use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use insim::{
    Packet,
    identifiers::{ClickId, ConnectionId, RequestId},
    insim::{Bfn, BfnType, Btn},
};

use super::{Node, NodeKind, TypeInMapper, View, id_pool::ClickIdPool};

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

#[derive(Clone)]
struct ButtonBinding<Msg> {
    click: Option<Msg>,
    typein: Option<TypeInMapper<Msg>>,
}

pub(super) struct Canvas<V: View> {
    ucid: ConnectionId,
    pool: ClickIdPool,
    // map stable hash -> button state (click id + rendered hash for diffing)
    buttons: HashMap<u64, ButtonState>,
    click_map: HashMap<ClickId, ButtonBinding<V::Message>>,
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

    #[allow(clippy::too_many_arguments)]
    fn visit<M: Clone + 'static>(
        node: Node<M>,
        parent_hash: u64,
        ucid: ConnectionId,
        buttons: &HashMap<u64, ButtonState>,
        pool: &mut ClickIdPool,
        new_buttons: &mut HashMap<u64, ButtonState>,
        tree: &mut taffy::TaffyTree,
        node_map: &mut Vec<(taffy::NodeId, u64, Btn)>,
        click_map: &mut HashMap<ClickId, ButtonBinding<M>>,
    ) -> Option<taffy::NodeId> {
        match node.kind {
            NodeKind::Container(Some(children)) => {
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
                    .new_with_children(node.style.unwrap_or_default(), &child_ids)
                    .expect("Could not add container to taffy layout");

                Some(node_id)
            },

            NodeKind::Container(None) => None,

            NodeKind::Button {
                text,
                msg,
                key,
                bstyle,
                typein,
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

                let typein_limit = typein.as_ref().map(|(limit, _)| *limit);
                let typein_mapper = typein.map(|(_, mapper)| mapper);

                if msg.is_some() || typein_mapper.is_some() {
                    let _ = click_map.insert(
                        click_id,
                        ButtonBinding {
                            click: msg,
                            typein: typein_mapper,
                        },
                    );
                }

                let node_id = tree
                    .new_leaf(node.style.unwrap_or_default())
                    .expect("Could not add a new child to taffy layout.. too many buttons?");
                node_map.push((
                    node_id,
                    stable_hash,
                    Btn {
                        text,
                        ucid,
                        reqi: RequestId(click_id.0),
                        clickid: click_id,
                        typein: typein_limit,
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
        self.click_map
            .get(clickid)
            .and_then(|binding| binding.click.clone())
    }

    pub(super) fn translate_typein_clickid(
        &self,
        clickid: &ClickId,
        text: String,
    ) -> Option<V::Message> {
        self.click_map
            .get(clickid)
            .and_then(|binding| binding.typein.as_ref())
            .map(|mapper| mapper(text))
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

#[cfg(test)]
mod tests {
    use insim::insim::BtnStyle;
    use tokio::sync::mpsc;

    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    enum TestMsg {
        Click,
        TypeIn(String),
    }

    struct TestView;

    impl View for TestView {
        type GlobalProps = ();
        type ConnectionProps = ();
        type Message = TestMsg;

        fn mount(_tx: mpsc::UnboundedSender<Self::Message>) -> Self {
            TestView
        }

        fn render(
            &self,
            _global_props: Self::GlobalProps,
            _connection_props: Self::ConnectionProps,
        ) -> Node<Self::Message> {
            Node::empty()
        }
    }

    #[test]
    fn test_canvas_diff_merge_empty() {
        let diff = CanvasDiff::default();
        let packets = diff.merge();
        assert!(packets.is_empty());
    }

    #[test]
    fn test_canvas_diff_merge_order() {
        let ucid = ConnectionId(1);
        let diff = CanvasDiff {
            update: vec![Btn {
                ucid,
                clickid: ClickId(1),
                text: "test".into(),
                ..Default::default()
            }],
            remove: vec![Bfn {
                ucid,
                subt: BfnType::DelBtn,
                clickid: ClickId(2),
                ..Default::default()
            }],
        };

        let packets = diff.merge();
        assert_eq!(packets.len(), 2);
        assert!(matches!(packets[0], Packet::Bfn(_)));
        assert!(matches!(packets[1], Packet::Btn(_)));
    }

    #[test]
    fn test_canvas_new() {
        let ucid = ConnectionId(5);
        let canvas = Canvas::<TestView>::new(ucid);
        assert_eq!(canvas.ucid, ucid);
        assert!(canvas.buttons.is_empty());
        assert!(canvas.click_map.is_empty());
    }

    #[test]
    fn test_reconcile_empty_tree_returns_none() {
        let mut canvas = Canvas::<TestView>::new(ConnectionId(1));
        let root: Node<TestMsg> = Node::empty();
        let diff = canvas.reconcile(root);
        assert!(diff.is_none());
    }

    #[test]
    fn test_reconcile_container_only_returns_none() {
        let mut canvas = Canvas::<TestView>::new(ConnectionId(1));
        let root: Node<TestMsg> = Node::container();
        let diff = canvas.reconcile(root);
        assert!(diff.is_none());
    }

    #[test]
    fn test_reconcile_single_button_returns_update() {
        let mut canvas = Canvas::<TestView>::new(ConnectionId(1));
        let root: Node<TestMsg> = Node::container()
            .w(200.0)
            .h(100.0)
            .with_child(Node::text("Hello", BtnStyle::default()).w(50.0).h(10.0));

        let diff = canvas.reconcile(root);
        assert!(diff.is_some());
        let diff = diff.unwrap();
        assert_eq!(diff.update.len(), 1);
        assert!(diff.remove.is_empty());
        assert_eq!(diff.update[0].text, "Hello");
    }

    #[test]
    fn test_reconcile_unchanged_returns_none() {
        let mut canvas = Canvas::<TestView>::new(ConnectionId(1));

        let make_tree = || {
            Node::container()
                .w(200.0)
                .h(100.0)
                .with_child(Node::text("Hello", BtnStyle::default()).w(50.0).h(10.0))
        };

        let diff1 = canvas.reconcile(make_tree());
        assert!(diff1.is_some());

        let diff2 = canvas.reconcile(make_tree());
        assert!(diff2.is_none());
    }

    #[test]
    fn test_reconcile_text_change_returns_update() {
        let mut canvas = Canvas::<TestView>::new(ConnectionId(1));

        let tree1: Node<TestMsg> = Node::container()
            .w(200.0)
            .h(100.0)
            .with_child(Node::text("Hello", BtnStyle::default()).w(50.0).h(10.0));

        let _ = canvas.reconcile(tree1);

        let tree2: Node<TestMsg> = Node::container()
            .w(200.0)
            .h(100.0)
            .with_child(Node::text("World", BtnStyle::default()).w(50.0).h(10.0));

        let diff = canvas.reconcile(tree2);
        assert!(diff.is_some());
        let diff = diff.unwrap();
        assert_eq!(diff.update.len(), 1);
        assert_eq!(diff.update[0].text, "World");
    }

    #[test]
    fn test_reconcile_removed_button_returns_removal() {
        let mut canvas = Canvas::<TestView>::new(ConnectionId(1));

        let tree1: Node<TestMsg> = Node::container().w(200.0).h(100.0).with_child(
            Node::text("Button1", BtnStyle::default())
                .w(50.0)
                .h(10.0)
                .key("btn1"),
        );

        let _ = canvas.reconcile(tree1);

        let tree2: Node<TestMsg> = Node::container().w(200.0).h(100.0);

        let diff = canvas.reconcile(tree2);
        assert!(diff.is_some());
        let diff = diff.unwrap();
        assert!(diff.update.is_empty());
        assert_eq!(diff.remove.len(), 1);
    }

    #[test]
    fn test_translate_clickid_known() {
        let mut canvas = Canvas::<TestView>::new(ConnectionId(1));

        let root: Node<TestMsg> = Node::container().w(200.0).h(100.0).with_child(
            Node::clickable("Click", BtnStyle::default(), TestMsg::Click)
                .w(50.0)
                .h(10.0),
        );

        let _ = canvas.reconcile(root);

        let click_id = ClickId(1);
        let msg = canvas.translate_clickid(&click_id);
        assert_eq!(msg, Some(TestMsg::Click));
    }

    #[test]
    fn test_translate_clickid_unknown() {
        let canvas = Canvas::<TestView>::new(ConnectionId(1));

        let click_id = ClickId(99);
        let msg = canvas.translate_clickid(&click_id);
        assert!(msg.is_none());
    }

    #[test]
    fn test_translate_typein_clickid_known() {
        let mut canvas = Canvas::<TestView>::new(ConnectionId(1));

        let root: Node<TestMsg> = Node::container().w(200.0).h(100.0).with_child(
            Node::text("Input", BtnStyle::default())
                .typein(32, TestMsg::TypeIn)
                .w(50.0)
                .h(10.0),
        );

        let _ = canvas.reconcile(root);

        let click_id = ClickId(1);
        let msg = canvas.translate_typein_clickid(&click_id, "hello".to_string());
        assert_eq!(msg, Some(TestMsg::TypeIn("hello".to_string())));
    }

    #[test]
    fn test_clear_resets_canvas() {
        let mut canvas = Canvas::<TestView>::new(ConnectionId(1));

        let root: Node<TestMsg> = Node::container().w(200.0).h(100.0).with_child(
            Node::clickable("Click", BtnStyle::default(), TestMsg::Click)
                .w(50.0)
                .h(10.0),
        );

        let _ = canvas.reconcile(root);
        assert!(!canvas.buttons.is_empty());
        assert!(!canvas.click_map.is_empty());

        canvas.clear();
        assert!(canvas.buttons.is_empty());
        assert!(canvas.click_map.is_empty());
    }

    #[test]
    fn test_reconcile_multiple_buttons() {
        let mut canvas = Canvas::<TestView>::new(ConnectionId(1));

        let root: Node<TestMsg> = Node::container()
            .w(200.0)
            .h(100.0)
            .flex()
            .flex_col()
            .with_child(Node::text("A", BtnStyle::default()).w(50.0).h(10.0))
            .with_child(Node::text("B", BtnStyle::default()).w(50.0).h(10.0))
            .with_child(Node::text("C", BtnStyle::default()).w(50.0).h(10.0));

        let diff = canvas.reconcile(root);
        assert!(diff.is_some());
        let diff = diff.unwrap();
        assert_eq!(diff.update.len(), 3);
    }

    #[test]
    fn test_keyed_buttons_maintain_identity() {
        let mut canvas = Canvas::<TestView>::new(ConnectionId(1));

        let tree1: Node<TestMsg> = Node::container()
            .w(200.0)
            .h(100.0)
            .flex()
            .flex_col()
            .with_child(
                Node::text("A", BtnStyle::default())
                    .w(50.0)
                    .h(10.0)
                    .key("a"),
            )
            .with_child(
                Node::text("B", BtnStyle::default())
                    .w(50.0)
                    .h(10.0)
                    .key("b"),
            );

        let _ = canvas.reconcile(tree1);
        let buttons_after_first = canvas.buttons.clone();

        let tree2: Node<TestMsg> = Node::container()
            .w(200.0)
            .h(100.0)
            .flex()
            .flex_col()
            .with_child(
                Node::text("B", BtnStyle::default())
                    .w(50.0)
                    .h(10.0)
                    .key("b"),
            )
            .with_child(
                Node::text("A", BtnStyle::default())
                    .w(50.0)
                    .h(10.0)
                    .key("a"),
            );

        let _ = canvas.reconcile(tree2);

        for (hash, state) in &buttons_after_first {
            if let Some(new_state) = canvas.buttons.get(hash) {
                assert_eq!(state.click_id, new_state.click_id);
            }
        }
    }
}
