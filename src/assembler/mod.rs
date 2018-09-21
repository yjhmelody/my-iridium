use ::vm::VM;
use assembler::symbols::*;
use instruction::Opcode;
use nom::types::CompleteStr;
use self::program_parsers::*;

pub mod opcode_parsers;
pub mod register_parsers;
pub mod operand_parsers;
pub mod instruction_parsers;
pub mod program_parsers;
pub mod directive_parsers;
pub mod label_parsers;
pub mod symbols;


#[derive(Debug, PartialEq)]
pub enum Token {
    Op { code: Opcode },
    Register { reg_num: u8 },
    IntegerOperand { value: i32 },
    LabelDeclaration { name: String },
    LabelUsage { name: String },
    Directive { name: String },
}

#[derive(Debug, Default)]
pub struct Assembler {
    pub phase: AssemblerPhase,
    pub symbols: SymbolTable,
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            phase: AssemblerPhase::First,
            symbols: SymbolTable::new(),
        }
    }

    pub fn assemble(&mut self, raw: &str) -> Option<Vec<u8>> {
        match parse_program(CompleteStr(raw)) {
            Ok((_remainder, program)) => {
                // first pass
                self.process_first_phase(&program);
                // second pass
                Some(self.process_second_phase(&program))
            },

            Err(e) => {
                println!("There was an error assembling the code: {:?}", e);
                None
            }
        }
    }

    fn process_first_phase(&mut self, p: &Program) {
        self.extract_labels(p);
        self.phase = AssemblerPhase::Second;
    }

    fn process_second_phase(&mut self, p: &Program) -> Vec<u8> {
        let mut program = vec![];
        for i in &p.instructions {
            let mut bytes = i.to_bytes();
            program.append(&mut bytes);
        }

        program
    }

    fn extract_labels(&mut self, p: &Program) {
        let mut count = 0;
        // build symbol table for all instructions which contains label
        for ins in &p.instructions {
            if ins.is_label() {
                match ins.label_name() {
                    Some(name) => {
                        let symbol = Symbol::new(name, count, SymbolType::Label);
                        self.symbols.add_symbol(symbol);
                    },

                    None => {},
                }
            }
            count += 4;
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
}
