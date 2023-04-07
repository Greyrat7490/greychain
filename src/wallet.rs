use std::{
    net::{TcpStream, SocketAddr, Ipv4Addr, IpAddr, TcpListener},
    thread::{JoinHandle, self},
    time::Duration,
    sync::{Arc, Mutex}, fs
};

use crate::{
    net::{
        tcp::{init_receiver, recv, send},
        pkg::{Package, PackageType},
        network::Network, serialize::Serializer, node::Node
    },
    blockchain::{Blockchain, Transaction, Block},
    crypto::create_key_pair
};

use rsa::{RsaPrivateKey, RsaPublicKey, sha2::Sha256, pss::BlindedSigningKey, pkcs8::EncodePublicKey};

const TIMEOUT: Duration = Duration::from_secs(5);
const BLOCKCHAINS_DIR: &str = "blockchains";

pub struct Wallet {
    pub port: u16,

    pub pub_key_pem: String,
    pub_key: RsaPublicKey,
    priv_key: RsaPrivateKey,
    sign_key: BlindedSigningKey<Sha256>,

    blochchain: Arc<Mutex<Blockchain>>,
    online: Arc<Mutex<bool>>,
    recv_thread: JoinHandle<()>,
    network: Arc<Mutex<Network>>
}

impl Wallet {
    pub fn new(master_nodes: &Vec<Node>) -> Wallet {
        let (pub_key, priv_key) = create_key_pair();
        let pub_key_pem = pub_key.to_public_key_pem(rsa::pkcs8::LineEnding::LF).unwrap();
        let sign_key = BlindedSigningKey::<Sha256>::from(priv_key.clone());

        let blochchain = Arc::new(Mutex::new(Blockchain::new()));

        let online = Arc::new(Mutex::new(true));
        let (port, listener) = init_receiver().expect("ERROR: could not create socket");

        let network = Arc::new(Mutex::new(Network::new(master_nodes)));
        let recv_thread = recv_loop(pub_key_pem.clone(), sign_key.clone(), Arc::clone(&online), listener, Arc::clone(&blochchain), Arc::clone(&network));
        network.lock().unwrap().update_status(pub_key_pem.clone(), port, true, sign_key.clone());

        println!("created new wallet at port {}", port);
        return Wallet{ port, online, recv_thread, priv_key, pub_key, blochchain, network, pub_key_pem, sign_key };
    }

    pub fn send_tx(&self, payee: &String, amount: f64) -> bool {
        let port = self.network.lock().unwrap().get_port(payee);
        if port == 0 {
            return false;
        }
        
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
        let client = TcpStream::connect_timeout(&addr, TIMEOUT);

        if let Ok(stream) = client {
            let tx = Transaction::new(&self.pub_key_pem, payee, amount);
            let sender = tx.payer.clone();
            send(stream, Package::new(tx, PackageType::Tx, sender, self.sign_key.clone()));
            return true;
        } else {
            println!("could not properly connect to server");
            return false;
        }
    }

    pub fn shutdown(self) {
        let pub_key_pem = self.pub_key.to_public_key_pem(rsa::pkcs8::LineEnding::LF).unwrap();
        let sign_key = BlindedSigningKey::<Sha256>::from(self.priv_key.clone());

        self.network.lock().unwrap().update_status(pub_key_pem, self.port, false, sign_key);
        *self.online.lock().unwrap() = false;
        self.recv_thread.join().unwrap();

        save_blockchain(&self.blochchain.lock().unwrap(), &format!("wallet{}", self.port));

        println!("wallet is offline now");
    }

    pub fn save_blockchain(&self) {
        save_blockchain(&self.blochchain.lock().unwrap(), &self.get_name());
    }

    pub fn show_network(&self) {
        println!("------- {} network -------\n{}", self.get_name(), self.network.lock().unwrap());
    }

    pub fn get_network_len(&self) -> usize {
        return self.network.lock().unwrap().get_len();
    }

    pub fn get_name(&self) -> String {
        return format!("wallet{}", self.port);
    }
}

impl PartialEq for Wallet {
    fn eq(&self, other: &Self) -> bool {
        return self.pub_key == other.pub_key;
    }
}

fn recv_loop(pub_key: String, sign_key: BlindedSigningKey::<Sha256>, online: Arc<Mutex<bool>>,
             listener: TcpListener, blockchain: Arc<Mutex<Blockchain>>, network: Arc<Mutex<Network>>) -> JoinHandle<()> {

    return thread::spawn(move || {
        // TODO: better spin lock?
        while *online.lock().unwrap() {
            let stream = listener.accept();

            if let Ok((stream, _)) = stream {
                let pkg = recv(stream);
                println!("received pkg type: {:?}", pkg.typ);
                handle_pkg(&pub_key, &sign_key, pkg, &blockchain, &network);
            }
        }
    });
}

fn handle_pkg(pub_key: &String, sign_key: &BlindedSigningKey::<Sha256>, pkg: Package,
              blockchain: &Arc<Mutex<Blockchain>>, network: &Arc<Mutex<Network>>) {
    match pkg.typ {
        PackageType::Tx => {
            let tx = Transaction::deserialize(&pkg.content).1;

            let blockchain = &mut blockchain.lock().unwrap();
            let block = Block::new(tx, blockchain.cur_hash, blockchain.get_round());
            blockchain.add_block(block);
        }
        PackageType::Status => {
            let node = Node::deserialize(&pkg.content).1;

            let network = &mut network.lock().unwrap();
            if node.online {
                let is_forwarded = pkg.is_forwarded;

                if network.contains(&node.pub_key) {
                    network.broadcast_forward(pkg);
                    network.register(node.pub_key.clone(), node.port);
                }

                if !is_forwarded {
                    let nodes = network.get_nodes_except(&node.pub_key);
                    let nodes_pkg = Package::new(nodes, PackageType::NodesRes, pub_key.to_string(), sign_key.to_owned());

                    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), node.port);
                    let client = TcpStream::connect_timeout(&addr, TIMEOUT);

                    if let Ok(stream) = client {
                        send(stream, nodes_pkg);
                    } else {
                        println!("could not properly connect to server");
                    }
                }
            } else {
                network.deregister(node.pub_key);
            }
        }
        PackageType::NodesRes => {
            let nodes = Vec::<Node>::deserialize(&pkg.content).1;

            let network = &mut network.lock().unwrap();
            for node in nodes {
                network.register(node.pub_key, node.port);
            }
        }
        PackageType::Block => { todo!() }
        PackageType::Fork => { todo!() }
    }
}

fn save_blockchain(blockchain: &Blockchain, name: &str) {
    fs::create_dir_all(BLOCKCHAINS_DIR).unwrap();

    let content = blockchain.to_string();

    if let Err(err) = fs::write(BLOCKCHAINS_DIR.to_owned() + "/" + name, &content) {
        println!("{}", err);
    } else {
        println!("saved blockchain to {}", name);
    }
}
