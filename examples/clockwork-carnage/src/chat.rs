//! Chat commands

use insim::{identifiers::ConnectionId, insim::Mso};
use kitcar::{chat::Parse, presence::PresenceHandle};
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use crate::ClockworkCarnageError;

// Just derive and you're done!
#[derive(Debug, Clone, PartialEq, kitcar::chat::Parse)]
#[chat(prefix = '!')]
/// Chat Commands
pub enum Chat {
    /// Only valid during Idle scene - start the game
    Start,
    /// Only valid during Event scene - stops the game
    End,
    /// Echo a string back from the server
    Echo { message: String },
    /// Help
    Help,
    /// Quit
    Quit,
}

#[derive(Debug, Clone)]
pub struct ChatHandle {
    broadcast: broadcast::Sender<(Chat, ConnectionId)>
}

impl ChatHandle {
    pub fn subscribe(&self) -> broadcast::Receiver<(Chat, ConnectionId)> {
        self.broadcast.subscribe()
    }
}

impl Chat {
    /// Respond to commands globally and provide a bus
    pub fn spawn(insim: insim::builder::SpawnedHandle, presence: PresenceHandle) -> (ChatHandle, CancellationToken) {
        let (tx, _rx) = broadcast::channel(100);

        let cancel_token = CancellationToken::new();
        let cancel_token2 = cancel_token.clone();
        let h = ChatHandle { broadcast: tx.clone() };

        let _ = tokio::spawn(async move {
            let result: Result<(), ClockworkCarnageError> = async {
                let mut packets = insim.subscribe();

                loop {
                    if let insim::Packet::Mso(mso) = packets.recv().await? {
                        match Self::try_from(&mso) {
                            Ok(Self::Echo { message }) => {
                                insim
                                    .send_message(format!("Echo: {}", message), mso.ucid)
                                    .await?;
                            },
                            Ok(Self::Help) => {
                                insim.send_message("Available commands:", mso.ucid).await?;
                                for cmd in Self::help() {
                                    insim.send_message(cmd, mso.ucid).await?;
                                }
                            },
                            Ok(Self::Quit) => {
                                if_chain::if_chain! {
                                    if let Some(conn_info) = presence.connection(&mso.ucid).await;
                                    if conn_info.admin;
                                    then {
                                        tracing::info!("Requested quit..");
                                        cancel_token2.cancel();
                                        return Ok(());
                                    }
                                }
                            },
                            Ok(o) => {
                                let _ = tx.send((o, mso.ucid))?;
                            },
                            _ => {}
                        }
                    }
                }
            }
            .await;

            if let Err(e) = result {
                tracing::error!("Chat background task failed: {:?}", e);
            }
        });

        (h, cancel_token)
    }
}
