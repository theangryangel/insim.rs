use std::time::Duration;

use insim::builder::InsimTask;
use kitcar::{
    scenes::{Scene, SceneError, SceneResult},
    time::Countdown,
    ui::{self, Component},
};

use crate::{
    chat,
    components::{HelpDialog, HelpDialogMsg, topbar},
};

#[derive(Clone, Debug)]
enum ClockworkLobbyMessage {
    Help(HelpDialogMsg),
}

struct ClockworkLobbyView {
    help_dialog: HelpDialog,
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
            return self.help_dialog.render(()).map(ClockworkLobbyMessage::Help);
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

impl ui::View for ClockworkLobbyView {
    type GlobalState = Duration;
    type ConnectionState = ();

    fn mount(_invalidator: ui::InvalidateHandle) -> Self {
        Self {
            help_dialog: HelpDialog::default(),
        }
    }

    fn compose(global: Self::GlobalState, _connection: Self::ConnectionState) -> Self::Props {
        ClockworkLobbyProps { remaining: global }
    }
}

/// Lobby scene - 20 second warm up period
#[derive(Clone)]
pub struct Lobby {
    pub insim: InsimTask,
    pub chat: chat::Chat,
}

impl Scene for Lobby {
    type Output = ();

    async fn run(self) -> Result<SceneResult<Self::Output>, SceneError> {
        tracing::info!("Lobby: 20 second warm up");
        let mut countdown = Countdown::new(Duration::from_secs(1), 20);
        let (ui, _ui_handle) = ui::attach_with::<ClockworkLobbyView, _, _>(
            self.insim.clone(),
            Duration::ZERO,
            self.chat.subscribe(),
            |(ucid, msg)| {
                matches!(msg, chat::ChatMsg::Help)
                    .then_some((ucid, ClockworkLobbyMessage::Help(HelpDialogMsg::Show)))
            },
        );

        while countdown.tick().await.is_some() {
            let remaining = countdown.remaining_duration();
            ui.set_global_state(remaining);
        }

        Ok(SceneResult::Continue(()))
    }
}
