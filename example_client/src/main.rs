extern crate profix;

mod messages;

use std::net::TcpStream;
use std::thread::spawn;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;

use profix::*;

use messages::*;

#[derive(Debug, PartialEq, FixDeserialize)]
enum ExampleSessionMessage {
    LogonResp(LogonResp)
}

#[derive(Debug, PartialEq, FixDeserialize)]
enum ExampleAppMessage {
    ExecReport(ExecReportResp),
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
}

impl profix::FixHandler<ExampleSessionMessage, ExampleAppMessage, Action> for ExampleHandler {
    fn handle_session(&mut self, _client: &mut FixClient, msg: ExampleSessionMessage) -> Result<(), HandleErr> {
        match msg {
            ExampleSessionMessage::LogonResp(_) => {
                self.is_logged = true;
            },
        }

        Ok(())
    }

    fn handle_app(&mut self, _client: &mut FixClient, msg: ExampleAppMessage) -> Result<(), HandleErr> {
        match msg {
            ExampleAppMessage::ExecReport(_) => {
                if let Err(e) = self.tx.send(HandlerFeedback::OrderPlaced) {
                    eprintln!("failure during sending feedback: {:?}", e);
                }
            },
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
}

impl profix::FixFactory<ExampleHandler> for Factory {
    fn connection_factory(&self) -> Result<FixClient, ConnectionFailure> {
        Ok(FixClient::new(CompIds { sender : "client".to_string(), target : "server".to_string() },
                          Box::new(PlainStreamWrapper::new(TcpStream::connect("127.0.0.1:3213").expect("server not found."))),
        ))
    }

    fn handler_factory(&self) -> ExampleHandler {
        ExampleHandler {
            is_logged : false,
            tx : self.tx.clone(),
        }
    }
}


fn main() {
    let (action_tx, action_rx) = channel();
    let (feedback_tx, feedback_rx) = channel();
    let _fix_thread = spawn(move || profix::fix_loop(Factory{ tx : feedback_tx.clone()}, action_rx));

    action_tx.send(Action::SendMarketOrder).expect("sending action failure");

    for feedback in feedback_rx {
        match feedback {
            HandlerFeedback::OrderPlaced => {
                println!("Order placed. yey, can exit.");
                break;
            },
        }
    }
    //
}
