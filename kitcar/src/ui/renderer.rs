//! UI Renderer
use std::{any::TypeId, collections::HashMap};

use insim::{identifiers::ClickId, insim::Btn};
use taffy::{prelude::length, NodeId, Size, TaffyTree};

use crate::ui::{
    click_id_pool::ClickIdPool,
    node::{UINode, UINodeKey},
    tree::TreeState,
};

// Renderer - handles layout computation and packet generation
pub(crate) struct Renderer;

pub(crate) type RendererResult = Result<
    (
        // Btn packet to send to LFS
        Vec<Btn>,
        // Btn ClickIds to ask LFS to remove
        Vec<ClickId>,
        // ClickId to UINodeKey and hash
        HashMap<ClickId, (UINodeKey, u64)>,
        // UINodeKey to ClickId
        HashMap<UINodeKey, ClickId>,
    ),
    // FIXME: This should be a proper error type
    String,
>;

impl Renderer {
    /// Compute layout and render packets for a tree
    pub(crate) fn render_tree(
        tree_state: &TreeState,
        click_id_pool: &mut ClickIdPool,
        id_to_tree_map: &HashMap<ClickId, TypeId>,
        tree_id: TypeId,
    ) -> RendererResult {
        // Skip rendering if marked for deletion
        if tree_state.marked_for_deletion {
            return Ok((Vec::new(), Vec::new(), HashMap::new(), HashMap::new()));
        }

        // Step 1: Compute layout using Taffy
        let mut taffy = TaffyTree::new();
        let mut desired_style_map = HashMap::new();
        let mut node_id_map = HashMap::new();

        let root_node_id = Self::build_taffy_tree_and_collect_buttons(
            &mut taffy,
            &tree_state.ui_tree,
            &mut desired_style_map,
            &mut node_id_map,
        );

        taffy
            .compute_layout(
                root_node_id,
                Size {
                    width: length(200.0),
                    height: length(200.0),
                },
            )
            .unwrap();

        // Step 2: Diff and generate packets
        let mut packets = Vec::new();
        let mut next_active_buttons = HashMap::new();
        let mut next_active_keys = HashMap::new();
        let mut removed_ids = Vec::new();

        for (key, node) in desired_style_map {
            let layout = taffy.layout(node_id_map[&key]).unwrap();
            let hash = node.hash(layout).unwrap();

            let (click_id, needs_update) =
                if let Some(old_click_id) = tree_state.active_keys.get(&key) {
                    // Validate that we still own this click ID
                    if id_to_tree_map.get(old_click_id) == Some(&tree_id) {
                        if let Some((_, old_hash)) = tree_state.active_buttons.get(old_click_id) {
                            (*old_click_id, *old_hash != hash)
                        } else {
                            // Inconsistent state - allocate new ID
                            let click_id = click_id_pool
                                .lease()
                                .ok_or_else(|| "Ran out of button IDs".to_string())?;
                            (click_id, true)
                        }
                    } else {
                        // We no longer own this click ID - allocate new one
                        let click_id = click_id_pool
                            .lease()
                            .ok_or_else(|| "Ran out of button IDs".to_string())?;
                        (click_id, true)
                    }
                } else {
                    // New button - allocate click ID
                    let click_id = click_id_pool
                        .lease()
                        .ok_or_else(|| "Ran out of button IDs".to_string())?;
                    (click_id, true)
                };

            if needs_update {
                packets.push(Btn {
                    clickid: click_id,
                    // Round to avoid visual oddities since LFS works in ints, and taffy with floats
                    l: layout.location.x.round() as u8,
                    t: layout.location.y.round() as u8,
                    w: layout.size.width.round() as u8,
                    h: layout.size.height.round() as u8,

                    // unwraps are fine here because we *know* we've got something we can actually
                    // render
                    text: node.text().unwrap().to_string(),
                    bstyle: node.bstyle().unwrap(),

                    // FIXME: UCID
                    ..Default::default()
                });
            }

            let _ = next_active_buttons.insert(click_id, (key, hash));
            let _ = next_active_keys.insert(key, click_id);
        }

        // Find removed buttons
        for (old_key, old_click_id) in &tree_state.active_keys {
            if !next_active_keys.contains_key(old_key) {
                removed_ids.push(*old_click_id);
            }
        }

        Ok((packets, removed_ids, next_active_buttons, next_active_keys))
    }

    // Helper to recursively build the Taffy tree and collect button data
    fn build_taffy_tree_and_collect_buttons<'a>(
        taffy: &mut TaffyTree,
        ui_node: &'a UINode,
        button_info: &mut HashMap<UINodeKey, &'a UINode>,
        node_id_map: &mut HashMap<UINodeKey, NodeId>,
    ) -> NodeId {
        let mut child_ids = Vec::new();
        if let UINode::Unrendered { children, .. } = ui_node {
            for child_node in children {
                let child_id = Self::build_taffy_tree_and_collect_buttons(
                    taffy,
                    child_node,
                    button_info,
                    node_id_map,
                );
                child_ids.push(child_id);
            }
        }

        match ui_node {
            UINode::Rendered { layout, key, .. } => {
                let _ = button_info.insert(*key, ui_node);
                let node_id = taffy.new_leaf(layout.clone()).unwrap();
                let _ = node_id_map.insert(*key, node_id);
                node_id
            },
            UINode::Unrendered { layout, .. } => {
                taffy.new_with_children(layout.clone(), &child_ids).unwrap()
            },
        }
    }
}
