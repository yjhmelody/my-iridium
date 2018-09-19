use assembler::directive_parsers::*;
use assembler::label_parsers::*;
use assembler::opcode_parsers::*;
use assembler::operand_parsers::*;
use assembler::register_parsers::*;
use assembler::Token;
use nom::multispace;
use nom::types::CompleteStr;
use std::process;


/// Stores a line assemble instruction
#[derive(Debug, PartialEq)]
pub struct AssemblerInstruction {
    label: Option<Token>,
    directive: Option<Token>,
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
                    process::exit(1);
                }
            }
        }

        // translate operands
        for operand in &[&self.operand1, &self.operand2, &self.operand3] {
            if let Some(token) = operand {
                AssemblerInstruction::extract_operand(token, &mut results);
            }
        }

        // padding to 32 bits
        while results.len() < 4 {
            results.push(0);
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

    pub fn is_label(&self) -> bool {
        self.label.is_some()
    }

    pub fn is_opcode(&self) -> bool {
        self.opcode.is_some()
    }

    pub fn is_directive(&self) -> bool {
        self.directive.is_some()
    }

    pub fn label_name(&self) -> Option<String> {
        if let Some(label) = &self.label {
            match label {
                Token::LabelDeclaration { name } => {
                    Some(name.clone())
                },
                _ => None,
            }
        } else {
            None
        }
    }
}


/// Handles instructions of the following form:
/// <opcode> <register> <operand>
named!(instruction_one<CompleteStr, AssemblerInstruction>,
    do_parse!(
        l: opt!(parse_label_decl) >>
        o: parse_opcode >>
        r: parse_register >>
        i: parse_operand >>
        (
            AssemblerInstruction{
                opcode: Some(o),
                label: l,
                directive: None,
                operand1: Some(r),
                operand2: Some(i),
                operand3: None
            }
        )
    )
);

/// Handles instructions of the following form:
/// <opcode>
named!(instruction_two<CompleteStr, AssemblerInstruction>,
    do_parse!(
        l: opt!(parse_label_decl) >>
        o: parse_opcode >>
        opt!(multispace) >>
        (
            AssemblerInstruction{
                opcode: Some(o),
                label: l,
                directive: None,
                operand1: None,
                operand2: None,
                operand3: None,
            }
        )
    )
);

/// Handles instructions of the following form:
/// <opcode> <register> <register> <register>
named!(instruction_three<CompleteStr, AssemblerInstruction>,
    do_parse!(
        l: opt!(parse_label_decl) >>
        o: parse_opcode >>
        r1: parse_register >>
        r2: parse_register >>
        r3: parse_register >>
        (
            AssemblerInstruction{
                opcode: Some(o),
                label: l,
                directive: None,
                operand1: Some(r1),
                operand2: Some(r2),
                operand3: Some(r3),
            }
        )
    )
);

/// Handles instructions of the following form:
/// <directive>
named!(instruction_four<CompleteStr, AssemblerInstruction>,
    do_parse!(
        d: parse_directive >>
        (
            AssemblerInstruction{
                label: None,
                opcode: None,
                directive: Some(d),
                operand1: None,
                operand2: None,
                operand3: None
            }
        )
    )
);

/// Will try to parse out any of the Instruction forms
named!(pub parse_instruction<CompleteStr, AssemblerInstruction>,
    do_parse!(
        ins: alt!(
            instruction_one |
            instruction_three |
            instruction_two |
            instruction_four
        ) >>
        (
            ins
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
                    label: None,
                    directive: None,
                    operand1: Some(Token::Register { reg_num: 0 }),
                    operand2: Some(Token::IntegerOperand { value: 100 }),
                    operand3: None,
                }
            ))
        );
    }

    #[test]
    fn test_parse_instruction_form_one_with_label() {
        let result = instruction_one(CompleteStr("load $0 @test1\n"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: Some(Token::Op { code: Opcode::LOAD }),
                    label: None,
                    directive: None,
                    operand1: Some(Token::Register { reg_num: 0 }),
                    operand2: Some(Token::LabelUsage { name: "test1".to_string() }),
                    operand3: None,
                }
            ))
        );
    }

    #[test]
    fn test_parse_instruction_form_two() {
        let result = instruction_two(CompleteStr("hlt\n"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: Some(Token::Op { code: Opcode::HLT }),
                    label: None,
                    directive: None,
                    operand1: None,
                    operand2: None,
                    operand3: None,
                }
            ))
        );
    }

    #[test]
    fn test_parse_instruction_form_three() {
        let result = instruction_three(CompleteStr("add $0 $1 $2\n"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: Some(Token::Op { code: Opcode::ADD }),
                    label: None,
                    directive: None,
                    operand1: Some(Token::Register { reg_num: 0 }),
                    operand2: Some(Token::Register { reg_num: 1 }),
                    operand3: Some(Token::Register { reg_num: 2 }),
                }
            ))
        );
    }

    #[test]
    fn test_parse_instruction_form_four() {
        let result = instruction_four(CompleteStr(".data\n"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: None,
                    label: None,
                    directive: Some(Token::Directive { name: "data".to_string() }),
                    operand1: None,
                    operand2: None,
                    operand3: None,
                }
            ))
        );
    }
}