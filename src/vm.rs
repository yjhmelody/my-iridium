use assembler::PIE_HEADER_PREFIX;
use byteorder::{LittleEndian, ReadBytesExt};
use chrono::prelude::*;
use instruction::Opcode;
use num_cpus;
use std;
use std::f64;
use std::io::Cursor;
use uuid::Uuid;

/// Virtual machine struct that will execute bytecode
#[derive(Default, Clone)]
pub struct VM {
    /// Array that simulates having hardware registers
    pub registers: [i32; 32],
    pub float_registers: [f64; 32],
    /// Program counter that tracks which byte is being executed
    pc: usize,
    /// The bytecode of the program being run
    pub program: Vec<u8>,
    /// Used for heap memory
    heap: Vec<u8>,
    /// Contains the remainder of modulo division ops
    remainder: usize,
    /// Contains the result of the last comparison operation
    equal_flag: bool,
    /// Contains the read-only section data
    ro_data: Vec<u8>,
    /// Is a unique, randomly generated UUID for identifying this VM
    id: Uuid,
    /// VMEvent recivie events
    events: Vec<VMEvent>,
    /// The core number of CPU
    pub logical_cores: usize,
}

#[derive(Debug, Clone)]
pub enum VMEventType {
    Start,
    GracefulStop {code: u32},
    Crash {code: u32},
}
#[derive(Debug, Clone)]

pub struct VMEvent {
    event: VMEventType,
    at: DateTime<Utc>,
    application_id: Uuid,
}


impl VM {
    /// Creates and returns a new VM
    pub fn new() -> VM {
        VM {
            registers: [0; 32],
            float_registers: [0.0; 32],
            program: vec![],
            ro_data: vec![],
            heap: vec![],
            pc: 0,
            remainder: 0,
            equal_flag: false,
            id: Uuid::new_v4(),
            events: Vec::new(),
            logical_cores: num_cpus::get(),
        }
    }

    /// Wraps execution in a loop so it will continue to run until done or there is an error
    /// executing instructions.
    pub fn run(&mut self) -> Vec<VMEvent> {
        self.events.push(VMEvent{
            event: VMEventType::Start,
            at: Utc::now(),
            application_id: self.id.clone(),
        });
        
        // TODO: Should setup custom errors here
        if !self.verify_header() {
            self.events.push(VMEvent{
                event: VMEventType::Crash{code:1},
                at: Utc::now(),
                application_id: self.id.clone(),
            });

            println!("Header was incorrect");
            return self.events.clone();
        }
        // If the header is valid, we need to change the PC to be at bit 65.
        self.pc = 64 + self.get_starting_offset();
        let mut is_done = None;
        while is_done.is_none() {
            is_done = self.execute_instruction();
        }

        self.events.push(VMEvent {
            event: VMEventType::GracefulStop{code: is_done.unwrap()},
            at: Utc::now(),
            application_id: self.id.clone(),
        });

        self.events.clone()
    }

    /// Executes one instruction. Meant to allow for more controlled execution of the VM
    pub fn run_once(&mut self) {
        self.execute_instruction();
    }

    /// Adds an arbitrary byte to the VM's program
    pub fn add_byte(&mut self, b: u8) {
        self.program.push(b);
    }

    /// Adds an arbitrary byte to the VM's program
    pub fn add_bytes(&mut self, mut b: Vec<u8>) {
        self.program.append(&mut b);
    }

    /// Executes an instruction and returns a bool. Meant to be called by the various public run
    /// functions.
    fn execute_instruction(&mut self) -> Option<u32> {
        if self.pc >= self.program.len() {
            return Some(1);
        }
        match self.decode_opcode() {
            Opcode::LOAD => {
                let register = self.next_8_bits() as usize;
                let number = u32::from(self.next_16_bits());
                self.registers[register] = number as i32;
            }
            Opcode::ADD => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 + register2;
            }
            Opcode::SUB => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 - register2;
            }
            Opcode::MUL => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 * register2;
            }
            Opcode::DIV => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 / register2;
                self.remainder = (register1 % register2) as usize;
            }
            Opcode::HLT => {
                println!("HLT encountered");
                return Some(0);
            }
            Opcode::JMP => {
                let target = self.registers[self.next_8_bits() as usize];
                self.pc = target as usize;
            }
            Opcode::JMPF => {
                let value = self.registers[self.next_8_bits() as usize] as usize;
                self.pc += value;
            }
            Opcode::JMPB => {
                let value = self.registers[self.next_8_bits() as usize] as usize;
                self.pc -= value;
            }
            Opcode::EQ => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 == register2;
                self.pc += 1;
            }
            Opcode::NEQ => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 != register2;
                self.pc += 1;
            }
            Opcode::GT => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 > register2;
                self.pc += 1;
            }
            Opcode::GTE => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 >= register2;
                self.pc += 1;
            }
            Opcode::LT => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 < register2;
                self.pc += 1;
            }
            Opcode::LTE => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 <= register2;
                self.pc += 1;
            }
            Opcode::JMPE => {
                if self.equal_flag {
                    let register = self.next_8_bits() as usize;
                    let target = self.registers[register];
                    self.pc = target as usize;
                } else {
                    // TODO: Fix the bits
                }
            }
            Opcode::NOP => {
                self.pc += 3;
            }
            Opcode::ALOC => {
                let register = self.next_8_bits() as usize;
                let bytes = self.registers[register];
                let new_end = self.heap.len() as i32 + bytes;
                self.heap.resize(new_end as usize, 0);
            }
            Opcode::INC => {
                let register_number = self.next_8_bits() as usize;
                self.registers[register_number] += 1;
                self.pc += 2;
            }
            Opcode::DEC => {
                let register_number = self.next_8_bits() as usize;
                self.registers[register_number] -= 1;
                self.pc += 2;
            }
            Opcode::DJMPE => {
                let destination = self.next_16_bits();
                if self.equal_flag {
                    self.pc = destination as usize;
                } else {
                    self.pc += 1;
                }
            }
            Opcode::PRTS => {
                // PRTS takes one operand, either a starting index in the read-only section of the bytecode
                // or a symbol (in the form of @symbol_name), which will look up the offset in the symbol table.
                // This instruction then reads each byte and prints it, until it comes to a 0x00 byte, which indicates
                // termination of the string
                let starting_offset = self.next_16_bits() as usize;
                let mut ending_offset = starting_offset;
                let slice = self.ro_data.as_slice();
                // TODO: Find a better way to do this. Maybe we can store the byte length and not null terminate? Or some form of caching where we
                // go through the entire ro_data on VM startup and find every string and its ending byte location?
                while slice[ending_offset] != 0 {
                    ending_offset += 1;
                }
                let result = std::str::from_utf8(&slice[starting_offset..ending_offset]);
                match result {
                    Ok(s) => { print!("{}", s); }
                    Err(e) => { println!("Error decoding string for prts instruction: {:#?}", e) }
                };
            }
            Opcode::LOADF64 => {
                let register = self.next_8_bits() as usize;
                let number = f64::from(self.next_16_bits());
                self.float_registers[register] = number;
            }
            Opcode::ADDF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.float_registers[self.next_8_bits() as usize] = register1 + register2;
            }
            Opcode::SUBF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.float_registers[self.next_8_bits() as usize] = register1 - register2;
            }
            Opcode::MULF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.float_registers[self.next_8_bits() as usize] = register1 * register2;
            }
            Opcode::DIVF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.float_registers[self.next_8_bits() as usize] = register1 / register2;
            }
            Opcode::EQF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.equal_flag = (register1 - register2).abs() < f64::EPSILON;
                self.next_8_bits();
            }
            Opcode::NEQF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.equal_flag = (register1 - register2).abs() > f64::EPSILON;
                self.next_8_bits();
            }
            Opcode::GTF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.equal_flag = register1 > register2;
                self.next_8_bits();
            }
            Opcode::GTEF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.equal_flag = register1 >= register2;
                self.next_8_bits();
            }
            Opcode::LTF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.equal_flag = register1 < register2;
                self.next_8_bits();
            }
            Opcode::LTEF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.equal_flag = register1 <= register2;
                self.next_8_bits();
            }
            Opcode::IGL => {
                println!("Illegal instruction encountered");
                return Some(1);
            }
        };
        
        None
    }

    /// Attempts to decode the byte the VM's program counter is pointing at into an opcode
    fn decode_opcode(&mut self) -> Opcode {
        let opcode = Opcode::from(self.program[self.pc]);
        self.pc += 1;
        opcode
    }

    /// Attempts to decode the next byte into an opcode
    fn next_8_bits(&mut self) -> u8 {
        let result = self.program[self.pc];
        self.pc += 1;
        result
    }

    /// Grabs the next 16 bits (2 bytes)
    fn next_16_bits(&mut self) -> u16 {
        let result = ((u16::from(self.program[self.pc])) << 8) | u16::from(self.program[self.pc + 1]);
        self.pc += 2;
        result
    }

    /// Processes the header of bytecode the VM is asked to execute
    fn verify_header(&self) -> bool {
        if self.program[0..4] != PIE_HEADER_PREFIX {
            return false;
        }
        true
    }

    // read 4 bytes after magical number
    fn get_starting_offset(&self) -> usize {
        let mut rdr = Cursor::new(&self.program[4..8]);
        rdr.read_u32::<LittleEndian>().unwrap() as usize
    }
}

#[cfg(test)]
mod tests {
    use assembler::PIE_HEADER_LENGTH;
    use super::*;

    fn get_test_vm() -> VM {
        let mut vm = VM::new();
        vm.registers[0] = 5;
        vm.registers[1] = 10;
        vm.float_registers[0] = 5.0;
        vm.float_registers[1] = 10.0;
        vm
    }

    fn prepend_header(mut b: Vec<u8>) -> Vec<u8> {
        let mut prepension = vec![];
        for byte in PIE_HEADER_PREFIX.into_iter() {
            prepension.push(byte.clone());
        }
        while prepension.len() < PIE_HEADER_LENGTH {
            prepension.push(0);
        }
        prepension.append(&mut b);
        prepension
    }

    #[test]
    fn test_create_vm() {
        let test_vm = VM::new();
        assert_eq!(test_vm.registers[0], 0)
    }

    #[test]
    fn test_opcode_hlt() {
        let mut test_vm = VM::new();
        let test_bytes = vec![5, 0, 0, 0];
        test_vm.program = test_bytes;
        test_vm.run_once();
        assert_eq!(test_vm.pc, 1);
    }

    #[test]
    fn test_opcode_igl() {
        let mut test_vm = VM::new();
        let test_bytes = vec![254, 0, 0, 0];
        test_vm.program = test_bytes;
        test_vm.run_once();
        assert_eq!(test_vm.pc, 1);
    }

    #[test]
    fn test_load_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![0, 0, 1, 244];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.registers[0], 500);
    }

    #[test]
    fn test_add_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![1, 0, 1, 2];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.registers[2], 15);
    }

    #[test]
    fn test_sub_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![2, 1, 0, 2];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.registers[2], 5);
    }

    #[test]
    fn test_mul_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![3, 0, 1, 2];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.registers[2], 50);
    }

    #[test]
    fn test_div_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![4, 1, 0, 2];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.registers[2], 2);
    }

    #[test]
    fn test_jmp_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 4;
        test_vm.program = vec![6, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.pc, 4);
    }

    #[test]
    fn test_jmpf_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 2;
        test_vm.program = vec![7, 0, 0, 0, 5, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.pc, 4);
    }

    #[test]
    fn test_jmpb_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[1] = 6;
        test_vm.program = vec![0, 0, 0, 10, 8, 1, 0, 0];
        test_vm.run_once();
        test_vm.run_once();
        assert_eq!(test_vm.pc, 0);
    }

    #[test]
    fn test_eq_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 10;
        test_vm.registers[1] = 10;
        test_vm.program = vec![9, 0, 1, 0, 9, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[1] = 20;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_neq_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 10;
        test_vm.registers[1] = 20;
        test_vm.program = vec![10, 0, 1, 0, 10, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[1] = 10;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }


    #[test]
    fn test_gt_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 20;
        test_vm.registers[1] = 10;
        test_vm.program = vec![11, 0, 1, 0, 11, 0, 1, 0, 11, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[0] = 10;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
        test_vm.registers[0] = 5;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_lt_opcode() {
        let mut vm = get_test_vm();
        vm.registers[0] = 20;
        vm.registers[1] = 10;
        vm.program = vec![12, 0, 1, 0, 12, 0, 1, 0, 12, 0, 1, 0];
        vm.run_once();
        assert_eq!(vm.equal_flag, false);
        vm.registers[0] = 10;
        vm.run_once();
        assert_eq!(vm.equal_flag, false);
        vm.registers[0] = 5;
        vm.run_once();
        assert_eq!(vm.equal_flag, true);
    }

    #[test]
    fn test_gte_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 20;
        test_vm.registers[1] = 10;
        test_vm.program = vec![13, 0, 1, 0, 13, 0, 1, 0, 13, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[0] = 10;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[0] = 5;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_lte_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 20;
        test_vm.registers[1] = 10;
        test_vm.program = vec![14, 0, 1, 0, 14, 0, 1, 0, 14, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
        test_vm.registers[0] = 10;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[0] = 5;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
    }

    #[test]
    fn test_jmpe_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 7;
        test_vm.equal_flag = true;
        test_vm.program = vec![15, 0, 0, 0, 15, 0, 0, 0, 15, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.pc, 7);
    }

    #[test]
    fn test_aloc_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 1024;
        test_vm.program = vec![17, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.heap.len(), 1024);
    }

    #[test]
    fn test_prts_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.ro_data.append(&mut vec![72, 101, 108, 108, 111, 0]);
        test_vm.program = vec![21, 0, 0, 0];
        test_vm.run_once();
        // TODO: How can we validate the output since it is just printing to stdout in a test?
    }

    #[test]
    fn test_load_floating_point_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![22, 0, 1, 244];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.float_registers[0], 500.0);
    }

    #[test]
    fn test_add_floating_point_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![23, 0, 1, 2];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.float_registers[2], 15.0);
    }

    #[test]
    fn test_sub_floating_point_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![24, 1, 0, 2];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.float_registers[2], 5.0);
    }

    #[test]
    fn test_mul_floating_point_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![25, 1, 0, 2];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.float_registers[2], 50.0);
    }

    #[test]
    fn test_div_floating_point_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![26, 1, 0, 2];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.float_registers[2], 2.0);
    }

    #[test]
    fn test_eq_floating_point_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.float_registers[0] = 10.0;
        test_vm.float_registers[1] = 10.0;
        test_vm.program = vec![27, 0, 1, 0, 27, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.float_registers[1] = 20.0;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_neq_floating_point_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.float_registers[0] = 10.0;
        test_vm.float_registers[1] = 20.0;
        test_vm.program = vec![28, 0, 1, 0, 28, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.float_registers[1] = 10.0;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_gt_floating_point_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.float_registers[0] = 20.0;
        test_vm.float_registers[1] = 10.0;
        test_vm.program = vec![29, 0, 1, 0, 29, 0, 1, 0, 29, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.float_registers[0] = 10.0;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
        test_vm.float_registers[0] = 5.0;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_gte_floating_point_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.float_registers[0] = 20.0;
        test_vm.float_registers[1] = 10.0;
        test_vm.program = vec![30, 0, 1, 0, 30, 0, 1, 0, 30, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.float_registers[0] = 10.0;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.float_registers[0] = 5.0;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_lt_floating_point_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.float_registers[0] = 20.0;
        test_vm.float_registers[1] = 10.0;
        test_vm.program = vec![31, 0, 1, 0, 31, 0, 1, 0, 31, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
        test_vm.float_registers[0] = 10.0;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
        test_vm.float_registers[0] = 5.0;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
    }

    #[test]
    fn test_lte_floating_point_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.float_registers[0] = 20.0;
        test_vm.float_registers[1] = 10.0;
        test_vm.program = vec![32, 0, 1, 0, 32, 0, 1, 0, 32, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
        test_vm.float_registers[0] = 10.0;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.float_registers[0] = 5.0;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
    }

}
