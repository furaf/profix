use std::time::Duration;
use std::thread::sleep;
use std::fmt::Debug;
use std::sync::mpsc::{Receiver, Sender};
use std::str;
use std;

use metrics::PerfMetric;

use exchange::Action;

use FixFactory;
use FixClient;
use FixHandler;
use deserialize;
use detail::{FixDeserializable, FixSerializable};
use CompIds;

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}


pub fn fix_loop<Factory, Sess, App, H>(
    fix_factory: Factory,
    perf_sender: Sender<PerfMetric>,
    action_rx: Receiver<Action>,
) where
    Sess: FixDeserializable,
    App: FixDeserializable,
    H: FixHandler<Sess, App>,
    Factory: FixFactory<H>,
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

        loop {
            let mut hard_break = false;


            let mut resp_buffer_all = [0; 20000];

            match client.poll(&mut resp_buffer_all) {
                Ok(size) => {
                    let mut slice_begin = 0;
                    while let Some(pos) = find_subsequence(&resp_buffer_all[slice_begin..], "\x0110=".as_bytes()) {
                        let slice_end = slice_begin + pos + 8;
                        let resp_buffer = &resp_buffer_all[slice_begin..slice_end];
                        slice_begin = slice_end;

                        let as_vec = resp_buffer.to_vec();
                        println!("g2ot size of {:?} buffer now is: {}", size, unsafe { String::from_utf8_unchecked(as_vec) });

                        FixClient::log_rcv(&resp_buffer, slice_end);
                        match deserialize::<Sess>(&resp_buffer) {
                            Ok(resp) => {

                                if let Err(err) = handler.handle_session(&mut client, resp) {
                                    error!("something went wrong while handling sess message: {:?} msg: {:?}", err, str::from_utf8(&resp_buffer));
                                    break;
                                }
                                continue;
                            }
                            Err(_) => {}
                        }
                        match deserialize::<App>(&resp_buffer) {
                            Ok(msg) => {
                                if let Err(err) = handler.handle_app(&mut client, msg) {
                                    error!(
                                        "something went wrong while handling app message: {:?} err: {:?}",
                                        str::from_utf8(&resp_buffer), err
                                    );
                                    hard_break = true;
                                    break;
                                }
                                continue;
                            }
                            Err(err) => {
                                error!("failed to derialize :( {}", err);

                                hard_break = true;
                                break;
                            }
                        }

                        if slice_begin >= size {
                            break;
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

            if hard_break {
                break;
            }

            if handler.is_logged() {
                if let Ok(action) = action_rx.try_recv() {
                    info!("got something to do. {:?}", action);
                    handler.handle_action(&mut client, action);
                }
            }

            handler.poll(&mut client);

            sleep(Duration::new(0, 1000));
        }
        sleep(Duration::from_secs(10));
    }
}