#[macro_use]
extern crate clap;
extern crate my_iridium;
extern crate num_cpus;

use clap::App;
use my_iridium::assembler;
use my_iridium::repl;
use my_iridium::vm;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

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

        None => {
            num_cpus::get()
        }
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
        None => { start_repl(); },
    }
}

fn start_repl() {
    let mut repl = repl::REPL::new();
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
