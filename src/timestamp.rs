use std::fmt;
use std::str;

use chrono::prelude::Local;
use chrono::NaiveDateTime;

use FixParse;
use ParseError;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Timestamp(pub NaiveDateTime);

impl Timestamp {
    pub fn now() -> Timestamp {
        Timestamp(Local::now().naive_utc())
    }
}

impl FixParse for Timestamp {
    fn parse(value: &[u8]) -> Result<Self, ParseError> {
        let value = match str::from_utf8(value) {
            Ok(value) => value,
            Err(_) => return Err("Could not parse timestamp because of UTF8"),
        };

        match NaiveDateTime::parse_from_str(value, "%Y%m%d-%H:%M:%S%.f") {
            Ok(t) => Ok(Timestamp(t)),
            Err(_) => Err("Could not parse timestamp (format: %Y%m%d-%H:%M:%S%.f)"),
        }
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.format("%Y%m%d-%H:%M:%S%.3f"))
    }
}
