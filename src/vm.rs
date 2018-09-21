use assembler::*;
use instruction::*;
use std;

/// Virtual machine struct that will execute bytecode
pub struct VM {
    /// 0-31 registers
    pub registers: [i32; 32],
    /// Program counter
    pc: usize,
    /// Saves the bytecode
    pub program: Vec<u8>,
    /// Used for heap memory
    heap: Vec<u8>,
    /// Contains the remainder of division
    remainder: u32,
    /// Contains the result of the last comparison operation
    equal_flag: bool,
    /// Contains the read-only section data
    ro_data: Vec<u8>,
}

impl VM {
    /// Creates a VM
    pub fn new() -> Self {
        Self {
            registers: [0; 32],
            pc: 0,
            program: Vec::new(),
            heap: Vec::new(),
            remainder: 0,
            equal_flag: false,
            ro_data: Vec::new(),
        }
    }

    /// Runs the VM with loop
    pub fn run(&mut self) {
        // need to verify pie header
        if !self.verify_header() {
            println!("Header was incorrect");
            std::process::exit(1);
        }
        self.pc = 65;
        let mut is_done = false;
        while !is_done {
            is_done = self.excute_instruction();
        }
    }

    /// Runs the VM only one cycle
    pub fn run_once(&mut self) {
        self.excute_instruction();
    }

    /// Executes an instruction and returns a bool
    fn excute_instruction(&mut self) -> bool {
        if self.pc >= self.program.len() {
            return true;
        }

        match self.decode_opcode() {
            Opcode::HLT => {
                println!("HLT encountered");
                return true;
            }

            Opcode::LOAD => {
                let register = self.next_8_bits() as usize;
                let number = self.next_16_bits() as u16;
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
                self.remainder = (register1 % register2) as u32;
            }

            Opcode::JMP => {
                let target = self.registers[self.next_8_bits() as usize];
                self.pc = target as usize;
            }

            Opcode::JMPF => {
                let value = self.registers[self.next_8_bits() as usize];
                self.pc += value as usize;
            }

            Opcode::JMPB => {
                let value = self.registers[self.next_8_bits() as usize];
                self.pc -= value as usize;
            }

            Opcode::EQ => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 == register2;
                self.next_8_bits();
            }

            Opcode::NEQ => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 != register2;
                self.next_8_bits();
            }

            Opcode::GT => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 > register2;
                self.next_8_bits();
            }

            Opcode::LT => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 < register2;
                self.next_8_bits();
            }

            Opcode::GTE => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 >= register2;
                self.next_8_bits();
            }

            Opcode::LTE => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 <= register2;
                self.next_8_bits();
            }

            Opcode::JMPE => {
                if self.equal_flag {
                    let register = self.next_8_bits() as usize;
                    let target = self.registers[register] as usize;
                    self.pc = target;
                }
            }

            Opcode::ALOC => {
                let register = self.next_8_bits() as usize;
                let bytes = self.registers[register];
                let new_end = self.heap.len() as i32 + bytes;
                self.heap.resize(new_end as usize, 0);
            }

            Opcode::Inc => {
                let register = self.next_8_bits() as usize;
                self.registers[register] += 1;
                self.next_8_bits();
                self.next_8_bits();
            }

            Opcode::Dec => {
                let register = self.next_8_bits() as usize;
                self.registers[register] -= 1;
                self.next_8_bits();
                self.next_8_bits();
            }

            Opcode::DJMPE => {
                let destination = self.next_16_bits();
                if self.equal_flag {
                    self.pc = destination as usize;
                } else {
                    self.next_8_bits();
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

            _ => {
                println!("Unrecognized opcode found! Terminating!");
                return true;
            }
        }
        false
    }

    /// Decodes the byte the VM's program counter is pointing at into an opcode
    fn decode_opcode(&mut self) -> Opcode {
        let opcode = Opcode::from(self.program[self.pc]);
        self.pc += 1;
        opcode
    }

    /// Decodes the next byte into an opcode
    fn next_8_bits(&mut self) -> u8 {
        let res = self.program[self.pc];
        self.pc += 1;
        res
    }

    /// Decodes the next 2 bytes into an opcode
    fn next_16_bits(&mut self) -> u16 {
        let res = ((self.program[self.pc] as u16) << 8) | (self.program[self.pc + 1] as u16);
        self.pc += 2;
        res
    }

    /// Adds an arbitrary byte to the VM's program
    pub fn add_byte(&mut self, b: u8) {
        self.program.push(b);
    }

    /// Adds some bytes to the VM's program
    pub fn add_bytes(&mut self, mut bytes: Vec<u8>) {
        self.program.append(&mut bytes);
    }

    fn verify_header(&self) -> bool {
        println!("{:?}", self.program);
        if self.program[0..4] != PIE_HEADER_PREFIX {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]

    use super::*;

    fn get_test_vm() -> VM {
        let mut test_vm = VM::new();
        test_vm.registers[0] = 5;
        test_vm.registers[1] = 10;
        test_vm
    }

    fn prepend_header(mut b: Vec<u8>) -> Vec<u8> {
        let mut prepension = vec![];
        for byte in PIE_HEADER_PREFIX.into_iter() {
            prepension.push(byte.clone());
        }
        while prepension.len() <= PIE_HEADER_LENGTH {
            prepension.push(0);
        }
        prepension.append(&mut b);
        prepension
    }

    #[test]
    fn create_vm() {
        let vm = VM::new();
        assert_eq!(vm.registers[0], 0);
        assert_eq!(vm.pc, 0);
        assert_eq!(vm.program, Vec::new());
    }

    #[test]
    fn run_vm() {
        let mut vm = get_test_vm();
        vm.program = vec![1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0];
        vm.program = prepend_header(vm.program);
        vm.run();
        assert_eq!(vm.registers[0], 5 + 10 * 3);
    }

    #[test]
    fn run_once_vm() {
        let mut vm = get_test_vm();
        vm.program = vec![1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0];
        vm.run_once();
        assert_eq!(vm.registers[0], 5 + 10 * 1);
    }

    #[test]
    fn opcode_igl() {
        let mut vm = VM::new();
        vm.program = vec![254, 0, 0, 0];
        vm.run_once();
        assert_eq!(vm.pc, 1);
    }

    #[test]
    fn opcode_load() {
        let mut vm = get_test_vm();
        // represent 500 using two u8s in little endian format
        vm.program = vec![0, 0, 1, 244];
        vm.program = prepend_header(vm.program);
        vm.run();
        assert_eq!(vm.registers[0], 500);
    }

    #[test]
    fn opcode_add() {
        let mut vm = get_test_vm();
        vm.program = vec![1, 0, 1, 2];
        vm.program = prepend_header(vm.program);
        vm.run();
        assert_eq!(vm.registers[2], 15);
    }

    #[test]
    fn opcode_sub() {
        let mut vm = get_test_vm();
        vm.program = vec![2, 1, 0, 2];
        vm.program = prepend_header(vm.program);
        vm.run();
        assert_eq!(vm.registers[2], 5);
    }

    #[test]
    fn opcode_mul() {
        let mut vm = get_test_vm();
        vm.program = vec![3, 0, 1, 2];
        vm.program = prepend_header(vm.program);
        vm.run();
        assert_eq!(vm.registers[2], 50);
    }

    #[test]
    fn opcode_div() {
        let mut vm = get_test_vm();
        vm.program = vec![4, 1, 0, 2];
        vm.program = prepend_header(vm.program);
        vm.run();
        assert_eq!(vm.registers[2], 2);
        assert_eq!(vm.remainder, 0);
    }

    #[test]
    fn opcode_hlt() {
        let mut vm = VM::new();
        vm.program = vec![5, 0, 0, 0];
        vm.run_once();
        assert_eq!(vm.pc, 1);
    }

    #[test]
    fn opcode_jmp() {
        let mut vm = get_test_vm();
        vm.registers[0] = 1;
        vm.program = vec![6, 0, 0, 0, 0, 0];
        vm.run_once();
        assert_eq!(vm.pc, 1);
    }

    #[test]
    fn opcode_jmpf() {
        let mut vm = get_test_vm();
        vm.registers[0] = 2;
        vm.program = vec![7, 0, 0, 0, 0, 0];
        vm.run_once();
        assert_eq!(vm.pc, 4);
    }

    #[test]
    fn opcode_jmpb() {
        let mut vm = get_test_vm();
        vm.registers[0] = 1;
        vm.program = vec![8, 0, 0, 0, 0, 0];
        vm.run_once();
        assert_eq!(vm.pc, 1);
    }

    #[test]
    fn opcode_eq() {
        let mut vm = get_test_vm();
        vm.program = vec![9, 0, 1, 0];
        vm.run_once();
        assert_eq!(vm.equal_flag, false);
    }

    #[test]
    fn opcode_neq() {
        let mut vm = get_test_vm();
        vm.program = vec![10, 0, 1, 0];
        vm.run_once();
        assert_eq!(vm.equal_flag, true);
    }

    #[test]
    fn opcode_gt() {
        let mut vm = get_test_vm();
        vm.program = vec![11, 0, 1, 0];
        vm.run_once();
        assert_eq!(vm.equal_flag, false);
    }


    #[test]
    fn opcode_lt() {
        let mut vm = get_test_vm();
        vm.program = vec![12, 0, 1, 0];
        vm.run_once();
        assert_eq!(vm.equal_flag, true);
    }

    #[test]
    fn opcode_gte() {
        let mut vm = get_test_vm();
        vm.registers[0] = 20;
        vm.registers[1] = 10;
        vm.program = vec![13, 0, 1, 0, 13, 0, 1, 0, 13, 0, 1, 0];
        vm.run_once();
        assert_eq!(vm.equal_flag, true);
        vm.registers[0] = 10;
        vm.run_once();
        assert_eq!(vm.equal_flag, true);
        vm.registers[0] = 5;
        vm.run_once();
        assert_eq!(vm.equal_flag, false);
    }

    #[test]
    fn opcode_lte() {
        let mut vm = get_test_vm();
        vm.registers[0] = 20;
        vm.registers[1] = 10;
        vm.program = vec![14, 0, 1, 0, 14, 0, 1, 0, 14, 0, 1, 0];
        vm.run_once();
        assert_eq!(vm.equal_flag, false);
        vm.registers[0] = 10;
        vm.run_once();
        assert_eq!(vm.equal_flag, true);
        vm.registers[0] = 5;
        vm.run_once();
        assert_eq!(vm.equal_flag, true);
    }

    #[test]
    fn opcode_jmpe() {
        let mut vm = get_test_vm();
        vm.registers[0] = 7;
        vm.equal_flag = true;
        vm.program = vec![15, 0, 0, 0, 15, 0, 0, 0, 15, 0, 0, 0];
        vm.run_once();
        assert_eq!(vm.pc, 7);
    }

    #[test]
    fn opcode_aloc() {
        let mut vm = get_test_vm();
        vm.registers[0] = 1024;
        vm.program = vec![17, 0, 0, 0];
        vm.run_once();
        assert_eq!(vm.heap.len(), 1024);
    }

    #[test]
    fn opcode_inc() {
        let mut vm = get_test_vm();
        vm.registers[0] = 1;
        vm.program = vec![18, 0, 0, 0];
        vm.run_once();
        assert_eq!(vm.registers[0], 2);
    }

    #[test]
    fn opcode_dec() {
        let mut vm = get_test_vm();
        vm.registers[0] = 2;
        vm.program = vec![19, 0, 0, 0];
        vm.run_once();
        assert_eq!(vm.registers[0], 1);
    }

    #[test]
    fn prts_opcode() {
        let mut vm = get_test_vm();
        vm.ro_data.append(&mut vec![72, 101, 108, 108, 111, 0]);
        vm.program = vec![21, 0, 0, 0];
        vm.run_once();
        // TODO: How can we validate the output since it is just printing to stdout in a test?
    }
}