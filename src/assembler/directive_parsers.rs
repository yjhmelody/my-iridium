use assembler::instruction_parsers::AssemblerInstruction;
use assembler::label_parsers::parse_label_decl;
use assembler::operand_parsers::parse_operand;
use assembler::Token;
use nom::alpha;
use nom::types::CompleteStr;


/// Parser for directive
named!(pub parse_directive_decl<CompleteStr, Token>,
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

named!(parse_directive_combined<CompleteStr, AssemblerInstruction>,
    ws!(
        do_parse!(
            label: opt!(parse_label_decl) >>
            name: parse_directive_decl >>
            operand1: opt!(parse_operand) >>
            operand2: opt!(parse_operand) >>
            operand3: opt!(parse_operand) >>
            (
                AssemblerInstruction{
                    opcode: None,
                    directive: Some(name),
                    label,
                    operand1,
                    operand2,
                    operand3,
                }
            )
        )
    )
);

mod tests {
    #![allow(unused_imports)]

    use super::*;

    #[test]
    fn test_parse_directive_decl() {
        let result = parse_directive_decl(CompleteStr(".data"));
        assert_eq!(result.is_ok(), true);
        let (_, directive) = result.unwrap();
        assert_eq!(directive, Token::Directive { name: "data".to_string() })
    }

    #[test]
    fn test_parse_directive_combined() {
        let directive2 = AssemblerInstruction {
            opcode: None,
            label: Some(Token::LabelDeclaration {
                name: "test".to_string()
            }),
            directive: Some(Token::Directive {
                name: "asciiz".to_string()
            }),
            operand1: Some(Token::IrString {
                name: "Hello".to_string()
            }),
            operand2: None,
            operand3: None,
        };

        let result = parse_directive_combined(CompleteStr("test: .asciiz 'Hello'"));
        assert_eq!(result.is_ok(), true);
        let (_, directive) = result.unwrap();
        assert_eq!(directive, directive2);
    }
}
