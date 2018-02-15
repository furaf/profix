use super::detail::FixSerializable;
use super::detail::FixDeserializable;
use super::detail::parse_fix_message;
use super::FixParse;
use super::ParseError;

use std::num::Wrapping;
use std::str;

#[inline]
pub fn serialize<T: FixSerializable>(t: &T) -> String {
    let body = t.serialize_body_to_fix();
    let header = format!("8=FIX.4.2\x019={}\x01", body.len());
    let chksum = checksum(header.as_bytes()) + checksum(body.as_bytes());
    format!("{}{}10={:03}\x01", header, body, chksum)
}

#[inline]
pub fn deserialize<T: FixDeserializable>(input: &[u8]) -> Result<T, ParseError> {
    let msg = parse_fix_message(input)?;
    FixDeserializable::deserialize_from_fix(msg)
}

pub fn checksum(input: &[u8]) -> Wrapping<u8> {
    let mut sum = Wrapping(0u8);
    for &c in input {
        sum += Wrapping(c as u8);
    }
    sum
}

impl<F: str::FromStr> FixParse for F {
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
