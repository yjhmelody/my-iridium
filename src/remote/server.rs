use remote::client::Client;
use std::io::BufReader;
use std::net::TcpListener;
use std::thread;

pub struct Server {
    hostname: String,
    port: String,
}

impl Server {
    pub fn new(hostname: String, port: String) -> Self {
        Self {
            hostname,
            port,
        }
    }

    pub fn listen(&mut self) {
        println!("Initializing TCP server...");
        let listener = TcpListener::bind(self.hostname.clone() + ":" + &self.port).unwrap();
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            thread::spawn(|| {
                let mut client = Client::new(stream);
            });
        }
    }
}