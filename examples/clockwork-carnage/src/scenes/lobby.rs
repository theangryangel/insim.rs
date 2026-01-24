use std::time::Duration;

use insim::builder::InsimTask;
use kitcar::{
    presence,
    scenes::{Scene, SceneError, SceneResult},
    time::Countdown,
    ui,
};

use crate::components::topbar;

struct ClockworkLobbyView {}
impl ui::View for ClockworkLobbyView {
    type GlobalProps = Duration;
    type ConnectionProps = ();
    type Message = ();

    fn mount(_tx: tokio::sync::mpsc::UnboundedSender<Self::Message>) -> Self {
        Self {}
    }

    fn render(
        &self,
        global_props: Self::GlobalProps,
        _connection_props: Self::ConnectionProps,
    ) -> ui::Node<Self::Message> {
        topbar(&format!("Warm up - {:?} remaining", global_props))
    }
}

/// Lobby scene - 20 second warm up period
#[derive(Clone)]
pub struct Lobby {
    pub insim: InsimTask,
    pub presence: presence::Presence,
}

impl Scene for Lobby {
    type Output = ();

    async fn run(self) -> Result<SceneResult<Self::Output>, SceneError> {
        tracing::info!("Lobby: 20 second warm up");
        let mut countdown = Countdown::new(Duration::from_secs(1), 20);
        let (ui, _ui_handle) = ui::attach::<ClockworkLobbyView>(
            self.insim.clone(),
            self.presence.clone(),
            Duration::ZERO,
        );

        while let Some(_) = countdown.tick().await {
            let remaining = countdown.remaining_duration();
            ui.update_global_props(remaining);
        }

        Ok(SceneResult::Continue(()))
    }
}
