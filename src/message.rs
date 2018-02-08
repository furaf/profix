use std::num::Wrapping;
use super::field::parse_fix_field;
use super::field::FixField;
use super::common::parse_int;

const VERSION_ID: u64 = 8u64;
const LENGTH_ID: u64 = 9u64;
const MSG_TYPE_ID: u64 = 35u64;

#[derive(Eq, PartialEq, Debug)]
pub struct FixMessage<'a> {
    pub msg_type: &'a [u8],
    pub body: &'a [u8],
    pub header_checksum: Wrapping<u8>,
}

fn parse_header_field(id: u64, input: &[u8]) -> Result<(FixField, &[u8]), &'static str> {
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
        },
        Err(e) => Err(e),
    }
}

pub fn parse_fix_message(input: &[u8]) -> Result<FixMessage, &'static str> {
    const CHECKSUM_LENGTH: usize = 7;

    if input.len() == 0 {
        return Err("FIX received empty message");
    }

    let (version, input) = parse_header_field(VERSION_ID, input)?;
    let (length, input) = parse_header_field(LENGTH_ID, input)?;

    if parse_int::<usize>(length.value)? + CHECKSUM_LENGTH != input.len() {
        return Err("FIX validation: invalid length");
    }

    let (msg_type, input) = parse_header_field(MSG_TYPE_ID, input)?;

    Ok(FixMessage {
        msg_type: msg_type.value,
        body: input,
        header_checksum: version.checksum + length.checksum + msg_type.checksum,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::common::checksum;
    use super::super::common::to_fix;

    #[test]
    fn test_header() {
        let msg = to_fix("8=FIX.4.2|9=10|35=A|34=1|10=123|");
        let result = parse_fix_message(&msg);
        assert_eq!(result, Ok(FixMessage {
            msg_type: b"A",
            body: &to_fix("34=1|10=123|"),
            header_checksum: checksum(&to_fix("8=FIX.4.2|9=10|35=A|")),
        }));
    }

    #[test]
    fn test_error_empty() {
        assert!(parse_fix_message(&to_fix("")).is_err());
    }

    #[test]
    fn test_error_no_length() {
        assert!(parse_fix_message(&to_fix("8=FIX.4.2|")).is_err());
    }

    #[test]
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