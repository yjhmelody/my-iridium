extern crate byteorder;
#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate my_iridium;
extern crate num_cpus;
extern crate uuid;

use clap::App;
use my_iridium::assembler;
use my_iridium::repl::REPL;
use my_iridium::vm;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

fn main() {
    env_logger::init();
    info!("Starting logging!");
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let data_root_dir = matches
        .value_of("DATA_ROOT_DIR")
        .unwrap_or("/var/lib/iridium");

    if make_directory(data_root_dir).is_err() {
        println!("There was an error creating the default root data directory");
        std::process::exit(1);
    }

    if matches.is_present("ENABLE_REMOTE_ACCESS") {
        // defaults to 127.0.0.1:2244
        let port = matches.value_of("LISTEN_PORT").unwrap_or("2244");
        let host = matches.value_of("LISTEN_HOST").unwrap_or("127.0.0.1");
        start_remote_server(host.to_string(), port.to_string());
    }

    let num_threads = match matches.value_of("THREADS") {
        Some(number) => {
            match number.parse::<usize>() {
                Ok(v) => { v },
                Err(_e) => {
                    println!("Invalid argument for number of threads: {}. Using default", number);
                    num_cpus::get()
                },
            }
        }

        None => num_cpus::get()
    };

    match matches.value_of("INPUT_FILE") {
        Some(filename) => {
            let program = read_file(filename);
            let mut asm = assembler::Assembler::new();
            let mut vm = vm::VM::new();
            vm.logical_cores = num_threads;
            let program = asm.assemble(&program);

            match program {
                Ok(p) => {
                    vm.add_bytes(p);
                    let events = vm.run();
                    println!("VM Events");
                    println!("--------------");
                    for event in &events {
                        println!("{:#?}", event);
                    };
                    process::exit(0);
                },

                Err(e) => {
                    println!("program running error {:?}", e);
                },
            }
        },
        None => {
            let mut repl = REPL::new();
            let mut rx = repl.rx_pipe.take();
            thread::spawn(move || {
                let chan = rx.unwrap();
                loop {
                    match chan.recv() {
                        Ok(msg) => println!("{}", msg),
                        Err(_e) => {},
                    }
                }
            });
            repl.run();
        },
    }
}


fn start_remote_server(host: String, port: String) {
    std::thread::spawn(move || {
        let mut sh = my_iridium::remote::server::Server::new(host, port);
        sh.listen();
    });
}

fn start_repl() {
    let mut repl = REPL::new();
    repl.run();
}

fn read_file(path: &str) -> String {
    match File::open(Path::new(&path)) {
        Ok(mut fh) => {
            let mut contents = String::new();
            match fh.read_to_string(&mut contents) {
                Ok(_) => {
                    contents
                },
                Err(e) => {
                    println!("There was an error reading file: {:?}", e);
                    std::process::exit(1);
                }
            }
        },
        Err(e) => {
            println!("File not found: {:?}", e);
            std::process::exit(1)
        }
    }
}

fn make_directory(dir: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    Ok(())
}


