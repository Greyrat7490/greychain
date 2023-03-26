use std::{net::{TcpListener, TcpStream, SocketAddr}, io::{Read, Write}, thread, str::FromStr, time::Duration};

fn main() {
    println!("Hello, world!");

    init_server();
    init_client();
}

fn init_server() {
    let listener = TcpListener::bind("127.0.0.1:6969").unwrap();

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
}

fn init_client() {
    let addr = SocketAddr::from_str("127.0.0.1:6969").unwrap();

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
