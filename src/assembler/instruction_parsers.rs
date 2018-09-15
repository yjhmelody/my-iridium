use assembler::opcode_parsers::*;
use assembler::operand_parsers::integer_operand;
use assembler::register_parsers::register;
use assembler::Token;

#[derive(Debug, PartialEq)]
pub struct AssemblerInstruction {
    opcocde: Token,
    operand1: Option<Token>,
    operand2: Option<Token>,
    operand3: Option<Token>,
}
