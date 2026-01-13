use std::collections::HashMap;

use insim::{
    Packet,
    identifiers::{ClickId, ConnectionId},
    insim::{Bfn, Btn},
};
use kitcar::ui::ClickIdPool;

use super::{Node, View};

#[derive(Debug, Default)]
pub struct CanvasDiff {
    pub update: Vec<Btn>,
    pub remove: Vec<Bfn>,
}

impl CanvasDiff {
    pub fn merge(self) -> Vec<Packet> {
        self.remove
            .into_iter()
            .map(Packet::from)
            .chain(self.update.into_iter().map(Packet::from))
            .collect()
    }
}

#[derive(Debug)]
pub struct Canvas<V: View> {
    ucid: ConnectionId,
    pool: ClickIdPool,
    // map stable hash (u64) -> leased click id
    // persists between frames.
    active: HashMap<u64, ClickId>,
    clicks: HashMap<ClickId, V::Message>,
    blocked: bool,
}

impl<V: View> Canvas<V> {
    pub fn new(ucid: ConnectionId) -> Self {
        Self {
            ucid,
            pool: ClickIdPool::new(),
            active: Default::default(),
            clicks: Default::default(),
            blocked: false,
        }
    }

    pub fn paint(&mut self, root: &Node<V::Message>) -> Option<CanvasDiff> {
        if self.blocked {
            return None;
        }
        // TODO: taffy layout, diffing, etc.
        todo!()
    }

    pub fn block(&mut self) {
        self.blocked = true;
        self.active.clear();
        self.clicks.clear();
    }

    pub fn unblock(&mut self) {
        self.blocked = false;
    }

    pub fn translate_clickid(&self, clickid: &ClickId) -> Option<V::Message> {
        self.clicks.get(clickid).cloned()
    }
}
