#[derive(Debug, Clone)]
pub enum Node<Msg> {
    Container(Vec<Node<Msg>>),
    Button {
        text: String,
        msg: Msg,
        key: Option<String>,
    },
    Empty,
}

impl<Msg> Node<Msg> {
    // Maps/wraps child msg -> parent msg
    // Usage: component.render(ctx).map(RootMsg::ParentVariant)
    pub fn map<F, ParentMsg>(self, f: F) -> Node<ParentMsg>
    where
        F: Fn(Msg) -> ParentMsg + Clone,
    {
        match self {
            Node::Container(c) => {
                Node::Container(c.into_iter().map(|k| k.map(f.clone())).collect())
            },
            Node::Button { text, msg, key } => Node::Button {
                text,
                msg: f(msg),
                key,
            },
            Node::Empty => Node::Empty,
        }
    }
}
