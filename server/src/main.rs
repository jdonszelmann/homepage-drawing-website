mod error;
mod line;
mod server;
mod client;
mod rest;

use ringbuffer;
use ringbuffer::RingBuffer;
use ws;
use log::error;
use std::collections::{HashMap, HashSet};
use flexi_logger::{Logger, opt_format, Duplicate};
use std::net::{IpAddr, SocketAddr, SocketAddrV4};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::fs;
use std::env;
use crate::server::Server;
use crate::client::Client;

create_error!(JsonError, "couldn't deserialize");
create_error!(LockError, "couldn't lock");

fn parse_ip(data: &Vec<u8>) -> Option<IpAddr>{
    String::from_utf8_lossy(data.as_slice()).parse().ok()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let root;
    let port: u16;
    if args.len() > 1 {
        root = args[1].to_owned();
    }else{
        root = String::from("/");
    }
    if args.len() > 2 {
        port = args[2].parse().expect("Second argument should be port");

    }else{
        port = 80;
    }

    Logger::with_env_or_str("info, ws = warn, rocket = warn")
        .log_to_file()
        .duplicate_to_stderr(Duplicate::Info)
        .directory("/logs")
        .format(opt_format)
        .start()
        .unwrap();

    let socketaddress = SocketAddr::V4(SocketAddrV4::new("0.0.0.0".parse().unwrap(), port));
    let webaddress = SocketAddr::V4(SocketAddrV4::new("0.0.0.0".parse().unwrap(), port + 1));
    let capacity = 1024;
    let whitelist = Arc::new(Mutex::new(HashSet::new()));
    let blacklist = Arc::new(Mutex::new(HashSet::new()));
    let history = Arc::new(Mutex::new(RingBuffer::with_capacity(
        capacity
    )));
    let clients: Arc<Mutex<HashMap<u32, Client>>> =  Arc::new(Mutex::new(HashMap::new()));

    let inner_whitelist = whitelist.clone();
    let inner_clients = clients.clone();
    let inner_blacklist = blacklist.clone();

    let whitelistname  = root.clone() + "/config/whitelist";
    let blacklistname  = root.clone() + "/config/blacklist";

    let whitelistnametmp = whitelistname.clone();
    let blacklistnametmp = blacklistname.clone();
    thread::spawn(move ||{
        loop{
            thread::sleep(Duration::from_secs(5));
            let whitelistcontents = match fs::read_to_string(whitelistnametmp.clone()){
                Err(_) => {
                    error!("couldn't read whitelist file");
                    continue;
                }
                Ok(i) => i,
            };
            let blacklistcontents = match fs::read_to_string(blacklistnametmp.clone()){
                Err(_) => {
                    error!("couldn't read blacklist file");
                    continue;
                }
                Ok(i) => i,
            };
            
            let mut whitelock = match inner_whitelist.lock(){
                Ok(i) => i,
                Err(_) => {
                    error!("failed to lock whitelist.");
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

            let mut blacklock = match inner_blacklist.lock(){
                Ok(i) => i,
                Err(_) => {
                    error!("failed to lock blacklist.");
                    continue;
                }
            };

            whitelock.clear();
            for addr in whitelistcontents.split("\n"){
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
                whitelock.insert(address);
            }

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
            drop(whitelock);
        }
    });

    let passwordsfilename = root.clone() + "/config/passwords";
    thread::spawn(move ||{
        rest::main(webaddress, passwordsfilename.clone(), whitelistname.clone(), blacklistname.clone());
    });

    ws::Builder::new()
        .build(|out : ws::Sender| Server::new(whitelist.clone(),history.clone(),clients.clone(), capacity.clone(), blacklist.clone(), out))
        .unwrap()
        .listen(socketaddress)
        .unwrap();
}
