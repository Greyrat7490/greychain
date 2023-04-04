use std::{
    net::{TcpStream, SocketAddr, Ipv4Addr, IpAddr, TcpListener},
    thread::{JoinHandle, self},
    time::Duration, 
    sync::{Arc, Mutex}, fs
};

use crate::{
    net::{tcp::init_receiver, tcp::recv, tcp::send, pkg::{Package, PackageType}},
    blockchain::{Blockchain, Transaction, Block}, crypto::create_key_pair
};

use rsa::{RsaPrivateKey, RsaPublicKey, sha2::Sha256, pss::BlindedSigningKey};

const TIMEOUT: Duration = Duration::from_secs(5);
const BLOCKCHAINS_DIR: &str = "blockchains";

pub struct Wallet {
    pub port: u16,
    online: Arc<Mutex<bool>>,
    recv_thread: JoinHandle<()>,
    pub pub_key: RsaPublicKey,
    priv_key: RsaPrivateKey,
    blochchain: Arc<Mutex<Blockchain>>,
}

impl Wallet {
    pub fn new() -> Wallet {     
        let (pub_key, priv_key) = create_key_pair();
        let blochchain = Arc::new(Mutex::new(Blockchain::new()));

        let online = Arc::new(Mutex::new(true));
        let (port, listener) = init_receiver().expect("ERROR: could not create socket");

        let recv_thread = recv_loop(Arc::clone(&online), listener, Arc::clone(&blochchain));

        println!("created new wallet at port {}", port);
        return Wallet{ port, online, recv_thread, priv_key, pub_key, blochchain };
    }

    pub fn send_tx(&self, port: u16, payee: &RsaPublicKey, amount: f64) -> bool {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
        let client = TcpStream::connect_timeout(&addr, TIMEOUT);

        if let Ok(stream) = client {
            let tx = Transaction::new(&self.pub_key, payee, amount);
            let sign_key = BlindedSigningKey::<Sha256>::from(self.priv_key.clone());

            send(stream, Package::new_tx(tx, sign_key));

            return true;
        } else {
            println!("could not properly connect to server");
            return false;
        }
    }

    pub fn shutdown(self) {
        *self.online.lock().unwrap() = false;
        self.recv_thread.join().unwrap();

        save_blockchain(&self.blochchain.lock().unwrap(), &format!("wallet{}", self.port));

        println!("wallet is offline now");
    }

    pub fn save_blockchain(&self) {
        save_blockchain(&self.blochchain.lock().unwrap(), &self.get_name());
    }

    pub fn get_name(&self) -> String {
        return format!("wallet{}", self.port);
    }
}

fn recv_loop(online: Arc<Mutex<bool>>, listener: TcpListener, blockchain: Arc<Mutex<Blockchain>>) -> JoinHandle<()> {
    return thread::spawn(move || {
        // TODO: better spin lock?
        while *online.lock().unwrap() {
            let stream = listener.accept();

            if let Ok((stream, _)) = stream {
                let pkg = recv(stream);
                println!("received pkg type: {:?}", pkg.typ);
                handle_pkg(pkg, &blockchain);
            }
        }
    });
}

fn handle_pkg(pkg: Package, blockchain: &Arc<Mutex<Blockchain>>) {
    match pkg.typ {
        PackageType::Tx => {
            let tx = Transaction::deserialize(pkg.content);

            let blockchain = &mut blockchain.lock().unwrap();
            let block = Block::new(tx, blockchain.cur_hash, blockchain.get_round());
            blockchain.add_block(block);
        }
        PackageType::Block => {}
        PackageType::Fork => {}
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
