//! Chat commands

use insim::{builder::InsimTask, insim::Mso};
use kitcar::chat::Parse;
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

/// Chat command bus.
pub type Chat = kitcar::chat::Chat<ChatMsg>;

/// Chat runtime error.
pub type ChatError = kitcar::chat::RuntimeError;

/// Respond to commands globally and provide a bus
pub fn spawn(insim: InsimTask) -> (Chat, JoinHandle<Result<(), ChatError>>) {
    kitcar::chat::spawn_with_handler(insim, 100, handle_chat_command)
}

async fn handle_chat_command(insim: InsimTask, mso: Mso, msg: ChatMsg) -> Result<(), ChatError> {
    match msg {
        ChatMsg::Echo { message } => {
            insim
                .send_message(format!("Echo: {message}"), mso.ucid)
                .await?;
        },
        ChatMsg::Help => {
            insim.send_message("Available commands:", mso.ucid).await?;
            for cmd in ChatMsg::help() {
                insim.send_message(cmd, mso.ucid).await?;
            }
        },
        _ => {},
    }
    Ok(())
}
