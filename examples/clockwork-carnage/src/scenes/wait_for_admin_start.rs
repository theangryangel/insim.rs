use insim::{builder::InsimTask, identifiers::ConnectionId};
use kitcar::{presence, scenes, ui};

use crate::{
    chat,
    components::{Marquee, topbar},
};

struct WaitForAdminStartView {
    marquee: Marquee,
}

impl WaitForAdminStartView {
    fn new(invalidator: ui::InvalidateHandle) -> Self {
        Self {
            marquee: Marquee::new("Hello World!!!!!", 10, invalidator),
        }
    }
}

impl ui::Component for WaitForAdminStartView {
    type Props = ();
    type Message = ();

    fn render(&self, _props: Self::Props) -> ui::Node<Self::Message> {
        let m = self.marquee.render(()).w(12.).h(5.);

        ui::container()
            .flex()
            .flex_col()
            .w(200.)
            .with_child(topbar("No game in progress").with_child(m))
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
        let (_ui, _ui_handle) = ui::mount(self.insim.clone(), (), |_ucid, invalidator| {
            WaitForAdminStartView::new(invalidator)
        });

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
