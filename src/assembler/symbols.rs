use ::vm::VM;
use assembler::Assembler;

#[derive(Debug)]
pub struct Symbol {
    name: String,
    offset: u32,
    symbol_type: SymbolType,
}

impl Symbol {
    pub fn new(name: String, offset: u32, symbol_type: SymbolType) -> Self {
        Self { name, offset, symbol_type }
    }
}

#[derive(Debug)]
pub enum SymbolType {
    Label,
}

#[derive(Debug, Default)]
pub struct SymbolTable {
    // todo: use HashTable to replace Vec
    symbols: Vec<Symbol>,
}

impl SymbolTable {
    /// Creates a symbol table
    pub fn new() -> Self {
        Self { symbols: Vec::new() }
    }

    /// Add a symbol to table
    pub fn add_symbol(&mut self, s: Symbol) {
        self.symbols.push(s);
    }

    /// Gived a s, return its mapping value
    pub fn symbol_value(&self, s: &str) -> Option<u32> {
        for symbol in &self.symbols {
            if symbol.name == s {
                return Some(symbol.offset);
            }
        }

        None
    }
}

/// Two pass for Assembler
#[derive(Debug, PartialEq, Clone)]
pub enum AssemblerPhase {
    First,
    Second,
}

impl Default for AssemblerPhase {
    fn default() -> Self {
        AssemblerPhase::First
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table() {
        let mut sym = SymbolTable::new();
        let new_symbol = Symbol::new("test".to_string(), 12, SymbolType::Label);
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
    fn test_assemble_program() {
        let mut asm = Assembler::new();
        let test_string = "load $0 #100\nload $1 #1\nload $2 #0\ntest: inc $0\nneq $0 $2\njmpe @test\nhlt";
        let program = asm.assemble(test_string).unwrap();
        let mut vm = VM::new();
        assert_eq!(program.len(), 21);
        vm.add_bytes(program);
        assert_eq!(vm.program.len(), 21);
    }
}
