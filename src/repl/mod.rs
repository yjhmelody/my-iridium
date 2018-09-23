#![allow(dead_code)]

use assembler::Assembler;
use assembler::program_parsers::parse_program;
use nom::types::CompleteStr;
use std;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::Write;
use std::num::ParseIntError;
use std::path::Path;
use std::u8;
use vm::VM;


/// Core structure for the REPL for the Assembler
pub struct REPL {
    command_buffer: Vec<String>,
    vm: VM,
    asm: Assembler,
}

impl Default for REPL {
    fn default() -> Self {
        Self::new()
    }
}

impl REPL {
    /// Creates a REPL
    pub fn new() -> Self {
        Self {
            command_buffer: Vec::new(),
            vm: VM::new(),
            asm: Assembler::new(),
        }
    }

    /// Runs the repl
    pub fn run(&mut self) {
        println!("Welcome to Iridium!");
        loop {
            let mut buffer = String::new();
            // Block call
            let stdin = io::stdin();
            print!(">>>");
            io::stdout().flush().expect("Unable to flush stdout");
            stdin.read_line(&mut buffer).expect("Unable to read line from user");

            if buffer.trim() != ".history" {
                self.command_buffer.push(buffer.to_string());
            }

            let buffer = buffer.trim();
            match buffer {
                ".quit" => {
                    println!("Farewell!");
                    std::process::exit(0);
                }

                ".history" => {
                    for command in &self.command_buffer {
                        print!("{}", command);
                    }
                }

                ".program" => {
                    println!("Listing instructions currently in VM's program vector:");
                    for inst in &self.vm.program {
                        println!("{}", inst);
                    }
                    println!("End of Program Listing");
                }

                ".registers" => {
                    println!("Listing registers and all contents:");
                    println!("{:#?}", self.vm.registers);
                    println!("End of Register Listing");
                }

                ".clear_program" => {
                    println!("Removing all bytes from VM's program vector...");
                    self.vm.program.truncate(0);
                }

                ".clear_registers" => {
                    println!("Setting all registers to 0");
                    for i in 0..self.vm.registers.len() {
                        self.vm.registers[i] = 0;
                    }
                }

                ".load_file" => {
                    print!("Please enter the path to the file you wish to load: ");
                    io::stdout().flush().expect("Unable to flush stdout");
                    let mut tmp = String::new();
                    stdin.read_line(&mut tmp).expect("Unable to read line from user");
                    let tmp = tmp.trim();
                    let filename = Path::new(&tmp);
                    let mut f = match File::open(Path::new(&filename)) {
                        Ok(f) => f,
                        Err(e) => {
                            println!("There was an error opening that file: {:?}", e);
                            continue;
                        }
                    };

                    let mut contents = String::new();
                    f.read_to_string(&mut contents).expect("There was an error reading from the file");
                    let program = match parse_program(CompleteStr(&contents)) {
                        // Rusts pattern matching is pretty powerful an can even be nested
                        Ok((_remainder, program)) => {
                            program
                        },
                        Err(e) => {
                            println!("Unable to parse input: {:?}", e);
                            continue;
                        }
                    };
                    self.vm.program.append(&mut program.to_bytes(&self.asm.symbols));
                }

                _ => {
                    let program = match parse_program(buffer.into()) {
                        Ok((_, program)) => program,
                        Err(_) => {
                            println!("Unable to parse input");
                            continue;
                        }
                    };
                    self.vm.program.append(&mut program.to_bytes(&self.asm.symbols));
                    self.vm.run_once();
                }
            }
        }
    }

    /// Accepts a hexadecimal string WITHOUT a leading `0x` and returns a Vec of u8
    /// Example for a LOAD command: 00 01 03 E8
    fn parse_hex(&mut self, i: &str) -> Result<Vec<u8>, ParseIntError> {
        let split = i.split(" ").collect::<Vec<&str>>();
        let mut results: Vec<u8> = vec![];

        for hex_string in split {
            let byte = u8::from_str_radix(&hex_string, 16);
            match byte {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(results)
    }
}
