use std::time::Duration;

use insim::builder::InsimTask;
use kitcar::{
    scenes::{Scene, SceneError, SceneResult},
    time::Countdown,
    ui::{self, Component},
};

use crate::{
    chat,
    components::{Dialog, DialogMsg, DialogProps, topbar},
};

const EVENT_HELP_LINES: &[&str] = &[
    " - Match the target lap time as closely as possible.",
    " - Crossing the first checkpoint starts your timed attempt.",
    " - Find one of the finishes as close to the target time as possible.",
    " - Full contact is permitted.",
    " - Don't be a dick.",
    " - Lower delta ranks higher and earns more points.",
    " - Retry as many times as you want each round.",
    "",
    "Good luck.",
];

#[derive(Clone, Debug)]
enum ClockworkLobbyMessage {
    Help(DialogMsg),
}

struct ClockworkLobbyView {
    help_dialog: Dialog,
}

#[derive(Debug, Clone, Default)]
struct ClockworkLobbyProps {
    remaining: Duration,
}

impl ui::Component for ClockworkLobbyView {
    type Props = ClockworkLobbyProps;
    type Message = ClockworkLobbyMessage;

    fn update(&mut self, msg: Self::Message) {
        match msg {
            ClockworkLobbyMessage::Help(help_msg) => {
                Component::update(&mut self.help_dialog, help_msg);
            },
        }
    }

    fn render(&self, props: Self::Props) -> ui::Node<Self::Message> {
        if self.help_dialog.is_visible() {
            return self
                .help_dialog
                .render(DialogProps {
                    title: "Welcome to Clockwork Carnage",
                    lines: EVENT_HELP_LINES,
                })
                .map(ClockworkLobbyMessage::Help);
        }

        ui::container()
            .flex()
            .flex_col()
            .w(200.)
            .with_child(topbar(&format!(
                "Warm up - {:?} remaining",
                props.remaining
            )))
    }
}

impl From<ui::UiState<Duration, ()>> for ClockworkLobbyProps {
    fn from(state: ui::UiState<Duration, ()>) -> Self {
        Self {
            remaining: state.global,
        }
    }
}

/// Lobby scene - 20 second warm up period
#[derive(Clone)]
pub struct Lobby {
    pub insim: InsimTask,
    pub chat: chat::EventChat,
}

impl Scene for Lobby {
    type Output = ();

    async fn run(self) -> Result<SceneResult<Self::Output>, SceneError> {
        tracing::info!("Lobby: 20 second warm up");
        let mut countdown = Countdown::new(Duration::from_secs(1), 20);
        let (ui, _ui_handle) = ui::mount_with(
            self.insim.clone(),
            Duration::ZERO,
            |_ucid, _invalidator| ClockworkLobbyView {
                help_dialog: Dialog::default(),
            },
            self.chat.subscribe(),
            |(ucid, msg)| {
                matches!(msg, chat::EventChatMsg::Help)
                    .then_some((ucid, ClockworkLobbyMessage::Help(DialogMsg::Show)))
            },
        );

        while countdown.tick().await.is_some() {
            let remaining = countdown.remaining_duration();
            ui.set_global_state(remaining);
        }

        Ok(SceneResult::Continue(()))
    }
}
