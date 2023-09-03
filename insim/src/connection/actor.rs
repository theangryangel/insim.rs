use std::time::Duration;

use tokio::{
    sync::{broadcast, mpsc},
    time,
};

use super::{command::Command, event::Event, ConnectionOptions};

use crate::{
    error::Error,
    result::Result,
    tools::maybe_keepalive,
    traits::{ReadPacket, WritePacket},
};

pub(crate) const TIMEOUT_SECS: u64 = 90;

pub(crate) async fn run_actor(
    options: &ConnectionOptions,
    mut rx: mpsc::Receiver<Command>,
) -> Result<()> {
    let mut attempt: u64 = 0;

    loop {
        let (broadcast_rx, _) = broadcast::channel(32);

        // connect
        let isi = options.as_isi();
        let mut stream = match options
            .transport
            .connect(isi, Duration::from_secs(TIMEOUT_SECS))
            .await
        {
            Ok(stream) => {
                attempt = 0;
                stream
            }
            Err(_) => {
                (attempt, _) = attempt.overflowing_add(1);
                match options.reconnect.retry(&attempt) {
                    (true, Some(delay)) => {
                        time::sleep(delay).await;
                        continue;
                    }
                    (true, None) => {
                        continue;
                    }
                    _ => {
                        return Err(Error::Disconnected);
                    }
                }
            }
        };

        // TODO: how do we communicate connection state safely.
        // If we create a client it will connect before we can get a stream of events from
        // broadcast.
        broadcast_rx.send(Event::Connected).expect("????");

        loop {
            tokio::select! {

                res = rx.recv() => match res {
                    Some(Command::Send(packet)) => {
                        stream.write(packet).await?;
                    },
                    Some(Command::Firehose(respond_to)) => {
                        let _ = respond_to.send(broadcast_rx.subscribe());
                    },
                    Some(Command::Shutdown) => return Ok(()),
                    None => break,
                },

                res = time::timeout(
                    Duration::from_secs(TIMEOUT_SECS), stream.read()
                ) => {

                    // FIXME we canot just ?? we need to handle the disconnects, etc.
                    match res {

                        Err(e) => {
                            broadcast_rx.send(
                                Event::Disconnected
                            ).expect("????");

                            broadcast_rx.send(
                                Event::Error(e.into())
                            ).expect("????");
                            break;
                        },

                        Ok(inner) => match inner {
                            Ok(Some(packet)) => {
                                maybe_keepalive(&mut stream, &packet).await?;

                                broadcast_rx.send(
                                    Event::Data(packet)
                                ).expect("????");
                            },

                            Ok(None) => {
                                broadcast_rx.send(
                                    Event::Disconnected
                                ).expect("????");
                                break;
                            }

                            Err(e) => {
                                broadcast_rx.send(
                                    Event::Disconnected
                                ).expect("????");

                                broadcast_rx.send(
                                    Event::Error(e)
                                ).expect("????");

                                break;
                            }
                        },
                    }
                }

            }
        }
    }
}
