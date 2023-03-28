use std::{
    net::{TcpStream, SocketAddr, Ipv4Addr, IpAddr},
    thread::JoinHandle,
    time::Duration, 
    sync::{Arc, Mutex}
};

use crate::{package::Package, net, transaction::Transaction};

use rsa::{RsaPrivateKey, RsaPublicKey};

const TIMEOUT: Duration = Duration::from_secs(5);
const RSA_BITS: usize = 2048;

pub struct Wallet {
    pub port: u16,
    online: Arc<Mutex<bool>>,
    recv_thread: JoinHandle<()>,
    pub pub_key: RsaPublicKey,
    priv_key: RsaPrivateKey,
}

impl Wallet {
    pub fn new() -> Wallet {
        let mut rng = rand::thread_rng();
        let priv_key = RsaPrivateKey::new(&mut rng, RSA_BITS).expect("ERROR: could not create key");
        let pub_key = RsaPublicKey::from(&priv_key);
        
        let (port, listener) = net::init_receiver().expect("ERROR: could not create socket");
        let online = Arc::new(Mutex::new(true));
        let recv_thread = net::recv_loop(Arc::clone(&online), listener);

        println!("created new wallet at port {}", port);
        return Wallet{ port, online, recv_thread, priv_key, pub_key };
    }

    pub fn send_msg(&self, port: u16, msg: &str) -> bool {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
        let client = TcpStream::connect_timeout(&addr, TIMEOUT);

        if let Ok(stream) = client {
            net::send(stream, Package::new_msg(msg));
            return true;
        } else {
            println!("could not properly connect to server");
            return false;
        }
    }

    pub fn send_tx(&self, port: u16, payee: &RsaPublicKey, amount: f64) -> bool {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
        let client = TcpStream::connect_timeout(&addr, TIMEOUT);

        if let Ok(stream) = client {
            let tx = Transaction::new(&self.pub_key, payee, &self.priv_key, amount);
            net::send(stream, Package::new_tx(tx));
            return true;
        } else {
            println!("could not properly connect to server");
            return false;
        }
    }

    pub fn shutdown(self) {
        *self.online.lock().unwrap() = false;
        self.recv_thread.join().unwrap();

        println!("wallet is offline now");
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
