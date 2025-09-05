//! UI Node and associated items
use std::{
    borrow::Cow,
    fmt,
    hash::{DefaultHasher, Hash, Hasher},
    ops::{Deref, DerefMut},
};

use insim::insim::BtnStyle;
use taffy::{Layout, Style};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
/// Unique key for a UINode.
/// Allows the Renderer to only send minimal updates to LFS.
pub struct UINodeKey(pub u8);

impl fmt::Display for UINodeKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for UINodeKey {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UINodeKey {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<u8> for UINodeKey {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug)]
/// UINode
pub enum UINode {
    /// Rendered items generate buttons in LFS
    Rendered {
        /// Requested layout
        layout: Style,
        /// Button style
        style: BtnStyle,
        /// Button Text
        text: Cow<'static, str>,
        /// *Your* ClickId - the ClickId sent to LFS will be assigned
        key: UINodeKey,
        /// Child nodes
        children: Vec<Self>,
    },
    /// Unrendered items are just used to help the layout generation
    Unrendered {
        /// Requested layout
        layout: Style,
        /// Child nodes
        children: Vec<Self>,
    },
}

impl UINode {
    pub(crate) fn bstyle(&self) -> Option<BtnStyle> {
        match self {
            UINode::Rendered { style, .. } => Some(style.clone()),
            UINode::Unrendered { .. } => None,
        }
    }

    pub(crate) fn text(&self) -> Option<Cow<'static, str>> {
        match self {
            UINode::Rendered { text, .. } => Some(text.clone()),
            UINode::Unrendered { .. } => None,
        }
    }

    pub(crate) fn hash(&self, layout: &Layout) -> Option<u64> {
        if let UINode::Rendered {
            text, key, style, ..
        } = self
        {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            text.hash(&mut hasher);
            style.hash(&mut hasher);
            layout.location.x.to_bits().hash(&mut hasher);
            layout.location.y.to_bits().hash(&mut hasher);
            layout.size.width.to_bits().hash(&mut hasher);
            layout.size.height.to_bits().hash(&mut hasher);
            Some(hasher.finish())
        } else {
            None
        }
    }

    pub(crate) fn children(&self) -> &[Self] {
        match self {
            Self::Unrendered { children, .. } => children,
            Self::Rendered { children, .. } => children,
        }
    }
}
