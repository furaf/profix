use std;
use std::fmt::Debug;
use std::str;
use std::sync::mpsc::Receiver;
use std::thread::sleep;
use std::time::Duration;

use deserialize;
use detail::{FixDeserializable};
use FixClient;
use FixFactory;
use FixHandler;

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

pub fn fix_loop<Factory, Sess, App, H, Action>(
    fix_factory: Factory,
    action_rx: Receiver<Action>,
) where
    Sess: FixDeserializable + Debug,
    App: FixDeserializable + Debug,
    H: FixHandler<Sess, App, Action>,
    Factory: FixFactory<H>,
    Action: Debug,
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

        let mut handler = fix_factory.handler_factory();
        //        let logon = (fix_factory.logon_factory)(&mut client);
        //      client.send(&logon);

        loop {
            let mut hard_break = false;

            let mut resp_buffer_all = [0; 200000];

            match client.poll(&mut resp_buffer_all) {
                Ok(size) => {
                    FixClient::log_rcv(&resp_buffer_all, size);
                    let mut slice_begin = 0;
                    while let Some(pos) =
                        find_subsequence(&resp_buffer_all[slice_begin..], "\x0110=".as_bytes())
                    {
                        let slice_end = slice_begin + pos + 8;
                        let resp_buffer = &resp_buffer_all[slice_begin..slice_end];
                        slice_begin = slice_end;

                        let as_vec = resp_buffer.to_vec();

                        FixClient::log_rcv(&resp_buffer, slice_end);
                        match deserialize::<Sess>(&resp_buffer) {
                            Ok(resp) => {
                                info!("sess << {:?}", resp);
                                if let Err(err) = handler.handle_session(&mut client, resp) {
                                    error!("something went wrong while handling sess message: {:?} msg: {:?}", err, str::from_utf8(&resp_buffer));
                                    hard_break = true;
                                    break;
                                }
                                continue;
                            }
                            Err(_) => {}
                        }
                        match deserialize::<App>(&resp_buffer) {
                            Ok(msg) => {
                                info!("app << {:?}", msg);
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
