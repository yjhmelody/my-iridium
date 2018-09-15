use instruction::*;

/// Virtual machine struct that will execute bytecode
pub struct VM {
    /// 0-31 registers
    pub registers: [i32; 32],
    /// Program counter
    pc: usize,
    /// Saves the bytecode
    pub program: Vec<u8>,
    /// Contains the remainder of division
    remainder: u32,
    /// Contains the result of the last comparison operation
    equal_flag: bool,
}

impl VM {
    /// Creates a VM
    pub fn new() -> Self {
        Self {
            registers: [0; 32],
            pc: 0,
            program: Vec::new(),
            remainder: 0,
            equal_flag: false,
        }
    }

    /// Runs the VM with loop
    pub fn run(&mut self) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_vm() -> VM {
        let mut test_vm = VM::new();
        test_vm.registers[0] = 5;
        test_vm.registers[1] = 10;
        test_vm
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
        vm.run();
        assert_eq!(vm.registers[0], 500);
    }

    #[test]
    fn opcode_add() {
        let mut vm = get_test_vm();
        vm.program = vec![1, 0, 1, 2];
        vm.run();
        assert_eq!(vm.registers[2], 15);
    }

    #[test]
    fn opcode_sub() {
        let mut vm = get_test_vm();
        vm.program = vec![2, 1, 0, 2];
        vm.run();
        assert_eq!(vm.registers[2], 5);
    }

    #[test]
    fn opcode_mul() {
        let mut vm = get_test_vm();
        vm.program = vec![3, 0, 1, 2];
        vm.run();
        assert_eq!(vm.registers[2], 50);
    }

    #[test]
    fn opcode_div() {
        let mut vm = get_test_vm();
        vm.program = vec![4, 1, 0, 2];
        vm.run();
        assert_eq!(vm.registers[2], 2);
        assert_eq!(vm.remainder, 0);
    }

    #[test]
    fn opcode_hlt() {
        let mut vm = VM::new();
        vm.program = vec![5, 0, 0, 0];
        vm.run();
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
}