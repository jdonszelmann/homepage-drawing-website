mod error;

use ringbuffer;
use ringbuffer::RingBuffer;
use ws;
use log::{info, warn, error, debug};
use serde::{Serialize,Deserialize};
use serde_json::{Value,json};
use std::collections::{HashMap, HashSet};
use rand::Rng;
use flexi_logger::{Logger, opt_format, Duplicate};
use std::mem;
use std::net::{IpAddr, SocketAddr, SocketAddrV4};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::fs;

create_error!(JsonError, "couldn't deserialize");
create_error!(LockError, "couldn't lock");

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
struct Line(
    pub f64, // x
    pub f64, // y
    pub f64, // oldx
    pub f64, // oldy
    pub u8,  // color
);

#[derive(Debug)]
struct Client{
    pub connection_id: u32,
    pub oldx: f64,
    pub oldy: f64,
    pub color: u8,
    pub ip: IpAddr,
}

impl Client{
    fn new(connection_id: u32, ip: IpAddr) -> Self{
        let mut rng = rand::weak_rng();

        Client{
            connection_id,
            oldx: -1f64,
            oldy: -1f64,
            color: rng.gen_range(0,100),
            ip,
        }
    }
}

struct Server {
    channel: ws::Sender,
    blacklist: Arc<Mutex<HashSet<IpAddr>>>,
    readonlylist: Arc<Mutex<HashSet<IpAddr>>>,
    history: Arc<Mutex<RingBuffer<Line>>>,
    clients: Arc<Mutex<HashMap<u32, Client>>>,
    capacity: usize
}

impl Server {

    pub fn new(blacklist: Arc<Mutex<HashSet<IpAddr>>>, history: Arc<Mutex<RingBuffer<Line>>>, clients: Arc<Mutex<HashMap<u32, Client>>>, capacity: usize, readonlylist: Arc<Mutex<HashSet<IpAddr>>>, channel: ws::Sender) -> Self{

        return Self{
            channel,
            blacklist,
            history,
            clients,
            capacity,
            readonlylist,
        }
    }
}

impl ws::Handler for Server {

    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()>{
        match shake.peer_addr{
            Some(i) => {
                if self.blacklist.lock().or(Err(Box::new(LockError)))?.contains(&i.ip()){
                    self.channel.close(ws::CloseCode::Policy)?;
                    warn!("A blacklisted ip ({}) tried to connect", i);
                    return Ok(());
                }

                let conn_id = self.channel.connection_id();
                let client = Client::new(conn_id, i.ip());
                self.clients.lock().or(Err(Box::new(LockError)))?.insert(conn_id, client);

                info!("New connection from {}", i.ip());

                let response = json!({
                    "command": "history",
                    "numonline": self.clients.lock().or(Err(Box::new(LockError)))?.len(),
                    "capacity": self.capacity,
                    "history": self.history.lock().or(Err(Box::new(LockError)))?.to_vec(),
                });

                match self.channel.send(response.to_string()){
                    Ok(_) => (),
                    Err(i) => {
                        error!("An error occurred during the sending");
                        return Err(i);
                    }
                };
            },
            None => {
                self.channel.close(ws::CloseCode::Error)?;
                warn!("Someone connected without peer address");
            }
        }

        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        debug!("{}",msg);

        let conn_id = self.channel.connection_id();
        let numonline = self.clients.lock().or(Err(Box::new(LockError)))?.len();

        let mut clients = self.clients.lock().or(Err(Box::new(LockError)))?;
        let client = match clients.get_mut(&conn_id){
            Some(i) => i,
            None => {
                warn!("Found user that wasn't in client map. Kicking.");
                match self.channel.close(ws::CloseCode::Abnormal){
                    Ok(_) => (),
                    Err(_) => {
                        error!("couldn't close socket");
                        return Ok(());
                    }
                };
                return Ok(());
            }
        };

        if self.readonlylist.lock().or(Err(Box::new(LockError)))?.contains(&client.ip){
            return Ok(());
        }

        let data: Value = serde_json::from_str(msg.as_text()?)
            .or(Err(Box::new(JsonError)))?;

        let x = match data["x"].as_f64(){
            Some(i) => i,
            None => {
                return Ok(warn!("Received malformed update data"));
            }
        };
        let y = match data["y"].as_f64(){
            Some(i) => i,
            None => {
                return Ok(warn!("Received malformed update data"));
            }
        };

        let oldx = mem::replace(&mut client.oldx, x);
        let oldy = mem::replace(&mut client.oldy, y);

        if x < 0f64 || y < 0f64{
            return Ok(());
        }

        let response = json!({
            "command": "update",
            "x":x,
            "y":y,
            "oldx":oldx,
            "oldy":oldy,
            "color":client.color,
            "numonline":numonline,
        });

        self.channel.broadcast(response.to_string())?;

        if oldx < 0f64 || oldy < 0f64{
            return Ok(());
        }

        self.history.lock().or(Err(Box::new(LockError)))?.push(Line(
            x,y,oldx,oldy,client.color
        ));
        Ok(())
    }

    fn on_close(&mut self, _code: ws::CloseCode, _reason: &str) {
        let removed = match match self.clients.lock(){
            Ok(i) => i,
            Err(_) => {
                error!("couldn't lock blacklist");
                return;
            }
        }.remove(&self.channel.connection_id()){
            Some(i) => i,
            None => {
                warn!("Lost connection from someone not in the client map (possibly due to blacklist)");
                return;
            }
        };
        info!("Dropping connection from {}", removed.ip);

    }
}

fn main() {
    Logger::with_env_or_str("info, ws = warn")
        .log_to_file()
        .duplicate_to_stderr(Duplicate::Info)
        .directory("/logs")
        .format(opt_format)
        .start()
        .unwrap();

    let port = 4242;
    let address = SocketAddr::V4(SocketAddrV4::new("0.0.0.0".parse().unwrap(), port));
    let capacity = 1024;
    let blacklist = Arc::new(Mutex::new(HashSet::new()));
    let readonlylist = Arc::new(Mutex::new(HashSet::new()));
    let history = Arc::new(Mutex::new(RingBuffer::with_capacity(
        capacity
    )));
    let clients: Arc<Mutex<HashMap<u32, Client>>> =  Arc::new(Mutex::new(HashMap::new()));

    let inner_blacklist = blacklist.clone();
    let inner_clients = clients.clone();
    let inner_readonlylist = readonlylist.clone();
    thread::spawn(move ||{
        loop{
            thread::sleep(Duration::from_secs(5));
            let blacklistcontents = fs::read_to_string("/config/blacklist").expect("Couldn't read file");
            let readonlylistcontents = fs::read_to_string("/config/readonlylist").expect("Couldn't read file");

            let mut blacklock = match inner_blacklist.lock(){
                Ok(i) => i,
                Err(_) => {
                    error!("failed to lock blacklist.");
                    continue;
                }
            };

            let mut clientlock = match inner_clients.lock(){
                Ok(i) => i,
                Err(_) => {
                    error!("failed to lock clientlist.");
                    continue;
                }
            };

            let mut readonlylock = match inner_readonlylist.lock(){
                Ok(i) => i,
                Err(_) => {
                    error!("failed to lock readonlylist.");
                    continue;
                }
            };

            blacklock.clear();
            for addr in blacklistcontents.split("\n"){
                if addr.is_empty() || addr.trim_start().starts_with("#"){
                    continue;
                }
                let address = match addr.parse(){
                    Ok(i) => i,
                    Err(i) => {
                        error!("parse error: {}", i);
                        continue;
                    }
                };
                blacklock.insert(address);
            }

            readonlylock.clear();
            for addr in readonlylistcontents.split("\n"){
                if addr.is_empty() || addr.trim_start().starts_with("#"){
                    continue;
                }
                let address = match addr.parse(){
                    Ok(i) => i,
                    Err(i) => {
                        error!("parse error: {}", i);
                        continue;
                    }
                };
                readonlylock.insert(address);
            }

            let blacklisted: Vec<_> = clientlock
                .iter()
                .filter(|&(_, v)| blacklock.contains(&v.ip) )
                .map(|(k, _) | k.clone())
                .collect();
            for value in blacklisted {
                clientlock.remove(&value);
            }
            drop(blacklock);
            drop(clientlock);
            drop(readonlylock);
        }
    });

    ws::Builder::new()
        .build(|out : ws::Sender| Server::new(blacklist.clone(),history.clone(),clients.clone(), capacity.clone(), readonlylist.clone(), out))
        .unwrap()
        .listen(address)
        .unwrap();
}
