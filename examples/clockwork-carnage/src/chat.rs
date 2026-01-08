//! Chat commands

use insim::insim::Mso;
use kitcar::chat::Parse;

use crate::ClockworkCarnageError;

// Just derive and you're done!
#[derive(Debug, PartialEq, kitcar::chat::Parse)]
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
}

impl Chat {
    /// Respond to global commands
    pub fn spawn(insim: insim::builder::SpawnedHandle) {
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
    }
}
