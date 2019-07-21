extern crate nom;

use nom::{
    branch::*, bytes::streaming::*, character::streaming::*, combinator::*, multi::*, sequence::*,
    IResult,
};
use std::convert::TryInto;
use std::string::String;

/// Parses a full string
pub fn string(input: &str) -> IResult<&str, String> {
    delimited(
        char('"'),
        fold_many0(string_element, String::new(), |mut acc: String, item| {
            acc.push_str(&item);
            acc
        }),
        char('"'),
    )(input)
}

/// Parses one element of a string
fn string_element(input: &str) -> IResult<&str, String> {
    alt((
        map(take_till1(|c| c == '"' || c == '\\'), |s| String::from(s)),
        mnemonic_escape,
        map(tag("\\\""), |_| String::from("\"")),
        map(tag("\\\\"), |_| String::from("\\")),
        whitespace,
        inline_hex_escape,
    ))(input)
}

fn whitespace(input: &str) -> IResult<&str, String> {
    let (s, _) = tag("\\")(input)?;
    let (s, _) = space0(s)?;
    let (s, _) = line_ending(s)?;
    let (s, _) = space0(s)?;
    Ok((s, String::from("")))
}

fn mnemonic_escape(input: &str) -> IResult<&str, String> {
    preceded(
        tag("\\"),
        map_res(take(1usize), |c| match c {
            "a" => Ok(String::from("\u{0007}")),
            "b" => Ok(String::from("\u{0008}")),
            "t" => Ok(String::from("\t")),
            "n" => Ok(String::from("\n")),
            "r" => Ok(String::from("\r")),
            _ => Err(format!("Invalid escape sequence \\{}", c)),
        }),
    )(input)
}

fn inline_hex_escape(input: &str) -> IResult<&str, String> {
    let (s, _) = tag("\\x")(input)?;
    map_res(
        take_while_m_n(1, 8, |c: char| c.is_ascii_hexdigit()),
        |num| {
            let str_num = u32::from_str_radix(num, 16).unwrap();
            let character: Result<char, _> = str_num.try_into();
            character.map(|c| c.to_string())
        },
    )(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string() {
        assert_eq!(string("\"\"").unwrap(), ("", String::from("")));
    }

    #[test]
    fn raw_string() {
        assert_eq!(string("\"abc\"").unwrap(), ("", String::from("abc")));
    }

    #[test]
    fn raw_string_element() {
        assert_eq!(
            string_element("abc\"").unwrap(),
            ("\"", String::from("abc"))
        );
    }

    #[test]
    fn inline_hex_escape_valid() {
        assert_eq!(
            inline_hex_escape("\\x20!").unwrap(),
            ("!", String::from(" "))
        )
    }

    #[test]
    fn inline_hex_escape_invalid() {
        let res = inline_hex_escape("\\xd800!");
        assert!(res.is_err());
    }

    #[test]
    fn strip_inline_whitespace() {
        let str = "\"blah blah\\   \n    blah blah\"";
        let res = string(str);
        assert_eq!(res.unwrap(), (("", String::from("blah blahblah blah"))))
    }

    mod mnemonic_escape {
        use super::*;

        #[test]
        fn test_mnemonic_alert() {
            assert_eq!(
                mnemonic_escape("\\a").unwrap(),
                ("", String::from("\u{0007}"))
            )
        }

        #[test]
        fn test_mnemonic_bell() {
            assert_eq!(
                mnemonic_escape("\\b").unwrap(),
                ("", String::from("\u{0008}"))
            )
        }

        #[test]
        fn test_mnemonic_tab() {
            assert_eq!(mnemonic_escape("\\t").unwrap(), ("", String::from("\t")))
        }

        #[test]
        fn test_mnemonic_newline() {
            assert_eq!(mnemonic_escape("\\n").unwrap(), ("", String::from("\n")))
        }

        #[test]
        fn test_mnemonic_carriage_return() {
            assert_eq!(mnemonic_escape("\\r").unwrap(), ("", String::from("\r")))
        }

        #[test]
        fn test_mnemonic_invalid() {
            let res = mnemonic_escape("\\p");
            assert!(res.is_err())
        }
    }
}
