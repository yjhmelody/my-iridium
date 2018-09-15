#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Opcode {
    /// Load opcode
    LOAD,

    /// Add src1 src2 dst
    ADD,
    /// Sub src1 src2 dst
    SUB,
    /// Mul src1 src2 dst
    MUL,
    /// Div src1 src2 dst
    DIV,
    /// Absolute Jumps by using register
    JMP,
    /// Relative Jumps for jump forwards
    JMPF,
    /// Relative Jumps for jump backwards
    JMPB,

    /// Halt the vm
    HLT,

    /// If src1 == src2 then set true
    EQ,
    /// If src1 != src2 then set true
    NEQ,
    /// If src1 > src2 then set true
    GT,
    /// If src1 < src2 then set true
    LT,
    /// If src1 >= src2 then set true
    GTE,
    /// If src1 <= src2 then set true
    LTE,
    /// If equaly_bool(A special register for storing last equality result) == true then jmp
    JMPE,

    /// Illegal opcode
    IGL,
}

/// Converts from a u8 to an Opcode
impl From<u8> for Opcode {
    fn from(v: u8) -> Self {
        use self::Opcode::*;
        match v {
            0 => LOAD,

            1 => ADD,
            2 => SUB,
            3 => MUL,
            4 => DIV,

            5 => HLT,

            6 => JMP,
            7 => JMPF,
            8 => JMPB,

            9 => EQ,
            10 => NEQ,
            11 => GT,
            12 => LT,
            13 => GTE,
            14 => LTE,
            15 => JMPE,

            _ => IGL,
        }
    }
}

/// Represents a combination of an opcode and operands for the VM to execute
#[derive(Debug, PartialEq)]
pub struct Instruction {
    opcode: Opcode,
}

impl Instruction {
    /// Creates a new Instruction
    pub fn new (opcode: Opcode) -> Self {
        Self {opcode}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_hlt() {
        let opcode = Opcode::HLT;
        assert_eq!(opcode, Opcode::HLT);
    }

    #[test]
    fn create_instruction() {
        let instr = Instruction::new(Opcode::HLT);
        assert_eq!(instr.opcode, Opcode::HLT);
    }
}