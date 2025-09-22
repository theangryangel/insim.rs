//! Component trait

use super::vdom::Element;

pub trait Component {
    type Props: Clone + PartialEq;

    fn render(&self, props: &Self::Props) -> Element;

    fn should_render(&self, old_props: &Self::Props, new_props: &Self::Props) -> bool {
        old_props != new_props
    }
}

// A wrapper to turn a function into a Component
#[derive(Debug)]
pub struct FnComponent<F, P>
where
    F: Fn(&P) -> Element,
    P: Clone + PartialEq,
{
    f: F,
    _props: std::marker::PhantomData<P>,
}

// Implement the Component trait for FnComponent
impl<F, P> Component for FnComponent<F, P>
where
    F: Fn(&P) -> Element,
    P: Clone + PartialEq,
{
    type Props = P;
    fn render(&self, props: &Self::Props) -> Element {
        (self.f)(props)
    }
}

// Convert a function into a FnComponent
impl<F, P> From<F> for FnComponent<F, P>
where
    F: Fn(&P) -> Element,
    P: Clone + PartialEq,
{
    fn from(f: F) -> Self {
        Self {
            f,
            _props: std::marker::PhantomData,
        }
    }
}
