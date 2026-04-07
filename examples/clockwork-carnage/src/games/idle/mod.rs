//! IdleGame — runs between scheduled events.

pub mod chat;

use kitcar::{scenes::SceneError, ui};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use super::MiniGameCtx;
use crate::{ChatError, hud::topbar};

struct IdleView;

impl ui::Component for IdleView {
    type Props<'a> = (&'a (), &'a ());
    type Message = ();

    fn render(&self, _: Self::Props<'_>) -> ui::Node<Self::Message> {
        ui::container()
            .flex()
            .flex_col()
            .w(200.)
            .with_child(topbar("No event currently in progress"))
    }
}

pub struct IdleGuard {
    chat_handle: JoinHandle<Result<(), ChatError>>,
}

impl Drop for IdleGuard {
    fn drop(&mut self) {
        self.chat_handle.abort();
    }
}

pub async fn run(ctx: &MiniGameCtx, cancel: CancellationToken) -> Result<(), SceneError> {
    let (_, chat_handle) = chat::spawn(ctx.insim.clone(), ctx.pool.clone());
    let _guard = IdleGuard { chat_handle };

    let (_ui, _ui_handle) = ui::mount(ctx.insim.clone(), (), |_ucid, _| IdleView);

    cancel.cancelled().await;
    Ok(())
}
