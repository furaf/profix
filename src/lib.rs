#[cfg(test)]
extern crate quickcheck;

extern crate chrono;
#[macro_use]
extern crate log;
extern crate native_tls;

extern crate exchange;
extern crate metrics;

mod parsing;
mod serialization;
mod timestamp;
mod client;
mod handler;
mod factory;
mod fix_loop;

pub type ParseError = &'static str;

pub use serialization::serialize;
pub use serialization::deserialize;
pub use timestamp::Timestamp;
pub use client::FixClient;
pub use handler::{FixHandler, HandleErr};
pub use factory::{CompIds, ConnectionFailure, FixFactory};
pub use fix_loop::fix_loop;

pub trait FixParse: Sized {
    fn parse(value: &[u8]) -> Result<Self, ParseError>;
}

pub trait FixParseGroup: Sized {
    fn parse_group(value: &[u8]) -> Result<Vec<Self>, ParseError>;
}

pub trait FixHeader {
    fn seq(&self) -> u64;
    fn target(&self) -> &str;
    fn sender(&self) -> &str;
}

pub mod detail {
    use super::ParseError;

    pub use super::parsing::FixField;
    pub use super::parsing::FixMessage;
    pub use super::parsing::ParserContinuation;
    pub use super::parsing::parse_fix_field;
    pub use super::parsing::parse_fix_message;

    pub trait FixSerializable {
        fn serialize_body_to_fix(&self) -> String;
    }

    pub trait FixDeserializable: Sized {
        fn deserialize_from_fix(msg: FixMessage) -> Result<Self, ParseError>;
    }

    pub trait FixDeserializableGroup: Sized {
        fn deserialize_group_from_fix(
            expected_length: usize,
            input: &[u8],
        ) -> Result<(Vec<Self>, ParserContinuation), ParseError>;
    }

    pub trait FixMessageType {
        const MSG_TYPE: &'static [u8];
    }
}
