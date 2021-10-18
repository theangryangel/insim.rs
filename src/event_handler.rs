use super::{client::Ctx, packets};

// TODO find a more fun name for EventHandler thats more fitting with racing.

#[allow(unused)]
pub trait EventHandler: Send + Sync {
    fn on_raw(&self, ctx: Ctx, data: packets::Insim) {}

    fn on_connect(&self, ctx: Ctx) {}
    fn on_disconnect(&self) {}
    fn on_timeout(&self) {}
}
