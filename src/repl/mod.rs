use assembler::program_parsers::parse_program;
use nom::types::CompleteStr;
use std;
use std::io;
use std::io::Write;
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
                    let program = parse_program(CompleteStr(buffer));
                    if !program.is_ok() {
                        println!("Unable to parse input");
                        continue;
                    }

                    let (_, program) = program.unwrap();
                    let bytecodes = program.to_bytes();
                    // TODO: Make a function to let us add bytes to the VM
                    for byte in bytecodes {
                        self.vm.add_byte(byte);
                    }
                    self.vm.run_once();
                }
            }
        }
    }
}
