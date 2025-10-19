use crate::ui::{Element, Scope};

pub type ComponentPath = Vec<usize>;

/// Trait to implement a reusable series/group of buttons
pub trait Component: 'static {
    type Props: Send + Sync + Clone;

    /// Render
    fn render(props: Self::Props, cx: &mut Scope) -> Option<Element>;
}
