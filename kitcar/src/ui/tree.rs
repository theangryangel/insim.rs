//! UI TreeManager and TreeState
use std::{any::TypeId, collections::HashMap};

use insim::identifiers::ClickId;

use crate::ui::node::{UINode, UINodeKey};

// Per-tree state tracking
#[derive(Debug)]
pub(crate) struct TreeState {
    pub(crate) ui_tree: UINode,
    // Maps click_id to (UINode Key, hash) for this specific tree
    pub(crate) active_buttons: HashMap<ClickId, (UINodeKey, u64)>,
    // Maps UINode Key to click_id for this specific tree
    pub(crate) active_keys: HashMap<UINodeKey, ClickId>,
    // Mark for deletion in next render cycle
    pub(crate) marked_for_deletion: bool,
}

impl TreeState {
    pub(crate) fn new(ui_tree: UINode) -> Self {
        Self {
            ui_tree,
            active_buttons: HashMap::new(),
            active_keys: HashMap::new(),
            marked_for_deletion: false,
        }
    }
}

/// Tree Manager - handles tree lifecycle and state
#[derive(Debug)]
pub(crate) struct TreeManager {
    // Maps tree ID to its state
    pub(crate) trees: HashMap<TypeId, TreeState>,
}

impl TreeManager {
    pub(crate) fn new() -> Self {
        Self {
            trees: HashMap::new(),
        }
    }

    /// Register or update a UI tree
    pub(crate) fn set_tree<T: 'static>(&mut self, ui_tree: UINode) -> TypeId {
        let tree_id = TypeId::of::<T>();

        if let Some(tree_state) = self.trees.get_mut(&tree_id) {
            // Update existing tree and unmark for deletion
            tree_state.ui_tree = ui_tree;
            tree_state.marked_for_deletion = false;
        } else {
            // Register new tree
            let tree_state = TreeState::new(ui_tree);
            let _ = self.trees.insert(tree_id, tree_state);
        }

        tree_id
    }

    /// Mark a tree for removal
    pub(crate) fn remove_tree<T: 'static>(&mut self) -> bool {
        let tree_id = TypeId::of::<T>();
        if let Some(tree_state) = self.trees.get_mut(&tree_id) {
            tree_state.marked_for_deletion = true;
            true
        } else {
            false
        }
    }

    /// Check if a tree exists and is not marked for deletion
    pub(crate) fn has_tree<T: 'static>(&self) -> bool {
        let tree_id = TypeId::of::<T>();
        self.trees
            .get(&tree_id)
            .map(|state| !state.marked_for_deletion)
            .unwrap_or(false)
    }

    /// Get all tree IDs
    pub(crate) fn get_tree_ids(&self) -> Vec<TypeId> {
        self.trees.keys().copied().collect()
    }

    /// Get tree state (immutable)
    pub(crate) fn get_tree_state(&self, tree_id: TypeId) -> Option<&TreeState> {
        self.trees.get(&tree_id)
    }

    /// Get tree state (mutable)
    pub(crate) fn get_tree_state_mut(&mut self, tree_id: TypeId) -> Option<&mut TreeState> {
        self.trees.get_mut(&tree_id)
    }

    /// Remove trees that are marked for deletion and return their click IDs
    pub(crate) fn cleanup_deleted_trees(&mut self) -> Vec<ClickId> {
        let mut removed_click_ids = Vec::new();
        self.trees.retain(|_tree_id, state| {
            if state.marked_for_deletion {
                // This tree is being removed, so collect its click IDs.
                removed_click_ids.extend(state.active_buttons.keys());
                // Return false to remove it from the map.
                false
            } else {
                // Return true to keep it.
                true
            }
        });

        removed_click_ids
    }
}
