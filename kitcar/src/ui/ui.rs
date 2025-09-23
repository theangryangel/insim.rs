// ui.rs
use std::{collections::HashMap, fmt::Debug};

use insim::{
    identifiers::{ClickId, ConnectionId, RequestId},
    insim::{Bfn, BfnType, Btn},
    Packet,
};

use super::{id_pool::ClickIdPool, vdom::Element};

#[derive(Debug, Default)]
pub struct UiDiff {
    pub to_update: Vec<Btn>,
    pub to_remove: Vec<Bfn>,
}

impl UiDiff {
    pub fn into_merged(self) -> Vec<Packet> {
        self.to_remove
            .into_iter()
            .map(|x| Packet::from(x))
            .chain(self.to_update.into_iter().map(|x| Packet::from(x)))
            .collect()
    }
}

/// Ui for a single connection
#[derive(Debug)]
pub struct Ui<F, P>
where
    F: Fn(&P) -> Option<Element>,
    P: Clone + PartialEq,
{
    id_pool: ClickIdPool,
    // The root component for this connection
    render_fn: F,
    // Stable mapping from component keys to button IDs
    key_to_click_id: HashMap<String, ClickId>,
    click_id_to_key: HashMap<ClickId, String>,
    // last layout and props
    last_layout: Option<HashMap<String, Btn>>,
    last_props: Option<P>,
    ucid: ConnectionId,
}

impl<F, P> Ui<F, P>
where
    F: Fn(&P) -> Option<Element>,
    P: Clone + PartialEq,
{
    pub fn new(id_pool: ClickIdPool, ucid: ConnectionId, root_component: F) -> Self
    where
        F: Fn(&P) -> Option<Element> + 'static,
        P: Clone + PartialEq + 'static,
    {
        Self {
            id_pool,
            render_fn: root_component,
            key_to_click_id: HashMap::new(),
            click_id_to_key: HashMap::new(),
            last_layout: None,
            last_props: None,
            ucid,
        }
    }

    pub fn render(&mut self, props: &P) -> Option<UiDiff> {
        let should_render = if let Some(old_props) = self.last_props.as_ref() {
            old_props != props
        } else {
            true
        };

        if !should_render {
            self.last_props = Some(props.clone());
            return None;
        }

        let ucid = self.ucid;

        let new_vdom = (self.render_fn)(props);

        // Handle the case where render function returns None
        let new_vdom = match new_vdom {
            Some(vdom) => vdom,
            None => {
                // No UI to render - remove everything
                let last_layout = self.last_layout.take().unwrap_or_default();
                let to_remove = last_layout
                    .iter()
                    .map(|(key, _btn)| Bfn {
                        ucid,
                        subt: BfnType::DelBtn,
                        clickid: self.release_clickid(key).unwrap(), // FIXME: don't unwrap
                        ..Default::default()
                    })
                    .collect();

                self.last_props = Some(props.clone());

                return Some(UiDiff {
                    to_update: vec![],
                    to_remove,
                });
            },
        };

        let mut taffy = taffy::TaffyTree::new();
        let mut node_map = HashMap::new();

        let root = populate_taffy_and_map(&mut taffy, &mut node_map, &new_vdom);
        let _ = taffy.compute_layout(root, taffy::Size::length(200.0));

        let new_layout: HashMap<String, Btn> = new_vdom
            .collect_renderable()
            .iter()
            .map(|(k, v)| {
                let node_id = node_map.get(&k.to_string()).unwrap();
                let (x, y) = get_taffy_abs_position(&taffy, node_id);
                let layout = taffy.layout(*node_id).unwrap();

                let renderable = Btn {
                    l: x as u8,
                    t: y as u8,
                    w: layout.size.width as u8,
                    h: layout.size.height as u8,
                    text: v.text().to_string(),
                    bstyle: v.bstyle().unwrap().clone(),
                    ..Default::default()
                };
                (k.to_string(), renderable)
            })
            .collect();

        let last_layout = self.last_layout.take().unwrap_or_default();

        // Find buttons to remove
        let to_remove = last_layout
            .iter()
            .filter_map(|(key, _btn)| {
                if !new_layout.contains_key(key) {
                    Some(Bfn {
                        ucid,
                        subt: BfnType::DelBtn,
                        clickid: self.release_clickid(key).unwrap(), // FIXME: don't unwrap
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
                        // New button
                        let mut btn = btn.clone();
                        let clickid = self.lease_clickid(key).unwrap();
                        btn.clickid = clickid;
                        btn.reqi = RequestId(clickid.0);
                        Some(btn)
                    },
                    Some(existing_btn) => {
                        // Check if button actually changed
                        if existing_btn != btn {
                            let mut btn = btn.clone();
                            let clickid = self.lease_clickid(key).unwrap();
                            btn.clickid = clickid;
                            btn.reqi = RequestId(clickid.0);
                            Some(btn)
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
        if let Some(&existing_id) = self.key_to_click_id.get(key) {
            return Some(existing_id);
        }

        if let Some(new_id) = self.id_pool.lease() {
            let _ = self.key_to_click_id.insert(key.to_string(), new_id);
            let _ = self.click_id_to_key.insert(new_id, key.to_string());
            return Some(new_id);
        }

        // TODO: is probably really an error
        None
    }

    fn release_clickid(&mut self, key: &str) -> Option<ClickId> {
        if let Some(existing_id) = self.key_to_click_id.remove(key) {
            let _ = self.click_id_to_key.remove(&existing_id);
            self.id_pool.release(&[existing_id]);
            return Some(existing_id);
        }

        None
    }

    pub fn click_id_to_key(&self, click_id: &ClickId) -> Option<&String> {
        self.click_id_to_key.get(click_id)
    }

    pub fn key_to_click_id(&self, key: &str) -> Option<&ClickId> {
        self.key_to_click_id.get(key)
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{super::Styled, *};

    #[test]
    fn test_centered_button_layout() {
        let button = Element::button("test_button", "Test").w(10.0).h(10.0);

        let container = Element::container()
            .w(200.0)
            .h(200.0)
            .flex()
            .justify_center()
            .items_center()
            .with_child(button);

        let mut taffy = taffy::TaffyTree::new();
        let mut node_map = HashMap::new();

        let root = populate_taffy_and_map(&mut taffy, &mut node_map, &container);

        taffy
            .compute_layout(root, taffy::Size::length(200.0))
            .unwrap();

        let button_node = node_map
            .get("test_button")
            .expect("Button node should exist");
        let (x, y) = get_taffy_abs_position(&taffy, button_node);

        assert_eq!(x, 95.0, "Button X position should be 95");
        assert_eq!(y, 95.0, "Button Y position should be 95");

        let layout = taffy.layout(*button_node).unwrap();
        assert_eq!(layout.size.width, 10.0, "Button width should be 10");
        assert_eq!(layout.size.height, 10.0, "Button height should be 10");
    }

    #[test]
    fn test_multiple_buttons_layout() {
        // Test with multiple buttons to ensure positioning works correctly
        let button1 = Element::button("button1", "Button 1").w(20.0).h(10.0);

        let button2 = Element::button("button2", "Button 2").w(20.0).h(10.0);

        let container = Element::container()
            .w(200.0)
            .h(200.0)
            .flex()
            .flex_col()
            .justify_center()
            .items_center()
            .with_child(button1)
            .with_child(button2);

        let mut taffy = taffy::TaffyTree::new();
        let mut node_map = HashMap::new();

        let root = populate_taffy_and_map(&mut taffy, &mut node_map, &container);
        taffy
            .compute_layout(root, taffy::Size::length(200.0))
            .unwrap();

        let button1_node = node_map.get("button1").unwrap();
        let (x1, y1) = get_taffy_abs_position(&taffy, button1_node);

        let button2_node = node_map.get("button2").unwrap();
        let (x2, y2) = get_taffy_abs_position(&taffy, button2_node);

        assert_eq!(x1, 90.0, "Button1 X should be 90");
        assert_eq!(x2, 90.0, "Button2 X should be 90");

        assert_eq!(y1, 90.0);
        assert_eq!(y2, 100.0);
    }
}
