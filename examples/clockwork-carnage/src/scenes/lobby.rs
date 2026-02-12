use std::time::Duration;

use insim::builder::InsimTask;
use kitcar::{
    presence,
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
            .with_child(topbar(&format!(
                "Warm up - {:?} remaining",
                props.remaining
            )))
    }
}

impl ui::View for ClockworkLobbyView {
    type GlobalState = Duration;
    type ConnectionState = ();

    fn mount(_tx: tokio::sync::mpsc::UnboundedSender<Self::Message>) -> Self {
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
    pub presence: presence::Presence,
}

impl Scene for Lobby {
    type Output = ();

    async fn run(self) -> Result<SceneResult<Self::Output>, SceneError> {
        tracing::info!("Lobby: 20 second warm up");
        let mut countdown = Countdown::new(Duration::from_secs(1), 1);
        let (ui, _ui_handle) = ui::attach::<ClockworkLobbyView>(
            self.insim.clone(),
            self.presence.clone(),
            Duration::ZERO,
        );

        let _chat_task = ui.update_from_broadcast(self.chat.subscribe(), |msg, _ucid| {
            matches!(msg, chat::ChatMsg::Help)
                .then_some(ClockworkLobbyMessage::Help(HelpDialogMsg::Show))
        });

        while let Some(_) = countdown.tick().await {
            let remaining = countdown.remaining_duration();
            ui.set_global_state(remaining);
        }

        Ok(SceneResult::Continue(()))
    }
}
