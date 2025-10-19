use std::{any::Any, collections::HashMap, fmt::Debug};

use crate::ui::{
    Component, ComponentPath, Element,
    component_state::ComponentState,
    vdom::{Button, Container},
};

pub struct Scope<'a> {
    path: ComponentPath,
    use_state_index: usize,
    child_index: usize,
    component_states: &'a mut HashMap<ComponentPath, Box<dyn Any>>,
    chat_commands: &'a mut HashMap<String, Vec<Box<dyn Fn()>>>,
    current_element_id: usize,
}

impl<'a> Debug for Scope<'a> {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<'a> Scope<'a> {
    /// New!
    pub fn new(
        component_states: &'a mut HashMap<ComponentPath, Box<dyn Any>>,
        chat_commands: &'a mut HashMap<String, Vec<Box<dyn Fn()>>>,
    ) -> Self {
        Self {
            path: vec![0], // FIXME: dont just alias the type
            use_state_index: 0,
            child_index: 0,
            component_states,
            chat_commands,
            current_element_id: 0,
        }
    }

    fn next_element_id(&mut self) -> usize {
        self.current_element_id = self.current_element_id.saturating_add(1);
        self.current_element_id
    }

    /// Helper to create a standard "foreground" button
    /// This is a best effort. Once the ClickIdPool is completely fragmented this can fail.
    /// Releastically this can only happen when the pool of 1-239 ids is almost fully exhausted.
    pub fn button(&mut self, text: String) -> Element {
        let id = self.next_element_id();

        Element::Button(Button {
            id,
            text: Some(text),
            on_click: None,
            btnstyle: Default::default(),
            style: Default::default(),
            children: None,
            lease_strategy: Some(super::id_pool::LeaseStrategy::High),
        })
    }

    /// Helper to create a "background" button
    /// This is a best effort. Once the ClickIdPool is completely fragmented this can fail.
    /// Releastically this can only happen when the pool of 1-239 ids is almost fully exhausted.
    pub fn background(&mut self) -> Element {
        let id = self.next_element_id();

        Element::Button(Button {
            id,
            text: None,
            on_click: None,
            btnstyle: Default::default(),
            style: Default::default(),
            children: None,
            lease_strategy: Some(super::id_pool::LeaseStrategy::Low),
        })
    }

    /// Helper to create a non-rendered container
    /// Useful for layout
    pub fn container(&mut self) -> Element {
        Element::Container(Container {
            children: None,
            style: Default::default(),
        })
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
    pub fn use_state<T: 'static>(
        &mut self,
        initial_state: impl FnOnce() -> T,
    ) -> ComponentState<T> {
        let mut hook_path = self.path.clone();
        hook_path.push(self.use_state_index);
        self.use_state_index += 1;

        let state = self
            .component_states
            .entry(hook_path.clone())
            .or_insert_with(|| Box::new(ComponentState::new(initial_state())));

        state.downcast_ref::<ComponentState<T>>().unwrap().clone()
    }

    /// On a chat command
    pub fn use_chat(&mut self, command: String, f: impl Fn() + 'static) {
        self.chat_commands
            .entry(command)
            .or_default()
            .push(Box::new(f));

        println!("{:?}", self.chat_commands.len());
    }
}
