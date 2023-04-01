use std::{
    net::{TcpListener, TcpStream, SocketAddr, Ipv4Addr, IpAddr},
    io::{Read, Write}, 
    thread::{self, JoinHandle},
    sync::{atomic::{AtomicU16, Ordering::Relaxed}, Arc, Mutex}
};

use super::pkg::{Package, PKG_SIZE};

fn get_next_port() -> u16 {
    static NEXT_PORT: AtomicU16 = AtomicU16::new(6969);
    return NEXT_PORT.fetch_add(1, Relaxed);
}

pub fn init_receiver() -> Option<(u16, TcpListener)> {
    let port = get_next_port(); 
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    let listener = TcpListener::bind(addr).unwrap();
    listener.set_nonblocking(true).unwrap();
    // TODO: handle error (if port in use next port)

    return Some((port, listener));
}

pub fn recv_loop(online: Arc<Mutex<bool>>, listener: TcpListener) -> JoinHandle<()> {
    return thread::spawn(move || {
        // TODO: better spin lock?
        while *online.lock().unwrap() {
            let stream = listener.accept();

            if let Ok((stream, _)) = stream {
                recv(stream);
            }
        }
    });
}

pub fn recv(mut stream: TcpStream) {
    let mut buf = [0; PKG_SIZE];
    stream.read_exact(&mut buf).unwrap();

    let content = Package::deserialize(buf);
    if !content.verify() {
        println!("ERROR: package is corrupted");
    }
    println!("received:\n{}", content);
}

pub fn send(mut stream: TcpStream, pkg: Package) {
    stream.write(&pkg.serialize()).unwrap();
}
