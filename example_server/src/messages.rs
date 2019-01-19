use std::fmt;

use profix::*;

#[derive(Debug, PartialEq, FixHeader, FixDeserialize)]
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
}


#[derive(Debug, PartialEq, FixHeader, FixSerialize)]
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

    #[id = "98"]
    pub encrypt_method : char,
    #[id = "108"]
    pub heartbeat_interval: i32,
    #[id = "1137"]
    pub default_app_ver_id : char,
}



#[derive(Debug, PartialEq, FixHeader, FixDeserialize)]
#[msg_type = "0"]
pub struct Heartbeat {
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


#[derive(PartialEq, Debug)]
pub struct Hax<T>(pub Vec<T>);

impl<T> ::std::fmt::Display for Hax<T> where T: profix::detail::FixSerializableGroup {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f, "{}", self.0.len());

        for e in &self.0 {
            write!(f, "\x01{}", e.serialize_group_to_fix());
        };

        Ok(())
    }
}

#[derive(FixDeserializeGroup, FixSerializeGroup, Debug, PartialEq)]
pub struct PartyIdGroup {
    #[id = "448"]
    pub party_id: String,
    #[id = "447"]
    pub party_id_source: String,
    #[id = "452"]
    pub party_role: i32,
}

#[derive(FixDeserializeGroup, Debug, PartialEq)]
pub struct QuoteSetsGroup {
    #[id = "302"]
    pub quote_set_id: String,
    #[id = "304"]
    pub tot_no_quote_entries: i32,
    #[id = "295"]
    pub quote_entries: Vec<QuoteEntryGroup>,
}

#[derive(FixSerializeGroup, Debug, PartialEq)]
pub struct QuoteSetsGroupOut {
    #[id = "302"]
    pub quote_set_id: String,
    #[id = "304"]
    pub tot_no_quote_entries: i32,
    #[id = "295"]
    pub quote_entries: Hax<QuoteEntryGroupOut>,
}

#[derive(FixDeserializeGroup, FixSerializeGroup, Debug, PartialEq)]
pub struct QuoteEntryGroupMini {
    #[id = "55"]
    pub symbol: String,
    #[id = "48"]
    pub security_id: String,
    #[id = "22"]
    pub security_id_source: i32,
}

#[derive(FixDeserializeGroup, FixSerializeGroup, Debug, PartialEq)]
pub struct QuoteEntryGroup {
    #[id = "299"]
    pub quote_entry_id: String,
    #[id = "55"]
    pub symbol: String,
    #[id = "48"]
    pub security_id: String,
    #[id = "22"]
    pub security_id_source: i32,

    #[id = "132"]
    pub bid_price : f64,
    #[id = "133"]
    pub ask_price : f64,

    #[id = "134"]
    pub bid_size : f64,
    #[id = "135"]
    pub offer_size : f64,

//    #[id = "60"]
//    pub transact_time : Option<Timestamp>,
}

#[derive(FixDeserializeGroup, FixSerializeGroup, Debug, PartialEq)]
pub struct QuoteEntryGroupOut {
    #[id = "299"]
    pub quote_entry_id: String,
    #[id = "55"]
    pub symbol: String,
    #[id = "48"]
    pub security_id: String,
    #[id = "22"]
    pub security_id_source: i32,

    #[id = "132"]
    pub bid_price : f64,
    #[id = "133"]
    pub ask_price : f64,

    #[id = "134"]
    pub bid_size : f64,
    #[id = "135"]
    pub offer_size : f64,

    #[id = "60"]
    pub transact_time : Timestamp,
}


#[derive(Debug, PartialEq, FixHeader, FixDeserialize)]
#[msg_type = "i"]
pub struct MassQuote {
    #[id = "34"]
    pub seq: u64,
    #[id = "49"]
    pub sender: String,
    #[id = "56"]
    pub target: String,
    #[id = "52"]
    pub sending_time: Timestamp,

    #[id = "117"]
    pub quote_id : String,

    #[id = "453"]
    pub party_ids : Vec<PartyIdGroup>,
    #[id = "296"]
    pub quote_sets : Vec<QuoteSetsGroup>,
}

#[derive(Debug, PartialEq, FixHeader, FixSerialize)]
#[msg_type = "b"]
pub struct MassQuoteAck {
    #[id = "34"]
    pub seq: u64,
    #[id = "49"]
    pub sender: String,
    #[id = "56"]
    pub target: String,
    #[id = "52"]
    pub sending_time: Timestamp,

    #[id = "117"]
    pub quote_id : String,
    #[id = "297"]
    pub quote_status : i32,

    #[id = "60"]
    pub transact_time : Timestamp,

    #[id = "296"]
    pub quote_sets : Hax<QuoteSetsGroupOut>,
}
//
#[derive(Debug, PartialEq, FixHeader, FixDeserialize)]
#[msg_type = "Z"]
pub struct QuoteCancel {
    #[id = "34"]
    pub seq: u64,
    #[id = "49"]
    pub sender: String,
    #[id = "56"]
    pub target: String,
    #[id = "52"]
    pub sending_time: Timestamp,

    #[id = "131"]
    pub quote_req_id : String,

    #[id = "295"]
    pub quote_entries : Vec<QuoteEntryGroupMini>
}
//
#[derive(Debug, PartialEq, FixHeader, FixSerialize)]
#[msg_type = "AI"]
pub struct QuoteStatusReport {
    #[id = "34"]
    pub seq: u64,
    #[id = "49"]
    pub sender: String,
    #[id = "56"]
    pub target: String,
    #[id = "52"]
    pub sending_time: Timestamp,

    #[id = "131"]
    pub quote_req_id : String,
//    #[id = "117"]
//    pub quote_id: String,

    #[id = "55"]
    pub symbol: String,
    #[id = "48"]
    pub security_id: String,
    #[id = "22"]
    pub security_id_source: i32,

    #[id = "297"]
    pub quote_status : i32,

    #[id = "453"]
    pub party_id : Hax<PartyIdGroup>,

    #[id = "60"]
    pub transact_time : Timestamp,
}

#[cfg(test)]
mod messages_test {
    use super::*;

    pub fn to_fix(s : &str) -> Vec<u8> {
        s.replace('|', "\x01").as_bytes().iter().map(|a| *a).collect()
    }
    #[test]
    fn mass_quote_parse() {
        let mq = to_fix("8=FIXT.1.1|9=407|35=i|49=Fake1|56=FakeExchange|34=3|52=20190119-16:53:28.997|117=TC.D.A0.opt0.IP#mq_1547916808997138090_5|453=1|448=IG_MM|447=D|452=66|296=1|302=1|304=3|295=3|299=0|55=TC.D.A0.opt0.IP|48=TC.D.A0.opt0.IP|22=8|132=48.0|133=50.0|134=5.0|135=5.0|299=1|55=TC.D.A0.opt0.IP|48=TC.D.A0.opt0.IP|22=8|132=48.0|133=51.0|134=5.0|135=5.0|299=2|55=TC.D.A0.opt0.IP|48=TC.D.A0.opt0.IP|22=8|132=47.0|133=51.0|134=5.0|135=5.0|10=099|");

        let result : Result<MassQuote, _> = profix::deserialize(&mq);
        assert!(result.is_ok())
    }

    #[test]
    fn quote_cancel_parse() {
        let qc = to_fix("8=FIXT.1.1|9=170|35=Z|49=Fake1|56=FakeExchange|34=4|52=20190119-16:53:28.997|131=TC.D.A0.opt0.IP#mqc_5|298=1|453=1|448=IG_MM|447=D|452=66|295=1|55=TC.D.A0.opt0.IP|48=TC.D.A0.opt0.IP|22=8|10=042|");

        let result : Result<QuoteCancel, _> = profix::deserialize(&qc);
        assert!(result.is_ok())
    }

    #[test]
    fn quote_ack_serialize() {
        let ack = MassQuoteAck {
            target : "FakeExchange".into(),
            sender : "Fake1".into(),
            seq : 4,
            sending_time : Timestamp::now(),
            quote_id : "TC.D.A0.opt0.IP#mqc_5".into(),
            quote_sets : Hax(vec![
                QuoteSetsGroupOut {
                    quote_set_id : "1".into(),
                    tot_no_quote_entries : 1,
                    quote_entries : Hax(vec![
//                        QuoteEntryGroupOut {
//                            symbol : "TC.D.A0.opt0.IP".into(),
//                            security_id : "TC.D.A0.opt0.IP".into(),
//                            security_id_source : 8,
//                            ask_price : 10.0,
//                            bid_price : 5.0,
//                            offer_size : 10.0,
//                            bid_size : 5.0,
//                            quote_entry_id : "0".into(),
//                            transact_time : Timestamp::now(),
//                        },
                    ]),
                }]),
            quote_status : 0,
            transact_time : Timestamp::now(),
        };

        let serialized = profix::serialize(&ack);
        assert_eq!("", serialized);
    }

    #[test]
    fn quote_status_report_serialize() {
        let qs = QuoteStatusReport {
            target : "FakeExchange".into(),
            sender : "Fake1".into(),
            seq : 4,
            sending_time : Timestamp::now(),

            quote_status : 1,
            security_id : "DummySecurityID".into(),
            security_id_source : 8,
            symbol : "TC.D.A0.opt0.IP".into(),
            quote_id : "TC.D.A0.opt0.IP#mq_1547916808997138090_5".into(),
            quote_req_id : "TC.D.A0.opt0.IP#mqc_5".into(),

            party_id : Hax(vec![ PartyIdGroup {
                party_id : "1".into(),
                party_id_source : "D".into(),
                party_role : 66,

            } ]),

            transact_time : Timestamp::now(),
        };

        let serialized = profix::serialize(&qs);
    }
}