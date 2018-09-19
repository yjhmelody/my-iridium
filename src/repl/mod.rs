use assembler::program_parsers::parse_program;
use std;
use std::io;
use std::io::Write;
use std::num::ParseIntError;
use std::u8;
use vm::VM;

/// Core structure for the REPL for the Assembler
pub struct REPL {
    command_buffer: Vec<String>,
    vm: VM,
}

impl REPL {
    /// Creates a REPL
    pub fn new() -> Self {
        Self {
            command_buffer: Vec::new(),
            vm: VM::new(),
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

            if buffer != ".history" {
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
                ".registers" => {
                    println!("{:#?}", self.vm.registers);
                }

                _ => {
                    let program = match parse_program(buffer.into()) {
                        Ok((_, program)) => program,
                        Err(_) => {
                            println!("Unable to parse input");
                            continue;
                        }
                    };
                    self.vm.program.append(&mut program.to_bytes());
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
