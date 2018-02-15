use std::num::Wrapping;
use std::result::Result;
use std::ops::AddAssign;
use std::ops::MulAssign;
use super::ParseError;

const VERSION_ID: u64 = 8u64;
const LENGTH_ID: u64 = 9u64;
const MSG_TYPE_ID: u64 = 35u64;

#[derive(Eq, PartialEq, Debug)]
pub struct FixMessage<'a> {
    pub msg_type: &'a [u8],
    pub body: &'a [u8],
    pub header_checksum: Wrapping<u8>,
}

#[derive(Eq, PartialEq, Debug)]
pub struct FixField<'a> {
    pub id: u64,
    pub value: &'a [u8],
    pub length: usize,
    pub checksum: Wrapping<u8>,
}

// Please only pass non-empty inputs
pub fn parse_fix_field(input: &[u8]) -> Result<FixField, ParseError> {
    const DELIMITER: u8 = 1u8;

    let input_len = input.len();

    let c = input[0];
    let Wrapping(digit) = Wrapping(c) - Wrapping('0' as u8);
    if digit > 9u8 {
        return Err("FIX id invalid initial character");
    }

    let mut id = digit as u64;
    let mut checksum = Wrapping(c);
    for k in 1usize.. {
        if k >= input_len {
            return Err("FIX id too big");
        }
        let c = input[k];
        checksum += Wrapping(c);
        if c == '=' as u8 {
            let start = k + 1;
            for k in start.. {
                if k >= input_len {
                    return Err("FIX value too big");
                }
                let c = input[k];
                checksum += Wrapping(c);
                if c == DELIMITER {
                    let field = FixField {
                        id,
                        value: &input[start..k],
                        length: (k + 1),
                        checksum,
                    };
                    return Ok(field);
                }
            }
        }
        let Wrapping(digit) = Wrapping(c) - Wrapping('0' as u8);
        if digit > 9u8 {
            return Err("FIX id invalid character");
        }
        id = 10u64 * id + digit as u64;
    }

    Err("FIX out of the infinite loop")
}

pub fn parse_fix_message(input: &[u8]) -> Result<FixMessage, ParseError> {
    const CHECKSUM_LENGTH: usize = 7;

    if input.len() == 0 {
        return Err("FIX received empty message");
    }

    let (version, input) = parse_header_field(VERSION_ID, input)?;
    let (length, input) = parse_header_field(LENGTH_ID, input)?;

    // TODO: Fix this
/*
    if parse_int::<usize>(length.value)? + CHECKSUM_LENGTH != input.len() {
        return Err("FIX validation: invalid length");
    }
*/
    let (msg_type, input) = parse_header_field(MSG_TYPE_ID, input)?;

    Ok(FixMessage {
        msg_type: msg_type.value,
        body: input,
        header_checksum: version.checksum + length.checksum + msg_type.checksum,
    })
}

fn parse_header_field(id: u64, input: &[u8]) -> Result<(FixField, &[u8]), ParseError> {
    match parse_fix_field(input) {
        Ok(field) => {
            if field.id != id {
                return Err("FIX unexpected id when parsing header");
            }
            if input.len() > field.length {
                let len = field.length;
                Ok((field, &input[len..]))
            } else {
                Err("FIX unexpected end of input when parsing header")
            }
        }
        Err(e) => Err(e),
    }
}

fn parse_int<T>(value: &[u8]) -> Result<T, ParseError>
where
    T: From<u8> + MulAssign + AddAssign + Copy,
{
    let value_len = value.len();
    if value_len == 0 {
        return Err("FIX could not parse integer from empty string");
    }
    let c = value[0];
    let Wrapping(digit) = Wrapping(c) - Wrapping('0' as u8);
    if digit > 9u8 {
        return Err("FIX could not parse integer - invalid initial digit");
    }
    let mut result = From::from(digit);
    let ten: T = From::from(10u8);
    for k in 1..value_len {
        let c = value[k];
        let Wrapping(digit) = Wrapping(c) - Wrapping('0' as u8);
        if digit > 9u8 {
            return Err("FIX could not parse length value");
        }
        result *= ten;
        result += From::from(digit);
    }
    Ok(result)
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::serialization::checksum;
    use quickcheck::*;

    pub fn to_fix(s: &str) -> Vec<u8> {
        s.replace('|', "\x01")
            .as_bytes()
            .iter()
            .map(|a| *a)
            .collect()
    }

    #[test]
    fn test_valid() {
        let msg = to_fix("8=FIX|asdf");
        let result = parse_fix_field(&msg);
        assert_eq!(
            result,
            Ok(FixField {
                id: 8u64,
                value: b"FIX",
                length: 6usize,
                checksum: checksum(&to_fix("8=FIX|")),
            })
        );
    }

    #[test]
    fn test_error_no_id() {
        assert!(parse_fix_field(&to_fix("=asdf|")).is_err());
    }

    #[test]
    fn test_error_just_id() {
        assert!(parse_fix_field(&to_fix("123")).is_err());
    }

    #[test]
    fn test_error_no_end() {
        assert!(parse_fix_field(&to_fix("123=asdf")).is_err());
    }

    #[test]
    fn test_error_invalid_id() {
        assert!(parse_fix_field(&to_fix("123x=y|")).is_err());
    }

    #[test]
    fn qc_parse_fix_field() {
        quickcheck(prop_parse_fix_field_never_crashes as fn(String) -> bool);
    }

    fn prop_parse_fix_field_never_crashes(input: String) -> bool {
        if !input.is_empty() {
            parse_fix_field(input.as_bytes()).ok();
        }
        true
    }

    #[test]
    fn qc_parse_valid_input() {
        quickcheck(prop_parse_valid_input as fn(u64, String, String) -> bool);
    }

    fn prop_parse_valid_input(id: u64, value: String, postfix: String) -> bool {
        if value.as_bytes().contains(&1u8) {
            return true;
        }
        let fix_field = format!("{}={}\x01", id, value);
        let fix_stream = format!("{}{}", fix_field, postfix);
        let expected = FixField {
            id,
            value: value.as_bytes(),
            length: fix_field.len(),
            checksum: checksum(fix_field.as_bytes()),
        };

        parse_fix_field(fix_stream.as_bytes()) == Ok(expected)
    }

    #[test]
    fn test_header() {
        let msg = to_fix("8=FIX.4.2|9=10|35=A|34=1|10=123|");
        let result = parse_fix_message(&msg);
        assert_eq!(
            result,
            Ok(FixMessage {
                msg_type: b"A",
                body: &to_fix("34=1|10=123|"),
                header_checksum: checksum(&to_fix("8=FIX.4.2|9=10|35=A|")),
            })
        );
    }

    #[test]
    fn test_error_empty() {
        assert!(parse_fix_message(&to_fix("")).is_err());
    }

    #[test]
    fn test_error_no_length() {
        assert!(parse_fix_message(&to_fix("8=FIX.4.2|")).is_err());
    }

    // TODO: Make it pass.
    #[test]
    #[ignore]
    fn test_error_invalid_length() {
        assert!(parse_fix_message(&to_fix("8=FIX.4.2|9=5|35=A|10=123|")).is_ok());
        assert!(parse_fix_message(&to_fix("8=FIX.4.2|9=6|35=A|10=123|")).is_err());
    }

    #[test]
    fn test_error_no_body() {
        assert!(parse_fix_message(&to_fix("8=FIX.4.2|9=166|35=A|")).is_err());
    }

    #[test]
    fn test_error_different_order() {
        assert!(parse_fix_message(&to_fix("9=5|8=FIX.4.2|35=A|10=111|")).is_err());
    }

    #[test]
    fn test_error_weird_field() {
        assert!(parse_fix_message(&to_fix("8=FIX.4.2|3=5|9=5|35=A|10=111|")).is_err());
    }
}
