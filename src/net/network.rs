use std::{net::{TcpStream, SocketAddr, IpAddr, Ipv4Addr}, time::Duration, collections::HashMap, fmt::Display};

use rsa::{pss::BlindedSigningKey, sha2::Sha256};

use super::{pkg::Package, tcp::send};

const TIMEOUT: Duration = Duration::from_secs(1);

pub struct Node {
    pub pub_key: String,
    pub port: u16
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "127.0.0.1:{}", self.port);
    }
}

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

    pub fn go_online(&self, pub_key: String, port: u16, sign_key: BlindedSigningKey::<Sha256>) {
        let pkg = Package::new_status(pub_key, port, true, sign_key);
        self.broadcast(pkg);
    }

    pub fn go_offline(&self, pub_key: String, port: u16, sign_key: BlindedSigningKey::<Sha256>) {
        let pkg = Package::new_status(pub_key, port, false, sign_key);
        self.broadcast(pkg);
    }

    pub fn register(&mut self, pub_key: String, port: u16) {
        if self.nodes.insert(pub_key, port) == None {
            println!("registered wallet{}", port);
        }
    }

    pub fn contains(&mut self, pub_key: &String) -> bool {
        return self.nodes.get(pub_key) == None;
    }

    pub fn deregister(&mut self, pub_key: String) {
        if let Some(port) = self.nodes.remove(&pub_key) {
            println!("deregistered wallet{}", port);
        }
    }

    pub fn get_nodes_except(&self, except_key: &String) -> Vec<Node>{
        let mut nodes = Vec::<Node>::with_capacity(self.nodes.len());

        for (pub_key, port) in &self.nodes {
            if pub_key != except_key {
                nodes.push(Node { pub_key: pub_key.clone(), port: *port });
            }
        }

        return nodes;
    }

    pub fn broadcast(&self, pkg: Package) {
        for (_, port) in &self.nodes {
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), *port);
            let stream = TcpStream::connect_timeout(&addr, TIMEOUT);

            if let Ok(stream) = stream {
                send(stream, pkg.clone());
            } else {
                println!("could not properly connect with {}", addr);
            }
        }
    }

    pub fn broadcast_forward(&self, pkg: Package) {
        for (_, port) in &self.nodes {
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), *port);
            let stream = TcpStream::connect_timeout(&addr, TIMEOUT);

            if let Ok(stream) = stream {
                let mut forwarded_pkg = pkg.clone();
                forwarded_pkg.is_forwarded = true;
                send(stream, forwarded_pkg);
            } else {
                println!("could not properly connect with {}", addr);
            }
        }
    }

    pub fn get_port(&self, pub_key: &String) -> u16 {
        if let Some(port) = self.nodes.get(pub_key) {
            return *port;
        } else {
            return 0;
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
