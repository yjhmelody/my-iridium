use assembler::errors::AssemblerError;
use assembler::instruction_parsers::AssemblerInstruction;
use assembler::program_parsers::*;
use assembler::symbols::*;
use byteorder::{LittleEndian, WriteBytesExt};
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
    FloatOperand { value: f64 },
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
    /// Creates a assembler to deal with asm
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

    /// write a fixed header to instructions vec
    fn write_pie_header(&self) -> Vec<u8> {
        let mut header = vec![];
        for byte in PIE_HEADER_PREFIX.iter() {
            header.push(byte.clone());
        }
        // Now we need to calculate the starting offset so that the
        // VM knows where the `read-only section` ends
        let mut wtr: Vec<u8> = vec![];

        // Write the length of the read-only section to the vector and convert it to a u32
        // This is important because byteorder crate will pad with zeros as needed
        wtr.write_u32::<LittleEndian>(self.ro.len() as u32).unwrap();

        // Append those 4 bytes to the header directly after the first four bytes
        header.append(&mut wtr);

        while header.len() < PIE_HEADER_LENGTH {
            header.push(0 as u8);
        }

        header
    }

    /// assemble asm to instructions
    pub fn assemble(&mut self, raw: &str) -> AssemblerResult {
        match parse_program(CompleteStr(raw)) {
            Ok((_remainder, program)) => {
                // todo: add a check for `remainder` which should be ""

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
                // write header after second pass
                let mut assembled_program = self.write_pie_header();
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
    use vm::VM;

    #[test]
    /// Tests assembly a small but correct program
    fn test_assemble_program() {
        let mut asm = Assembler::new();
        let test_string = r"
        .data
        .code
        load $0 #100
        load $1 #1
        load $2 #0
        test: inc $0
        neq $0 $2
        jmpe @test
        hlt
        ";
        let program = asm.assemble(test_string).unwrap();
        let mut vm = VM::new();
        assert_eq!(program.len(), 92);
        vm.add_bytes(program);
        assert_eq!(vm.program.len(), 92);
    }

    #[test]
    /// Simple test of data that goes into the read only section
    fn test_code_start_offset_written() {
        let mut asm = Assembler::new();
        let test_string = r"
        .data
        test1: .asciiz 'Hello'
        .code
        load $0 #100
        load $1 #1
        load $2 #0
        test: inc $0
        neq $0 $2
        jmpe @test
        hlt
        ";
        let program = asm.assemble(test_string);
        assert_eq!(program.is_ok(), true);
        let unwrapped = program.unwrap();
        assert_eq!(unwrapped[4], 6);
    }

    #[test]
    /// Tests that we can add things to the symbol table
    fn test_symbol_table() {
        let mut sym = SymbolTable::new();
        let new_symbol = Symbol::new_with_offset("test".to_string(), SymbolType::Label, 12);
        sym.add_symbol(new_symbol);
        assert_eq!(sym.symbols.len(), 1);
        let v = sym.symbol_value("test");
        assert_eq!(true, v.is_some());
        let v = v.unwrap();
        assert_eq!(v, 12);
        let v = sym.symbol_value("does_not_exist");
        assert_eq!(v.is_some(), false);
    }

    #[test]
    /// Simple test of data that goes into the read only section
    fn test_ro_data() {
        let mut asm = Assembler::new();
        let test_string = r"
        .data
        test: .asciiz 'This is a test'
        .code
        ";
        let program = asm.assemble(test_string);
        assert_eq!(program.is_ok(), true);
    }

    #[test]
    /// This tests that a section name that isn't `code` or `data` throws an error
    fn test_bad_ro_data() {
        let mut asm = Assembler::new();
        let test_string = r"
        .code
        test: .asciiz 'This is a test'
        .wrong
        ";
        let program = asm.assemble(test_string);
        assert_eq!(program.is_ok(), false);
    }

    #[test]
    /// Tests that code which does not declare a segment first does not work
    fn test_first_phase_no_segment() {
        let mut asm = Assembler::new();
        let test_string = "hello: .asciiz 'Fail'";
        let result = parse_program(CompleteStr(test_string));
        assert_eq!(result.is_ok(), true);
        let (_, p) = result.unwrap();
        asm.process_first_phase(&p);
        assert_eq!(asm.errors.len(), 1);
    }

    #[test]
    /// Tests that code inside a proper segment works
    fn test_first_phase_inside_segment() {
        let mut asm = Assembler::new();
        let test_string = r"
        .data
        test: .asciiz 'Hello'
        ";
        let result = parse_program(CompleteStr(test_string));
        assert_eq!(result.is_ok(), true);
        let (_, p) = result.unwrap();
        asm.process_first_phase(&p);
        assert_eq!(asm.errors.len(), 0);
    }
}
