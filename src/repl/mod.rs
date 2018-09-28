use assembler::Assembler;
use assembler::program_parsers::parse_program;
use repl::command_parser::CommandParser;
use scheduler::Scheduler;
use std;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::Write;
use std::num::ParseIntError;
use std::path::Path;
use std::u8;
use vm::VM;

pub mod command_parser;


/// Core structure for the REPL for the Assembler
pub struct REPL {
    command_buffer: Vec<String>,
    vm: VM,
    asm: Assembler,
    scheduler: Scheduler,
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
            scheduler: Scheduler::new(),
        }
    }

    /// Runs the repl
    pub fn run(&mut self) {
        println!("Welcome to Iridium!");
        loop {
            let mut buffer = String::new();
            // Block call
            let stdin = io::stdin();
            // print! does not automatically flush stdout
            print!(">>> ");
            io::stdout().flush().expect("Unable to flush stdout");
            stdin
                .read_line(&mut buffer)
                .expect("Unable to read line from user");

            self.command_buffer.push(buffer.clone());
            let buffer = buffer.trim();
            // commands are start with `!`
            if buffer.starts_with("!") {
                self.excute_command(&buffer);
            } else {
                let program = match parse_program(buffer.into()) {
                    Ok((_, program)) => program,
                    Err(e) => {
                        println!("Unable to parse input {:?}", e);
                        continue;
                    }
                };
                self.vm
                    .program
                    .append(&mut program.to_bytes(&self.asm.symbols));
                self.vm.run_once();
            }
        }
    }

    /// Accepts a hexadecimal string WITHOUT a leading `0x` and returns a Vec of u8
    /// Example for a LOAD command: 00 01 03 E8
    #[allow(dead_code)]
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

    fn excute_command(&mut self, input: &str) {
        let args = CommandParser::tokenize(input);
        match args[0] {
            "!quit" => self.quit(&args[1..]),
            "!history" => self.history(&args[1..]),
            "!program" => self.program(&args[1..]),
            "!clear_program" => self.clear_program(&args[1..]),
            "!clear_registers" => self.clear_registers(&args[1..]),
            "!registers" => self.registers(&args[1..]),
            "!symbols" => self.symbols(&args[1..]),
            "!load_file" => self.load_file(&args[1..]),
            "!spawn" => self.spawn(&args[1..]),
            _ => { println!("Invalid command!") }
        };
    }

    fn quit(&mut self, _args: &[&str]) {
        println!("Farewell!");
        std::process::exit(0);
    }

    fn history(&mut self, _args: &[&str]) {
        let mut results = vec![];
        for command in &self.command_buffer {
            results.push(command.clone());
        }
    }

    fn program(&mut self, _args: &[&str]) {
        let mut results = vec![];
        for instruction in &self.vm.program {
            results.push(instruction.clone())
        }
        println!("End of Program Listing");
    }

    fn clear_program(&mut self, _args: &[&str]) {
        self.vm.program.clear();
    }

    fn clear_registers(&mut self, _args: &[&str]) {
        println!("Setting all registers to 0");
        for i in 0..self.vm.registers.len() {
            self.vm.registers[i] = 0;
        }
        println!("Done!");
    }

    fn registers(&self, _args: &[&str]) {
        println!("Listing registers and all contents:");
        println!("{:#?}", self.vm.registers);
        println!("End of Register Listing")
    }


    fn symbols(&self, _args: &[&str]) {
        println!("Listing symbols table:");
        println!("{:#?}", self.asm.symbols);
        println!("End of Symbols Listing");
    }

    fn load_file(&mut self, _args: &[&str]) {
        let contents = self.get_data_from_load();
        if let Some(contents) = contents {
            match self.asm.assemble(&contents) {
                Ok(mut program) => {
                    println!("Sending assembled program to VM");
                    self.vm.program.append(&mut program);
                    println!("{:#?}", self.vm.program);
                    self.vm.run();
                },
                Err(errs) => {
                    for err in errs {
                        println!("Unable to parse input: {}", err);
                    }
                    return;
                }
            }
        } else {
            return;
        }
    }

    fn spawn(&mut self, _args: &[&str]) {
        let contents = self.get_data_from_load();
        println!("Loaded contents: {:#?}", contents);
        if let Some(contents) = contents {
            match self.asm.assemble(&contents) {
                Ok(mut program) => {
                    println!("Sending assembled program to VM");
                    self.vm.program.append(&mut program);
                    println!("{:#?}", self.vm.program);
                    self.scheduler.get_thread(self.vm.clone());
                },
                Err(errs) => {
                    for err in errs {
                        println!("Unable to parse input: {}", err);
                    }
                    return;
                }
            }
        } else {
            return;
        }
    }

    fn get_data_from_load(&mut self) -> Option<String> {
        let stdin = io::stdin();
        print!("Please enter the path to the file you wish to load: ");
        io::stdout().flush().expect("Unable to flush stdout");

        let mut tmp = String::new();
        stdin
            .read_line(&mut tmp)
            .expect("Unable to read line from user");
        println!("Attempting to load program from file...");

        let tmp = tmp.trim();
        let filename = Path::new(&tmp);
        let mut f = match File::open(&filename) {
            Ok(f) => f,
            Err(e) => {
                println!("There was an error opening that file: {:?}", e);
                return None;
            }
        };
        let mut contents = String::new();
        match f.read_to_string(&mut contents) {
            Ok(_bytes_read) => Some(contents),
            Err(e) => {
                println!("there was an error reading that file: {:?}", e);
                None
            }
        }
    }
}


