use std::marker::Sized;
use std::str;

use super::ParseError;
use super::message::FixMessage;
use super::message::parse_fix_message;

pub trait FixParse: Sized {
    fn parse(value: &[u8]) -> Result<Self, ParseError>;
}


impl <F: str::FromStr> FixParse for F {
    fn parse(value: &[u8]) -> Result<Self, ParseError> {
        let value = match str::from_utf8(value) {
            Ok(x) => x,
            Err(_) => return Err("Could not convert to UTF8"),
        };

        match value.parse() {
            Ok(v) => Ok(v),
            Err(_) => Err("Could not convert to target type"),
        }
    }
}

pub trait FixDeserializable: Sized {
    const MSG_TYPE: &'static str;
    fn deserialize_from_fix(msg: FixMessage) -> Result<Self, ParseError>;
}

#[inline]
pub fn deserialize<T: FixDeserializable>(input: &[u8]) -> Result<T, ParseError> {
    let msg = parse_fix_message(input)?;
    FixDeserializable::deserialize_from_fix(msg)
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::common::to_fix;

    #[derive(FixDeserialize, Debug, PartialEq)]
    #[msg_type = "A"]
    struct Message {
        #[id = "50"] seq: u64,
        #[id = "51"] value: f64,
    }

    #[test]
    fn test() {
        let fix = to_fix("8=FIX.4.2|9=18|35=A|50=5|51=1.23|10=038|");
        let msg = Message { seq: 5, value: 1.23 };
        assert_eq!(deserialize(&fix), Ok(msg));
    }
}
