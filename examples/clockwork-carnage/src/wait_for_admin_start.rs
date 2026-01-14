use insim::{builder::SpawnedHandle, core::string::colours::Colourify, identifiers::ConnectionId};
use kitcar::{
    presence, scenes,
    ui::{self, Component},
};
use tokio::sync::broadcast;

use crate::{chat, marquee, topbar::topbar};

#[derive(Clone, Debug)]
enum WaitForAdminStartMsg {
    Marquee(marquee::MarqueeMsg),
}

struct WaitForAdminStartView {
    marquee: marquee::Marquee,
}

impl ui::View for WaitForAdminStartView {
    type GlobalProps = ();
    type ConnectionProps = ();
    type Message = WaitForAdminStartMsg;

    fn mount(tx: tokio::sync::mpsc::UnboundedSender<Self::Message>) -> Self {
        Self {
            marquee: marquee::Marquee::new(&"Hello World!!!!!".white(), 10, tx, |m| {
                WaitForAdminStartMsg::Marquee(m)
            }),
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            WaitForAdminStartMsg::Marquee(m) => ui::Component::update(&mut self.marquee, m),
        }
    }

    fn render(
        &self,
        _global_props: Self::GlobalProps,
        _connection_props: Self::ConnectionProps,
    ) -> ui::Node<Self::Message> {
        let m = self
            .marquee
            .render(())
            .map(WaitForAdminStartMsg::Marquee)
            .w(38.)
            .h(5.);

        topbar("No game in progress").with_child(m)
    }
}

/// Wait for admin to start
#[derive(Clone)]
pub struct WaitForAdminStart {
    pub insim: SpawnedHandle,
    pub presence: presence::Presence,
    pub chat: chat::Chat,
}

impl scenes::Scene for WaitForAdminStart {
    type Output = ();

    async fn run(self) -> Result<scenes::SceneResult<()>, scenes::SceneError> {
        let _ui =
            ui::attach::<WaitForAdminStartView>(self.insim.clone(), self.presence.clone(), ());

        self.insim
            .send_message("Ready for admin !start command", ConnectionId::ALL)
            .await?;

        let mut chat = self.chat.subscribe();

        loop {
            match chat.recv().await {
                Ok((chat::ChatMsg::Start, ucid)) => {
                    if let Some(conn) = self.presence.connection(&ucid).await {
                        if conn.admin {
                            tracing::info!("Admin started game");
                            return Ok(scenes::SceneResult::Continue(()));
                        }
                    }
                },
                Ok(_) => {},
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    tracing::warn!("Chat commands lost due to lag");
                },
                Err(broadcast::error::RecvError::Closed) => {
                    return Err(scenes::SceneError::Custom {
                        scene: "WaitForAdminStart",
                        cause: Box::new(chat::ChatError::HandleLost),
                    });
                },
            }
        }
    }
}
