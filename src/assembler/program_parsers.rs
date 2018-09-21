use assembler::directive_parsers::parse_directive;
use assembler::instruction_parsers::*;
use nom::types::CompleteStr;

/// Stores a assemble program
#[derive(Debug, PartialEq)]
pub struct Program {
    pub instructions: Vec<AssemblerInstruction>
}

impl Program {
    /// Translates instruction into bytes for eval.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut program = vec![];
        for instr in &self.instructions {
            program.append(&mut instr.to_bytes());
        }

        program
    }
}

/// parse the program to a vector of instructions
named!(pub parse_program<CompleteStr, Program>,
    do_parse!(
        instructions: many1!(alt!(parse_instruction | parse_directive)) >>
        (
            Program {
                instructions: instructions
            }
        )
    )
);


mod tests {
    use super::*;

    #[test]
    fn test_parse_program() {
        let result = parse_program(CompleteStr("load $0 #100\n"));
        assert_eq!(result.is_ok(), true);
        let (leftover, p) = result.unwrap();
        assert_eq!(leftover, CompleteStr(""));
        assert_eq!(1, p.instructions.len());
    }

    #[test]
    fn test_program_to_bytes() {
        let result = parse_program(CompleteStr("load $0 #100\n"));
        assert_eq!(result.is_ok(), true);
        let (_, p) = result.unwrap();
        let bytecode = p.to_bytes();
        assert_eq!(bytecode.len(), 4);
        println!("{:?}", bytecode);
    }
}