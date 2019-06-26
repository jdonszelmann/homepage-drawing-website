use core::mem;
use std::sync::{Arc, Mutex};
use std::net::IpAddr;
use crate::client::Client;
use std::collections::{HashMap, HashSet};
use ringbuffer::RingBuffer;
use crate::line::Line;
use log::{info, warn, error, debug};
use serde_json::{Value,json};
use crate::parse_ip;
use crate::LockError;
use crate::JsonError;

pub struct Server {
    channel: ws::Sender,
    whitelist: Arc<Mutex<HashSet<IpAddr>>>,
    readonlylist: Arc<Mutex<HashSet<IpAddr>>>,
    history: Arc<Mutex<RingBuffer<Line>>>,
    clients: Arc<Mutex<HashMap<u32, Client>>>,
    capacity: usize
}

impl Server {

    pub fn new(whitelist: Arc<Mutex<HashSet<IpAddr>>>, history: Arc<Mutex<RingBuffer<Line>>>, clients: Arc<Mutex<HashMap<u32, Client>>>, capacity: usize, readonlylist: Arc<Mutex<HashSet<IpAddr>>>, channel: ws::Sender) -> Self{

        return Self{
            channel,
            whitelist,
            history,
            clients,
            capacity,
            readonlylist,
        }
    }
}

impl ws::Handler for Server {

    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()>{
        let mut tmp_ip = None;

        for (name, value) in shake.request.headers(){
            if name == &String::from("X-Real-Ip"){
                tmp_ip = parse_ip(value);
            }
        }
        let ip = match tmp_ip{
            Some(i) => i,
            None => match shake.request.client_addr()? {
                Some(i) => match i.parse() {
                    Ok(i) => i,
                    Err(_) => {
                        error!("couldn't parse address {}", i);
                        return Ok(());
                    }
                },
                None => match shake.peer_addr {
                    Some(i) => i.ip(),
                    None => {
                        self.channel.close(ws::CloseCode::Error)?;
                        warn!("Someone connected without peer address");
                        return Ok(());
                    }
                }
            }
        };

        if !self.whitelist.lock().or(Err(Box::new(LockError)))?.contains(&ip){
            self.channel.close(ws::CloseCode::Policy)?;
            warn!("A non whitelisted ip ({}) tried to connect", ip);
            return Ok(());
        }

        let conn_id = self.channel.connection_id();
        let client = Client::new(conn_id, ip);
        let client2 = client.clone();
        self.clients.lock().or(Err(Box::new(LockError)))?.insert(conn_id, client);

        info!("New connection from {}", ip);

        let response = json!({
            "command": "history",
            "color": client2.color,
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
                error!("couldn't lock client");
                return;
            }
        }.remove(&self.channel.connection_id()){
            Some(i) => i,
            None => {
                warn!("Lost connection from someone not in the client map (possibly due to whitelist)");
                return;
            }
        };
        info!("Dropping connection from {}", removed.ip);

    }
}
