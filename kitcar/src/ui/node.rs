use insim::insim::BtnStyle;

#[derive(Debug, Clone)]
pub enum NodeKind<Msg> {
    Container(Option<Vec<Node<Msg>>>),
    Button {
        text: String,
        msg: Option<Msg>,
        key: Option<String>,
        bstyle: BtnStyle,
    },
    Empty,
}

#[derive(Debug, Clone)]
pub struct Node<Msg> {
    pub(super) style: Option<taffy::Style>,
    pub(super) kind: NodeKind<Msg>,
}

impl<Msg> Node<Msg> {
    /// Container node: Usually some sort flexbox
    pub fn container() -> Self {
        Self {
            style: Default::default(),
            kind: NodeKind::Container(None),
        }
    }

    /// A clickable button
    pub fn clickable(text: impl Into<String>, mut bstyle: BtnStyle, msg: Msg) -> Self {
        bstyle = bstyle.clickable();
        Self {
            style: Default::default(),
            kind: NodeKind::Button {
                text: text.into(),
                msg: Some(msg),
                key: None,
                bstyle,
            },
        }
    }

    /// A text only button, non-clickable
    pub fn text(text: impl Into<String>, bstyle: BtnStyle) -> Self {
        Self {
            style: Default::default(),
            kind: NodeKind::Button {
                text: text.into(),
                msg: None,
                key: None,
                bstyle,
            },
        }
    }

    /// No output. Effectively this is the same as Option<Node>, however we don't use Option for
    /// convenience.
    pub fn empty() -> Self {
        Self {
            style: Default::default(),
            kind: NodeKind::Empty,
        }
    }

    /// For buttons that perhaps have changable text, or siblings, or things in a loop, we ideally
    /// want to manually set a key to ensure that the system knows what buttons to update more
    /// sensibly.
    pub fn key<K: Into<String>>(mut self, val: K) -> Self {
        if let NodeKind::Button { ref mut key, .. } = self.kind {
            *key = Some(val.into());
        }

        self
    }

    pub fn with_child<E: Into<Option<Node<Msg>>>>(mut self, val: E) -> Self {
        let val = if let Some(val) = val.into() {
            val
        } else {
            return self;
        };

        if let NodeKind::Container(ref mut children) = self.kind {
            children.get_or_insert_default().push(val);
        }

        self
    }

    pub fn with_child_if(mut self, val: Self, condition: bool) -> Self {
        if condition {
            self = self.with_child(val);
        }
        self
    }

    pub fn with_children<I>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        for child in children.into_iter() {
            self = self.with_child(child);
        }
        self
    }

    pub fn with_children_if<I>(mut self, children: I, condition: bool) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        if condition {
            self = self.with_children(children);
        }
        self
    }

    pub fn style(&self) -> Option<&taffy::Style> {
        self.style.as_ref()
    }

    pub fn style_mut(&mut self) -> &mut taffy::Style {
        self.style.get_or_insert_with(Default::default)
    }

    pub fn w(mut self, val: f32) -> Self {
        self.style_mut().size.width = taffy::Dimension::length(val);
        self
    }

    pub fn w_auto(mut self) -> Self {
        self.style_mut().size.width = taffy::Dimension::auto();
        self
    }

    pub fn h(mut self, val: f32) -> Self {
        self.style_mut().size.height = taffy::Dimension::length(val);
        self
    }

    pub fn h_auto(mut self) -> Self {
        self.style_mut().size.height = taffy::Dimension::auto();
        self
    }

    pub fn block(mut self) -> Self {
        self.style_mut().display = taffy::Display::Block;
        self
    }

    pub fn flex(mut self) -> Self {
        self.style_mut().display = taffy::Display::Flex;
        self
    }

    pub fn flex_col(mut self) -> Self {
        self.style_mut().flex_direction = taffy::FlexDirection::Column;
        self
    }

    pub fn flex_row(mut self) -> Self {
        self.style_mut().flex_direction = taffy::FlexDirection::Row;
        self
    }

    pub fn flex_col_reverse(mut self) -> Self {
        self.style_mut().flex_direction = taffy::FlexDirection::ColumnReverse;
        self
    }

    pub fn flex_row_reverse(mut self) -> Self {
        self.style_mut().flex_direction = taffy::FlexDirection::RowReverse;
        self
    }

    pub fn flex_grow(mut self, val: f32) -> Self {
        self.style_mut().flex_grow = val;
        self
    }

    pub fn flex_shrink(mut self, val: f32) -> Self {
        self.style_mut().flex_shrink = val;
        self
    }

    pub fn flex_wrap(mut self) -> Self {
        self.style_mut().flex_wrap = taffy::FlexWrap::Wrap;
        self
    }

    pub fn flex_nowrap(mut self) -> Self {
        self.style_mut().flex_wrap = taffy::FlexWrap::NoWrap;
        self
    }

    pub fn flex_wrap_reverse(mut self) -> Self {
        self.style_mut().flex_wrap = taffy::FlexWrap::WrapReverse;
        self
    }

    pub fn items_start(mut self) -> Self {
        self.style_mut().align_items = Some(taffy::AlignItems::Start);
        self
    }

    pub fn items_end(mut self) -> Self {
        self.style_mut().align_items = Some(taffy::AlignItems::End);
        self
    }

    pub fn items_center(mut self) -> Self {
        self.style_mut().align_items = Some(taffy::AlignItems::Center);
        self
    }

    pub fn items_baseline(mut self) -> Self {
        self.style_mut().align_items = Some(taffy::AlignItems::Baseline);
        self
    }

    pub fn justify_start(mut self) -> Self {
        self.style_mut().justify_content = Some(taffy::JustifyContent::Start);
        self
    }

    pub fn justify_end(mut self) -> Self {
        self.style_mut().justify_content = Some(taffy::JustifyContent::End);
        self
    }

    pub fn justify_center(mut self) -> Self {
        self.style_mut().justify_content = Some(taffy::JustifyContent::Center);
        self
    }

    pub fn justify_between(mut self) -> Self {
        self.style_mut().justify_content = Some(taffy::JustifyContent::SpaceBetween);
        self
    }

    pub fn justify_around(mut self) -> Self {
        self.style_mut().justify_content = Some(taffy::JustifyContent::SpaceAround);
        self
    }

    pub fn content_start(mut self) -> Self {
        self.style_mut().align_content = Some(taffy::AlignContent::Start);
        self
    }

    pub fn content_end(mut self) -> Self {
        self.style_mut().align_content = Some(taffy::AlignContent::End);
        self
    }

    pub fn content_around(mut self) -> Self {
        self.style_mut().align_content = Some(taffy::AlignContent::SpaceAround);
        self
    }

    pub fn content_between(mut self) -> Self {
        self.style_mut().align_content = Some(taffy::AlignContent::SpaceBetween);
        self
    }

    pub fn content_evenly(mut self) -> Self {
        self.style_mut().align_content = Some(taffy::AlignContent::SpaceEvenly);
        self
    }

    pub fn content_stretch(mut self) -> Self {
        self.style_mut().align_content = Some(taffy::AlignContent::Stretch);
        self
    }

    pub fn self_start(mut self) -> Self {
        self.style_mut().align_self = Some(taffy::AlignSelf::FlexStart);
        self
    }

    pub fn self_end(mut self) -> Self {
        self.style_mut().align_self = Some(taffy::AlignSelf::FlexEnd);
        self
    }

    pub fn self_center(mut self) -> Self {
        self.style_mut().align_self = Some(taffy::AlignSelf::Center);
        self
    }

    pub fn self_stretch(mut self) -> Self {
        self.style_mut().align_self = Some(taffy::AlignSelf::Stretch);
        self
    }

    pub fn m(mut self, val: f32) -> Self {
        self.style_mut().margin = taffy::Rect::length(val);
        self
    }

    pub fn mt_auto(mut self) -> Self {
        self.style_mut().margin.top = taffy::LengthPercentageAuto::auto();
        self
    }

    pub fn mt(mut self, val: f32) -> Self {
        self.style_mut().margin.top = taffy::LengthPercentageAuto::length(val);
        self
    }

    pub fn mb(mut self, val: f32) -> Self {
        self.style_mut().margin.bottom = taffy::LengthPercentageAuto::length(val);
        self
    }

    pub fn ml(mut self, val: f32) -> Self {
        self.style_mut().margin.left = taffy::LengthPercentageAuto::length(val);
        self
    }

    pub fn mr(mut self, val: f32) -> Self {
        self.style_mut().margin.right = taffy::LengthPercentageAuto::length(val);
        self
    }

    pub fn mx(mut self, val: f32) -> Self {
        self.style_mut().margin.left = taffy::LengthPercentageAuto::length(val);
        self.style_mut().margin.right = taffy::LengthPercentageAuto::length(val);
        self
    }

    pub fn my(mut self, val: f32) -> Self {
        self.style_mut().margin.top = taffy::LengthPercentageAuto::length(val);
        self.style_mut().margin.bottom = taffy::LengthPercentageAuto::length(val);
        self
    }

    pub fn mx_auto(mut self) -> Self {
        self.style_mut().margin.left = taffy::LengthPercentageAuto::auto();
        self.style_mut().margin.right = taffy::LengthPercentageAuto::auto();
        self
    }

    pub fn my_auto(mut self) -> Self {
        self.style_mut().margin.top = taffy::LengthPercentageAuto::auto();
        self.style_mut().margin.bottom = taffy::LengthPercentageAuto::auto();
        self
    }

    pub fn p(mut self, val: f32) -> Self {
        self.style_mut().padding = taffy::Rect::length(val);
        self
    }

    pub fn pt(mut self, val: f32) -> Self {
        self.style_mut().padding.top = taffy::LengthPercentage::length(val);
        self
    }

    pub fn pb(mut self, val: f32) -> Self {
        self.style_mut().padding.bottom = taffy::LengthPercentage::length(val);
        self
    }

    pub fn pl(mut self, val: f32) -> Self {
        self.style_mut().padding.left = taffy::LengthPercentage::length(val);
        self
    }

    pub fn pr(mut self, val: f32) -> Self {
        self.style_mut().padding.right = taffy::LengthPercentage::length(val);
        self
    }

    pub fn px(mut self, val: f32) -> Self {
        self.style_mut().padding.left = taffy::LengthPercentage::length(val);
        self.style_mut().padding.right = taffy::LengthPercentage::length(val);
        self
    }

    pub fn py(mut self, val: f32) -> Self {
        self.style_mut().padding.top = taffy::LengthPercentage::length(val);
        self.style_mut().padding.bottom = taffy::LengthPercentage::length(val);
        self
    }

    pub fn fit_content(mut self) -> Self {
        self = self.w_auto().h_auto();
        self
    }

    pub fn sized(mut self, width: f32, height: f32) -> Self {
        self = self.w(width).h(height);
        self
    }

    // Maps/wraps child msg -> parent msg
    // Usage: component.render(ctx).map(RootMsg::ParentVariant)
    pub fn map<F, ParentMsg>(self, f: F) -> Node<ParentMsg>
    where
        F: Fn(Msg) -> ParentMsg + Clone,
    {
        let kind = match self.kind {
            NodeKind::Container(Some(c)) => {
                NodeKind::Container(Some(c.into_iter().map(|k| k.map(f.clone())).collect()))
            },
            NodeKind::Container(None) => NodeKind::Container(None),
            NodeKind::Button {
                text,
                msg,
                key,
                bstyle,
            } => NodeKind::Button {
                text,
                msg: msg.map(&f),
                key,
                bstyle,
            },
            NodeKind::Empty => NodeKind::Empty,
        };

        Node {
            kind,
            style: self.style,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    enum TestMsg {
        Click,
    }

    #[derive(Debug, Clone, PartialEq)]
    enum ParentMsg {
        Child(TestMsg),
    }

    #[test]
    fn test_container_creation() {
        let node: Node<TestMsg> = Node::container();
        assert!(matches!(node.kind, NodeKind::Container(None)));
        assert!(node.style.is_none());
    }

    #[test]
    fn test_text_node_creation() {
        let node: Node<TestMsg> = Node::text("Hello", BtnStyle::default());
        match node.kind {
            NodeKind::Button { text, msg, key, .. } => {
                assert_eq!(text, "Hello");
                assert!(msg.is_none());
                assert!(key.is_none());
            },
            _ => panic!("Expected Button node"),
        }
    }

    #[test]
    fn test_clickable_node_creation() {
        let node = Node::clickable("Click me", BtnStyle::default(), TestMsg::Click);
        match node.kind {
            NodeKind::Button { text, msg, key, .. } => {
                assert_eq!(text, "Click me");
                assert_eq!(msg, Some(TestMsg::Click));
                assert!(key.is_none());
            },
            _ => panic!("Expected Button node"),
        }
    }

    #[test]
    fn test_empty_node_creation() {
        let node: Node<TestMsg> = Node::empty();
        assert!(matches!(node.kind, NodeKind::Empty));
    }

    #[test]
    fn test_with_child() {
        let child = Node::text("child", BtnStyle::default());
        let container: Node<TestMsg> = Node::container().with_child(child);

        match container.kind {
            NodeKind::Container(Some(children)) => {
                assert_eq!(children.len(), 1);
            },
            _ => panic!("Expected Container with children"),
        }
    }

    #[test]
    fn test_with_child_on_button_noop() {
        let button: Node<TestMsg> = Node::text("button", BtnStyle::default());
        let child = Node::text("child", BtnStyle::default());
        let result = button.with_child(child);

        assert!(matches!(result.kind, NodeKind::Button { .. }));
    }

    #[test]
    fn test_with_child_if_true() {
        let child = Node::text("child", BtnStyle::default());
        let container: Node<TestMsg> = Node::container().with_child_if(child, true);

        match container.kind {
            NodeKind::Container(Some(children)) => {
                assert_eq!(children.len(), 1);
            },
            _ => panic!("Expected Container with children"),
        }
    }

    #[test]
    fn test_with_child_if_false() {
        let child: Node<TestMsg> = Node::text("child", BtnStyle::default());
        let container: Node<TestMsg> = Node::container().with_child_if(child, false);

        assert!(matches!(container.kind, NodeKind::Container(None)));
    }

    #[test]
    fn test_with_children() {
        let children = vec![
            Node::text("a", BtnStyle::default()),
            Node::text("b", BtnStyle::default()),
            Node::text("c", BtnStyle::default()),
        ];
        let container: Node<TestMsg> = Node::container().with_children(children);

        match container.kind {
            NodeKind::Container(Some(children)) => {
                assert_eq!(children.len(), 3);
            },
            _ => panic!("Expected Container with children"),
        }
    }

    #[test]
    fn test_with_children_if_true() {
        let children = vec![
            Node::text("a", BtnStyle::default()),
            Node::text("b", BtnStyle::default()),
        ];
        let container: Node<TestMsg> = Node::container().with_children_if(children, true);

        match container.kind {
            NodeKind::Container(Some(children)) => {
                assert_eq!(children.len(), 2);
            },
            _ => panic!("Expected Container with children"),
        }
    }

    #[test]
    fn test_with_children_if_false() {
        let children: Vec<Node<TestMsg>> = vec![
            Node::text("a", BtnStyle::default()),
            Node::text("b", BtnStyle::default()),
        ];
        let container: Node<TestMsg> = Node::container().with_children_if(children, false);

        assert!(matches!(container.kind, NodeKind::Container(None)));
    }

    #[test]
    fn test_key_on_button() {
        let node: Node<TestMsg> = Node::text("text", BtnStyle::default()).key("my-key");
        match node.kind {
            NodeKind::Button { key, .. } => {
                assert_eq!(key, Some("my-key".to_string()));
            },
            _ => panic!("Expected Button node"),
        }
    }

    #[test]
    fn test_key_on_container_noop() {
        let node: Node<TestMsg> = Node::container().key("my-key");
        match node.kind {
            NodeKind::Container(None) => {},
            _ => panic!("Expected Container without key modification"),
        }
    }

    #[test]
    fn test_map_transforms_messages() {
        let node = Node::clickable("click", BtnStyle::default(), TestMsg::Click);
        let mapped: Node<ParentMsg> = node.map(ParentMsg::Child);

        match mapped.kind {
            NodeKind::Button { msg, .. } => {
                assert_eq!(msg, Some(ParentMsg::Child(TestMsg::Click)));
            },
            _ => panic!("Expected Button node"),
        }
    }

    #[test]
    fn test_map_preserves_container_structure() {
        let container: Node<TestMsg> = Node::container()
            .with_child(Node::clickable("a", BtnStyle::default(), TestMsg::Click))
            .with_child(Node::text("b", BtnStyle::default()));

        let mapped: Node<ParentMsg> = container.map(ParentMsg::Child);

        match mapped.kind {
            NodeKind::Container(Some(children)) => {
                assert_eq!(children.len(), 2);
            },
            _ => panic!("Expected Container with children"),
        }
    }

    #[test]
    fn test_map_preserves_empty() {
        let node: Node<TestMsg> = Node::empty();
        let mapped: Node<ParentMsg> = node.map(ParentMsg::Child);
        assert!(matches!(mapped.kind, NodeKind::Empty));
    }

    #[test]
    fn test_map_preserves_empty_container() {
        let node: Node<TestMsg> = Node::container();
        let mapped: Node<ParentMsg> = node.map(ParentMsg::Child);
        assert!(matches!(mapped.kind, NodeKind::Container(None)));
    }

    #[test]
    fn test_w_h_sets_dimensions() {
        let node: Node<TestMsg> = Node::container().w(100.0).h(50.0);
        let style = node.style.unwrap();
        assert_eq!(style.size.width, taffy::Dimension::length(100.0));
        assert_eq!(style.size.height, taffy::Dimension::length(50.0));
    }

    #[test]
    fn test_w_auto_h_auto() {
        let node: Node<TestMsg> = Node::container().w_auto().h_auto();
        let style = node.style.unwrap();
        assert_eq!(style.size.width, taffy::Dimension::auto());
        assert_eq!(style.size.height, taffy::Dimension::auto());
    }

    #[test]
    fn test_flex_direction_methods() {
        let col: Node<TestMsg> = Node::container().flex().flex_col();
        assert_eq!(
            col.style.unwrap().flex_direction,
            taffy::FlexDirection::Column
        );

        let row: Node<TestMsg> = Node::container().flex().flex_row();
        assert_eq!(row.style.unwrap().flex_direction, taffy::FlexDirection::Row);

        let col_rev: Node<TestMsg> = Node::container().flex().flex_col_reverse();
        assert_eq!(
            col_rev.style.unwrap().flex_direction,
            taffy::FlexDirection::ColumnReverse
        );

        let row_rev: Node<TestMsg> = Node::container().flex().flex_row_reverse();
        assert_eq!(
            row_rev.style.unwrap().flex_direction,
            taffy::FlexDirection::RowReverse
        );
    }

    #[test]
    fn test_flex_wrap_methods() {
        let wrap: Node<TestMsg> = Node::container().flex_wrap();
        assert_eq!(wrap.style.unwrap().flex_wrap, taffy::FlexWrap::Wrap);

        let nowrap: Node<TestMsg> = Node::container().flex_nowrap();
        assert_eq!(nowrap.style.unwrap().flex_wrap, taffy::FlexWrap::NoWrap);

        let wrap_rev: Node<TestMsg> = Node::container().flex_wrap_reverse();
        assert_eq!(
            wrap_rev.style.unwrap().flex_wrap,
            taffy::FlexWrap::WrapReverse
        );
    }

    #[test]
    fn test_flex_grow_shrink() {
        let node: Node<TestMsg> = Node::container().flex_grow(2.0).flex_shrink(0.5);
        let style = node.style.unwrap();
        assert_eq!(style.flex_grow, 2.0);
        assert_eq!(style.flex_shrink, 0.5);
    }

    #[test]
    fn test_align_items_methods() {
        let start: Node<TestMsg> = Node::container().items_start();
        assert_eq!(
            start.style.unwrap().align_items,
            Some(taffy::AlignItems::Start)
        );

        let end: Node<TestMsg> = Node::container().items_end();
        assert_eq!(end.style.unwrap().align_items, Some(taffy::AlignItems::End));

        let center: Node<TestMsg> = Node::container().items_center();
        assert_eq!(
            center.style.unwrap().align_items,
            Some(taffy::AlignItems::Center)
        );

        let baseline: Node<TestMsg> = Node::container().items_baseline();
        assert_eq!(
            baseline.style.unwrap().align_items,
            Some(taffy::AlignItems::Baseline)
        );
    }

    #[test]
    fn test_justify_content_methods() {
        let start: Node<TestMsg> = Node::container().justify_start();
        assert_eq!(
            start.style.unwrap().justify_content,
            Some(taffy::JustifyContent::Start)
        );

        let end: Node<TestMsg> = Node::container().justify_end();
        assert_eq!(
            end.style.unwrap().justify_content,
            Some(taffy::JustifyContent::End)
        );

        let center: Node<TestMsg> = Node::container().justify_center();
        assert_eq!(
            center.style.unwrap().justify_content,
            Some(taffy::JustifyContent::Center)
        );

        let between: Node<TestMsg> = Node::container().justify_between();
        assert_eq!(
            between.style.unwrap().justify_content,
            Some(taffy::JustifyContent::SpaceBetween)
        );

        let around: Node<TestMsg> = Node::container().justify_around();
        assert_eq!(
            around.style.unwrap().justify_content,
            Some(taffy::JustifyContent::SpaceAround)
        );
    }

    #[test]
    fn test_align_content_methods() {
        let start: Node<TestMsg> = Node::container().content_start();
        assert_eq!(
            start.style.unwrap().align_content,
            Some(taffy::AlignContent::Start)
        );

        let end: Node<TestMsg> = Node::container().content_end();
        assert_eq!(
            end.style.unwrap().align_content,
            Some(taffy::AlignContent::End)
        );

        let around: Node<TestMsg> = Node::container().content_around();
        assert_eq!(
            around.style.unwrap().align_content,
            Some(taffy::AlignContent::SpaceAround)
        );

        let between: Node<TestMsg> = Node::container().content_between();
        assert_eq!(
            between.style.unwrap().align_content,
            Some(taffy::AlignContent::SpaceBetween)
        );

        let evenly: Node<TestMsg> = Node::container().content_evenly();
        assert_eq!(
            evenly.style.unwrap().align_content,
            Some(taffy::AlignContent::SpaceEvenly)
        );

        let stretch: Node<TestMsg> = Node::container().content_stretch();
        assert_eq!(
            stretch.style.unwrap().align_content,
            Some(taffy::AlignContent::Stretch)
        );
    }

    #[test]
    fn test_align_self_methods() {
        let start: Node<TestMsg> = Node::container().self_start();
        assert_eq!(
            start.style.unwrap().align_self,
            Some(taffy::AlignSelf::FlexStart)
        );

        let end: Node<TestMsg> = Node::container().self_end();
        assert_eq!(
            end.style.unwrap().align_self,
            Some(taffy::AlignSelf::FlexEnd)
        );

        let center: Node<TestMsg> = Node::container().self_center();
        assert_eq!(
            center.style.unwrap().align_self,
            Some(taffy::AlignSelf::Center)
        );

        let stretch: Node<TestMsg> = Node::container().self_stretch();
        assert_eq!(
            stretch.style.unwrap().align_self,
            Some(taffy::AlignSelf::Stretch)
        );
    }

    #[test]
    fn test_margin_methods() {
        let m: Node<TestMsg> = Node::container().m(5.0);
        let style = m.style.unwrap();
        assert_eq!(style.margin, taffy::Rect::length(5.0));

        let mt: Node<TestMsg> = Node::container().mt(1.0);
        assert_eq!(
            mt.style.unwrap().margin.top,
            taffy::LengthPercentageAuto::length(1.0)
        );

        let mb: Node<TestMsg> = Node::container().mb(2.0);
        assert_eq!(
            mb.style.unwrap().margin.bottom,
            taffy::LengthPercentageAuto::length(2.0)
        );

        let ml: Node<TestMsg> = Node::container().ml(3.0);
        assert_eq!(
            ml.style.unwrap().margin.left,
            taffy::LengthPercentageAuto::length(3.0)
        );

        let mr: Node<TestMsg> = Node::container().mr(4.0);
        assert_eq!(
            mr.style.unwrap().margin.right,
            taffy::LengthPercentageAuto::length(4.0)
        );
    }

    #[test]
    fn test_margin_xy_methods() {
        let mx: Node<TestMsg> = Node::container().mx(10.0);
        let style = mx.style.unwrap();
        assert_eq!(style.margin.left, taffy::LengthPercentageAuto::length(10.0));
        assert_eq!(
            style.margin.right,
            taffy::LengthPercentageAuto::length(10.0)
        );

        let my: Node<TestMsg> = Node::container().my(10.0);
        let style = my.style.unwrap();
        assert_eq!(style.margin.top, taffy::LengthPercentageAuto::length(10.0));
        assert_eq!(
            style.margin.bottom,
            taffy::LengthPercentageAuto::length(10.0)
        );
    }

    #[test]
    fn test_margin_auto_methods() {
        let mt_auto: Node<TestMsg> = Node::container().mt_auto();
        assert_eq!(
            mt_auto.style.unwrap().margin.top,
            taffy::LengthPercentageAuto::auto()
        );

        let mx_auto: Node<TestMsg> = Node::container().mx_auto();
        let style = mx_auto.style.unwrap();
        assert_eq!(style.margin.left, taffy::LengthPercentageAuto::auto());
        assert_eq!(style.margin.right, taffy::LengthPercentageAuto::auto());

        let my_auto: Node<TestMsg> = Node::container().my_auto();
        let style = my_auto.style.unwrap();
        assert_eq!(style.margin.top, taffy::LengthPercentageAuto::auto());
        assert_eq!(style.margin.bottom, taffy::LengthPercentageAuto::auto());
    }

    #[test]
    fn test_padding_methods() {
        let p: Node<TestMsg> = Node::container().p(5.0);
        let style = p.style.unwrap();
        assert_eq!(style.padding, taffy::Rect::length(5.0));

        let pt: Node<TestMsg> = Node::container().pt(1.0);
        assert_eq!(
            pt.style.unwrap().padding.top,
            taffy::LengthPercentage::length(1.0)
        );

        let pb: Node<TestMsg> = Node::container().pb(2.0);
        assert_eq!(
            pb.style.unwrap().padding.bottom,
            taffy::LengthPercentage::length(2.0)
        );

        let pl: Node<TestMsg> = Node::container().pl(3.0);
        assert_eq!(
            pl.style.unwrap().padding.left,
            taffy::LengthPercentage::length(3.0)
        );

        let pr: Node<TestMsg> = Node::container().pr(4.0);
        assert_eq!(
            pr.style.unwrap().padding.right,
            taffy::LengthPercentage::length(4.0)
        );
    }

    #[test]
    fn test_padding_xy_methods() {
        let px: Node<TestMsg> = Node::container().px(10.0);
        let style = px.style.unwrap();
        assert_eq!(style.padding.left, taffy::LengthPercentage::length(10.0));
        assert_eq!(style.padding.right, taffy::LengthPercentage::length(10.0));

        let py: Node<TestMsg> = Node::container().py(10.0);
        let style = py.style.unwrap();
        assert_eq!(style.padding.top, taffy::LengthPercentage::length(10.0));
        assert_eq!(style.padding.bottom, taffy::LengthPercentage::length(10.0));
    }

    #[test]
    fn test_display_methods() {
        let block: Node<TestMsg> = Node::container().block();
        assert_eq!(block.style.unwrap().display, taffy::Display::Block);

        let flex: Node<TestMsg> = Node::container().flex();
        assert_eq!(flex.style.unwrap().display, taffy::Display::Flex);
    }

    #[test]
    fn test_fit_content() {
        let node: Node<TestMsg> = Node::container().fit_content();
        let style = node.style.unwrap();
        assert_eq!(style.size.width, taffy::Dimension::auto());
        assert_eq!(style.size.height, taffy::Dimension::auto());
    }

    #[test]
    fn test_sized() {
        let node: Node<TestMsg> = Node::container().sized(100.0, 50.0);
        let style = node.style.unwrap();
        assert_eq!(style.size.width, taffy::Dimension::length(100.0));
        assert_eq!(style.size.height, taffy::Dimension::length(50.0));
    }

    #[test]
    fn test_chained_style_methods() {
        let node: Node<TestMsg> = Node::container()
            .flex()
            .flex_col()
            .w(200.0)
            .h(100.0)
            .p(5.0)
            .m(10.0)
            .items_center()
            .justify_between();

        let style = node.style.unwrap();
        assert_eq!(style.display, taffy::Display::Flex);
        assert_eq!(style.flex_direction, taffy::FlexDirection::Column);
        assert_eq!(style.size.width, taffy::Dimension::length(200.0));
        assert_eq!(style.size.height, taffy::Dimension::length(100.0));
        assert_eq!(style.padding, taffy::Rect::length(5.0));
        assert_eq!(style.margin, taffy::Rect::length(10.0));
        assert_eq!(style.align_items, Some(taffy::AlignItems::Center));
        assert_eq!(
            style.justify_content,
            Some(taffy::JustifyContent::SpaceBetween)
        );
    }
}
