use assembler::Token;
use instruction::Opcode;
use nom::alpha1;
use nom::types::CompleteStr;

/// Parser for opcode
named!(pub parse_opcode<CompleteStr, Token>,
    do_parse!(
        opcode: alpha1 >>
        (
            Token::Op {code: Opcode::from(opcode)}
        )
    )
);


#[cfg(test)]
mod tests {
    #![allow(unused_imports)]

    use super::*;

    #[test]
    fn test_opcode() {
        let result = parse_opcode(CompleteStr("load"));
        assert_eq!(result.is_ok(), true);
        let (res, token) = result.unwrap();
        assert_eq!(token, Token::Op{code: Opcode::LOAD});
        assert_eq!(res, CompleteStr(""));

        let result = parse_opcode(CompleteStr("aold"));
        let (_, token) = result.unwrap();
        assert_eq!(token, Token::Op { code: Opcode::IGL });
    }
}