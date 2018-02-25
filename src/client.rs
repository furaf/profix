use std::net::TcpStream;
use std::fmt::Debug;
use std::io::Error;
use std::io::{Read, Write};
use std;

use native_tls::TlsStream;

use detail::FixSerializable;
use serialize;
use FixHeader;

pub struct FixClient {
    stream: TlsStream<TcpStream>,

    send_seq_num: u64,
    rcv_seq_num: u64,

    sender_comp_id : String,
    target_comp_id : String,
}

impl FixClient {
    pub fn new(sender_comp_id : &str, target_comp_id : &str, stream: TlsStream<TcpStream>) -> FixClient {
        FixClient {
            stream,
            send_seq_num: 1u64,
            rcv_seq_num: 1u64,

            sender_comp_id : sender_comp_id.to_owned(),
            target_comp_id : target_comp_id.to_owned(),
        }
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
            let underlying = self.stream.get_ref();

            underlying.peek(&mut peek_buffer)?;
        }

        self.stream.read(&mut buf)
    }

    pub fn log_send(serialized : &str) {
        println!(">> {}", serialized);
        info!(">> {}", serialized);
    }

    pub fn log_rcv(buff : &[u8], size : usize) {
        if let Ok(as_str) = std::str::from_utf8(buff) {
            println!("<< {}", &as_str[0..size]);
            info!("<< {}", &as_str[0..size]);
        } else {
            error!("couldnt view rcv as utf8?");
        }
    }

    pub fn validate_msg<T : FixHeader>(&mut self, m : &T) -> Result<(),&'static str> {
        if m.seq() != self.rcv_seq_num {
            Err("rcv seq num out of order")
        } else if m.sender() != self.target_comp_id {
            Err("Sender comp id mismatch")
        } else if m.target() != self.sender_comp_id {
            Err("Sender comp id mismatch")
        } else {

            self.rcv_seq_num += 1;
            Ok(())
        }
    }
}