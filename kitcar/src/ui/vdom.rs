use std::collections::HashMap;

use insim::insim::{BtnStyle, BtnStyleFlags};

use crate::ui::styled::Styled;

#[derive(Debug, Clone, PartialEq)]
pub enum Element {
    Button {
        key: String, // TODO: auto generate this somehow if not supplied
        text: String,
        style: taffy::Style,
        btnstyle: BtnStyle,
    },
    Container {
        children: Vec<Element>,
        style: taffy::Style,
    },
}

impl Element {
    pub fn container() -> Self {
        Self::Container {
            children: vec![],
            style: taffy::Style::DEFAULT,
        }
    }

    pub fn button(key: &str, text: &str) -> Self {
        Self::Button {
            key: key.to_string(),
            text: text.to_string(),
            style: taffy::Style::DEFAULT,
            btnstyle: BtnStyle::default().dark(),
        }
    }

    pub fn clickable(mut self) -> Self {
        if let Element::Button {
            ref mut btnstyle, ..
        } = self
        {
            btnstyle.flags.set(BtnStyleFlags::CLICK, true);
        }
        self
    }

    pub fn with_child(mut self, val: Element) -> Self {
        if let Element::Container {
            ref mut children, ..
        } = self
        {
            children.push(val);
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
        I: IntoIterator<Item = Option<Element>>,
    {
        for child in children.into_iter().flatten() {
            self = self.with_child(child);
        }
        self
    }

    pub fn with_children_if<I>(mut self, children: I, condition: bool) -> Self
    where
        I: IntoIterator<Item = Option<Element>>,
    {
        if condition {
            self = self.with_children(children);
        }
        self
    }

    pub fn children(&self) -> &[Element] {
        match self {
            Element::Container { children, .. } => &children,
            _ => &[],
        }
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

    pub fn collect_renderable(&self) -> HashMap<&str, &Element> {
        let mut result = HashMap::new();
        collect_renderable_recursively(&self, &mut result);
        result
    }
}

impl Styled for Element {
    fn style(&self) -> &taffy::Style {
        match self {
            Element::Button { style, .. } => style,
            Element::Container { style, .. } => style,
        }
    }

    fn style_mut(&mut self) -> &mut taffy::Style {
        match self {
            Element::Button { ref mut style, .. } => style,
            Element::Container { ref mut style, .. } => style,
        }
    }
}

fn collect_renderable_recursively<'a>(
    vdom: &'a Element,
    result: &mut HashMap<&'a str, &'a Element>,
) {
    match vdom {
        &Element::Button { ref key, .. } => {
            let _ = result.insert(key.as_str(), vdom);
        },
        _ => {
            for child in vdom.children().iter() {
                collect_renderable_recursively(child, result);
            }
        },
    }
}
