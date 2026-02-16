use insim::{builder::InsimTask, identifiers::ConnectionId};
use kitcar::{presence, scenes, ui};

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

impl WaitForAdminStartView {
    fn new(tx: tokio::sync::mpsc::UnboundedSender<WaitForAdminStartMsg>) -> Self {
        Self {
            marquee: Marquee::new("Hello World!!!!!", 10, tx, |m| {
                WaitForAdminStartMsg::Marquee(m)
            }),
        }
    }
}

impl ui::Component for WaitForAdminStartView {
    type Props = ();
    type Message = WaitForAdminStartMsg;

    fn update(&mut self, msg: Self::Message) {
        match msg {
            WaitForAdminStartMsg::Marquee(m) => ui::Component::update(&mut self.marquee, m),
        }
    }

    fn render(&self, _props: Self::Props) -> ui::Node<Self::Message> {
        let m = self
            .marquee
            .render(())
            .map(WaitForAdminStartMsg::Marquee)
            .w(38.)
            .h(5.);

        topbar("No game in progress").with_child(m)
    }
}

impl ui::View for WaitForAdminStartView {
    type GlobalState = ();
    type ConnectionState = ();

    fn mount(tx: tokio::sync::mpsc::UnboundedSender<Self::Message>) -> Self {
        Self::new(tx)
    }

    fn compose(_global: Self::GlobalState, _connection: Self::ConnectionState) -> Self::Props {}
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
        let (_ui, _ui_handle) = ui::attach::<WaitForAdminStartView>(self.insim.clone(), ());

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
