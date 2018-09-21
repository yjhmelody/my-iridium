use assembler::instruction_parsers::AssemblerInstruction;
use assembler::operand_parsers::parse_operand;
use assembler::Token;
use nom::alpha;
use nom::types::CompleteStr;


/// Parser for directive
named!(pub parse_directive_decl <CompleteStr, Token>,
    ws!(
        do_parse!(
            tag!(".") >>
            name: alpha >>
            (
                Token::Directive{
                  name: name.to_string(),
                }
            )
        )
    )
);

named!(parse_directive_combined<CompleteStr, AssemblerInstruction>,
    ws!(
        do_parse!(
            tag!(".") >>
            name: parse_directive_decl >>
            o1: opt!(parse_operand) >>
            o2: opt!(parse_operand) >>
            o3: opt!(parse_operand) >>
            (
                AssemblerInstruction{
                    opcode: None,
                    directive: Some(name),
                    label: None,
                    operand1: o1,
                    operand2: o2,
                    operand3: o3,
                }
            )
        )
    )
);

/// Will try to parse out any of the Directive forms
named!(pub parse_directive<CompleteStr, AssemblerInstruction>,
    do_parse!(
        ins: alt!(
            parse_directive_combined
        ) >>
        (
            ins
        )
    )
);

mod tests {
    #![allow(unused_imports)]

    use super::*;

    #[test]
    fn test_parser_directive_decl() {
        let result = parse_directive_decl(CompleteStr(".data"));
        assert_eq!(result.is_ok(), true);
        let (_, directive) = result.unwrap();
        assert_eq!(directive, Token::Directive { name: "data".to_string() })
    }
}
