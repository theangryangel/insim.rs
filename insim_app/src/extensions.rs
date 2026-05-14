//! Type-keyed extension registry.
//!
//! [`App::extension`](crate::App::extension) inserts cloneable values into a
//! `TypeId`-keyed map. Handlers/middleware reach them through their context
//! by calling `cx.extension::<T>()`, which downcasts and clones.
//!
//! Used by middleware that wants to expose itself as an extractor — see
//! [`crate::PresenceMiddleware`] for the canonical example.

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

/// Type-keyed map of cloneable values made available to handlers/middleware.
#[derive(Default)]
pub struct Extensions {
    map: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Extensions {
    /// Create an empty extensions registry.
    pub(crate) fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Insert a pre-`Arc`-wrapped value. Used by [`App::extension`](crate::App::extension)
    /// so that one `Arc<E>` allocation can be shared between the registry
    /// (extractor lookup) and the dispatch chain (`on_event`).
    pub(crate) fn insert_arc<T: Any + Send + Sync + 'static>(&mut self, value: Arc<T>) {
        let _ = self
            .map
            .insert(TypeId::of::<T>(), value as Arc<dyn Any + Send + Sync>);
    }

    /// Get a cloned value of type `T`, or `None` if not registered.
    pub fn get<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|a| a.downcast_ref::<T>())
            .cloned()
    }
}

impl std::fmt::Debug for Extensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Extensions")
            .field("len", &self.map.len())
            .finish()
    }
}
