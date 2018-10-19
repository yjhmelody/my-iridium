use assembler::Assembler;
use assembler::program_parsers::parse_program;
use nom::types::CompleteStr;
use repl::command_parser::CommandParser;
use scheduler::Scheduler;
use std;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::Write;
use std::num::ParseIntError;
use std::path::Path;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;
use std::u8;
use vm::VM;

pub mod command_parser;

pub static REMOTE_BANNER: &'static str = "Welcome to Iridium! Let's be productive!";
pub static PROMPT: &'static str = ">>> ";
const COMMAND_PREFIX: char = '!';

/// Core structure for the REPL for the Assembler
pub struct REPL {
    command_buffer: Vec<String>,
    vm: VM,
    asm: Assembler,
    scheduler: Scheduler,
    pub tx_pipe: Option<Box<Sender<String>>>,
    pub rx_pipe: Option<Box<Receiver<String>>>,
}

impl Default for REPL {
    fn default() -> Self {
        Self::new()
    }
}

impl REPL {
    /// Creates a REPL
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            command_buffer: Vec::new(),
            vm: VM::new(),
            asm: Assembler::new(),
            scheduler: Scheduler::new(),
            tx_pipe: Some(Box::new(tx)),
            rx_pipe: Some(Box::new(rx)),
        }
    }

    /// Runs the repl
    pub fn run(&mut self) {
        self.send_message(REMOTE_BANNER.to_string());
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
                self.execute_command(&buffer);
            } else {
                let program = match parse_program(buffer.into()) {
                    Ok((_, program)) => program,
                    Err(e) => {
                        self.send_message(format!("Unable to parse input: {:?}", e));
                        self.send_prompt();
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

    /// Sends single cmd to remote
    pub fn run_single(&mut self, buf: &str) -> Option<String> {
        if buf.starts_with(COMMAND_PREFIX) {
            self.execute_command(&buf);
            None
        } else {
            let program = match parse_program(CompleteStr(buf)) {
                Ok((_remainder, program)) => Some(program),
                Err(err) => {
                    self.send_message(format!("Unable to parse input: {:?}", err));
                    self.send_prompt();
                    None
                }
            };

            program.map(|p| {
                let mut bytes = p.to_bytes(&self.asm.symbols);
                self.vm.program.append(&mut bytes);
                self.vm.run_once();
                p
            });
            None
        }
    }

    /// Sends message to remote
    pub fn send_message(&mut self, msg: String) {
        match &self.tx_pipe {
            Some(pipe) => {
                match pipe.send(msg) {
                    Ok(_) => {}
                    Err(_e) => {}
                };
            }
            None => {}
        }
    }

    /// Sends prompt to remote
    pub fn send_prompt(&mut self) {
        match &self.tx_pipe {
            Some(pipe) => match pipe.send(PROMPT.to_owned()) {
                Ok(_) => {}
                Err(_e) => {}
            },
            None => {}
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

    /// Execute a command which starts with `!`
    fn execute_command(&mut self, input: &str) {
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
            _ => {
                self.send_message("Invalid command!".to_string());
                self.send_prompt();
            }
        };
    }

    fn quit(&mut self, _args: &[&str]) {
        self.send_message("Farewell! Have a great day!".to_string());
        std::process::exit(0);
    }

    fn history(&mut self, _args: &[&str]) {
        let mut results = vec![];
        for command in &self.command_buffer {
            results.push(command.clone());
        }
        self.send_message(format!("{:#?}", results));
        self.send_prompt();
    }

    fn program(&mut self, _args: &[&str]) {
        self.send_message("Listing instructions currently in VM's program vector: ".to_string());
        let mut results = vec![];
        for instruction in &self.vm.program {
            results.push(instruction.clone())
        }
        self.send_message(format!("{:#?}", results));
        self.send_message("End of Program Listing".to_string());
        self.send_prompt();
    }

    fn clear_program(&mut self, _args: &[&str]) {
        self.vm.program.clear();
    }

    fn clear_registers(&mut self, _args: &[&str]) {
        self.send_message("Setting all registers to 0".to_string());
        for i in 0..self.vm.registers.len() {
            self.vm.registers[i] = 0;
        }
        self.send_message("Done!".to_string());
        self.send_prompt();
    }

    fn registers(&mut self, _args: &[&str]) {
        self.send_message("Listing registers and all contents:".to_string());
        let mut results = vec![];
        for register in &self.vm.registers {
            results.push(register.clone());
        }
        self.send_message(format!("{:#?}", results));
        self.send_message("End of Register Listing".to_string());
        self.send_prompt();
    }

    fn symbols(&mut self, _args: &[&str]) {
//        todo: fix it
//        let mut results = vec![];
//        for symbol in &self.asm.symbols.symbols {
//            results.push(symbol.clone());
//        }
//        self.send_message("Listing symbols table:".to_string());
//        self.send_message(format!("{:#?}", results));
//        self.send_message("End of Symbols Listing".to_string());
//        self.send_prompt();
    }

    fn load_file(&mut self, _args: &[&str]) {
        let contents = self.get_data_from_load();
        if let Some(contents) = contents {
            match self.asm.assemble(&contents) {
                Ok(mut program) => {
                    self.send_message("Sending assembled program to VM".to_string());
                    self.vm.program.append(&mut program);
                    self.vm.run();
                },
                Err(errs) => {
                    for err in errs {
                        self.send_message(format!("Unable to parse input: {}", err));
                        self.send_prompt();
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
        self.send_message(format!("Loaded contents: {:#?}", contents));
        if let Some(contents) = contents {
            match self.asm.assemble(&contents) {
                Ok(mut program) => {
                    self.send_message("Sending assembled program to VM".to_string());
                    self.vm.program.append(&mut program);
                    self.scheduler.get_thread(self.vm.clone());
                },
                Err(errs) => {
                    for err in errs {
                        self.send_message(format!("Unable to parse input: {}", err));
                        self.send_prompt();
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
        self.send_message("Please enter the path to the file you wish to load: ".to_string());

        let mut tmp = String::new();
        stdin
            .read_line(&mut tmp)
            .expect("Unable to read line from user");
        self.send_message("Attempting to load program from file...".to_string());

        let filename = Path::new(tmp.trim());
        let mut f = match File::open(filename) {
            Ok(f) => f,
            Err(e) => {
                self.send_message(format!("There was an error opening that file: {:?}", e));
                return None;
            }
        };
        let mut contents = String::new();
        match f.read_to_string(&mut contents) {
            Ok(_bytes_read) => Some(contents),
            Err(e) => {
                self.send_message(format!("there was an error reading that file: {:?}", e));
                None
            }
        }
    }
}


