use assembler::label_parsers::parse_label_usage;
use assembler::register_parsers::parse_register;
use assembler::Token;
use nom::digit;
use nom::types::CompleteStr;

/// Parser for all kinds of operand
named!(pub parse_operand<CompleteStr, Token>,
    alt!(
        parse_integer_operand |
        parse_register |
        parse_label_usage |
        parse_irstring
    )
);

/// Parser for integer numbers, which we preface with `#` in our assembly language
named!(pub parse_integer_operand<CompleteStr, Token>,
    ws!(
        do_parse!(
            tag!("#") >>
            reg_num: digit >>
            (
                Token::IntegerOperand{value: reg_num.parse::<i32>().unwrap()}
            )
        )
    )
);

/// Parser for vm's string constants
named!(parse_irstring<CompleteStr, Token>,
    do_parse!(
        tag!("'") >>
        content: take_until!("'") >>
        tag!("'") >>
        (
            Token::IrString{ name: content.to_string()}
        )
    )
);


#[cfg(test)]
mod tests {
    #![allow(unused_imports)]

    use super::*;

    #[test]
    fn test_parse_integer_oprand() {
        let result = parse_integer_operand(CompleteStr("#10"));
        assert_eq!(result.is_ok(), true);
        let (res, token) = result.unwrap();
        assert_eq!(res, CompleteStr(""));
        assert_eq!(token, Token::IntegerOperand { value: 10 });

        let result = parse_integer_operand(CompleteStr("10"));
        assert_eq!(result.is_ok(), false);
    }

    #[test]
    fn test_parse_string_operand() {
        let result = parse_irstring(CompleteStr("'This is a test'"));
        assert_eq!(result.is_ok(), true);

        let result = parse_irstring(CompleteStr("\"This is a test\""));
        assert_eq!(result.is_ok(), false);
    }
}