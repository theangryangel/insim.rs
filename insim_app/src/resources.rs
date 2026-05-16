//! Type-keyed resource registry.
//!
//! [`App::resource`](crate::App::resource) inserts cloneable values into a
//! `TypeId`-keyed map. Handlers reach them through their context by calling
//! `cx.resources.get::<T>()`, which downcasts and clones.
//!
//! Used by every resource that wants to expose itself as an extractor - see
//! [`crate::Presence`] for the canonical example.

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

/// Type-keyed map of cloneable values made available to handlers.
#[derive(Default)]
pub struct Resources {
    map: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Resources {
    /// Create an empty resource registry.
    pub(crate) fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Insert a pre-`Arc`-wrapped value. Used by [`App::resource`](crate::App::resource)
    /// so that one `Arc<T>` allocation can be shared between the registry
    /// and any installable's handler closure captures.
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

impl std::fmt::Debug for Resources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resources")
            .field("len", &self.map.len())
            .finish()
    }
}
