use nom::types::CompleteStr;
use assembler::Token;
use instruction::Opcode;

named!(
    pub opcode_load<CompleteStr, Token>,
    do_parse!(
        tag!("load") >> (Token::Op{code: Opcode::LOAD})
    )
);


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_load() {
        let result = opcode_load(CompleteStr("load"));
        assert!(result.is_ok());
        let (res, token) = result.unwrap();
        assert_eq!(token, Token::Op{code: Opcode::LOAD});
        assert_eq!(res, CompleteStr(""));

        let result = opcode_load(CompleteStr("aold"));
        assert_eq!(result.is_ok(), false);
    }
}