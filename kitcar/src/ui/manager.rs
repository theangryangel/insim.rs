//! Main UI Manager - coordinates between TreeManager, ClickIdPool, and Renderer

use std::{any::TypeId, collections::HashMap};

use insim::{
    identifiers::{ClickId, ConnectionId},
    insim::{Bfn, BfnType, Btn},
};

use crate::ui::{
    id_pool::IdPool,
    node::{UINode, UINodeKey},
    renderer::Renderer,
    tree::TreeManager,
};

#[derive(Debug)]
/// UI / Button Manager - Give it UINode and it will render LFS packets
/// It will do a basic view diff to ensure that the minimum number of updates are sent to LFS
pub struct UIManager {
    tree_manager: TreeManager,
    click_id_pool: IdPool<1, 239>,
    // Maps click_id back to which tree it belongs to
    id_to_tree: HashMap<ClickId, TypeId>,
}

impl UIManager {
    /// New!
    pub fn new() -> Self {
        UIManager {
            tree_manager: TreeManager::new(),
            click_id_pool: IdPool::new(),
            id_to_tree: HashMap::new(),
        }
    }

    /// Register or update a UI tree using the TypeId of an empty struct
    pub fn set_tree<T: 'static>(&mut self, ui_tree: UINode) -> TypeId {
        self.tree_manager.set_tree::<T>(ui_tree)
    }

    /// Mark a tree for removal (will be removed in next render cycle)
    pub fn remove_tree<T: 'static>(&mut self) -> bool {
        self.tree_manager.remove_tree::<T>()
    }

    /// Get all registered tree IDs
    pub fn get_tree_ids(&self) -> Vec<TypeId> {
        self.tree_manager.get_tree_ids()
    }

    /// Check if a tree is registered and not marked for deletion
    pub fn has_tree<T: 'static>(&self) -> bool {
        self.tree_manager.has_tree::<T>()
    }

    /// Render a specific tree using its TypeId
    pub fn render_tree<T: 'static>(
        &mut self,
        ucid: ConnectionId,
    ) -> Result<(Vec<Btn>, Vec<ClickId>), String> {
        let tree_id = TypeId::of::<T>();
        self.render_tree_by_id(tree_id, ucid)
    }

    /// Internal method to render by tree ID
    fn render_tree_by_id(
        &mut self,
        tree_id: TypeId,
        ucid: ConnectionId,
    ) -> Result<(Vec<Btn>, Vec<ClickId>), String> {
        let tree_state = self
            .tree_manager
            .get_tree_state(tree_id)
            .ok_or_else(|| format!("Tree {:?} not found", tree_id))?;

        let (packets, removed_ids, next_active_buttons, next_active_keys) = Renderer::render_tree(
            tree_state,
            &mut self.click_id_pool,
            &self.id_to_tree,
            tree_id,
            ucid,
        )?;

        // Update the tree state with new button mappings
        if let Some(tree_state) = self.tree_manager.get_tree_state_mut(tree_id) {
            tree_state.active_buttons = next_active_buttons.clone();
            tree_state.active_keys = next_active_keys;
        }

        // Update id_to_tree mapping for newly allocated click IDs
        for (click_id, _) in &next_active_buttons {
            let _ = self.id_to_tree.insert(*click_id, tree_id);
        }

        // Remove mappings for removed click IDs
        for click_id in &removed_ids {
            let _ = self.id_to_tree.remove(click_id);
        }

        Ok((packets, removed_ids))
    }

    /// Render all active trees and return combined packets
    pub fn render_all(&mut self, ucid: ConnectionId) -> (Vec<Btn>, Vec<Bfn>) {
        let mut all_packets = Vec::new();
        let mut all_removed = Vec::new();

        // First, handle trees marked for deletion
        let deleted_tree_click_ids = self.tree_manager.cleanup_deleted_trees();
        all_removed.extend(deleted_tree_click_ids.clone());

        // Remove mappings for deleted tree click IDs
        for click_id in &deleted_tree_click_ids {
            let _ = self.id_to_tree.remove(click_id);
        }

        // Render remaining active trees
        let tree_ids: Vec<TypeId> = self.tree_manager.get_tree_ids();

        for tree_id in tree_ids {
            match self.render_tree_by_id(tree_id, ucid) {
                Ok((mut packets, mut removed)) => {
                    all_packets.append(&mut packets);
                    all_removed.append(&mut removed);
                },
                Err(e) => {
                    eprintln!("Error rendering tree {:?}: {}", tree_id, e);
                },
            }
        }

        // Batch deallocate all removed click IDs at the end
        self.click_id_pool.release(&all_removed);

        (
            all_packets,
            all_removed
                .into_iter()
                .map(|clickid| Bfn {
                    ucid,
                    clickid,
                    subt: BfnType::DelBtn,
                    ..Default::default()
                })
                .collect(),
        )
    }

    /// Handle a click and return both the tree ID and button key that was clicked
    pub fn handle_click(&self, click_id: ClickId) -> Option<(TypeId, UINodeKey)> {
        let tree_id = self.id_to_tree.get(&click_id)?;
        let tree_state = self.tree_manager.get_tree_state(*tree_id)?;
        let (button_key, _) = tree_state.active_buttons.get(&click_id)?;
        Some((*tree_id, *button_key))
    }

    /// Get debug info about click ID allocation
    pub fn debug_info(&self) -> String {
        let (total, available, allocated) = self.click_id_pool.stats();
        format!(
            "Active trees: {}, Total click IDs: {}, Available: {}, Allocated: {}",
            self.tree_manager.trees.len(),
            total,
            available,
            allocated
        )
    }

    /// Get the current click ID pool size (for debugging)
    pub fn available_click_ids(&self) -> usize {
        self.click_id_pool.available_count()
    }

    /// Get detailed pool statistics
    pub fn pool_stats(&self) -> (usize, usize, usize) {
        self.click_id_pool.stats()
    }
}

#[cfg(test)]
mod tests {
    use super::{super::components, *};

    // Test view types
    struct TestViewA;
    struct TestViewB;
    struct TestViewC;

    #[test]
    fn test_adding_a_view() {
        let mut ui_manager = UIManager::new();

        // Initially no trees should be present
        assert_eq!(ui_manager.get_tree_ids().len(), 0);
        assert!(!ui_manager.has_tree::<TestViewA>());

        // Create a simple view
        let test_view = components::page_layout(vec![components::primary_button(
            "Test Button".into(),
            1.into(),
        )]);

        // Add the view
        let tree_id = ui_manager.set_tree::<TestViewA>(test_view);

        // Verify the view was added
        assert_eq!(ui_manager.get_tree_ids().len(), 1);
        assert!(ui_manager.has_tree::<TestViewA>());
        assert!(ui_manager.get_tree_ids().contains(&tree_id));

        // Verify we can render it
        let result = ui_manager.render_tree::<TestViewA>(ConnectionId::LOCAL);
        assert!(result.is_ok());

        let (packets, removed) = result.unwrap();
        assert_eq!(packets.len(), 1);
        assert_eq!(removed.len(), 0);
        assert_eq!(packets[0].text, "Test Button");
    }

    #[test]
    fn test_adding_multiple_views() {
        let mut ui_manager = UIManager::new();

        // Add multiple different view types
        let view_a = components::page_layout(vec![components::primary_button(
            "Button A".into(),
            1.into(),
        )]);
        let view_b = components::page_layout(vec![
            components::primary_button("Button B1".into(), 10.into()),
            components::primary_button("Button B2".into(), 11.into()),
        ]);

        let _ = ui_manager.set_tree::<TestViewA>(view_a);
        let _ = ui_manager.set_tree::<TestViewB>(view_b);

        // Verify both views exist
        assert_eq!(ui_manager.get_tree_ids().len(), 2);
        assert!(ui_manager.has_tree::<TestViewA>());
        assert!(ui_manager.has_tree::<TestViewB>());

        // Render all and verify we get packets from both trees
        let (all_packets, all_removed) = ui_manager.render_all(ConnectionId::LOCAL);
        assert_eq!(all_packets.len(), 3); // 1 from A, 2 from B
        assert_eq!(all_removed.len(), 0);

        // Verify packet contents
        let button_texts: Vec<&String> = all_packets.iter().map(|p| &p.text).collect();
        assert!(button_texts.contains(&&"Button A".to_string()));
        assert!(button_texts.contains(&&"Button B1".to_string()));
        assert!(button_texts.contains(&&"Button B2".to_string()));
    }

    #[test]
    fn test_updating_existing_view() {
        let mut ui_manager = UIManager::new();

        // Add initial view
        let initial_view = components::page_layout(vec![components::primary_button(
            "Initial Button".into(),
            1.into(),
        )]);
        let _ = ui_manager.set_tree::<TestViewA>(initial_view);

        // Render to allocate click IDs
        let (initial_packets, _) = ui_manager
            .render_tree::<TestViewA>(ConnectionId::LOCAL)
            .unwrap();
        assert_eq!(initial_packets.len(), 1);
        assert_eq!(initial_packets[0].text, "Initial Button");

        // Update the same view type with new content
        let updated_view = components::page_layout(vec![
            components::primary_button("Updated Button".into(), 1.into()),
            components::primary_button("New Button".into(), 2.into()),
        ]);
        let _ = ui_manager.set_tree::<TestViewA>(updated_view); // Same method, different content

        // Verify it's still the same tree (no new tree added)
        assert_eq!(ui_manager.get_tree_ids().len(), 1);
        assert!(ui_manager.has_tree::<TestViewA>());

        // Render and verify updated content
        let (updated_packets, _) = ui_manager
            .render_tree::<TestViewA>(ConnectionId::LOCAL)
            .unwrap();
        assert_eq!(updated_packets.len(), 2); // Now has 2 buttons

        let button_texts: Vec<&String> = updated_packets.iter().map(|p| &p.text).collect();
        assert!(button_texts.contains(&&"Updated Button".to_string()));
        assert!(button_texts.contains(&&"New Button".to_string()));
    }

    #[test]
    fn test_view_not_marked_for_deletion_after_update() {
        let mut ui_manager = UIManager::new();

        // Add and mark for deletion
        let view =
            components::page_layout(vec![components::primary_button("Test".into(), 1.into())]);
        let _ = ui_manager.set_tree::<TestViewA>(view);
        let _ = ui_manager.remove_tree::<TestViewA>();

        // Verify it's marked for deletion
        assert!(!ui_manager.has_tree::<TestViewA>()); // has_tree returns false for marked trees

        // Update the tree (should unmark it)
        let updated_view =
            components::page_layout(vec![components::primary_button("Updated".into(), 1.into())]);
        let _ = ui_manager.set_tree::<TestViewA>(updated_view);

        // Verify it's no longer marked for deletion
        assert!(ui_manager.has_tree::<TestViewA>());

        // Should be able to render successfully
        let result = ui_manager.render_tree::<TestViewA>(ConnectionId::LOCAL);
        assert!(result.is_ok());
        let (packets, _) = result.unwrap();
        assert_eq!(packets.len(), 1);
        assert_eq!(packets[0].text, "Updated");
    }

    #[test]
    fn test_adding_empty_view() {
        let mut ui_manager = UIManager::new();

        // Add a view with no rendered buttons
        let empty_view = components::page_layout(vec![]);
        let _ = ui_manager.set_tree::<TestViewA>(empty_view);

        // Verify the tree exists
        assert!(ui_manager.has_tree::<TestViewA>());

        // Render should succeed but return no packets
        let (packets, removed) = ui_manager
            .render_tree::<TestViewA>(ConnectionId::LOCAL)
            .unwrap();
        assert_eq!(packets.len(), 0);
        assert_eq!(removed.len(), 0);
    }

    #[test]
    fn test_adding_view_allocates_click_ids() {
        let mut ui_manager = UIManager::new();
        let initial_available = ui_manager.available_click_ids();

        // Add a view with 3 buttons
        let view = components::page_layout(vec![
            components::primary_button("Button 1".into(), 1.into()),
            components::primary_button("Button 2".into(), 2.into()),
            components::primary_button("Button 3".into(), 3.into()),
        ]);
        let _ = ui_manager.set_tree::<TestViewA>(view);

        // Render to allocate click IDs
        let (packets, _) = ui_manager
            .render_tree::<TestViewA>(ConnectionId::LOCAL)
            .unwrap();
        assert_eq!(packets.len(), 3);

        // Verify click IDs were allocated
        let available_after = ui_manager.available_click_ids();
        assert_eq!(initial_available - available_after, 3);

        // Verify all packets have unique click IDs
        let mut click_ids: Vec<ClickId> = packets.iter().map(|p| p.clickid).collect();
        click_ids.sort();
        click_ids.dedup();
        assert_eq!(click_ids.len(), 3); // Should still be 3 after dedup
    }

    #[test]
    fn test_type_safety_different_views() {
        let mut ui_manager = UIManager::new();

        // Add views of different types
        let view_a = components::page_layout(vec![components::primary_button(
            "A Button".into(),
            1.into(),
        )]);
        let view_b = components::page_layout(vec![components::primary_button(
            "B Button".into(),
            10.into(),
        )]);

        let _ = ui_manager.set_tree::<TestViewA>(view_a);
        let _ = ui_manager.set_tree::<TestViewB>(view_b);

        // Each type should be tracked separately
        assert!(ui_manager.has_tree::<TestViewA>());
        assert!(ui_manager.has_tree::<TestViewB>());
        assert!(!ui_manager.has_tree::<TestViewC>()); // Not added

        // Should be able to render each independently
        let (packets_a, _) = ui_manager
            .render_tree::<TestViewA>(ConnectionId::LOCAL)
            .unwrap();
        let (packets_b, _) = ui_manager
            .render_tree::<TestViewB>(ConnectionId::LOCAL)
            .unwrap();

        assert_eq!(packets_a.len(), 1);
        assert_eq!(packets_b.len(), 1);
        assert_eq!(packets_a[0].text, "A Button");
        assert_eq!(packets_b[0].text, "B Button");
    }

    #[test]
    fn test_render_nonexistent_view() {
        let mut ui_manager = UIManager::new();

        // Try to render a view that was never added
        let result = ui_manager.render_tree::<TestViewA>(ConnectionId::LOCAL);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }
}
