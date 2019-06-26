use std::net::IpAddr;
use rand;
use rand::Rng;

#[derive(Debug, Copy, Clone)]
pub struct Client{
    pub connection_id: u32,
    pub oldx: f64,
    pub oldy: f64,
    pub color: u8,
    pub ip: IpAddr,
}

impl Client{
    pub fn new(connection_id: u32, ip: IpAddr) -> Self{
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
