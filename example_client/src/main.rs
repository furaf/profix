#[macro_use]
extern crate profix;

use profix::Timestamp;

#[derive(Debug, PartialEq, FixHeader, FixSerialize)]
#[msg_type = "A"]
pub struct LogonReq {
    #[id = "34"]
    pub seq: u64,
    #[id = "49"]
    pub sender: String,
    #[id = "56"]
    pub target: String,
    #[id = "52"]
    pub sending_time: Timestamp,
    #[id = "98"]
    pub encrypt_method: u32,
    #[id = "108"]
    pub heartbeat_interval: u32,
    #[id = "554"]
    pub password: String,
    #[id = "96"]
    pub raw_data: String,
    #[id = "8013"]
    pub cancel_orders_on_disconnect: char,
}


fn main() {
    println!("Hello, world!");
}
