use crate::config::definition::Server;
use insim::client::prelude::*;
use miette::{IntoDiagnostic, Result};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub(crate) type Task = (JoinHandle<Result<()>>, Arc<crate::state::State>);

pub(crate) fn spawn(server: &Server) -> Result<Task> {
    let (insim_tx, mut insim_rx) = mpsc::unbounded_channel::<Event>();

    let state = Arc::new(crate::state::State::new(insim_tx.clone()));

    let conf = server.as_insim_config().into_diagnostic()?;

    let task_insim = tokio::task::spawn({
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

    Ok((task_insim, state))
}
