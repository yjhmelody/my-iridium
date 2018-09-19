use assembler::Token;
use nom::digit;
use nom::types::CompleteStr;

/// Parser for integer numbers, which we preface with `#` in our assembly language
named!(pub integer_operand<CompleteStr, Token>,
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_integer_oprand() {
        let result = integer_operand(CompleteStr("#10"));
        assert_eq!(result.is_ok(), true);
        let (res, token) = result.unwrap();
        assert_eq!(res, CompleteStr(""));
        assert_eq!(token, Token::IntegerOperand { value: 10 });

        let result = integer_operand(CompleteStr("10"));
        assert_eq!(result.is_ok(), false);
    }
}