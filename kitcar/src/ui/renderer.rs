// ui.rs
use std::{collections::HashMap, fmt::Debug};

use indexmap::IndexMap;
use insim::{
    identifiers::{ClickId, ConnectionId, RequestId},
    insim::{Bfn, BfnType, Btn},
    Packet,
};

use super::{id_pool::ClickIdPool, vdom::Element};
use crate::ui::vdom::ElementKey;

#[derive(Debug, Default)]
pub struct UiRendererDiff {
    pub to_update: Vec<Btn>,
    pub to_remove: Vec<Bfn>,
}

impl UiRendererDiff {
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
pub struct UiRenderer {
    id_pool: ClickIdPool,
    // Stable mapping from component instance and key to button ClickId
    key_to_click_id: HashMap<ElementKey, ClickId>,
    click_id_to_key: HashMap<ClickId, ElementKey>,
    // last layout and props
    last_layout: Option<IndexMap<ElementKey, Btn>>,
}

impl UiRenderer {
    pub fn new(id_pool: ClickIdPool) -> Self {
        Self {
            id_pool,
            key_to_click_id: HashMap::new(),
            click_id_to_key: HashMap::new(),
            last_layout: None,
        }
    }

    /// Forcefully clear the ui. Useful for bfn clear
    pub fn clear(&mut self) {
        self.last_layout = None;
        for key in self.click_id_to_key.keys() {
            self.id_pool.release(&[*key]);
        }
        self.click_id_to_key.clear();
        self.key_to_click_id.clear();
    }

    pub fn render(&mut self, vdom: Option<Element>, ucid: &ConnectionId) -> Option<UiRendererDiff> {
        // Handle the case where render function returns None
        let new_vdom = match vdom {
            Some(vdom) => vdom,
            None => {
                // No UI to render - remove everything
                let last_layout = self.last_layout.take().unwrap_or_default();
                let to_remove = last_layout
                    .iter()
                    .map(|(key, _btn)| Bfn {
                        ucid: *ucid,
                        subt: BfnType::DelBtn,
                        clickid: self.release_clickid(key).unwrap(), // FIXME: don't unwrap
                        ..Default::default()
                    })
                    .collect();
                self.last_layout = None;

                return Some(UiRendererDiff {
                    to_update: vec![],
                    to_remove,
                });
            },
        };

        let mut taffy = taffy::TaffyTree::new();
        let mut node_map = HashMap::new();

        let root = populate_taffy_and_map(&mut taffy, &mut node_map, &new_vdom);
        let _ = taffy.compute_layout(root, taffy::Size::length(200.0));

        let new_layout: IndexMap<ElementKey, Btn> = new_vdom
            .collect_renderable()
            .into_iter()
            .map(|(k, v)| {
                let node_id = node_map.get(k).unwrap();
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
                (k.clone(), renderable)
            })
            .collect();

        let last_layout = self.last_layout.take().unwrap_or_default();

        // Find buttons to remove
        let to_remove: Vec<Bfn> = last_layout
            .iter()
            .filter_map(|(key, _btn)| {
                if !new_layout.contains_key(key) {
                    Some(Bfn {
                        ucid: *ucid,
                        subt: BfnType::DelBtn,
                        clickid: self.release_clickid(key).unwrap(), // FIXME: don't unwrap
                        ..Default::default()
                    })
                } else {
                    None
                }
            })
            .collect();

        let to_update: Vec<Btn> = new_layout
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

        if to_update.is_empty() && to_remove.is_empty() {
            None
        } else {
            Some(UiRendererDiff {
                to_update,
                to_remove,
            })
        }
    }

    fn lease_clickid(&mut self, key: &ElementKey) -> Option<ClickId> {
        // If we already have an ID for this key, reuse it
        if let Some(&existing_id) = self.key_to_click_id.get(key) {
            return Some(existing_id);
        }

        if let Some(new_id) = self.id_pool.lease() {
            let _ = self.key_to_click_id.insert(key.clone(), new_id);
            let _ = self.click_id_to_key.insert(new_id, key.clone());
            return Some(new_id);
        }

        // TODO: is probably really an error
        None
    }

    fn release_clickid(&mut self, key: &ElementKey) -> Option<ClickId> {
        if let Some(existing_id) = self.key_to_click_id.remove(key) {
            let _ = self.click_id_to_key.remove(&existing_id);
            self.id_pool.release(&[existing_id]);
            return Some(existing_id);
        }

        None
    }

    pub fn click_id_to_key(&self, click_id: &ClickId) -> Option<&ElementKey> {
        self.click_id_to_key.get(click_id)
    }

    pub fn key_to_click_id(&self, key: &ElementKey) -> Option<&ClickId> {
        self.key_to_click_id.get(key)
    }
}

fn populate_taffy_and_map(
    tree: &mut taffy::TaffyTree,
    node_map: &mut HashMap<ElementKey, taffy::NodeId>,
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
        Element::Button {
            key,
            style,
            children,
            ..
        } => {
            let taffy_id = if children.len() == 0 {
                tree.new_leaf(style.clone()).unwrap()
            } else {
                let child_ids: Vec<taffy::NodeId> = children
                    .into_iter()
                    .map(|c| populate_taffy_and_map(tree, node_map, c))
                    .collect();

                tree.new_with_children(style.clone(), &child_ids).unwrap()
            };
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

    use insim::{identifiers::ConnectionId, insim::BtnStyle};

    use super::{super::Styled, *};

    #[derive(Clone, PartialEq, Default)]
    pub struct AppProps {
        pub empty: bool,
        pub bar: bool,
    }

    fn app(props: &AppProps) -> Option<Element> {
        if props.empty {
            return None;
        }

        let mut children = Vec::new();

        children.push(
            Element::Button {
                text: "foo".to_string(),
                key: ElementKey::new(1, "1"),
                style: taffy::Style::DEFAULT,
                btnstyle: BtnStyle::default(),
                children: vec![],
            }
            .w(5.0)
            .h(5.0),
        );

        if props.bar {
            children.push(Element::Button {
                text: "bar".to_string(),
                key: ElementKey::new(1, "2"),
                style: taffy::Style::DEFAULT,
                btnstyle: BtnStyle::default(),
                children: vec![],
            });
        }

        Some(Element::Container {
            children,
            style: taffy::Style::DEFAULT,
        })
    }

    #[test]
    fn test_centered_button_layout() {
        let button = Element::button(1, "test_button", "Test").w(10.0).h(10.0);

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
            .get(&ElementKey::new(1, "test_button"))
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
        let button1 = Element::button(1, "button1", "Button 1").w(20.0).h(10.0);

        let button2 = Element::button(1, "button2", "Button 2").w(20.0).h(10.0);

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

        let button1_node = node_map.get(&ElementKey::new(1, "button1")).unwrap();
        let (x1, y1) = get_taffy_abs_position(&taffy, button1_node);

        let button2_node = node_map.get(&ElementKey::new(1, "button2")).unwrap();
        let (x2, y2) = get_taffy_abs_position(&taffy, button2_node);

        assert_eq!(x1, 90.0, "Button1 X should be 90");
        assert_eq!(x2, 90.0, "Button2 X should be 90");

        assert_eq!(y1, 90.0);
        assert_eq!(y2, 100.0);
    }

    #[test]
    fn test_ui() {
        let mut renderer = UiRenderer::new(ClickIdPool::new());

        let vdom = app(&AppProps {
            empty: false,
            bar: false,
        });

        let diff = renderer
            .render(vdom, &ConnectionId::ALL)
            .expect("Initial render should render *something*");

        assert_eq!(diff.to_update.len(), 1);
        assert_eq!(diff.to_remove.len(), 0);

        let expected_click_id = diff.to_update[0].clickid;

        assert_eq!(
            renderer.key_to_click_id(&ElementKey::new(1, "1")),
            Some(&expected_click_id)
        );

        assert_eq!(diff.to_update[0].text, "foo");

        let vdom = app(&AppProps {
            empty: false,
            bar: false,
        });

        let diff = renderer.render(vdom, &ConnectionId::ALL);

        // nothing changed
        assert!(diff.is_none(), "{:?}", diff);

        assert_eq!(
            renderer.key_to_click_id(&ElementKey::new(1, "1")),
            Some(&expected_click_id)
        );

        let vdom = app(&AppProps {
            empty: false,
            bar: true,
        });

        let diff = renderer
            .render(vdom, &ConnectionId::ALL)
            .expect("when updating bar, we should get a diff");

        assert_eq!(diff.to_update.len(), 1);
        assert_eq!(diff.to_remove.len(), 0);

        assert_eq!(diff.to_update[0].text, "bar");
        assert_ne!(diff.to_update[0].clickid, expected_click_id); // we dont reuse an id

        let vdom = app(&AppProps {
            empty: true,
            bar: true,
        });

        let diff = renderer
            .render(vdom, &ConnectionId::ALL)
            .expect("when updating bar, we should get a diff");

        assert_eq!(diff.to_remove.len(), 2, "received diff: {:?}", diff);

        assert_eq!(renderer.key_to_click_id(&ElementKey::new(1, "1")), None);
    }
}
