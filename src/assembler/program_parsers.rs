use assembler::directive_parsers::parse_directive;
use assembler::instruction_parsers::*;
use assembler::symbols::SymbolTable;
use nom::types::CompleteStr;


/// Stores a assemble program
#[derive(Debug, PartialEq)]
pub struct Program {
    pub instructions: Vec<AssemblerInstruction>
}

impl Program {
    /// Translates instruction into bytes for eval.
    pub fn to_bytes(&self, symbols: &SymbolTable) -> Vec<u8> {
        let mut program = vec![];
        for instr in &self.instructions {
            program.append(&mut instr.to_bytes(symbols));
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
    #![allow(unused_imports)]

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
        let symbols = SymbolTable::new();
        let bytecode = p.to_bytes(&symbols);
        assert_eq!(bytecode.len(), 4);
        println!("{:?}", bytecode);
    }

    #[test]
    fn test_complete_program() {
        let program = CompleteStr(r"
        .data
        hello: .asciiz 'Hello everyone!'
        .code
        hlt");
        let result = parse_program(program);
        assert_eq!(result.is_ok(), true);
        let (rest, _) = result.unwrap();
        assert_eq!(rest, CompleteStr(""));
    }
}