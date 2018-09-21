#[derive(Debug)]
pub struct Symbol {
    name: String,
    offset: Option<u32>,
    symbol_type: SymbolType,
}

impl Symbol {
    pub fn new(name: String, symbol_type: SymbolType) -> Self {
        Self { name, symbol_type, offset: None }
    }

    pub fn new_with_offset(name: String, symbol_type: SymbolType, offset: u32) -> Self {
        Self {
            name,
            symbol_type,
            offset: Some(offset),
        }
    }
}

#[derive(Debug)]
pub enum SymbolType {
    Label,
}

#[derive(Debug, Default)]
pub struct SymbolTable {
    // todo: use HashTable to replace Vec
    pub symbols: Vec<Symbol>,
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

    pub fn has_symbol(&self, s: &str) -> bool {
        for symbol in &self.symbols {
            if symbol.name == s {
                return true;
            }
        }

        false
    }

    /// Gived a s, return its mapping value
    pub fn symbol_value(&self, s: &str) -> Option<u32> {
        for symbol in &self.symbols {
            if symbol.name == s {
                return symbol.offset;
            }
        }

        None
    }

    pub fn set_symbol_offset(&mut self, s: &str, offset: u32) -> bool {
        for symbol in &mut self.symbols {
            if symbol.name == s {
                symbol.offset = Some(offset);
                return true;
            }
        }

        false
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
    #![allow(unused_imports)]

    use super::*;

    #[test]
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
}
