#[derive(Debug, Clone)]
pub enum NodeKind<Msg> {
    Container(Vec<Node<Msg>>),
    Button {
        text: String,
        msg: Option<Msg>,
        key: Option<String>,
    },
    Empty,
}

#[derive(Debug, Clone)]
pub struct Node<Msg> {
    // TODO: add taffy style
    // style: taffy::Style,
    kind: NodeKind<Msg>,
}

impl<Msg> Node<Msg> {
    pub fn container(children: Vec<Node<Msg>>) -> Self {
        Self {
            kind: NodeKind::Container(children),
        }
    }
    pub fn button(text: impl Into<String>, msg: Msg) -> Self {
        Self {
            kind: NodeKind::Button {
                text: text.into(),
                msg: Some(msg),
                key: None,
            },
        }
    }
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            kind: NodeKind::Button {
                text: text.into(),
                msg: None,
                key: None,
            },
        }
    }
    pub fn empty() -> Self {
        Self {
            kind: NodeKind::Empty,
        }
    }

    // pub fn with_style(mut self, style: taffy::Style) -> Self {
    //     self.style = style;
    //     self
    // }

    // Maps/wraps child msg -> parent msg
    // Usage: component.render(ctx).map(RootMsg::ParentVariant)
    pub fn map<F, ParentMsg>(self, f: F) -> Node<ParentMsg>
    where
        F: Fn(Msg) -> ParentMsg + Clone,
    {
        let kind = match self.kind {
            NodeKind::Container(c) => {
                NodeKind::Container(c.into_iter().map(|k| k.map(f.clone())).collect())
            },
            NodeKind::Button { text, msg, key } => NodeKind::Button {
                text,
                msg: msg.map(&f),
                key,
            },
            NodeKind::Empty => NodeKind::Empty,
        };

        Node { kind }
    }
}
