use std::{any::Any, collections::HashMap};

use crate::ui::{Component, ComponentPath, Element};

#[derive(Debug)]
pub struct Scope<'a> {
    path: ComponentPath,
    use_state_index: usize,
    child_index: usize,
    component_states: &'a mut HashMap<ComponentPath, Box<dyn Any + Send + Sync>>,
    current_element_id: usize,
}

impl<'a> Scope<'a> {
    /// New!
    pub fn new(
        component_states: &'a mut HashMap<ComponentPath, Box<dyn Any + Send + Sync>>,
    ) -> Self {
        Self {
            path: vec![0], // FIXME: dont just alias the type
            use_state_index: 0,
            child_index: 0,
            component_states,
            current_element_id: 0,
        }
    }

    fn next_element_id(&mut self) -> usize {
        self.current_element_id = self.current_element_id.saturating_add(1);
        self.current_element_id
    }

    /// helper to create a standard button
    pub fn button(&mut self, text: String) -> Element {
        let id = self.next_element_id();

        Element::Button {
            id,
            text,
            on_click: None,
            btnstyle: Default::default(),
            style: Default::default(),
            children: Vec::new(),
        }
    }

    /// Helper to create a non-rendered container
    /// Useful for layout
    pub fn container(&mut self) -> Element {
        Element::Container {
            children: vec![],
            style: Default::default(),
        }
    }

    /// Helper method to render a component
    pub fn component<C: Component>(&mut self, props: C::Props) -> Option<Element> {
        let child_id = self.child_index;
        self.child_index = self.child_index.saturating_add(1);
        self.path.push(child_id);

        // reset the component indexes when we render the children
        let parent_use_state_index = self.use_state_index;
        self.use_state_index = 0;

        let result = C::render(props, self);

        // then put it back
        let _ = self.path.pop();
        self.use_state_index = parent_use_state_index;
        result
    }

    /// Provide some state to your component
    pub fn use_state<T: Send + Sync + 'static>(
        &mut self,
        initial_state: impl FnOnce() -> T,
    ) -> &mut T {
        let mut hook_path = self.path.clone();
        hook_path.push(self.use_state_index);
        self.use_state_index += 1;

        let state = self
            .component_states
            .entry(hook_path.clone())
            .or_insert_with(|| Box::new(initial_state()));

        state.downcast_mut::<T>().unwrap()
    }
}
