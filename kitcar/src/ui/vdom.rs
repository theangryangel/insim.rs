use std::fmt::Debug;

use indexmap::IndexMap;
use insim::insim::{BtnStyle, BtnStyleColour, BtnStyleFlags};

use crate::ui::{styled::Styled, AnyComponent};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ElementId(Vec<usize>);

// FIXME: we should finalise/hash this when all the building is done
impl ElementId {
    pub fn root() -> Self {
        ElementId(vec![])
    }

    pub fn child(&self, index: usize) -> Self {
        let mut path = self.0.clone();
        path.push(index);
        ElementId(path)
    }
}

pub type ElementOnClickFn = Option<Box<dyn Fn() + Send + Sync + 'static>>;

pub enum Element {
    Button {
        text: String,
        style: taffy::Style,
        btnstyle: BtnStyle,
        children: Vec<Element>,
        on_click: ElementOnClickFn,
    },
    Container {
        children: Vec<Element>,
        style: taffy::Style,
    },
    Component(Box<dyn AnyComponent>),
}

impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Element::Button {
                    text,
                    style,
                    children,
                    btnstyle,
                    ..
                },
                Element::Button {
                    text: other_text,
                    style: other_style,
                    children: other_children,
                    btnstyle: other_btnstyle,
                    ..
                },
            ) => {
                text == other_text
                    && style == other_style
                    && children == other_children
                    && btnstyle == other_btnstyle
            },
            (
                Element::Container { children, style },
                Element::Container {
                    children: other_children,
                    style: other_style,
                },
            ) => children == other_children && style == other_style,
            _ => false,
        }
    }
}

impl Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Element::Button { .. } => write!(f, "Element::Button"),
            Element::Container { .. } => write!(f, "Element::Container"),
            Element::Component(_) => write!(f, "Element::Component"),
        }
    }
}

impl Element {
    pub fn container() -> Self {
        Self::Container {
            children: vec![],
            style: taffy::Style::DEFAULT,
        }
    }

    pub fn button(text: &str) -> Self {
        Self::Button {
            text: text.to_string(),
            style: taffy::Style::DEFAULT,
            btnstyle: BtnStyle::default(),
            children: vec![],
            on_click: None,
        }
    }

    pub fn on_click(mut self, f: ElementOnClickFn) -> Self {
        if let Element::Button {
            ref mut btnstyle,
            ref mut on_click,
            ..
        } = self
        {
            btnstyle.flags.set(BtnStyleFlags::CLICK, f.is_some());
            *on_click = f;
        }
        self
    }

    pub fn dark(mut self) -> Self {
        if let Element::Button {
            ref mut btnstyle, ..
        } = self
        {
            btnstyle.flags.set(BtnStyleFlags::DARK, true);
            btnstyle.flags.set(BtnStyleFlags::LIGHT, false);
        }
        self
    }

    pub fn light(mut self) -> Self {
        if let Element::Button {
            ref mut btnstyle, ..
        } = self
        {
            btnstyle.flags.set(BtnStyleFlags::LIGHT, true);
            btnstyle.flags.set(BtnStyleFlags::DARK, false);
        }
        self
    }

    pub fn green(mut self) -> Self {
        if let Element::Button {
            ref mut btnstyle, ..
        } = self
        {
            btnstyle.colour = BtnStyleColour::Ok;
        }
        self
    }

    /// Align text left/start
    pub fn text_align_start(mut self) -> Self {
        if let Element::Button {
            ref mut btnstyle, ..
        } = self
        {
            btnstyle.flags.set(BtnStyleFlags::LEFT, true);
            btnstyle.flags.set(BtnStyleFlags::RIGHT, false);
        }
        self
    }

    /// Align text right/end
    pub fn text_align_end(mut self) -> Self {
        if let Element::Button {
            ref mut btnstyle, ..
        } = self
        {
            btnstyle.flags.set(BtnStyleFlags::RIGHT, true);
            btnstyle.flags.set(BtnStyleFlags::LEFT, false);
        }
        self
    }

    /// Align text center
    pub fn text_align_center(mut self) -> Self {
        if let Element::Button {
            ref mut btnstyle, ..
        } = self
        {
            btnstyle.flags.set(BtnStyleFlags::RIGHT, false);
            btnstyle.flags.set(BtnStyleFlags::LEFT, false);
        }
        self
    }

    pub fn with_child(mut self, val: Element) -> Self {
        match self {
            Self::Container {
                ref mut children, ..
            } => {
                children.push(val);
            },
            Self::Button {
                ref mut children, ..
            } => {
                children.push(val);
            },
            Self::Component(_) => {},
        }

        self
    }

    pub fn try_with_child(mut self, val: Option<Element>) -> Self {
        if let Some(inner) = val {
            self = self.with_child(inner);
        }
        self
    }

    pub fn with_child_if(mut self, val: Element, condition: bool) -> Self {
        if condition {
            self = self.with_child(val);
        }
        self
    }

    pub fn with_children<I>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = Element>,
    {
        for child in children.into_iter() {
            self = self.with_child(child);
        }
        self
    }

    pub fn with_children_if<I>(mut self, children: I, condition: bool) -> Self
    where
        I: IntoIterator<Item = Element>,
    {
        if condition {
            self = self.with_children(children);
        }
        self
    }

    pub fn text(&self) -> &str {
        match self {
            Element::Button { text, .. } => text,
            _ => "",
        }
    }

    pub fn bstyle(&self) -> Option<&BtnStyle> {
        match self {
            Element::Button { btnstyle, .. } => Some(btnstyle),
            _ => None,
        }
    }
}

impl Styled for Element {
    fn style(&self) -> &taffy::Style {
        match self {
            Element::Button { style, .. } => style,
            Element::Container { style, .. } => style,
            _ => {
                unimplemented!()
            },
        }
    }

    fn style_mut(&mut self) -> &mut taffy::Style {
        match self {
            Element::Button { ref mut style, .. } => style,
            Element::Container { ref mut style, .. } => style,
            _ => {
                unimplemented!()
            },
        }
    }
}
