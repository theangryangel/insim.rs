//! Chat commands

use kitcar::chat::Parse;

// Just derive and you're done!
#[derive(Debug, PartialEq, kitcar::chat::Parse)]
#[chat(prefix = '!')]
/// Chat Commands
pub enum Chat {
    /// Only valid during Idle scene - start the game
    Start,
    /// Echo a string back from the server
    Echo { message: String },
    /// Help
    Help,
}

impl Chat {
    /// Respond to global commands
    pub fn spawn(insim: insim::builder::SpawnedHandle) {
        let _ = tokio::spawn(async move {
            let result: anyhow::Result<()> = async {
                let mut packets = insim.subscribe();

                loop {
                    if let insim::Packet::Mso(mso) = packets.recv().await? {
                        match Self::parse(mso.msg_from_textstart()) {
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
