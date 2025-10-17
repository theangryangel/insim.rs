use crate::ui::{Element, Scope};

pub type ComponentPath = Vec<usize>;

/// Trait for users to implement a Ui for a single connection
pub trait Component: 'static {
    type Props: Send + Sync + Clone;

    /// Render
    fn render(props: Self::Props, cx: &mut Scope) -> Option<Element>;
}
