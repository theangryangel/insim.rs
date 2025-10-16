// ui.rs
use std::{any::Any, collections::HashMap, fmt::Debug};

use indexmap::IndexMap;
use insim::{
    identifiers::{ClickId, ConnectionId, RequestId},
    insim::{Bfn, BfnType, Btn},
    Packet,
};

use super::{id_pool::ClickIdPool, vdom::Element};
use crate::ui::{scope::Scope, vdom::ElementId, Component, ComponentPath};

#[derive(Debug, Default)]
pub struct RenderDiff {
    pub to_update: Vec<Btn>,
    pub to_remove: Vec<Bfn>,
}

impl RenderDiff {
    pub fn into_merged(self) -> Vec<Packet> {
        self.to_remove
            .into_iter()
            .map(|x| Packet::from(x))
            .chain(self.to_update.into_iter().map(|x| Packet::from(x)))
            .collect()
    }
}

/// Ui for a single connection
pub struct Runtime {
    id_pool: ClickIdPool,
    // Stable mapping from component instance and key to button ClickId
    element_id_to_click_id: HashMap<ElementId, ClickId>,
    // last layout
    last_layout: Option<IndexMap<ElementId, Btn>>,
    // Persisted component state
    component_states: HashMap<ComponentPath, Box<dyn Any + Send + Sync>>,
    // Click handlers
    clicks: HashMap<ClickId, Box<dyn Fn() + Send + Sync>>,
    // Has the UI been removed through user request?
    blocked: bool,
}

impl Debug for Runtime {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Runtime {
    pub fn new(id_pool: ClickIdPool) -> Self {
        Self {
            id_pool,
            element_id_to_click_id: HashMap::new(),
            last_layout: None,
            component_states: HashMap::new(),
            clicks: HashMap::new(),
            blocked: false,
        }
    }

    pub fn on_click(&mut self, click_id: &ClickId) {
        if let Some(f) = self.clicks.get_mut(click_id) {
            f()
        }
    }

    /// Unblock
    pub fn unblock(&mut self) {
        self.blocked = false;
    }

    /// Block
    pub fn block(&mut self) {
        self.blocked = true;
        self.clear();
    }

    /// Forcefully clear the ui. Useful for bfn clear
    pub fn clear(&mut self) {
        self.last_layout = None;
        self.element_id_to_click_id.clear();
        self.clicks.clear();
    }

    pub fn render<C: Component>(
        &mut self,
        props: C::Props,
        ucid: &ConnectionId,
    ) -> Option<RenderDiff> {
        let vdom = if self.blocked {
            None
        } else {
            let mut cx = Scope::new(&mut self.component_states);
            C::render(props, &mut cx)
        };

        // TODO: This is a bit brutal. but fuck it for now.
        self.clicks.clear();

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

                return Some(RenderDiff {
                    to_update: vec![],
                    to_remove,
                });
            },
        };

        let mut taffy = taffy::TaffyTree::new();
        let mut node_map = IndexMap::new();

        let root_id = flatten(&mut taffy, &mut node_map, new_vdom);
        let _ = taffy.compute_layout(root_id, taffy::Size::length(200.0));

        for (_element_id, (node_id, _on_click, btn)) in node_map.iter_mut() {
            let (x, y) = get_taffy_abs_position(&taffy, node_id);
            let layout = taffy.layout(*node_id).unwrap();
            btn.l = x as u8;
            btn.t = y as u8;
            btn.w = layout.size.width as u8;
            btn.h = layout.size.height as u8;

            // FIXME: we could probably flatten the 2 loops below into this
        }

        let last_layout = self.last_layout.take().unwrap_or_default();

        // Find buttons to remove
        let to_remove: Vec<Bfn> = last_layout
            .iter()
            .filter_map(|(key, _vals)| {
                if !node_map.contains_key(key) {
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

        let to_update: Vec<Btn> = node_map
            .iter_mut()
            .filter_map(|(element_id, (_node_id, on_click, btn))| {
                match last_layout.get(element_id) {
                    None => {
                        let click_id = self.lease_clickid(element_id).unwrap();
                        if let Some(on_click) = on_click.take() {
                            let _ = self.clicks.insert(click_id.clone(), on_click);
                        }

                        // New button
                        let mut btn = btn.clone();
                        btn.clickid = click_id;
                        btn.reqi = RequestId(click_id.0);
                        Some(btn)
                    },
                    Some(existing_btn) => {
                        // Check if button actually changed
                        if existing_btn != btn {
                            let mut btn = btn.clone();
                            let click_id = self.lease_clickid(element_id).unwrap();
                            if let Some(on_click) = on_click.take() {
                                let _ = self.clicks.insert(click_id.clone(), on_click);
                            }
                            btn.clickid = click_id;
                            btn.reqi = RequestId(click_id.0);
                            Some(btn)
                        } else {
                            None
                        }
                    },
                }
            })
            .collect();

        self.last_layout = Some(
            node_map
                .into_iter()
                .map(|(element_id, (_node_id, _on_click, btn))| (element_id, btn))
                .collect(),
        );

        if to_update.is_empty() && to_remove.is_empty() {
            None
        } else {
            Some(RenderDiff {
                to_update,
                to_remove,
            })
        }
    }

    fn lease_clickid(&mut self, key: &ElementId) -> Option<ClickId> {
        // If we already have an ID for this key, reuse it
        if let Some(&existing_id) = self.element_id_to_click_id.get(key) {
            return Some(existing_id);
        }

        if let Some(new_id) = self.id_pool.lease() {
            let _ = self.element_id_to_click_id.insert(key.clone(), new_id);
            return Some(new_id);
        }

        // TODO: is probably really an error
        None
    }

    fn release_clickid(&mut self, key: &ElementId) -> Option<ClickId> {
        if let Some(existing_id) = self.element_id_to_click_id.remove(key) {
            self.id_pool.release(&[existing_id]);
            return Some(existing_id);
        }

        None
    }

    pub fn key_to_click_id(&self, key: &ElementId) -> Option<&ClickId> {
        self.element_id_to_click_id.get(key)
    }
}

fn flatten(
    tree: &mut taffy::TaffyTree,
    node_map: &mut IndexMap<ElementId, (taffy::NodeId, Option<Box<dyn Fn() + Send + Sync>>, Btn)>,
    vdom: Element,
) -> taffy::NodeId {
    match vdom {
        Element::Container {
            children, style, ..
        } => {
            let child_ids: Vec<taffy::NodeId> = children
                .into_iter()
                .map(|c| flatten(tree, node_map, c))
                .collect();

            let taffy_id = tree.new_with_children(style.clone(), &child_ids).unwrap();
            taffy_id
        },
        Element::Button {
            id,
            style,
            children,
            on_click,
            btnstyle,
            text,
            ..
        } => {
            let taffy_id = if children.len() == 0 {
                tree.new_leaf(style.clone()).unwrap()
            } else {
                let child_ids: Vec<taffy::NodeId> = children
                    .into_iter()
                    .map(|c| flatten(tree, node_map, c))
                    .collect();

                tree.new_with_children(style.clone(), &child_ids).unwrap()
            };
            let _ = node_map.insert(
                id,
                (
                    taffy_id,
                    on_click,
                    Btn {
                        text,
                        bstyle: btnstyle,
                        ..Default::default()
                    },
                ),
            );
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

        let root = flatten(&mut taffy, &mut node_map, &container);

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

        let root = flatten(&mut taffy, &mut node_map, &container);
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
        let mut renderer = Runtime::new(ClickIdPool::new());

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
