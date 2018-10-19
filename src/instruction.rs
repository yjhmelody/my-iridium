use nom::types::CompleteStr;

/// VM's Opcode
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
    INC,
    /// Decrease one
    DEC,

    DJMPE,
    PRTS,
    /// Illegal opcode
    IGL,

    /// Float Operation
    LOADF64,
    ADDF64,
    SUBF64,
    MULF64,
    DIVF64,
    EQF64,
    NEQF64,
    GTF64,
    GTEF64,
    LTF64,
    LTEF64,
}


impl From<Opcode> for u8 {
    fn from(op: Opcode) -> Self {
        use self::Opcode::*;
        match op {
            LOAD => 0,

            ADD => 1,
            SUB => 2,
            MUL => 3,
            DIV => 4,

            HLT => 5,

            JMP => 6,
            JMPF => 7,
            JMPB => 8,

            EQ => 9,
            NEQ => 10,
            GT => 11,
            LT => 12,
            GTE => 13,
            LTE => 14,
            JMPE => 15,

            NOP => 16,
            ALOC => 17,

            INC => 18,
            DEC => 19,

            DJMPE => 20,
            PRTS => 21,

            LOADF64 => 22,
            ADDF64 => 23,
            SUBF64 => 24,
            MULF64 => 25,
            DIVF64 => 26,
            EQF64 => 27,
            NEQF64 => 28,
            GTF64 => 29,
            GTEF64 => 30,
            LTF64 => 31,
            LTEF64 => 32,

            IGL => 100,
        }
    }
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

            18 => INC,
            19 => DEC,

            20 => DJMPE,
            21 => PRTS,

            22 => LOADF64,
            23 => ADDF64,
            24 => SUBF64,
            25 => MULF64,
            26 => DIVF64,
            27 => EQF64,
            28 => NEQF64,
            29 => GTF64,
            30 => GTEF64,
            31 => LTF64,
            32 => LTEF64,

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
            CompleteStr("inc") => Opcode::INC,
            CompleteStr("dec") => Opcode::DEC,
            CompleteStr("djmpe") => Opcode::DJMPE,
            CompleteStr("prts") => Opcode::PRTS,
            CompleteStr("loadf64") => Opcode::LOADF64,
            CompleteStr("addf64") => Opcode::ADDF64,
            CompleteStr("subf64") => Opcode::SUBF64,
            CompleteStr("mulf64") => Opcode::MULF64,
            CompleteStr("divf64") => Opcode::DIVF64,
            CompleteStr("eqf64") => Opcode::EQF64,
            CompleteStr("neqf64") => Opcode::NEQF64,
            CompleteStr("gtf64") => Opcode::GTF64,
            CompleteStr("gtef64") => Opcode::GTEF64,
            CompleteStr("ltf64") => Opcode::LTF64,
            CompleteStr("ltef64") => Opcode::LTEF64,
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