//! Challenge mode chat commands

use insim::{builder::InsimTask, insim::Mso};
use insim_extras::chat::Parse;
use tokio::task::JoinHandle;

use crate::ChatError;

#[derive(Debug, Clone, PartialEq, insim_extras::chat::Parse)]
#[chat(prefix = '!')]
/// Chat Commands
pub enum ChallengeChatMsg {
    /// Help
    Help,
    /// Show altitude tracker
    Alt,
}

pub type ChallengeChat = insim_extras::chat::Chat<ChallengeChatMsg>;

pub fn spawn(insim: InsimTask) -> (ChallengeChat, JoinHandle<Result<(), ChatError>>) {
    insim_extras::chat::spawn_with_handler(insim, 100, handle_challenge_chat)
}

async fn handle_challenge_chat(
    insim: InsimTask,
    mso: Mso,
    msg: ChallengeChatMsg,
) -> Result<(), ChatError> {
    if msg == ChallengeChatMsg::Help {
        insim.send_message("Available commands:", mso.ucid).await?;
        for cmd in ChallengeChatMsg::help() {
            insim.send_message(cmd, mso.ucid).await?;
        }
    }
    Ok(())
}
