use std::collections::HashMap;

use insim::insim::BtnStyle;

#[derive(Debug, Default)]
pub struct ElementDiff<'a> {
    /// new keys
    pub new: HashMap<&'a str, &'a Element>,
    /// updated keys
    pub changed: HashMap<&'a str, &'a Element>,
    /// removed
    pub removed: Vec<&'a str>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Element {
    Empty,
    Button {
        key: String, // TODO: auto generate this somehow if not supplied
        text: String,
        style: taffy::Style,
        width: u8,
        height: u8,
        btnstyle: BtnStyle,
    },
    Container {
        children: Vec<Element>,
        style: taffy::Style,
    },
}

impl Element {
    pub fn children(&self) -> &[Element] {
        match self {
            Element::Container { children, .. } => &children,
            _ => &[],
        }
    }

    pub fn text(&self) -> &str {
        match self {
            Element::Button { ref text, .. } => &text,
            _ => "",
        }
    }

    pub fn bstyle(&self) -> Option<&BtnStyle> {
        match self {
            Element::Button { btnstyle, .. } => Some(&btnstyle),
            _ => None,
        }
    }

    pub fn width(&self) -> Option<u8> {
        match self {
            Element::Button { width, .. } => Some(*width),
            _ => None,
        }
    }

    pub fn height(&self) -> Option<u8> {
        match self {
            Element::Button { height, .. } => Some(*height),
            _ => None,
        }
    }

    pub fn collect_renderable(&self) -> HashMap<&str, &Element> {
        let mut result = HashMap::new();
        collect_renderable_recursively(&self, &mut result);
        result
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
