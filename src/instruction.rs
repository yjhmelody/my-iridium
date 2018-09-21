use nom::types::CompleteStr;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Opcode {
    /// Load data to register
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

    /// No operate
    NOP,
    /// For memory
    ALOC,

    /// Increase one
    Inc,
    /// Decrease one
    Dec,

    DJMPE,
    PRTS,
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

            16 => NOP,
            17 => ALOC,

            18 => Inc,
            19 => Dec,

            20 => DJMPE,
            21 => PRTS,
            _ => IGL,
        }
    }
}

impl<'a> From<CompleteStr<'a>> for Opcode {
    fn from(v: CompleteStr<'a>) -> Self {
        match v {
            CompleteStr("load") => Opcode::LOAD,
            CompleteStr("add") => Opcode::ADD,
            CompleteStr("sub") => Opcode::SUB,
            CompleteStr("mul") => Opcode::MUL,
            CompleteStr("div") => Opcode::DIV,
            CompleteStr("hlt") => Opcode::HLT,
            CompleteStr("jmp") => Opcode::JMP,
            CompleteStr("jmpf") => Opcode::JMPF,
            CompleteStr("jmpb") => Opcode::JMPB,
            CompleteStr("eq") => Opcode::EQ,
            CompleteStr("neq") => Opcode::NEQ,
            CompleteStr("gte") => Opcode::GTE,
            CompleteStr("gt") => Opcode::GT,
            CompleteStr("lte") => Opcode::LTE,
            CompleteStr("lt") => Opcode::LT,
            CompleteStr("jmpe") => Opcode::JMPE,
            CompleteStr("nop") => Opcode::NOP,
            CompleteStr("aloc") => Opcode::ALOC,
            CompleteStr("inc") => Opcode::Inc,
            CompleteStr("dec") => Opcode::Dec,
            CompleteStr("djmpe") => Opcode::DJMPE,
            CompleteStr("prts") => Opcode::PRTS,
            _ => Opcode::IGL,
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
    #![allow(unused_imports)]

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

    #[test]
    fn str_to_opcode() {
        let opcode = Opcode::from(CompleteStr("load"));
        assert_eq!(opcode, Opcode::LOAD);
        let opcode = Opcode::from(CompleteStr("illegal"));
        assert_eq!(opcode, Opcode::IGL);
    }
}