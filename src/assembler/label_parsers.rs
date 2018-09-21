use assembler::Token;
use nom::{alphanumeric, multispace};
use nom::types::CompleteStr;

/// Look for a user-defined label, such as `label1:`
named!(pub parse_label_decl<CompleteStr, Token>,
    ws!(
        do_parse!(
            name: alphanumeric >>
            tag!(":") >>
            opt!(multispace) >>
            (
                Token::LabelDeclaration{name: name.to_string()}
            )
        )
    )
);

/// Looks for a user-defined label which is used, such as `@label1`
named!(pub parse_label_usage<CompleteStr, Token>,
    ws!(
        do_parse!(
            tag!("@") >>
            name: alphanumeric >>
            opt!(multispace) >>
            (
                Token::LabelUsage{name: name.to_string()}
            )
        )
    )
);

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]

    use super::*;

    #[test]
    fn test_parse_label_declaration() {
        let result = parse_label_decl(CompleteStr("test:"));
        assert_eq!(result.is_ok(), true);
        let (_, token) = result.unwrap();
        assert_eq!(token, Token::LabelDeclaration { name: "test".to_string() });
        let result = parse_label_decl(CompleteStr("test"));
        assert_eq!(result.is_ok(), false);
    }

    #[test]
    fn test_parse_label_usage() {
        let result = parse_label_usage(CompleteStr("@test"));
        assert_eq!(result.is_ok(), true);
        let (_, token) = result.unwrap();
        assert_eq!(token, Token::LabelUsage { name: "test".to_string() });
        let result = parse_label_usage(CompleteStr("test"));
        assert_eq!(result.is_ok(), false);
    }
}