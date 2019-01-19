#[macro_use]
extern crate log;

extern crate env_logger;

extern crate profix;

mod messages;

use std::net::TcpStream;
use std::net::TcpListener;

use std::time::Duration;
use std::time::Instant;

use std::thread::spawn;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;

use profix::*;

use messages::*;

#[derive(Debug, PartialEq, FixDeserialize)]
enum ExampleSessionMessage {
    LogonReq(LogonReq)
}

#[derive(Debug, PartialEq, FixDeserialize)]
enum ExampleAppMessage {
    ExecReport(ExecReportResp),
    MassQuote(MassQuote),
}

#[derive(Debug)]
enum Action {
    SendMarketOrder
}

#[derive(Debug)]
enum HandlerFeedback {
    OrderPlaced,
}

struct ExampleHandler {
    is_logged : bool,

    tx : Sender<HandlerFeedback>,

    messages_this_second : i32,
    this_second : Instant,
}

impl profix::FixHandler<ExampleSessionMessage, ExampleAppMessage, Action> for ExampleHandler {
    fn handle_session(&mut self, client: &mut FixClient, msg: ExampleSessionMessage) -> Result<(), HandleErr> {
        match msg {
            ExampleSessionMessage::LogonReq(_) => {
                self.is_logged = true;
                println!("Client has logged in.");

                let resp = LogonResp {
                    sending_time : Timestamp::now(),
                    seq : client.get_next_send_seq(),
                    sender : client.comp_ids().sender.clone(),
                    target : client.comp_ids().target.clone(),
                };

                client.send(&resp);
            },
        }

        Ok(())
    }

    fn handle_app(&mut self, client: &mut FixClient, msg: ExampleAppMessage) -> Result<(), HandleErr> {
        match msg {
            ExampleAppMessage::ExecReport(_) => {
                if let Err(e) = self.tx.send(HandlerFeedback::OrderPlaced) {
                    eprintln!("failure during sending feedback: {:?}", e);
                }
            },
            ExampleAppMessage::MassQuote(mq) => {
//                println!("hai");
                println!("Message rate: {}", self.messages_this_second);

                if Instant::now().duration_since(self.this_second) >= Duration::from_secs(1) {
                    println!("Message rate: {}", self.messages_this_second);

                    self.messages_this_second = 0;
                    self.this_second = Instant::now();
                }
//                println!("got mq!");
                let mqa = MassQuoteAck {
                    sending_time : Timestamp::now(),
                    seq : client.get_next_send_seq(),
                    sender : client.comp_ids().sender.clone(),
                    target : client.comp_ids().target.clone(),
                };

                client.send(&mqa);
                self.messages_this_second += 1;

            }
        }

        Ok(())
    }

    fn handle_action(&mut self, client: &mut FixClient, action: Action) {
        match action {
            Action::SendMarketOrder => {
                let req = NewMarketOrder {
                    seq : client.get_next_send_seq(),
                    sender: client.comp_ids().sender.clone(),
                    target: client.comp_ids().target.clone(),
                    sending_time: Timestamp::now(),
                    our_order_id: "1".to_string(),
                    symbol: "BTCUSD".to_string(),
                    side: Side::Buy,
                    size: "10".to_string(),
                    order_type: OrderType::Market,
                };

                client.send(&req);
            }
        }
    }

    fn is_logged(&self) -> bool {
        self.is_logged
    }
}

struct Factory {
    tx : Sender<HandlerFeedback>,

    listener : TcpListener,
}

impl profix::FixFactory<ExampleHandler> for Factory {
    fn connection_factory(&self) -> Result<FixClient, ConnectionFailure> {
        Ok(FixClient::new(CompIds { sender : "server".to_string(), target : "client".to_string() },
                          Box::new(PlainStreamWrapper::new(self.listener.incoming().next().expect("failed to finish establshin connection").expect("2") )),
        ))
    }

    fn handler_factory(&self) -> ExampleHandler {
        ExampleHandler {
            is_logged : false,
            tx : self.tx.clone(),

            this_second : Instant::now(),
            messages_this_second : 0,
        }
    }
}


fn main() {
    env_logger::init();

    info!("Starting server");
    let listener = TcpListener::bind("0.0.0.0:3213").expect("couldnt create server");

    let (action_tx, action_rx) = channel();
    let (feedback_tx, feedback_rx) = channel();

    let factory = Factory {
        tx : feedback_tx.clone(),
        listener,
    };

    profix::fix_loop(factory, action_rx);

    println!("Hello, world!");
}
