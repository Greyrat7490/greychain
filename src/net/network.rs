use std::{net::{TcpStream, SocketAddr, IpAddr, Ipv4Addr}, time::Duration, collections::HashMap, fmt::Display, thread::sleep};

use rsa::{pss::BlindedSigningKey, sha2::Sha256};

use super::{pkg::{Package, PackageType}, tcp::send, node::Node};

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(1);
const RESPONSE_SLEEP: Duration = Duration::from_millis(100);
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(7);

pub struct Network {
    nodes: HashMap<String, u16> // TODO: add ip later
}

impl Network {
    pub fn new(master_nodes: &Vec<Node>) -> Network {
        let mut nodes = HashMap::<String, u16>::new();
        for node in master_nodes {
            nodes.insert(node.pub_key.to_string(), node.port);
        }

        return Network{ nodes };
    }

    pub fn new_empty() -> Network {
        return Network{ nodes: HashMap::<String, u16>::new() };
    }

    pub fn go_offline(&self, pub_key: String, port: u16, sign_key: BlindedSigningKey::<Sha256>) {
        let node = Node {pub_key: pub_key.clone(), port, online: false};
        let pkg = Package::new(node, PackageType::Status, pub_key, sign_key);
        self.broadcast(pkg);
    }

    pub fn go_online(&self, pub_key: String, port: u16, sign_key: BlindedSigningKey::<Sha256>) -> Result<(), &str> {
        let node = Node {pub_key: pub_key.clone(), port, online: true};
        let pkg = Package::new(node, PackageType::Status, pub_key, sign_key);
        self.broadcast(pkg);

        const MAX_ITER: usize = (RESPONSE_TIMEOUT.as_millis() / RESPONSE_SLEEP.as_millis()) as usize;

        for _ in 0..MAX_ITER {
            if !self.nodes.is_empty() { return Ok(()); }
            sleep(RESPONSE_SLEEP); 
        }

        return Err("no response");
    }

    pub fn register(&mut self, pub_key: String, port: u16) {
        self.nodes.insert(pub_key, port);
    }

    pub fn contains(&mut self, pub_key: &String) -> bool {
        return self.nodes.get(pub_key) == None;
    }

    pub fn deregister(&mut self, pub_key: String) {
        self.nodes.remove(&pub_key);
    }

    pub fn to_nodes(&self) -> Vec<Node> {
        return self.nodes.iter()
            .map(|(pub_key, port)| Node { pub_key: pub_key.clone(), port: *port, online: true })
            .collect();
    }

    pub fn broadcast(&self, pkg: Package) {
        for (_, port) in &self.nodes {
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), *port);
            let stream = TcpStream::connect_timeout(&addr, CONNECTION_TIMEOUT);

            if let Ok(stream) = stream {
                send(stream, pkg.clone());
            } else {
                println!("could not connect with {}", addr);
            }
        }
    }

    pub fn broadcast_forward(&self, pkg: Package) {
        for (_, port) in &self.nodes {
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), *port);
            let stream = TcpStream::connect_timeout(&addr, CONNECTION_TIMEOUT);

            if let Ok(stream) = stream {
                let mut forwarded_pkg = pkg.clone();
                forwarded_pkg.is_forwarded = true;
                send(stream, forwarded_pkg);
            } else {
                println!("could not connect with {}", addr);
            }
        }
    }

    pub fn get_len(&self) -> usize {
        return self.nodes.len();
    }
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.nodes.iter()
                        .map(|(_, port)| format!("127.0.0.1:{}\n", port))
                        .collect::<String>());
    }
}
