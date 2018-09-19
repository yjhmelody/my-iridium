use assembler::opcode_parsers::*;
use assembler::operand_parsers::integer_operand;
use assembler::register_parsers::register;
use assembler::Token;
use nom::types::CompleteStr;
use std::process;

/// Stores a line assemble instruction
#[derive(Debug, PartialEq)]
pub struct AssemblerInstruction {
    opcode: Option<Token>,
    operand1: Option<Token>,
    operand2: Option<Token>,
    operand3: Option<Token>,
}

impl AssemblerInstruction {
    /// Translates instruction into bytes for eval.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut results = vec![];
        // translate opcode
        if let Some(ref token) = self.opcode {
            match token {
                Token::Op { code } => match code {
                    _ => {
                        // impl From<u8> for Opcode, convert any opcode into its integer with code as u8
                        results.push(*code as u8);
                    }
                },
                _ => {
                    println!("Non-opcode found in opcode field");
                }
            }
        }

        // translate operands
        for operand in &[&self.operand1, &self.operand2, &self.operand3] {
            if let Some(token) = operand {
                AssemblerInstruction::extract_operand(token, &mut results);
            }
        }

        results
    }

    fn extract_operand(t: &Token, results: &mut Vec<u8>) {
        match t {
            Token::Register { reg_num } => { results.push(*reg_num) },
            Token::IntegerOperand { value } => {
                let converted = *value as u16;
                let byte1 = converted;
                let byte2 = converted >> 8;
                // pay attention to order
                results.push(byte2 as u8);
                results.push(byte1 as u8);
            }

            _ => {
                println!("Opcode found in operand field");
                process::exit(1);
            }
        }
    }
}


/// Handles instructions of the following form:
/// LOAD $0 #100
named!(pub instruction_one<CompleteStr, AssemblerInstruction>,
    do_parse!(
        o: opcode_load >>
        r: register >>
        i: integer_operand >>
        (
            AssemblerInstruction {
                opcode: Some(o),
                operand1: Some(r),
                operand2: Some(i),
                operand3: None,
            }
        )
    )
);


#[cfg(test)]
mod tests {
    use instruction::Opcode;
    use super::*;

    #[test]
    fn test_parse_instruction_form_one() {
        let result = instruction_one(CompleteStr("load $0 #100\n"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: Some(Token::Op { code: Opcode::LOAD }),
                    operand1: Some(Token::Register { reg_num: 0 }),
                    operand2: Some(Token::IntegerOperand { value: 100 }),
                    operand3: None,
                }
            ))
        );
    }
}