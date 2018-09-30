use assembler::errors::AssemblerError;
use assembler::instruction_parsers::AssemblerInstruction;
use assembler::program_parsers::*;
use assembler::symbols::*;
use instruction::Opcode;
use nom::types::CompleteStr;

pub mod opcode_parsers;
pub mod register_parsers;
pub mod operand_parsers;
pub mod instruction_parsers;
pub mod program_parsers;
pub mod directive_parsers;
pub mod label_parsers;
pub mod symbols;
pub mod errors;

pub const PIE_HEADER_PREFIX: [u8; 4] = [0x45, 0x50, 0x49, 0x45];
pub const PIE_HEADER_LENGTH: usize = 64;


#[derive(Debug, PartialEq)]
pub enum Token {
    Op { code: Opcode },
    Register { reg_num: u8 },
    IntegerOperand { value: i32 },
    LabelDeclaration { name: String },
    LabelUsage { name: String },
    Directive { name: String },
    IrString { name: String },
}

#[derive(Debug, Default)]
pub struct Assembler {
    /// Tracks which phase the assembler is in
    pub phase: AssemblerPhase,
    /// Symbol table for constants and variables
    pub symbols: SymbolTable,
    /// The read-only data section constants are put in
    pub ro: Vec<u8>,
    /// The compiled bytecode generated from assembly instructions
    pub bytecode: Vec<u8>,
    /// Tracks the current offset of the read-only section
    ro_offset: u32,
    /// A list of all the sections we've seen in the code
    sections: Vec<AssemblerSection>,
    /// The current section the assembler is in
    current_section: Option<AssemblerSection>,
    /// The current instruction the assembler is converting to bytecode
    current_instruction: u32,
    /// Any errors we find along the way. At the end, we'll present them to the user
    errors: Vec<AssemblerError>,
}

pub type AssemblerResult = Result<Vec<u8>, Vec<AssemblerError>>;

impl Assembler {
    pub fn new() -> Self {
        Self {
            phase: AssemblerPhase::First,
            symbols: SymbolTable::new(),
            ro: Vec::new(),
            bytecode: Vec::new(),
            ro_offset: 0,
            sections: Vec::new(),
            current_section: None,
            current_instruction: 0,
            errors: Vec::new(),
        }
    }

    fn write_pie_header(&self) -> Vec<u8> {
        let mut header = vec![];
        for byte in PIE_HEADER_PREFIX.iter() {
            header.push(byte.clone());
        }
        while header.len() <= PIE_HEADER_LENGTH {
            header.push(0 as u8);
        }

        header
    }

    pub fn assemble(&mut self, raw: &str) -> AssemblerResult {
        match parse_program(CompleteStr(raw)) {
            Ok((_remainder, program)) => {
                // todo: add a check for `remainder` which should be ""
                // write header
                let mut assembled_program = self.write_pie_header();

                // Start processing the AssembledInstructions. This is the first pass of our two-pass assembler.
                // We pass a read-only reference down to another function.
                self.process_first_phase(&program);
                if !self.errors.is_empty() {
                    return Err(self.errors.clone());
                }

                // Make sure that we have at least one data section and one code section
                if self.sections.len() != 2 {
                    // todo: detail out which ones are missing
                    println!("Did not find at least two sections");

                    self.errors.push(AssemblerError::InsufficientSections);
                    return Err(self.errors.clone());
                }

                // second pass which translates opcodes and operands into the bytecode
                let mut body = self.process_second_phase(&program);
                // add header
                assembled_program.append(&mut body);
                Ok(assembled_program)
            },

            Err(e) => {
                println!("There was an error assembling the code: {:?}", e);
                Err(vec![AssemblerError::ParseError { error: e.to_string() }])
            }
        }
    }

    /// Runs the first pass assembling process.
    /// It looks for labels and puts them in the symbol table
    fn process_first_phase(&mut self, p: &Program) {
        for inst in &p.instructions {
            // deal with label
            if inst.is_label() {
                if self.current_section.is_some() {
                    self.process_label_decl(&inst);
                } else {
                    self.errors.push(AssemblerError::NoSegmentDeclarationFound {
                        instruction: self.current_instruction,
                    });
                }
            }

            // deal with directive
            if inst.is_directive() {
                self.process_directive(inst);
            }
            self.current_instruction += 1;
        }

        // Once we're done with this function, set the phase to second
        self.phase = AssemblerPhase::Second;
    }

    /// Handles the declaration of a label such as:
    /// hello: .asciiz 'Hello'
    fn process_label_decl(&mut self, inst: &AssemblerInstruction) {
        let name = match inst.get_label_name() {
            Some(name) => name,
            None => {
                self.errors.push(AssemblerError::StringConstantDeclaredWithoutLabel {
                    instruction: self.current_instruction,
                });
                return;
            }
        };

        if self.symbols.has_symbol(&name) {
            self.errors.push(AssemblerError::SymbolAlreadyDeclared);
            return;
        }

        let symbol = Symbol::new(name, SymbolType::Label);
        self.symbols.add_symbol(symbol);
    }

    /// Runs the second pass of the assembler
    fn process_second_phase(&mut self, p: &Program) -> Vec<u8> {
        self.current_instruction = 0;
        let mut program = vec![];
        for inst in &p.instructions {
            if inst.is_opcode() {
                let mut bytes = inst.to_bytes(&self.symbols);
                program.append(&mut bytes);
            }
            if inst.is_directive() {
                self.process_directive(inst);
            }
        }

        program
    }

    fn process_directive(&mut self, inst: &AssemblerInstruction) {
        let directive_name = match inst.get_directive_name() {
            Some(name) => name,
            None => {
                println!("Directive has an invalid name: {:?}", inst);
                return;
            }
        };

        // check if there were any operands.
        if inst.has_operands() {
            match directive_name.as_ref() {
                "asciiz" => self.handle_asciiz(inst),
                _ => {
                    self.errors.push(AssemblerError::UnknownDirectiveFound {
                        directive: directive_name.clone(),
                    });
                    return;
                }
            }
        } else {
            self.process_section_header(&directive_name);
        }
    }

    /// Handles a declaration of a section header, such as:
    /// .code
    fn process_section_header(&mut self, header_name: &str) {
        let new_section: AssemblerSection = header_name.into();

        if new_section == AssemblerSection::Unknown {
            println!("Found an section header that is unknown: {:#?}", header_name);
            return;
        }

        // TODO: Check if we really need to keep a list of all sections seen
        self.sections.push(new_section.clone());
        self.current_section = Some(new_section);
    }

    /// Handles a declaration of a null-terminated string:
    /// hello: .asciiz 'Hello!'
    fn handle_asciiz(&mut self, inst: &AssemblerInstruction) {
        // Being a constant declaration, this is only meaningful in the first pass
        if self.phase != AssemblerPhase::First { return; }

        // In this case, operand1 will have the entire string we need to read in to RO memory
        match inst.get_string_constant() {
            Some(s) => {
                match inst.get_label_name() {
                    Some(name) => { self.symbols.set_symbol_offset(&name, self.ro_offset); }
                    None => {
                        // This would be someone typing:
                        // .asciiz 'Hello'
                        println!("Found a string constant with no associated label!");
                        return;
                    }
                };
                // We'll read the string into the read-only section byte-by-byte
                for byte in s.as_bytes() {
                    self.ro.push(*byte);
                    self.ro_offset += 1;
                }
                // This is the null termination bit we are using to indicate a string has ended
                self.ro.push(0);
                self.ro_offset += 1;
            }
            None => {
                // This just means someone typed `.asciiz` for some reason
                println!("String constant following an .asciiz was empty");
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssemblerSection {
    Data { starting_instruction: Option<u32> },
    Code { starting_instruction: Option<u32> },
    Unknown,
}

impl Default for AssemblerSection {
    fn default() -> Self {
        AssemblerSection::Unknown
    }
}

impl<'a> From<&'a str> for AssemblerSection {
    fn from(name: &str) -> AssemblerSection {
        match name {
            "data" => {
                AssemblerSection::Data { starting_instruction: None }
            }
            "code" => {
                AssemblerSection::Code { starting_instruction: None }
            }
            _ => {
                AssemblerSection::Unknown
            }
        }
    }
}


#[cfg(test)]
mod tests {
    #![allow(unused_imports)]

    use super::*;
}
