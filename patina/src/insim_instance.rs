use crate::{config::definition::Server as ServerConfig, state::State};
use futures::{SinkExt, StreamExt};
use insim::client::Event;
use miette::Result;
use std::sync::Arc;
use tokio::{sync::mpsc, task::JoinHandle};

pub(crate) struct Instance {
    pub(crate) config: ServerConfig,
    pub(crate) state: Arc<State>,
    pub(crate) handle: JoinHandle<Result<()>>,
}

impl From<&ServerConfig> for Instance {
    fn from(value: &ServerConfig) -> Self {
        // FIXME
        let conf = value.as_insim_config().unwrap();

        let (insim_tx, mut insim_rx) = mpsc::unbounded_channel::<Event>();

        // FIXME whats the easiest way for us to drop the insim_tx being passed to state?
        let state = Arc::new(crate::state::State::new(insim_tx));

        let handle = tokio::task::spawn({
            let state = state.clone();
            async move {
                let mut client = conf.into_client();

                loop {
                    tokio::select! {
                        msg = client.next() => match msg {
                            Some(msg) => {
                                state.handle_event(msg.clone())?;
                            },
                            None => {
                                break;
                            }
                        },

                        msg = insim_rx.recv() => match msg {
                            Some(msg) => match msg {
                                Event::Shutdown => {
                                    tracing::debug!("shuttingdown?");
                                    client.shutdown();
                                    break;
                                },
                                Event::Handshaking => unimplemented!(),
                                Event::Connected => unimplemented!(),
                                Event::Disconnected => unimplemented!(),
                                Event::Error(e) => {
                                    panic!("{}", e)
                                },
                                Event::Data(data) => {
                                    client.send(Event::Data(data)).await?;
                                },
                            },
                            None => {
                                break;
                            }
                        }
                    }
                }

                tracing::debug!("loop ended?");

                Ok(())
            }
        });

        Self {
            config: value.clone(),
            handle,
            state,
        }
    }
}
