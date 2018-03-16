use std::time::Duration;
use std::thread::sleep;
use std::fmt::Debug;
use std::sync::mpsc::{Sender,Receiver};
use std::str;
use std;

use metrics::PerfMetric;

use exchange::Action;

use FixFactory;
use FixClient;
use FixHandler;
use deserialize;
use detail::{FixSerializable, FixDeserializable};
use CompIds;

pub fn fix_loop<Factory, Sess, App, H>(fix_factory : Factory, perf_sender : Sender<PerfMetric>, action_rx : Receiver<Action>)
    where Sess : FixDeserializable,
          App : FixDeserializable,
          H : FixHandler<Sess, App>,
          Factory : FixFactory<H>
{
    loop {
        info!("initiating connection to gdax fix");
        let mut client = match fix_factory.connection_factory() {
            Ok(stream) => {
                info!("connected to gdax!");
                stream
            }

            Err(err) => {
                error!("connection to gdax failed with {:?}", err);
                sleep(Duration::from_secs(10));
                continue;
            }
        };

        let mut handler = fix_factory.handler_factory(perf_sender.clone());
//        let logon = (fix_factory.logon_factory)(&mut client);
  //      client.send(&logon);

        let mut resp_buffer = [0; 1000];
        loop {
            match client.poll(&mut resp_buffer) {
                Ok(size) => {
                    info!("got size of {:?}", size);
                    FixClient::log_rcv(&resp_buffer, size);
                    match deserialize::<Sess>(&resp_buffer) {
                        Ok(resp) => {
                            if let Err(err) = handler.handle_session(&mut client, resp) {
                                error!("something went wrong while handling sess message: {:?} msg: {:?}", err, str::from_utf8(&resp_buffer) );
                                break;
                            }
                            continue;
                        }
                        Err(_) => {}
                    }
                    match deserialize::<App>(&resp_buffer) {
                        Ok(msg) => {
                            if let Err(_) = handler.handle_app(&mut client, msg) {
                                error!("something went wrong while handling app message: {:?}", str::from_utf8(&resp_buffer) );
                                break;
                            }
                            continue;
                        }
                        Err(err) => {
                            println!("failed to derialize :( {}", err);
                        }
                    }

                }
                Err(err) => {
                    match err.kind() {
                        std::io::ErrorKind::WouldBlock => {
                            //its okay.
                        }
                        kind @ _ => {
                            println!("Unexpected poll result. resetting connection. {:?}", kind);
                            break;
                        }
                    }
                }
            };

            if handler.is_logged() {
                if let Ok(action) = action_rx.try_recv() {
                    info!("got something to do. {:?}", action);
                    handler.handle_action(&mut client, action);
                }
            }

            handler.poll(&mut client);

            sleep(Duration::new(0, 0));
        }

        sleep(Duration::from_secs(10));
    }
}