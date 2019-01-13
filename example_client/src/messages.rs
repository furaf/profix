use std::fmt;

use profix::*;

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

#[derive(Debug, PartialEq, FixHeader, FixDeserialize)]
#[msg_type = "A"]
pub struct LogonResp {
    #[id = "34"]
    pub seq: u64,
    #[id = "49"]
    pub sender: String,
    #[id = "56"]
    pub target: String,
    #[id = "52"]
    pub sending_time: Timestamp,
}


#[derive(Debug, Clone, Copy, PartialEq, FixParse)]
pub enum OrderType {
    #[fix_value = "1"]
    Market,
    #[fix_value = "2"]
    Limit,
    #[fix_value = "3"]
    StopMarket,
    #[fix_value = "4"]
    StopLimit,
}

//todo remove me after enum serialization is done.
impl fmt::Display for OrderType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &OrderType::Market => write!(f, "1"),
            &OrderType::Limit => write!(f, "2"),
            &OrderType::StopMarket => write!(f, "3"),
            &OrderType::StopLimit => write!(f, "4"),
        }
    }
}

#[derive(Debug, PartialEq, FixHeader, FixDeserialize, FixSerialize)]
#[msg_type = "D"]
pub struct NewMarketOrder {
    #[id = "34"]
    pub seq: u64,
    #[id = "49"]
    pub sender: String,
    #[id = "56"]
    pub target: String,
    #[id = "52"]
    pub sending_time: Timestamp,

    #[id = "11"]
    pub our_order_id: String,
    #[id = "55"]
    pub symbol: String,
    #[id = "54"]
    pub side: Side,
    #[id = "38"]
    pub size: String,
    #[id = "40"]
    pub order_type: OrderType,
}

#[derive(Debug, PartialEq, FixParse)]
pub enum OrderStatus {
    #[fix_value = "0"]
    New,
    #[fix_value = "1"]
    PartiallyFilled,
    #[fix_value = "2"]
    Filled,
    #[fix_value = "3"]
    Done,
    #[fix_value = "4"]
    Canceled,
    #[fix_value = "7"]
    Stopped,
    #[fix_value = "8"]
    Rejected,
}

#[derive(Debug, PartialEq, FixParse)]
pub enum Side {
    #[fix_value = "1"]
    Buy,
    #[fix_value = "2"]
    Sell,
}

//todo remove me after enum serialization is done.
impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Side::Buy => write!(f, "1"),
            &Side::Sell => write!(f, "2"),
        }
    }
}

#[derive(Debug, PartialEq, FixParse)]
pub enum OrderRejectReason {
    #[fix_value = "0"]
    Unknown,
    #[fix_value = "3"]
    InsufficientFunds,
    #[fix_value = "8"]
    PostOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, FixParse)]
pub enum ExecType {
    #[fix_value = "0"]
    NewOrder,
    #[fix_value = "1"]
    Fill,
    #[fix_value = "3"]
    Done,
    #[fix_value = "4"]
    Cancelled,
    #[fix_value = "7"]
    Stopped,
    #[fix_value = "8"]
    Rejected,
    #[fix_value = "D"]
    OrderChanged,
    #[fix_value = "I"]
    OrderStatus,
}

#[derive(Debug, PartialEq, FixHeader, FixDeserialize)]
#[msg_type = "8"]
pub struct ExecReportResp {
    #[id = "34"]
    pub seq: u64,
    #[id = "49"]
    pub sender: String,
    #[id = "56"]
    pub target: String,
    #[id = "52"]
    pub sending_time: Timestamp,

    #[id = "150"]
    pub exec_type: ExecType,

    #[id = "11"]
    pub our_order_id: Option<String>,

    #[id = "37"]
    pub assigned_id: Option<String>,

    #[id = "55"]
    pub symbol: Option<String>,

    #[id = "54"]
    pub side: Side,

    #[id = "32"]
    //LastShares	Amount filled (if ExecType=1). Also called LastQty as of FIX 4.3
    pub qty_filled: Option<f64>,

    #[id = "44"]
    pub price: Option<f64>,

    #[id = "38"]
    pub original_qty: Option<f64>,

    #[id = "152"]
    //mo only.
    pub cash_qty: Option<f64>,

    #[id = "60"]
    pub transact_time: String,

    #[id = "39"]
    pub ord_status: Option<OrderStatus>,

    #[id = "103"]
    pub ord_rej_reason: Option<OrderRejectReason>,

    #[id = "136"]
    pub no_misc_fees: Option<String>,

    #[id = "137"]
    pub misc_fee_amt: Option<String>,

    #[id = "139"]
    pub misc_fee_type: Option<String>,

    #[id = "1003"]
    pub trade_id: Option<String>,
    #[id = "1057"]
    pub aggressor_indicator: Option<String>,
}