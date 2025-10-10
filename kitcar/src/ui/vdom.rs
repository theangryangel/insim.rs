use std::collections::HashMap;

use indexmap::IndexMap;
use insim::insim::{BtnStyle, BtnStyleColour, BtnStyleFlags};

use crate::ui::styled::Styled;

#[derive(Debug, Clone, PartialEq)]
pub enum Element {
    Button {
        key: String, // TODO: auto generate this somehow if not supplied
        text: String,
        style: taffy::Style,
        btnstyle: BtnStyle,
        children: Vec<Element>,
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
            btnstyle: BtnStyle::default(),
            children: vec![],
        }
    }

    pub fn multi_line_text(key: &str, text: &[&str], height: f32) -> Vec<Self> {
        text.iter()
            .enumerate()
            .map(|(i, f)| {
                Self::button(&format!("{}-{}", key, i), f)
                    .h(height)
                    .text_align_start()
            })
            .collect()
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

    pub fn children(&self) -> &[Element] {
        match self {
            Element::Container { children, .. } => &children,
            Element::Button { children, .. } => &children,
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

    pub fn collect_renderable(&self) -> IndexMap<&str, &Element> {
        let mut result = IndexMap::new();
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
    result: &mut IndexMap<&'a str, &'a Element>,
) {
    // XXX: IndexMap because buttons are Z-Index'ed by ClickId it seems. And this is the best
    // "workaround" we have for this, for now.
    // When the ClickIdPool nears exhaustion, this all goes to shit.

    if let Element::Button { ref key, .. } = vdom {
        let _ = result.insert(key.as_str(), vdom);
    }

    for child in vdom.children().iter() {
        collect_renderable_recursively(child, result);
    }
}
