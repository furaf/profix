use std;
use std::fmt::Debug;
use std::io::Error;
use std::io::{Read, Write};
use std::net::TcpStream;

use native_tls::TlsStream;

use detail::FixSerializable;
use serialize;
use CompIds;
use FixHeader;

pub trait Stream {
    fn get_mut(&mut self) -> &mut TcpStream;

    fn read(&mut self, buf : &mut [u8]) -> Result<usize, Error>;
    fn write_all(&mut self, buf : &[u8]) -> Result<(), Error>;
}

pub struct PlainStreamWrapper {
    stream: TcpStream,
}

impl PlainStreamWrapper {
    pub fn new(stream: TcpStream) -> PlainStreamWrapper {
        PlainStreamWrapper { stream }
    }
}

impl Stream for PlainStreamWrapper {
    fn get_mut(&mut self) -> &mut TcpStream {
        &mut self.stream
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.stream.read(buf)
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Error> {
        self.stream.write_all(buf)
    }
}

pub struct TlsStreamWrapper {
    stream: TlsStream<TcpStream>,
}

impl TlsStreamWrapper {
    pub fn new(stream: TlsStream<TcpStream>) -> TlsStreamWrapper {
        TlsStreamWrapper { stream }
    }
}

impl Stream for TlsStreamWrapper {
    fn get_mut(&mut self) -> &mut TcpStream {
        self.stream.get_mut()
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.stream.read(buf)
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Error> {
        self.stream.write_all(buf)
    }
}

pub struct FixClient {
    stream: Box<Stream>, //TlsStream<TcpStream>,
//    stream: TlsStream<TcpStream>,

    send_seq_num: u64,
    rcv_seq_num: u64,
    comp_ids: CompIds,
}

#[derive(Debug)]
pub enum MessageValidationErr {
    SeqNumOutOfOrder,
    SenderMismatch(String),
    TargetMismatch(String),
}

impl FixClient {
    pub fn new(comp_ids: CompIds, stream: Box<Stream>) -> FixClient {
        FixClient {
            stream,
            send_seq_num: 1u64,
            rcv_seq_num: 1u64,

            comp_ids,
        }
    }

    pub fn comp_ids(&self) -> &CompIds {
        &self.comp_ids
    }

    pub fn get_next_send_seq(&mut self) -> u64 {
        let seq = self.send_seq_num;
        self.send_seq_num += 1;

        seq
    }

    pub fn send<Msg: FixSerializable + Debug>(&mut self, msg: &Msg) {
        let fix_msg = serialize(msg);
        Self::log_send(&fix_msg);
        self.stream.write_all(fix_msg.as_bytes()).unwrap();
    }

    pub fn poll(&mut self, mut buf: &mut [u8]) -> Result<usize, Error> {
        {
            let mut peek_buffer = [0; 1];
            let underlying = self.stream.get_mut();

            underlying.peek(&mut peek_buffer)?;
        }

        self.stream.read(&mut buf)
    }

    pub fn log_send(serialized: &str) {
        //        println!(">> {}", serialized);
        info!(">> {}", serialized);
    }

    pub fn log_rcv(buff: &[u8], size: usize) {
        if let Ok(as_str) = std::str::from_utf8(buff) {
            //            println!("<< {}", &as_str);
            info!("<< {}", &as_str);
        } else {
            error!("couldnt view rcv as utf8?");
        }
    }

    pub fn validate_msg<T: FixHeader>(&mut self, m: &T) -> Result<(), MessageValidationErr> {
        if m.seq() != self.rcv_seq_num {
            Err(MessageValidationErr::SeqNumOutOfOrder)
        } else if m.sender() != self.comp_ids.target {
            Err(MessageValidationErr::SenderMismatch(format!(
                "expected {} got {}",
                self.comp_ids.target,
                m.sender()
            )))
        } else if m.target() != self.comp_ids.sender {
            Err(MessageValidationErr::TargetMismatch(format!(
                "expected {} got {}",
                self.comp_ids.sender,
                m.target()
            )))
        } else {
            self.rcv_seq_num += 1;
            Ok(())
        }
    }
}