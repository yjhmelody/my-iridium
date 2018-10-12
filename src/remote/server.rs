use remote::client::Client;
use std::io::BufReader;
use std::net::TcpListener;
use std::thread;

pub struct Server {
    hostname: String,
    port: String,
}

impl Server {
    /// Creates a server to deal with remote access
    pub fn new(hostname: String, port: String) -> Self {
        Self {
            hostname,
            port,
        }
    }

    /// When anyone remote access, creates a new thread to deal with remote commands
    pub fn listen(&mut self) {
        println!("Initializing TCP server...");
        let listener = TcpListener::bind(self.hostname.clone() + ":" + &self.port).unwrap();
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            thread::spawn(|| {
                let mut client = Client::new(stream);
                client.run();
            });
        }
    }
}