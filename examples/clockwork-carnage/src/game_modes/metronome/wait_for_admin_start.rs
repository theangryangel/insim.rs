use insim::{builder::InsimTask, identifiers::ConnectionId};
use kitcar::{presence, scenes::{self, FromContext}, ui};

use super::chat;
use crate::components::{Marquee, topbar};

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
    pub chat: chat::EventChat,
}

impl<Ctx> scenes::Scene<Ctx> for WaitForAdminStart
where
    InsimTask: FromContext<Ctx>,
    presence::Presence: FromContext<Ctx>,
    Ctx: Sync,
{
    type Output = ();

    async fn run(self, ctx: &Ctx) -> Result<scenes::SceneResult<()>, scenes::SceneError> {
        let insim = InsimTask::from_context(ctx);
        let presence = presence::Presence::from_context(ctx);

        let (_ui, _ui_handle) = ui::mount(insim.clone(), (), |_ucid, invalidator| {
            WaitForAdminStartView::new(invalidator)
        });

        insim
            .send_message("Ready for admin !start command", ConnectionId::ALL)
            .await?;

        self.chat
            .wait_for_admin_cmd(presence, |msg| {
                matches!(msg, chat::EventChatMsg::Start)
            })
            .await?;

        tracing::info!("Admin started game");
        Ok(scenes::SceneResult::Continue(()))
    }
}
