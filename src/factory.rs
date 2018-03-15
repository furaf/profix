use std::fmt::Debug;
use std::net::TcpStream;
use std::sync::mpsc::Sender;
use std;

use native_tls::TlsStream;
use native_tls;

use detail::FixSerializable;
use FixClient;

use metrics::PerfMetric;

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
    pub sender : String,
    pub target : String,
}

pub struct FixFactory<Logon : FixSerializable + Debug, Handler> {
    pub logon_factory : fn(&mut FixClient) -> Logon,
    pub connection_factory : fn() -> Result<TlsStream<TcpStream>, ConnectionFailure>,

    pub handler_factory : fn(perf_sneder : Sender<PerfMetric>) -> Handler,
}