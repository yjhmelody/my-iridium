#![allow(unused_imports)]
#![allow(dead_code)]

use repl;
use std::io::{BufRead, Read, Write};
use std::io::{BufReader, BufWriter};
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;
use std::thread;

pub struct Client {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    raw_stream: TcpStream,
    repl: repl::REPL,
}

impl Client {
    /// Creates a client to deal with r/w data
    pub fn new(stream: TcpStream) -> Self {
        let reader = BufReader::new(stream.try_clone().unwrap());
        let writer = BufWriter::new(stream.try_clone().unwrap());
        let raw_stream = stream;
        let repl = repl::REPL::new();
        Self {
            reader,
            writer,
            raw_stream,
            repl,
        }
    }

    /// write all message
    fn w(&mut self, msg: &str) -> bool {
        match self.writer.write_all(msg.as_bytes()) {
            Ok(_) => match self.writer.flush() {
                Ok(_) => true,
                Err(err) => {
                    println!("Error flushing to client: {}", err);
                    false
                }
            },

            Err(err) => {
                println!("Error writing to client: {}", err);
                false
            }
        }
    }

    /// Write `>>> ` to the user
    fn write_prompt(&mut self) {
        self.w(repl::PROMPT);
    }

    // Creates a new thread to receive message looply
    fn recv_loop(&mut self) {
        // TODO: Make this safer on unwrap
        let rx = self.repl.rx_pipe.take();
        let mut writer = self.raw_stream.try_clone().unwrap();
        thread::spawn(move || {
            let chan = rx.unwrap();
            loop {
                match chan.recv() {
                    Ok(msg) => {
                        writer.write_all(msg.as_bytes());
                        writer.flush();
                    }
                    Err(_e) => {}
                }
            }
        });
    }

    /// Runs the client to connect the remote
    pub fn run(&mut self) {
        self.recv_loop();
        let mut buf = String::new();
        let banner = repl::REMOTE_BANNER.to_owned() + "\n" + repl::PROMPT;
        self.w(&banner);
        loop {
            match self.reader.read_line(&mut buf) {
                Ok(_) => {
                    buf.trim_right();
                    self.repl.run_single(&buf);
                }
                Err(e) => {
                    println!("Error receiving: {:#?}", e);
                }
            }
        }
    }
}

