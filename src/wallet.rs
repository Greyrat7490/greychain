use std::{
    net::{TcpListener, TcpStream, SocketAddr, Ipv4Addr, IpAddr},
    io::{Read, Write}, 
    thread::{self, JoinHandle},
    time::Duration, 
    sync::{atomic::{AtomicU16, Ordering::Relaxed}, Arc, Mutex}
};

use crate::{package::{Package, PACKAGE_SIZE}, transaction::Transaction};

const TIMEOUT: Duration = Duration::from_secs(5);

pub struct Wallet {
    pub port: u16,
    online: Arc<Mutex<bool>>,
    recv_thread: JoinHandle<()>
}

impl Wallet {
    pub fn new() -> Wallet {
        if let Some((port, listener)) = init_receiver() {
            let online = Arc::new(Mutex::new(true));
            let recv_thread = recv_loop(Arc::clone(&online), listener);

            println!("created new wallet at port {}", port);
            return Wallet{ port, online, recv_thread };
        } else {
            panic!("ERROR: could not create socket");
        }
    }

    pub fn send_msg(&self, port: u16, msg: &str) -> bool {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
        let client = TcpStream::connect_timeout(&addr, TIMEOUT);

        if let Ok(stream) = client {
            send(stream, Package::new_msg(msg));
            return true;
        } else {
            println!("could not properly connect to server");
            return false;
        }
    }

    pub fn send_tx(&self, port: u16, tx: Transaction) -> bool {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
        let client = TcpStream::connect_timeout(&addr, TIMEOUT);

        if let Ok(stream) = client {
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

        println!("wallet is offline now");
    }
}

fn get_next_port() -> u16 {
    static NEXT_PORT: AtomicU16 = AtomicU16::new(6969);
    return NEXT_PORT.fetch_add(1, Relaxed);
}

fn init_receiver() -> Option<(u16, TcpListener)> {
    let port = get_next_port(); 
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    let listener = TcpListener::bind(addr).unwrap();
    listener.set_nonblocking(true).unwrap();
    // TODO: handle error (if port in use next port)

    return Some((port, listener));
}

fn recv_loop(online: Arc<Mutex<bool>>, listener: TcpListener) -> JoinHandle<()> {
    return thread::spawn(move || {
        // TODO: better spin lock
        while *online.lock().unwrap() {
            let stream = listener.accept();

            if let Ok((stream, _)) = stream {
                recv(stream);
            }
        }
    });
}

fn recv(mut stream: TcpStream) {
    let mut buf = [0; PACKAGE_SIZE];
    stream.read_exact(&mut buf).unwrap();

    let msg = Package::from(buf);

    println!("received:\n{}", msg);
}

fn send(mut stream: TcpStream, pkg: Package) {
    stream.write(&pkg.as_bytes()).unwrap();
}
