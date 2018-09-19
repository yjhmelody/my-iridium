use assembler::Token;
use nom::alpha;
use nom::types::CompleteStr;

/// Parser for directive
named!(pub parse_directive <CompleteStr, Token>,
    ws!(
        do_parse!(
            tag!(".") >>
            d: alpha >>
            (
                Token::Directive{
                  name: d.to_string(),
                }
            )
        )
    )
);

mod tests {
    use super::*;

    #[test]
    fn test_parser_directive() {
        let result = parse_directive(CompleteStr(".data"));
        assert_eq!(result.is_ok(), true);
        let (_, directive) = result.unwrap();
        assert_eq!(directive, Token::Directive { name: "data".to_string() })
    }
}
