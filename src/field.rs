use std::result::Result;
use std::num::Wrapping;

// Please only pass non-empty inputs
pub fn parse_fix_field(input: &[u8]) -> Result<FixField, &'static str> {
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

#[derive(Eq, PartialEq, Debug)]
pub struct FixField<'a> {
    pub id: u64,
    pub value: &'a [u8],
    pub length: usize,
    pub checksum: Wrapping<u8>,
}


#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::*;
    use super::super::common::checksum;
    use super::super::common::to_fix;

    #[test]
    fn test_valid() {
        let msg = to_fix("8=FIX|asdf");
        let result = parse_fix_field(&msg);
        assert_eq!(result, Ok(FixField{
            id: 8u64,
            value: b"FIX",
            length: 6usize,
            checksum: checksum(&to_fix("8=FIX|")),
        }));
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
}