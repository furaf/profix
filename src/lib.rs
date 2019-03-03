#[cfg(test)]
extern crate quickcheck;

extern crate chrono;
#[macro_use]
extern crate log;
extern crate native_tls;

#[macro_use]
#[allow(unused_imports)]
extern crate profix_derive;
#[doc(hidden)]
pub use self::profix_derive::*;

mod client;
mod factory;
pub mod fix_loop;
mod handler;
mod parsing;
mod serialization;
mod timestamp;

pub mod profix;

pub type ParseError = &'static str;

pub use client::FixClient;
pub use client::PlainStreamWrapper;
pub use client::TlsStreamWrapper;
pub use factory::{CompIds, ConnectionFailure, FixFactory};
pub use fix_loop::fix_loop;
pub use handler::{FixHandler, HandleErr};
pub use serialization::deserialize;
pub use serialization::serialize;
pub use timestamp::Timestamp;

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

    pub use super::parsing::parse_fix_field;
    pub use super::parsing::parse_fix_message;
    pub use super::parsing::FixField;
    pub use super::parsing::FixMessage;
    pub use super::parsing::ParserContinuation;

    pub trait FixSerializable {
        fn serialize_body_to_fix(&self) -> String;
    }

    pub trait FixSerializableGroup {
        fn serialize_group_to_fix(&self) -> String;
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
