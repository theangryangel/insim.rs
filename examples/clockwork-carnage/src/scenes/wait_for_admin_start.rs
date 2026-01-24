use insim::{builder::InsimTask, core::string::colours::Colourify, identifiers::ConnectionId};
use kitcar::{
    presence, scenes,
    ui::{self, Component},
};

use crate::{
    chat,
    components::{Marquee, MarqueeMsg, topbar},
};

#[derive(Clone, Debug)]
enum WaitForAdminStartMsg {
    Marquee(MarqueeMsg),
}

struct WaitForAdminStartView {
    marquee: Marquee,
}

impl ui::View for WaitForAdminStartView {
    type GlobalProps = ();
    type ConnectionProps = ();
    type Message = WaitForAdminStartMsg;

    fn mount(tx: tokio::sync::mpsc::UnboundedSender<Self::Message>) -> Self {
        Self {
            marquee: Marquee::new(&"Hello World!!!!!".white(), 10, tx, |m| {
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
    pub insim: InsimTask,
    pub presence: presence::Presence,
    pub chat: chat::Chat,
}

impl scenes::Scene for WaitForAdminStart {
    type Output = ();

    async fn run(self) -> Result<scenes::SceneResult<()>, scenes::SceneError> {
        let _ = ui::attach::<WaitForAdminStartView>(self.insim.clone(), self.presence.clone(), ());

        self.insim
            .send_message("Ready for admin !start command", ConnectionId::ALL)
            .await?;

        self.chat
            .wait_for_admin_cmd(self.presence.clone(), |msg| {
                matches!(msg, chat::ChatMsg::Start)
            })
            .await?;

        tracing::info!("Admin started game");
        Ok(scenes::SceneResult::Continue(()))
    }
}
