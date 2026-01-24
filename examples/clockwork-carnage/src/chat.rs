//! Chat commands

use insim::{identifiers::ConnectionId, insim::Mso};
use kitcar::{chat::Parse, presence, scenes::SceneError};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

// Just derive and you're done!
#[derive(Debug, Clone, PartialEq, kitcar::chat::Parse)]
#[chat(prefix = '!')]
/// Chat Commands
pub enum ChatMsg {
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
pub struct Chat {
    broadcast: broadcast::Sender<(ChatMsg, ConnectionId)>,
}

impl Chat {
    pub fn subscribe(&self) -> broadcast::Receiver<(ChatMsg, ConnectionId)> {
        self.broadcast.subscribe()
    }

    /// Wait for an admin to send a specific chat command.
    /// Reminder, you should probably pin this if its use in a loop { tokio::select! { .. } }
    pub async fn wait_for_admin_cmd<F>(
        &self,
        presence: presence::Presence,
        matches: F,
    ) -> Result<(), SceneError>
    where
        F: Fn(&ChatMsg) -> bool,
    {
        let mut chat = self.subscribe();

        loop {
            match chat.recv().await {
                Ok((msg, ucid)) if matches(&msg) => {
                    if let Some(conn) = presence.connection(&ucid).await {
                        if conn.admin {
                            return Ok(());
                        }
                    }
                },
                Ok(_) => {},
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    tracing::warn!("Chat commands lost due to lag");
                },
                Err(broadcast::error::RecvError::Closed) => {
                    return Err(SceneError::Custom {
                        scene: "wait_for_admin_cmd",
                        cause: Box::new(ChatError::HandleLost),
                    });
                },
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChatError {
    #[error("Insim: {0}")]
    Insim(#[from] insim::Error),
    #[error("Lost Insim Handle")]
    InsimHandleLost,
    #[error("Lost Chat chandle")]
    HandleLost,
}

/// Respond to commands globally and provide a bus
pub fn spawn(insim: insim::builder::InsimTask) -> (Chat, JoinHandle<()>) {
    let (tx, _rx) = broadcast::channel(100);

    let h = Chat {
        broadcast: tx.clone(),
    };

    let handle = tokio::spawn(async move {
        let result: Result<(), ChatError> = async {
            let mut packets = insim.subscribe();

            loop {
                if let insim::Packet::Mso(mso) = packets
                    .recv()
                    .await
                    .map_err(|_| ChatError::InsimHandleLost)?
                {
                    match ChatMsg::try_from(&mso) {
                        Ok(ChatMsg::Echo { message }) => {
                            insim
                                .send_message(format!("Echo: {}", message), mso.ucid)
                                .await?;
                        },
                        Ok(ChatMsg::Help) => {
                            insim.send_message("Available commands:", mso.ucid).await?;
                            for cmd in ChatMsg::help() {
                                insim.send_message(cmd, mso.ucid).await?;
                            }
                        },
                        Ok(o) => {
                            let _ = tx.send((o, mso.ucid)).map_err(|_| ChatError::HandleLost);
                        },
                        _ => {},
                    }
                }
            }
        }
        .await;

        if let Err(e) = result {
            tracing::error!("Chat background task failed: {:?}", e);
        }
    });

    (h, handle)
}
