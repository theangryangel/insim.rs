//! Idle mode chat commands

use insim::{builder::InsimTask, insim::Mso};
use insim_extras::chat::Parse;

use crate::{ChatError, db};

#[derive(Debug, Clone, PartialEq, insim_extras::chat::Parse)]
#[chat(prefix = '!')]
/// Chat Commands
pub enum IdleChatMsg {
    /// Help - about Clockwork Carnage and available commands
    Help,
    /// Next - show the next scheduled event
    Next,
    /// Upcoming - show the next scheduled event
    Upcoming,
}

pub type IdleChat = insim_extras::chat::Chat<IdleChatMsg>;

pub fn spawn(
    insim: InsimTask,
    pool: db::Pool,
) -> (IdleChat, tokio::task::JoinHandle<Result<(), ChatError>>) {
    insim_extras::chat::spawn_with_handler(insim, 100, move |insim, mso, msg| {
        let pool = pool.clone();
        handle_idle_chat(insim, mso, msg, pool)
    })
}

async fn handle_idle_chat(
    insim: InsimTask,
    mso: Mso,
    msg: IdleChatMsg,
    pool: db::Pool,
) -> Result<(), ChatError> {
    match msg {
        IdleChatMsg::Help => {
            insim
                .send_message(
                    "Clockwork Carnage: a series of timed mini-game events. No event is running right now.",
                    mso.ucid,
                )
                .await?;
            insim.send_message("Available commands:", mso.ucid).await?;
            for cmd in IdleChatMsg::help() {
                insim.send_message(cmd, mso.ucid).await?;
            }
        },
        IdleChatMsg::Next | IdleChatMsg::Upcoming => match db::next_scheduled_event(&pool).await {
            Ok(Some((event, secs))) => {
                let mode = match &*event.mode {
                    db::EventMode::Metronome { .. } => "Metronome",
                    db::EventMode::Shortcut => "Shortcut",
                    db::EventMode::Bomb { .. } => "Bomb",
                };
                let name = event
                    .name
                    .as_deref()
                    .map(str::to_owned)
                    .unwrap_or_else(|| format!("{} / {}", event.track, event.layout));
                insim
                    .send_message(
                        format!("Next event: {name} ({mode}) in {}", format_secs(secs)),
                        mso.ucid,
                    )
                    .await?;
            },
            Ok(None) => {
                insim.send_message("No events scheduled.", mso.ucid).await?;
            },
            Err(e) => {
                tracing::warn!("Failed to query next scheduled event: {e}");
            },
        },
    }
    Ok(())
}

fn format_secs(secs: i64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    if h > 0 {
        format!("{h}h {m}m")
    } else {
        format!("{m}m")
    }
}
