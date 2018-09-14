mod opcode;
mod opcode_parser;
use instruction::Opcode;

#[derive(Debug, PartialEq)]
pub enum Token {
    Op{code: Opcode},
}

