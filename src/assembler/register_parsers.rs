use assembler::Token;
use nom::digit;
use nom::types::CompleteStr;
use super::*;

named!(
    register<CompleteStr, Token>,
    ws!(
        do_parse!(
            tag!("$") >>
            reg_num: digit >>
            (
                Token::Register{
                    reg_num: reg_num.parse::<u8>().unwrap()
                }
            )
        )
    )
);


#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_register() {
        let res = register(CompleteStr("$0"));
        assert_eq!(res.is_ok(), true);
        let res = register(CompleteStr("0"));
        assert_eq!(res.is_ok(), false);
        let res = register(CompleteStr("$a"));
        assert_eq!(res.is_ok(), false);
    }
}
