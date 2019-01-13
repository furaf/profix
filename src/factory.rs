use std;
use std::fmt::Debug;
use std::net::TcpStream;
use std::sync::mpsc::Sender;

use native_tls;
use native_tls::TlsStream;

use detail::FixSerializable;
use FixClient;

#[derive(Debug)]
pub enum ConnectionFailure {
    TlsError(native_tls::Error),
    TlsHandshakeError(native_tls::HandshakeError<std::net::TcpStream>),
    TcpStreamError(std::io::Error),
}

impl From<native_tls::Error> for ConnectionFailure {
    fn from(tls_error: native_tls::Error) -> ConnectionFailure {
        ConnectionFailure::TlsError(tls_error)
    }
}

impl From<native_tls::HandshakeError<std::net::TcpStream>> for ConnectionFailure {
    fn from(tls_error: native_tls::HandshakeError<std::net::TcpStream>) -> ConnectionFailure {
        ConnectionFailure::TlsHandshakeError(tls_error)
    }
}

impl From<std::io::Error> for ConnectionFailure {
    fn from(io_error: std::io::Error) -> ConnectionFailure {
        ConnectionFailure::TcpStreamError(io_error)
    }
}

#[derive(Clone)]
pub struct CompIds {
    pub sender: String,
    pub target: String,
}

pub trait FixFactory<Handler> {
    fn connection_factory(&self) -> Result<FixClient, ConnectionFailure>;
    fn handler_factory(&self) -> Handler;
}
