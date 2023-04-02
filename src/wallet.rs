use std::{
    net::{TcpStream, SocketAddr, Ipv4Addr, IpAddr, TcpListener},
    thread::{JoinHandle, self},
    time::Duration, 
    sync::{Arc, Mutex}, fs
};

use crate::{
    net::{tcp::init_receiver, tcp::recv, tcp::send, pkg::{Package, PackageType}},
    blockchain::{Blockchain, Transaction, Block}
};

use rsa::{RsaPrivateKey, RsaPublicKey};

const TIMEOUT: Duration = Duration::from_secs(5);
const RSA_BITS: usize = 2048;
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
        let mut rng = rand::thread_rng();
        let priv_key = RsaPrivateKey::new(&mut rng, RSA_BITS).expect("ERROR: could not create key");
        let pub_key = RsaPublicKey::from(&priv_key);
        
        let (port, listener) = init_receiver().expect("ERROR: could not create socket");
        let online = Arc::new(Mutex::new(true));

        let blochchain = Arc::new(Mutex::new(Blockchain::new()));

        let recv_thread = recv_loop(Arc::clone(&online), listener, Arc::clone(&blochchain));

        println!("created new wallet at port {}", port);
        return Wallet{ port, online, recv_thread, priv_key, pub_key, blochchain };
    }

    pub fn send_tx(&self, port: u16, payee: &RsaPublicKey, amount: f64) -> bool {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
        let client = TcpStream::connect_timeout(&addr, TIMEOUT);

        if let Ok(stream) = client {
            let tx = Transaction::new(&self.pub_key, payee, &self.priv_key, amount);
            send(stream, Package::new_tx(tx));
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
            let block = Block::new(tx);

            blockchain.lock().unwrap().add_block(block);
        }
        PackageType::Block => {}
        PackageType::Fork => {}
    }

    println!("{}", pkg);
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


#[cfg(test)]
mod tests {
    use rsa::{
        RsaPrivateKey, RsaPublicKey,
        pss::{BlindedSigningKey, VerifyingKey},
        sha2::Sha256,
        signature::{Keypair, RandomizedSigner, Verifier}
    };

    use crate::wallet::RSA_BITS;

    #[test]
    pub fn sign() {
        let mut rng = rand::thread_rng();
        let priv_key = RsaPrivateKey::new(&mut rng, RSA_BITS).expect("ERROR: could not create key");

        let sign_key = BlindedSigningKey::<Sha256>::from(priv_key);
        let veri_key = sign_key.verifying_key();

        let msg = "test message";
        let sign = sign_key.sign_with_rng(&mut rng, msg.as_bytes());
        assert_ne!(msg.to_string(), sign.to_string());

        veri_key.verify(msg.as_bytes(), &sign).unwrap();
    }

    #[test]
    pub fn sign_verify_key_from_pub() {
        let mut rng = rand::thread_rng();
        let priv_key = RsaPrivateKey::new(&mut rng, RSA_BITS).expect("ERROR: could not create key");
        let pub_key = RsaPublicKey::from(&priv_key);

        let sign_key = BlindedSigningKey::<Sha256>::from(priv_key);
        let veri_key = VerifyingKey::<Sha256>::from(pub_key);

        let msg = "test message";
        let sign = sign_key.sign_with_rng(&mut rng, msg.as_bytes());
        assert_ne!(msg.to_string(), sign.to_string());

        veri_key.verify(msg.as_bytes(), &sign).unwrap();
    }

    #[test]
    pub fn sign_verify_key_from_pub_fail() {
        let mut rng = rand::thread_rng();
        let priv_key = RsaPrivateKey::new(&mut rng, RSA_BITS).expect("ERROR: could not create key");
        let pub_key = RsaPublicKey::from(&priv_key);

        let sign_key = BlindedSigningKey::<Sha256>::from(priv_key);
        let veri_key = VerifyingKey::<Sha256>::from(pub_key);

        let msg = "test message";
        let sign = sign_key.sign_with_rng(&mut rng, msg.as_bytes());
        assert_ne!(msg.to_string(), sign.to_string());

        let wrong_msg = "wrong test message";
        if let Ok(()) = veri_key.verify(wrong_msg.as_bytes(), &sign) {
            println!("verify should return error");
            assert!(false);
        }
    }
}
