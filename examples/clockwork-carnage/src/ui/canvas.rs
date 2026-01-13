use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use insim::{
    Packet,
    identifiers::{ClickId, ConnectionId, RequestId},
    insim::{Bfn, BfnType, Btn},
};
use kitcar::ui::ClickIdPool;

use super::{Node, NodeKind, View};

#[derive(Debug, Default)]
pub(super) struct CanvasDiff {
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

#[derive(Debug, Clone, Copy)]
struct ButtonState {
    click_id: ClickId,
    rendered_hash: u64,
}

#[derive(Debug)]
pub(super) struct Canvas<V: View> {
    ucid: ConnectionId,
    pool: ClickIdPool,
    // map stable hash -> button state (click id + rendered hash for diffing)
    buttons: HashMap<u64, ButtonState>,
    click_map: HashMap<ClickId, V::Message>,
}

impl<V: View> Canvas<V> {
    pub(super) fn new(ucid: ConnectionId) -> Self {
        Self {
            ucid,
            pool: ClickIdPool::new(),
            buttons: Default::default(),
            click_map: Default::default(),
        }
    }

    pub(super) fn reconcile(&mut self, root: Node<V::Message>) -> Option<CanvasDiff> {
        let mut click_map = HashMap::new();
        let mut new_buttons = HashMap::new();

        let mut tree = taffy::TaffyTree::new();
        let mut node_map = Vec::new();

        // start traversal with a seed hash (0)
        let root_id = Self::visit(
            root,
            0,
            self.ucid,
            &self.buttons,
            &mut self.pool,
            &mut new_buttons,
            &mut tree,
            &mut node_map,
            &mut click_map,
        );

        if let Some(root_id) = root_id {
            tree.compute_layout(root_id, taffy::Size::length(200.0))
                .expect("taffy compute layout failed");
        }

        // identify ids that were allocated in previous frames but not seen in this one
        let dead_ids: Vec<ClickId> = self
            .buttons
            .iter()
            .filter(|(hash, _)| !new_buttons.contains_key(hash))
            .map(|(_, state)| state.click_id)
            .collect();

        // release dead ids
        if !dead_ids.is_empty() {
            self.pool.release(&dead_ids);
        }

        self.click_map = click_map;

        let mut updates = Vec::new();

        for (nodeid, stable_hash, mut btn) in node_map {
            let (x, y) = get_taffy_abs_position(&tree, &nodeid).expect("");
            let layout = tree.layout(nodeid).expect("");

            btn.l = x as u8;
            btn.t = y as u8;
            btn.w = layout.size.width as u8;
            btn.h = layout.size.height as u8;

            // hash the relevant fields for diffing
            let mut hasher = DefaultHasher::new();
            btn.text.hash(&mut hasher);
            btn.l.hash(&mut hasher);
            btn.t.hash(&mut hasher);
            btn.w.hash(&mut hasher);
            btn.h.hash(&mut hasher);
            btn.bstyle.hash(&mut hasher);
            let rendered_hash = hasher.finish();

            // only update if changed
            let prev_state = self.buttons.get(&stable_hash);
            if prev_state.map(|s| s.rendered_hash) != Some(rendered_hash) {
                updates.push(btn);
            }

            // update the rendered hash in new_buttons
            if let Some(state) = new_buttons.get_mut(&stable_hash) {
                state.rendered_hash = rendered_hash;
            }
        }

        let removals: Vec<Bfn> = dead_ids
            .into_iter()
            .map(|clickid| Bfn {
                ucid: self.ucid,
                subt: BfnType::DelBtn,
                clickid,
                ..Default::default()
            })
            .collect();

        self.buttons = new_buttons;

        if updates.is_empty() && removals.is_empty() {
            None
        } else {
            Some(CanvasDiff {
                update: updates,
                remove: removals,
            })
        }
    }

    fn visit<M: Clone>(
        node: Node<M>,
        parent_hash: u64,
        ucid: ConnectionId,
        buttons: &HashMap<u64, ButtonState>,
        pool: &mut ClickIdPool,
        new_buttons: &mut HashMap<u64, ButtonState>,
        tree: &mut taffy::TaffyTree,
        node_map: &mut Vec<(taffy::NodeId, u64, Btn)>,
        click_map: &mut HashMap<ClickId, M>,
    ) -> Option<taffy::NodeId> {
        match node.kind {
            NodeKind::Container(children) => {
                let child_ids: Vec<taffy::NodeId> = children
                    .into_iter()
                    .enumerate()
                    .filter_map(|(idx, child)| {
                        // mix index into parent hash to differentiate siblings
                        let mut hasher = DefaultHasher::new();
                        parent_hash.hash(&mut hasher);
                        idx.hash(&mut hasher);
                        let my_hash = hasher.finish();

                        Self::visit(
                            child,
                            my_hash,
                            ucid,
                            buttons,
                            pool,
                            new_buttons,
                            tree,
                            node_map,
                            click_map,
                        )
                    })
                    .collect();

                let node_id = tree
                    .new_with_children(node.style, &child_ids)
                    .expect("Could not add container to taffy layout");

                Some(node_id)
            },

            NodeKind::Button {
                text,
                msg,
                key,
                mut bstyle,
            } => {
                // calculate stable identity
                let mut hasher = DefaultHasher::new();
                parent_hash.hash(&mut hasher);

                if let Some(ref k) = key {
                    // strong identity: user provided a key (e.g. "user-123")
                    // we do not hash the index here, allowing the item to move
                    // in the list while keeping the same id.
                    k.hash(&mut hasher);
                } else {
                    // weak identity: fallback to text + "btn" marker
                    // note: in a container, the 'parent_hash' already includes the index,
                    // so this unique enough for static lists.
                    text.hash(&mut hasher);
                    "btn".hash(&mut hasher);
                }

                let stable_hash = hasher.finish();

                // allocate or reuse click id
                let click_id = if let Some(state) = buttons.get(&stable_hash) {
                    state.click_id
                } else if let Some(state) = new_buttons.get(&stable_hash) {
                    state.click_id
                } else {
                    match pool.lease() {
                        Some(new_id) => new_id,
                        None => unimplemented!("pool exhausted"),
                    }
                };

                // track this button (rendered_hash will be set after layout)
                let _ = new_buttons.insert(
                    stable_hash,
                    ButtonState {
                        click_id,
                        rendered_hash: 0,
                    },
                );

                if let Some(msg) = msg {
                    let _ = click_map.insert(click_id, msg);
                    bstyle = bstyle.clickable(); // force clickable if the user didnt do it
                }

                let node_id = tree
                    .new_leaf(node.style)
                    .expect("Could not add a new child to taffy layout.. too many buttons?");
                node_map.push((
                    node_id,
                    stable_hash,
                    Btn {
                        text,
                        ucid,
                        reqi: RequestId(click_id.0),
                        clickid: click_id,
                        bstyle,
                        ..Default::default()
                    },
                ));

                Some(node_id)
            },
            NodeKind::Empty => None,
        }
    }

    // should be called if the user clears buttons
    pub(super) fn clear(&mut self) {
        self.buttons.clear();
        self.click_map.clear();
        self.pool.release_all();
    }

    pub(super) fn translate_clickid(&self, clickid: &ClickId) -> Option<V::Message> {
        self.click_map.get(clickid).cloned()
    }
}

fn get_taffy_abs_position(taffy: &taffy::TaffyTree, node_id: &taffy::NodeId) -> Option<(f32, f32)> {
    let mut current_node = *node_id;
    let mut absolute_location = (0.0, 0.0);

    loop {
        let layout = taffy.layout(current_node).ok()?;
        absolute_location.0 += layout.location.x;
        absolute_location.1 += layout.location.y;

        // Get the parent of the current node
        let parent = taffy.parent(current_node);

        if let Some(parent_node) = parent {
            // If there is a parent, move up the tree
            current_node = parent_node;
        } else {
            // If there is no parent, we've reached the root
            break;
        }
    }

    Some(absolute_location)
}
