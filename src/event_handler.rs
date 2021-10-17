use super::{client::Ctx, packets};

pub trait EventHandler: Send + Sync {
    fn raw(&self, ctx: Ctx, data: packets::Insim) {}

    fn connected(&self, ctx: Ctx) {}
    fn disconnected(&self) {}
    fn timeout(&self) {}
}
