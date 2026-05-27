use kitcar::{AppError, ChatEvent, Event, ui::Ui};

use crate::{
    components::DialogMsg,
    games::bomb::{BombConnectionProps, BombGlobal, BombMsg, BombView},
};

#[derive(Debug, Clone)]
pub(super) enum Cmd {
    Help,
}

impl std::str::FromStr for Cmd {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (head, _rest) = s.split_once(char::is_whitespace).unwrap_or((s, ""));
        match head {
            "help" => Ok(Cmd::Help),
            _ => Err(()),
        }
    }
}

pub(super) async fn handle_chat(
    Event(cmd): Event<ChatEvent<Cmd>>,
    ui: Ui<BombView, BombGlobal, BombConnectionProps>,
) -> Result<(), AppError> {
    match cmd.parsed {
        Cmd::Help => {
            let _ = ui.update(cmd.ucid, BombMsg::Help(DialogMsg::Show)).await;
        },
    }
    Ok(())
}
