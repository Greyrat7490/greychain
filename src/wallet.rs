use std::{
    net::{TcpListener, TcpStream, SocketAddr, Ipv4Addr, IpAddr},
    io::{Read, Write}, 
    thread,
    time::Duration, 
    sync::atomic::{AtomicU16, Ordering::Relaxed}
};

pub struct Wallet {
    pub port: u16
}

impl Wallet {
    pub fn new() -> Wallet {
        let port = init_server();
        if let Some(port) = port {
            init_client(port);

            println!("created new wallet at port {}", port);
            return Wallet{ port };
        } else {
            println!("ERROR: could not create socket");
            println!("INFO: wallet is offline");
        }

        return Wallet { port: 0 };
    }
}

fn get_next_port() -> u16 {
    static NEXT_PORT: AtomicU16 = AtomicU16::new(6969);
    return NEXT_PORT.fetch_add(1, Relaxed);
}

fn init_server() -> Option<u16> {
    let port = get_next_port(); 
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    let listener = TcpListener::bind(addr).unwrap();
    // TODO: handle error (if port in use next port)

    thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                println!("connected to client");

                send(stream);
            } else {
                panic!("could not properly connect to client")
            }
        }
    });

    return Some(port);
}

fn init_client(port: u16) {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    let client = TcpStream::connect_timeout(&addr, Duration::from_secs(5));

    if let Ok(stream) = client {
        println!("connected to server");

        recv(stream);
    } else {
        panic!("could not properly connect to server")
    }
}

fn recv(mut stream: TcpStream) {
    println!("recv msg");

    let mut buf = String::new();
    stream.read_to_string(&mut buf).unwrap();

    println!("recieved: {}", buf);
}

fn send(mut stream: TcpStream) {
    println!("send msg");

    stream.write(b"test message").unwrap();
}
