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
