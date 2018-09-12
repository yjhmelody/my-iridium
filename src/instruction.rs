#[derive(Debug, PartialEq)]
pub enum Opcode {
    /// load opcode
    LOAD,
    /// halt the vm
    HLT,
    /// ADD src1 src2 dst
    ADD,
    SUB,
    MUL,
    DIV,
    /// illegal opcode
    IGL,

}

/// convert u8 to an opcode
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
            _ => IGL,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Instruction {
    opcode: Opcode,
}

impl Instruction {
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