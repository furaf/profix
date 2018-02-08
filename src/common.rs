use std::num::Wrapping;
use std::ops::MulAssign;
use std::ops::AddAssign;
use super::ParseError;

pub fn checksum(input: &[u8]) -> Wrapping<u8> {
    let mut sum = Wrapping(0u8);
    for &c in input {
        sum += Wrapping(c as u8);
    }
    sum
}

pub fn parse_int<T>(value: &[u8]) -> Result<T, ParseError>
where T: From<u8> + MulAssign + AddAssign + Copy,
{
    let value_len = value.len();
    if value_len == 0 {
        return Err("FIX could not parse integer from empty string");
    }
    let c = value[0];
    let Wrapping(digit) = Wrapping(c) - Wrapping('0' as u8);
    if digit > 9u8 {
        return Err("FIX could not parse integer - invalid initial digit")
    }
    let mut result = From::from(digit);
    let ten: T = From::from(10u8);
    for k in 1..value_len {
        let c = value[k];
        let Wrapping(digit) = Wrapping(c) - Wrapping('0' as u8);
        if digit > 9u8 {
            return Err("FIX could not parse length value")
        }
        result *= ten;
        result += From::from(digit);
    }
    Ok(result)
}

#[cfg(test)]
pub fn to_fix(s: &str) -> Vec<u8> {
    s.replace('|', "\x01").as_bytes().iter().map(|a|*a).collect()
}
