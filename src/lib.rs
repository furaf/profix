extern crate chrono;
#[cfg(test)]
extern crate quickcheck;
#[macro_use]
#[cfg(test)]
extern crate fix_derive;

mod message;
mod field;
mod serialize;
mod deserialize;
mod common;

pub type ParseError = &'static str;

pub use serialize::FixSerializable;
pub use deserialize::FixDeserializable;
pub use deserialize::FixParse;
pub use message::FixMessage;

pub use serialize::serialize;
pub use deserialize::deserialize;