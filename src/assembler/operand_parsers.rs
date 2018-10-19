use assembler::label_parsers::parse_label_usage;
use assembler::register_parsers::parse_register;
use assembler::Token;
use nom::digit;
use nom::types::CompleteStr;

/// Parser for all kinds of operand
named!(pub parse_operand<CompleteStr, Token>,
    alt!(
        parse_integer_operand |
        parse_float_operand |
        parse_label_usage |
        parse_register |
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

named!(parse_float_operand<CompleteStr, Token>,
    ws!(
        do_parse!(
            tag!("#") >>
            sign: opt!(tag!("-")) >>
            left_nums: digit >>
            tag!(".") >>
            right_nums: digit >>
            (
                {
                    let mut num = String::from("");
                    if sign.is_some() {
                        num.push_str("-");
                    }
                    num.push_str(&left_nums.to_string());
                    num.push_str(".");
                    num.push_str(&right_nums.to_string());
                    Token::FloatOperand{value: num.parse::<f64>().unwrap()}
                }
            )
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

    #[test]
    fn test_parse_float_operand() {
        let test = vec!["#100.3", "#-100.3", "#1.0", "#0.0"];
        for i in &test {
            assert_eq!(parse_float_operand(CompleteStr(i)).is_ok(), true);
        }
    }
}