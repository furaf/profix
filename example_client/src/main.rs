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
    MassQuoteAck(MassQuoteAck),
}

#[derive(Debug)]
enum Action {
    SendMassQuote
}

#[derive(Debug)]
enum HandlerFeedback {
    LoggedIn,
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
                self.tx.send(HandlerFeedback::LoggedIn);
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
            ExampleAppMessage::MassQuoteAck(mqa) => {
//                println!("got mqa!");
//                let mq = MassQuote {
//                    sending_time : Timestamp::now(),
//                    seq : client.get_next_send_seq(),
//                    sender : client.comp_ids().sender.clone(),
//                    target : client.comp_ids().target.clone(),
//                };
//
//                client.send(&mq);
            },
        }

        Ok(())
    }

    fn handle_action(&mut self, client: &mut FixClient, action: Action) {
        match action {
            Action::SendMassQuote => {
                for i in 0..1000 {
                    let mq = MassQuote {
                        sending_time: Timestamp::now(),
                        seq: client.get_next_send_seq(),
                        sender: client.comp_ids().sender.clone(),
                        target: client.comp_ids().target.clone(),
                    };

                    client.send(&mq);
                };
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
        let mut client = FixClient::new(CompIds { sender : "client".to_string(), target : "server".to_string() },
                          Box::new(PlainStreamWrapper::new(TcpStream::connect("127.0.0.1:3213").expect("server not found."))));

        let logon = LogonReq {
            target : client.comp_ids().target.clone(),
            sender : client.comp_ids().sender.clone(),
            seq : client.get_next_send_seq(),
            sending_time : Timestamp::now(),
        };

        client.send(&logon);
        println!("Logon sent.");

        Ok(client)
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

    for feedback in feedback_rx {
        match feedback {
            HandlerFeedback::LoggedIn => {
                println!("Logged In");
                loop {
                    action_tx.send(Action::SendMassQuote).expect("sending action failure");
                    ::std::thread::sleep(::std::time::Duration::from_millis(500));
                }
            }
            HandlerFeedback::OrderPlaced => {
                println!("Order placed. yey, can exit.");
                break;
            },
        }
    }
    //
}
