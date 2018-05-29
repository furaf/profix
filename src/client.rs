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
    fn tcp(&mut self) -> &mut TcpStream;
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
    fn tcp(&mut self) -> &mut TcpStream {
        &mut self.stream
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
    fn tcp(&mut self) -> &mut TcpStream {
        self.stream.get_mut()
    }
}

pub struct FixClient {
    stream: Box<Stream>, //TlsStream<TcpStream>,

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
        self.stream.tcp().write_all(fix_msg.as_bytes()).unwrap();
    }

    pub fn poll(&mut self, mut buf: &mut [u8]) -> Result<usize, Error> {
        {
            let mut peek_buffer = [0; 1];
            let underlying = self.stream.tcp();

            underlying.peek(&mut peek_buffer)?;
        }

        self.stream.tcp().read(&mut buf)
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
