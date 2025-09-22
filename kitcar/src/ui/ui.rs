use std::collections::HashMap;

use insim::{
    identifiers::{ClickId, ConnectionId},
    insim::{Bfn, BfnType, Btn},
};

use super::{component::Component, id_pool::ClickIdPool, vdom::Element};

#[derive(Debug, Default)]
pub struct UiDiff {
    pub to_update: Vec<Btn>,
    pub to_remove: Vec<Bfn>,
}

/// Ui for a single connection
#[derive(Debug)]
pub struct Ui<C>
where
    C: Component,
{
    id_pool: ClickIdPool,
    // The root component for this connection
    root_component: C,
    // Stable mapping from component keys to button IDs
    key_to_button_id: HashMap<String, ClickId>,
    // last layout and props
    last_layout: Option<HashMap<String, Btn>>,
    last_props: Option<C::Props>,
}

impl<C> Ui<C>
where
    C: Component,
{
    pub fn new(id_pool: ClickIdPool, root_component: C) -> Self {
        Self {
            id_pool,
            root_component,
            key_to_button_id: HashMap::new(),
            last_layout: None,
            last_props: None,
        }
    }

    pub fn render(&mut self, ucid: ConnectionId, props: &C::Props) -> Option<UiDiff> {
        let should_render = if let Some(old_props) = self.last_props.as_ref() {
            self.root_component.should_render(old_props, props)
        } else {
            true
        };

        if !should_render {
            self.last_props = Some(props.clone());
            return None;
        }

        let new_vdom = self.root_component.render(props);

        let mut taffy = taffy::TaffyTree::new();
        // key to taffy::NodeId
        let mut node_map = HashMap::new();

        let root = populate_taffy_and_map(&mut taffy, &mut node_map, &new_vdom);
        let _ = taffy.compute_layout(root, taffy::Size::length(200.0));

        // build renderable layout hashmap

        let new_layout: HashMap<String, Btn> = new_vdom
            .collect_renderable()
            .iter()
            .map(|(k, v)| {
                let node_id = node_map.get(&k.to_string()).unwrap();
                let (x, y) = get_taffy_abs_position(&taffy, node_id);

                let renderable = Btn {
                    l: x as u8,
                    t: y as u8,
                    w: v.width().unwrap(),
                    h: v.height().unwrap(),
                    text: v.text().to_string(),
                    bstyle: v.bstyle().unwrap().clone(),
                    ..Default::default()
                };
                (k.to_string(), renderable)
            })
            .collect();

        let last_layout = self.last_layout.take().unwrap_or_default();

        // find buttons to remove
        let to_remove = last_layout
            .iter()
            .filter_map(|(key, _btn)| {
                if !new_layout.contains_key(key) {
                    Some(Bfn {
                        ucid,
                        subt: BfnType::DelBtn,
                        // FIXME: dont unwrap
                        clickid: self.release_clickid(key).unwrap(),
                        ..Default::default()
                    })
                } else {
                    None
                }
            })
            .collect();

        let to_update = new_layout
            .iter()
            .filter_map(|(key, btn)| {
                match last_layout.get(key) {
                    None => {
                        // new button
                        let mut btn = btn.clone();
                        btn.clickid = self.lease_clickid(key).unwrap();
                        Some(btn)
                    },
                    Some(existing_btn) => {
                        if !matches!(existing_btn, _btn) {
                            let mut btn = btn.clone();
                            btn.clickid = self.lease_clickid(key).unwrap();
                            Some(btn.into())
                        } else {
                            None
                        }
                    },
                }
            })
            .collect();

        self.last_layout = Some(new_layout);
        self.last_props = Some(props.clone());

        Some(UiDiff {
            to_update,
            to_remove,
        })
    }

    fn lease_clickid(&mut self, key: &str) -> Option<ClickId> {
        // If we already have an ID for this key, reuse it
        if let Some(&existing_id) = self.key_to_button_id.get(key) {
            return Some(existing_id);
        }

        if let Some(new_id) = self.id_pool.lease() {
            let _ = self.key_to_button_id.insert(key.to_string(), new_id);
            return Some(new_id);
        }

        // TODO: is probably really an error
        None
    }

    fn release_clickid(&mut self, key: &str) -> Option<ClickId> {
        // If we already have an ID for this key, reuse it
        if let Some(&existing_id) = self.key_to_button_id.get(key) {
            self.id_pool.release(&[existing_id]);
            return Some(existing_id);
        }

        None
    }
}

fn populate_taffy_and_map(
    tree: &mut taffy::TaffyTree,
    node_map: &mut HashMap<String, taffy::NodeId>,
    vdom: &Element,
) -> taffy::NodeId {
    match vdom {
        Element::Container { children, style } => {
            let child_ids: Vec<taffy::NodeId> = children
                .into_iter()
                .map(|c| populate_taffy_and_map(tree, node_map, c))
                .collect();

            let taffy_id = tree.new_with_children(style.clone(), &child_ids).unwrap();
            taffy_id
        },
        Element::Button { key, style, .. } => {
            let taffy_id = tree.new_leaf(style.clone()).unwrap();
            let _ = node_map.insert(key.clone(), taffy_id);
            taffy_id
        },
        Element::Empty => {
            // TODO: how do we avoid this
            tree.new_leaf(taffy::Style::DEFAULT).unwrap()
        },
    }
}

// FIXME: we need to get this into the loop somewhere so that we dont need to redo it every
// time
fn get_taffy_abs_position(taffy: &taffy::TaffyTree, node_id: &taffy::NodeId) -> (f32, f32) {
    let mut current_node = node_id.clone();
    let mut absolute_location = (0.0, 0.0);

    loop {
        let layout = taffy.layout(current_node).unwrap();
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

    absolute_location
}
