use assembler::label_parsers::*;
use assembler::opcode_parsers::*;
use assembler::operand_parsers::*;
use assembler::symbols::*;
use assembler::Token;
use nom::types::CompleteStr;
use std::process;

/// Stores a line assemble instruction
#[derive(Debug, PartialEq)]
pub struct AssemblerInstruction {
    pub label: Option<Token>,
    pub directive: Option<Token>,
    pub opcode: Option<Token>,
    pub operand1: Option<Token>,
    pub operand2: Option<Token>,
    pub operand3: Option<Token>,
}

impl AssemblerInstruction {
    /// Translates instruction into bytes for eval.
    pub fn to_bytes(&self, symbols: &SymbolTable) -> Vec<u8> {
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
                AssemblerInstruction::extract_operand(token, &mut results, symbols);
            }
        }

        // padding to 32 bits
        while results.len() < 4 {
            results.push(0);
        }

        results
    }

    fn extract_operand(t: &Token, results: &mut Vec<u8>, symbols: &SymbolTable) {
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

            Token::FloatOperand { value } => {
                unimplemented!();
            }

            Token::LabelUsage { name } => {
                if let Some(value) = symbols.symbol_value(name) {
                    let byte1 = value;
                    let byte2 = value >> 8;
                    results.push(byte2 as u8);
                    results.push(byte1 as u8);
                }
            }

            _ => {
                println!("Opcode found in operand field");
                process::exit(1);
            }
        }
    }

    /// Check it is label
    pub fn is_label(&self) -> bool {
        self.label.is_some()
    }

    /// Check it is opcode
    pub fn is_opcode(&self) -> bool {
        self.opcode.is_some()
    }

    /// Check it is directive
    pub fn is_directive(&self) -> bool {
        self.directive.is_some()
    }

    pub fn has_operands(&self) -> bool {
        self.operand1.is_some() || self.operand2.is_some() || self.operand3.is_some()
    }

    pub fn get_directive_name(&self) -> Option<String> {
        match &self.directive {
            Some(d) => {
                match d {
                    Token::Directive { name } => Some(name.to_string()),
                    _ => None,
                }
            }

            None => None,
        }
    }

    /// Get label's name
    pub fn get_label_name(&self) -> Option<String> {
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

    // Get the string constant from `.asciiz` directive
    pub fn get_string_constant(&self) -> Option<String> {
        match &self.operand1 {
            Some(d) => match d {
                Token::IrString { name } => Some(name.to_string()),
                _ => None,
            },
            None => None,
        }
    }
}


named!(parse_instruction_combined<CompleteStr, AssemblerInstruction>,
    do_parse!(
        label: opt!(parse_label_decl) >>
        o: parse_opcode >>
        operand1: opt!(parse_operand) >>
        operand2: opt!(parse_operand) >>
        operand3: opt!(parse_operand) >>
        (
            AssemblerInstruction {
                opcode: Some(o),
                label,
                directive: None,
                operand1,
                operand2,
                operand3,
            }
        )
    )
);

/// Will try to parse out any of the Instruction forms
named!(pub parse_instruction<CompleteStr, AssemblerInstruction>,
    do_parse!(
        ins: alt!(
            parse_instruction_combined
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
        let result = parse_instruction_combined(CompleteStr("load $0 #100\n"));
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
        let result = parse_instruction_combined(CompleteStr("load $0 @test1\n"));
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
        let result = parse_instruction_combined(CompleteStr("hlt"));
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
        let result = parse_instruction_combined(CompleteStr("add $0 $1 $2\n"));
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
}