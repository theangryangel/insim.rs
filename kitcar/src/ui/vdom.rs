use std::collections::HashMap;

use insim::insim::BtnStyle;

use crate::ui::styled::Styled;

#[derive(Debug, Clone, PartialEq)]
pub enum Element {
    Empty,
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
            btnstyle: BtnStyle::default(),
        }
    }

    pub fn with_child(mut self, val: Element) -> Self {
        match self {
            Element::Container {
                ref mut children, ..
            } => children.push(val),
            _ => panic!("Empty cannot have children"),
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
            Element::Empty => panic!("There is no style on an Empty Element"),
            Element::Button { style, .. } => style,
            Element::Container { style, .. } => style,
        }
    }

    fn style_mut(&mut self) -> &mut taffy::Style {
        match self {
            Element::Empty => panic!("cannot set a style on an empty Element"),
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
