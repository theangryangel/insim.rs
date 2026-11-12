use std::sync::Arc;

use insim::insim::BtnStyle;

pub type TypeInMapper<Msg> = Arc<dyn Fn(String) -> Msg + 'static>;

fn map_typein_mapper<Msg, ParentMsg, F>(mapper: TypeInMapper<Msg>, f: F) -> TypeInMapper<ParentMsg>
where
    F: Fn(Msg) -> ParentMsg + Clone + 'static,
    Msg: 'static,
{
    Arc::new(move |text| f(mapper(text)))
}

#[derive(Clone)]
pub enum NodeKind<Msg> {
    Container {
        children: Option<Vec<Node<Msg>>>,
        visible: bool,
        bstyle: BtnStyle,
    },
    Button {
        text: String,
        msg: Option<Msg>,
        key: Option<String>,
        bstyle: BtnStyle,
        typein: Option<(u8, TypeInMapper<Msg>)>,
    },
    Empty,
}

impl<Msg> std::fmt::Debug for NodeKind<Msg> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Container {
                children,
                visible,
                bstyle,
            } => f
                .debug_struct("Container")
                .field(
                    "children",
                    &children
                        .as_ref()
                        .map(|c| format!("Some({} children)", c.len()))
                        .unwrap_or_else(|| "None".to_string()),
                )
                .field("visible", visible)
                .field("bstyle", bstyle)
                .finish(),
            Self::Button {
                text,
                key,
                bstyle,
                typein,
                ..
            } => f
                .debug_struct("Button")
                .field("text", text)
                .field("key", key)
                .field("bstyle", bstyle)
                .field("typein", &typein.as_ref().map(|(limit, _)| *limit))
                .finish(),
            Self::Empty => f.write_str("Empty"),
        }
    }
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
            kind: NodeKind::Container {
                children: None,
                visible: false,
                bstyle: BtnStyle::default(),
            },
        }
    }

    /// Visible container rendered as a non-interactive background button.
    pub fn background(bstyle: BtnStyle) -> Self {
        Self {
            style: Default::default(),
            kind: NodeKind::Container {
                children: None,
                visible: true,
                bstyle,
            },
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
                typein: None,
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
                typein: None,
            },
        }
    }

    /// A type-in-able button
    pub fn typein<F>(text: impl Into<String>, mut bstyle: BtnStyle, limit: u8, mapper: F) -> Self
    where
        F: Fn(String) -> Msg + 'static,
    {
        bstyle = bstyle.clickable();

        Self {
            style: Default::default(),
            kind: NodeKind::Button {
                text: text.into(),
                msg: None,
                key: None,
                bstyle,
                typein: Some((limit, Arc::new(mapper))),
            },
        }
    }

    /// No output. Effectively this is the same as `Option<Node>`, however we don't use Option for
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

        if let NodeKind::Container {
            ref mut children, ..
        } = self.kind
        {
            children.get_or_insert_default().push(val);
        }

        self
    }

    /// Append a child only when `condition` is true.
    #[deprecated(
        since = "0.2.0",
        note = "use .when(condition, |node| node.with_child(child)) or .with_child(condition.then(|| child))"
    )]
    pub fn with_child_if(mut self, val: Self, condition: bool) -> Self {
        if condition {
            self = self.with_child(val);
        }
        self
    }

    /// Append nodes or optional nodes, skipping `None` entries.
    ///
    /// Items in a collection must share a type; wrap unconditional nodes in
    /// `Some` when mixing them with optional nodes.
    ///
    /// ```
    /// use insim::insim::BtnStyle;
    /// use insim_extra::ui::Node;
    ///
    /// let show_delta = false;
    /// let hud: Node<()> = Node::container().with_children([
    ///     Some(Node::text("Lap 3 / 10", BtnStyle::default())),
    ///     show_delta.then(|| Node::text("+0.5", BtnStyle::default())),
    /// ]);
    /// ```
    pub fn with_children<I, E>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = E>,
        E: Into<Option<Self>>,
    {
        for child in children.into_iter() {
            self = self.with_child(child);
        }
        self
    }

    /// Append nodes or optional nodes only when `condition` is true.
    #[deprecated(
        since = "0.2.0",
        note = "use .when(condition, |node| node.with_children(children))"
    )]
    pub fn with_children_if<I, E>(mut self, children: I, condition: bool) -> Self
    where
        I: IntoIterator<Item = E>,
        E: Into<Option<Self>>,
    {
        if condition {
            self = self.with_children(children);
        }
        self
    }

    /// Conditionally apply a run of builder steps, staying in the fluent chain.
    /// `f` is invoked with `self` only when `condition` holds; otherwise `self`
    /// is returned untouched. Construct children inside the closure to avoid
    /// building them when the condition is false.
    ///
    /// ```
    /// use insim::insim::BtnStyle;
    /// use insim_extra::ui::Node;
    ///
    /// let show_hud = true;
    /// let hud: Node<()> = Node::container()
    ///     .when(show_hud, |node| node.with_children([
    ///         Node::text("Lap 3 / 10", BtnStyle::default()),
    ///         Node::text("Fuel 20%", BtnStyle::default()),
    ///     ]))
    ///     .when(false, |_| panic!("skipped branches are never evaluated"));
    /// ```
    pub fn when<F: FnOnce(Self) -> Self>(self, condition: bool, f: F) -> Self {
        if condition { f(self) } else { self }
    }

    pub fn bstyle(&self) -> Option<&BtnStyle> {
        match self.kind {
            NodeKind::Container { ref bstyle, .. } => Some(bstyle),
            NodeKind::Button { ref bstyle, .. } => Some(bstyle),
            NodeKind::Empty => None,
        }
    }

    pub fn bstyle_mut(&mut self) -> &mut BtnStyle {
        match self.kind {
            NodeKind::Container { ref mut bstyle, .. } => bstyle,
            NodeKind::Button { ref mut bstyle, .. } => bstyle,
            NodeKind::Empty => unimplemented!("You cannot mutate a bstyle on an Empty node"),
        }
    }

    pub fn style(&self) -> Option<&taffy::Style> {
        self.style.as_ref()
    }

    pub fn style_mut(&mut self) -> &mut taffy::Style {
        self.style.get_or_insert_with(Default::default)
    }

    /// Configure Taffy properties while staying in the builder chain.
    ///
    /// The closure receives the existing style, preserving earlier builder steps.
    pub fn with_style(mut self, f: impl FnOnce(&mut taffy::Style)) -> Self {
        f(self.style_mut());
        self
    }

    /// Keep this node in normal flow and ignore its insets (the default).
    pub fn position_static(mut self) -> Self {
        self.style_mut().position = taffy::Position::Static;
        self
    }

    /// Offset this node in normal flow and anchor absolute descendants to it.
    pub fn position_relative(mut self) -> Self {
        self.style_mut().position = taffy::Position::Relative;
        self
    }

    /// Remove this node from normal flow and anchor it to its containing block.
    pub fn position_absolute(mut self) -> Self {
        self.style_mut().position = taffy::Position::Absolute;
        self
    }

    /// Remove this node from normal flow and anchor it to the 200 × 200 viewport.
    pub fn position_fixed(mut self) -> Self {
        self.style_mut().position = taffy::Position::Fixed;
        self
    }

    /// Set all four insets in UI units. Static nodes ignore insets.
    pub fn inset(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.style_mut().inset = taffy::Rect::length(val);
        self
    }

    /// Set left and right insets in UI units. Static nodes ignore insets.
    pub fn inset_x(self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.left(val).right(val)
    }

    /// Set top and bottom insets in UI units. Static nodes ignore insets.
    pub fn inset_y(self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.top(val).bottom(val)
    }

    /// Set the top inset in UI units. Static nodes ignore insets.
    pub fn top(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.style_mut().inset.top = taffy::LengthPercentageAuto::length(val);
        self
    }

    /// Reset the top inset to automatic positioning.
    pub fn top_auto(mut self) -> Self {
        self.style_mut().inset.top = taffy::LengthPercentageAuto::auto();
        self
    }

    /// Set the right inset in UI units. Static nodes ignore insets.
    pub fn right(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.style_mut().inset.right = taffy::LengthPercentageAuto::length(val);
        self
    }

    /// Reset the right inset to automatic positioning.
    pub fn right_auto(mut self) -> Self {
        self.style_mut().inset.right = taffy::LengthPercentageAuto::auto();
        self
    }

    /// Set the bottom inset in UI units. Static nodes ignore insets.
    pub fn bottom(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.style_mut().inset.bottom = taffy::LengthPercentageAuto::length(val);
        self
    }

    /// Reset the bottom inset to automatic positioning.
    pub fn bottom_auto(mut self) -> Self {
        self.style_mut().inset.bottom = taffy::LengthPercentageAuto::auto();
        self
    }

    /// Set the left inset in UI units. Static nodes ignore insets.
    pub fn left(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.style_mut().inset.left = taffy::LengthPercentageAuto::length(val);
        self
    }

    /// Reset the left inset to automatic positioning.
    pub fn left_auto(mut self) -> Self {
        self.style_mut().inset.left = taffy::LengthPercentageAuto::auto();
        self
    }

    pub fn w(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.style_mut().size.width = taffy::Dimension::length(val);
        self
    }

    pub fn w_auto(mut self) -> Self {
        self.style_mut().size.width = taffy::Dimension::auto();
        self
    }

    pub fn h(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.style_mut().size.height = taffy::Dimension::length(val);
        self
    }

    pub fn h_auto(mut self) -> Self {
        self.style_mut().size.height = taffy::Dimension::auto();
        self
    }

    /// Set width and height in UI units.
    pub fn size(self, width: impl Into<f64>, height: impl Into<f64>) -> Self {
        self.w(width).h(height)
    }

    /// Set equal width and height in UI units.
    pub fn square(self, size: impl Into<f64>) -> Self {
        let size = size.into();
        self.size(size, size)
    }

    /// Set width as a percentage of the containing block's width.
    /// Uses CSS percentage values: `50` means 50%, `100` means 100%.
    pub fn w_percent(mut self, val: impl Into<f64>) -> Self {
        self.style_mut().size.width = taffy::Dimension::percent((val.into() / 100.0) as f32);
        self
    }

    /// Set the minimum width in UI units.
    pub fn min_w(mut self, val: impl Into<f64>) -> Self {
        self.style_mut().min_size.width = taffy::LengthPercentageAuto::length(val.into() as f32);
        self
    }

    /// Set the maximum width in UI units.
    pub fn max_w(mut self, val: impl Into<f64>) -> Self {
        self.style_mut().max_size.width = taffy::LengthPercentageAuto::length(val.into() as f32);
        self
    }

    /// Set height as a percentage of the containing block's height.
    /// Uses CSS percentage values: `50` means 50%, `100` means 100%.
    pub fn h_percent(mut self, val: impl Into<f64>) -> Self {
        self.style_mut().size.height = taffy::Dimension::percent((val.into() / 100.0) as f32);
        self
    }

    /// Set the minimum height in UI units.
    pub fn min_h(mut self, val: impl Into<f64>) -> Self {
        self.style_mut().min_size.height = taffy::LengthPercentageAuto::length(val.into() as f32);
        self
    }

    /// Set the maximum height in UI units.
    pub fn max_h(mut self, val: impl Into<f64>) -> Self {
        self.style_mut().max_size.height = taffy::LengthPercentageAuto::length(val.into() as f32);
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

    /// Enable flex layout with col direction.
    pub fn flex_col(mut self) -> Self {
        self.style_mut().display = taffy::Display::Flex;
        self.style_mut().flex_direction = taffy::FlexDirection::Column;
        self
    }

    /// Enable flex layout with row direction.
    pub fn flex_row(mut self) -> Self {
        self.style_mut().display = taffy::Display::Flex;
        self.style_mut().flex_direction = taffy::FlexDirection::Row;
        self
    }

    /// Enable flex layout with col reverse direction.
    pub fn flex_col_reverse(mut self) -> Self {
        self.style_mut().display = taffy::Display::Flex;
        self.style_mut().flex_direction = taffy::FlexDirection::ColumnReverse;
        self
    }

    /// Enable flex layout with row reverse direction.
    pub fn flex_row_reverse(mut self) -> Self {
        self.style_mut().display = taffy::Display::Flex;
        self.style_mut().flex_direction = taffy::FlexDirection::RowReverse;
        self
    }

    /// Set horizontal and vertical spacing between flex or grid items.
    pub fn gap(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.style_mut().gap = taffy::Size::length(val);
        self
    }

    /// Set horizontal spacing between columns, independent of flex direction.
    pub fn gap_x(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.style_mut().gap.width = taffy::LengthPercentage::length(val);
        self
    }

    /// Set vertical spacing between rows, independent of flex direction.
    pub fn gap_y(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.style_mut().gap.height = taffy::LengthPercentage::length(val);
        self
    }

    pub fn flex_grow(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.style_mut().flex_grow = val;
        self
    }

    pub fn flex_shrink(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
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
        self.style_mut().align_items = taffy::AlignItems::START;
        self
    }

    pub fn items_end(mut self) -> Self {
        self.style_mut().align_items = taffy::AlignItems::END;
        self
    }

    pub fn items_center(mut self) -> Self {
        self.style_mut().align_items = taffy::AlignItems::CENTER;
        self
    }

    /// Stretch children with automatic cross sizes to fill the cross axis.
    pub fn items_stretch(mut self) -> Self {
        self.style_mut().align_items = taffy::AlignItems::STRETCH;
        self
    }

    pub fn items_baseline(mut self) -> Self {
        self.style_mut().align_items = taffy::AlignItems::BASELINE;
        self
    }

    pub fn justify_start(mut self) -> Self {
        self.style_mut().justify_content = taffy::JustifyContent::START;
        self
    }

    pub fn justify_end(mut self) -> Self {
        self.style_mut().justify_content = taffy::JustifyContent::END;
        self
    }

    pub fn justify_center(mut self) -> Self {
        self.style_mut().justify_content = taffy::JustifyContent::CENTER;
        self
    }

    pub fn justify_between(mut self) -> Self {
        self.style_mut().justify_content = taffy::JustifyContent::SPACE_BETWEEN;
        self
    }

    pub fn justify_around(mut self) -> Self {
        self.style_mut().justify_content = taffy::JustifyContent::SPACE_AROUND;
        self
    }

    pub fn content_start(mut self) -> Self {
        self.style_mut().align_content = taffy::AlignContent::START;
        self
    }

    pub fn content_end(mut self) -> Self {
        self.style_mut().align_content = taffy::AlignContent::END;
        self
    }

    pub fn content_around(mut self) -> Self {
        self.style_mut().align_content = taffy::AlignContent::SPACE_AROUND;
        self
    }

    pub fn content_between(mut self) -> Self {
        self.style_mut().align_content = taffy::AlignContent::SPACE_BETWEEN;
        self
    }

    pub fn content_evenly(mut self) -> Self {
        self.style_mut().align_content = taffy::AlignContent::SPACE_EVENLY;
        self
    }

    pub fn content_stretch(mut self) -> Self {
        self.style_mut().align_content = taffy::AlignContent::STRETCH;
        self
    }

    /// Reset self alignment to `auto`, following the container's item alignment.
    pub fn self_auto(mut self) -> Self {
        self.style_mut().align_self = None;
        self
    }

    pub fn self_start(mut self) -> Self {
        self.style_mut().align_self = Some(taffy::AlignSelf::FLEX_START);
        self
    }

    pub fn self_end(mut self) -> Self {
        self.style_mut().align_self = Some(taffy::AlignSelf::FLEX_END);
        self
    }

    pub fn self_center(mut self) -> Self {
        self.style_mut().align_self = Some(taffy::AlignSelf::CENTER);
        self
    }

    pub fn self_stretch(mut self) -> Self {
        self.style_mut().align_self = Some(taffy::AlignSelf::STRETCH);
        self
    }

    pub fn m(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.style_mut().margin = taffy::Rect::length(val);
        self
    }

    pub fn mt_auto(mut self) -> Self {
        self.style_mut().margin.top = taffy::LengthPercentageAuto::auto();
        self
    }

    pub fn mt(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.style_mut().margin.top = taffy::LengthPercentageAuto::length(val);
        self
    }

    pub fn mb(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.style_mut().margin.bottom = taffy::LengthPercentageAuto::length(val);
        self
    }

    pub fn ml(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.style_mut().margin.left = taffy::LengthPercentageAuto::length(val);
        self
    }

    pub fn mr(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.style_mut().margin.right = taffy::LengthPercentageAuto::length(val);
        self
    }

    pub fn mx(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.style_mut().margin.left = taffy::LengthPercentageAuto::length(val);
        self.style_mut().margin.right = taffy::LengthPercentageAuto::length(val);
        self
    }

    pub fn my(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
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

    pub fn p(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.style_mut().padding = taffy::Rect::length(val);
        self
    }

    pub fn pt(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.style_mut().padding.top = taffy::LengthPercentage::length(val);
        self
    }

    pub fn pb(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.style_mut().padding.bottom = taffy::LengthPercentage::length(val);
        self
    }

    pub fn pl(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.style_mut().padding.left = taffy::LengthPercentage::length(val);
        self
    }

    pub fn pr(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.style_mut().padding.right = taffy::LengthPercentage::length(val);
        self
    }

    pub fn px(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.style_mut().padding.left = taffy::LengthPercentage::length(val);
        self.style_mut().padding.right = taffy::LengthPercentage::length(val);
        self
    }

    pub fn py(mut self, val: impl Into<f64>) -> Self {
        let val = val.into() as f32;
        self.style_mut().padding.top = taffy::LengthPercentage::length(val);
        self.style_mut().padding.bottom = taffy::LengthPercentage::length(val);
        self
    }

    /// Reset width and height to `auto` (the behavior of `fit_content` before 0.2).
    pub fn size_auto(self) -> Self {
        self.w_auto().h_auto()
    }

    /// Set width to CSS `fit-content`, clamped between intrinsic content sizes
    /// according to the available space. This does not measure button text.
    pub fn w_fit_content(mut self) -> Self {
        self.style_mut().size.width = taffy::Dimension::fit_content();
        self
    }

    /// Set height to CSS `fit-content`. This does not measure button text.
    pub fn h_fit_content(mut self) -> Self {
        self.style_mut().size.height = taffy::Dimension::fit_content();
        self
    }

    /// Set both dimensions to CSS `fit-content`.
    ///
    /// Before 0.2 this set both dimensions to `auto`; use [`Self::size_auto`]
    /// to retain that behavior. Button text is not measured automatically.
    pub fn fit_content(self) -> Self {
        self.w_fit_content().h_fit_content()
    }

    // Maps/wraps child msg -> parent msg
    // Usage: component.render(ctx).map(RootMsg::ParentVariant)
    pub fn map<F, ParentMsg>(self, f: F) -> Node<ParentMsg>
    where
        F: Fn(Msg) -> ParentMsg + Clone + 'static,
        Msg: 'static,
    {
        let kind = match self.kind {
            NodeKind::Container {
                children,
                visible,
                bstyle,
            } => NodeKind::Container {
                children: children.map(|c| c.into_iter().map(|k| k.map(f.clone())).collect()),
                visible,
                bstyle,
            },
            NodeKind::Button {
                text,
                msg,
                key,
                bstyle,
                typein,
            } => NodeKind::Button {
                text,
                msg: msg.map(&f),
                key,
                bstyle,
                typein: typein.map(|(limit, mapper)| (limit, map_typein_mapper(mapper, f.clone()))),
            },
            NodeKind::Empty => NodeKind::Empty,
        };

        Node {
            kind,
            style: self.style,
        }
    }
}

impl Node<std::convert::Infallible> {
    /// Lift a node that provably emits no messages into any message type.
    ///
    /// A `Node<Infallible>` can never carry a message (no value of
    /// [`Infallible`](std::convert::Infallible) exists), so this is a total,
    /// panic-free cast - use it to embed a pure display component (one whose
    /// `Message = Infallible`) into a parent of any message type, instead of
    /// `.map(|_| unreachable!())`.
    pub fn cast<Msg: 'static>(self) -> Node<Msg> {
        self.map(|never| match never {})
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
        assert!(matches!(
            node.kind,
            NodeKind::Container {
                children: None,
                visible: false,
                ..
            }
        ));
        assert!(node.style.is_none());
    }

    #[test]
    fn test_background_creation() {
        let node: Node<TestMsg> = Node::background(BtnStyle::default().dark());
        match node.kind {
            NodeKind::Container {
                children,
                visible,
                bstyle,
            } => {
                assert!(children.is_none());
                assert!(visible);
                assert_eq!(bstyle, BtnStyle::default().dark());
            },
            _ => panic!("Expected Container node"),
        }
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
            NodeKind::Container {
                children: Some(children),
                ..
            } => {
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
    #[allow(deprecated)] // Verify compatibility until the deprecated helper is removed.
    fn test_with_child_if_true() {
        let child = Node::text("child", BtnStyle::default());
        let container: Node<TestMsg> = Node::container().with_child_if(child, true);

        match container.kind {
            NodeKind::Container {
                children: Some(children),
                ..
            } => {
                assert_eq!(children.len(), 1);
            },
            _ => panic!("Expected Container with children"),
        }
    }

    #[test]
    #[allow(deprecated)] // Verify compatibility until the deprecated helper is removed.
    fn test_with_child_if_false() {
        let child: Node<TestMsg> = Node::text("child", BtnStyle::default());
        let container: Node<TestMsg> = Node::container().with_child_if(child, false);

        assert!(matches!(
            container.kind,
            NodeKind::Container {
                children: None,
                visible: false,
                ..
            }
        ));
    }

    #[test]
    fn test_with_children() {
        let children = vec![
            Some(Node::text("a", BtnStyle::default())),
            None,
            Some(Node::text("b", BtnStyle::default())),
            false.then(|| Node::text("hidden", BtnStyle::default())),
            Some(Node::text("c", BtnStyle::default())),
        ];
        let container: Node<TestMsg> = Node::container().with_children(children);

        match container.kind {
            NodeKind::Container {
                children: Some(children),
                ..
            } => {
                let texts: Vec<_> = children
                    .iter()
                    .map(|child| match &child.kind {
                        NodeKind::Button { text, .. } => text.as_str(),
                        _ => panic!("Expected text button"),
                    })
                    .collect();
                assert_eq!(texts, ["a", "b", "c"]);
            },
            _ => panic!("Expected Container with children"),
        }
    }

    #[test]
    #[allow(deprecated)] // Verify compatibility until the deprecated helper is removed.
    fn test_with_children_if_true() {
        let children = vec![
            Node::text("a", BtnStyle::default()),
            Node::text("b", BtnStyle::default()),
        ];
        let container: Node<TestMsg> = Node::container().with_children_if(children, true);

        match container.kind {
            NodeKind::Container {
                children: Some(children),
                ..
            } => {
                assert_eq!(children.len(), 2);
            },
            _ => panic!("Expected Container with children"),
        }
    }

    #[test]
    #[allow(deprecated)] // Verify compatibility until the deprecated helper is removed.
    fn test_with_children_if_false() {
        let children: Vec<Node<TestMsg>> = vec![
            Node::text("a", BtnStyle::default()),
            Node::text("b", BtnStyle::default()),
        ];
        let container: Node<TestMsg> = Node::container().with_children_if(children, false);

        assert!(matches!(
            container.kind,
            NodeKind::Container {
                children: None,
                visible: false,
                ..
            }
        ));
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
            NodeKind::Container {
                children: None,
                visible: false,
                ..
            } => {},
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
            NodeKind::Container {
                children: Some(children),
                ..
            } => {
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
        assert!(matches!(
            mapped.kind,
            NodeKind::Container {
                children: None,
                visible: false,
                ..
            }
        ));
    }

    #[test]
    fn test_map_preserves_visible_container() {
        let node: Node<TestMsg> = Node::background(BtnStyle::default().dark());
        let mapped: Node<ParentMsg> = node.map(ParentMsg::Child);
        match mapped.kind {
            NodeKind::Container {
                children,
                visible,
                bstyle,
            } => {
                assert!(children.is_none());
                assert!(visible);
                assert_eq!(bstyle, BtnStyle::default().dark());
            },
            _ => panic!("Expected Container node"),
        }
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
        assert_eq!(start.style.unwrap().align_items, taffy::AlignItems::START);

        let end: Node<TestMsg> = Node::container().items_end();
        assert_eq!(end.style.unwrap().align_items, taffy::AlignItems::END);

        let center: Node<TestMsg> = Node::container().items_center();
        assert_eq!(center.style.unwrap().align_items, taffy::AlignItems::CENTER);

        let baseline: Node<TestMsg> = Node::container().items_baseline();
        assert_eq!(
            baseline.style.unwrap().align_items,
            taffy::AlignItems::BASELINE
        );
    }

    #[test]
    fn test_justify_content_methods() {
        let start: Node<TestMsg> = Node::container().justify_start();
        assert_eq!(
            start.style.unwrap().justify_content,
            taffy::JustifyContent::START
        );

        let end: Node<TestMsg> = Node::container().justify_end();
        assert_eq!(
            end.style.unwrap().justify_content,
            taffy::JustifyContent::END
        );

        let center: Node<TestMsg> = Node::container().justify_center();
        assert_eq!(
            center.style.unwrap().justify_content,
            taffy::JustifyContent::CENTER
        );

        let between: Node<TestMsg> = Node::container().justify_between();
        assert_eq!(
            between.style.unwrap().justify_content,
            taffy::JustifyContent::SPACE_BETWEEN
        );

        let around: Node<TestMsg> = Node::container().justify_around();
        assert_eq!(
            around.style.unwrap().justify_content,
            taffy::JustifyContent::SPACE_AROUND
        );
    }

    #[test]
    fn test_align_content_methods() {
        let start: Node<TestMsg> = Node::container().content_start();
        assert_eq!(
            start.style.unwrap().align_content,
            taffy::AlignContent::START
        );

        let end: Node<TestMsg> = Node::container().content_end();
        assert_eq!(end.style.unwrap().align_content, taffy::AlignContent::END);

        let around: Node<TestMsg> = Node::container().content_around();
        assert_eq!(
            around.style.unwrap().align_content,
            taffy::AlignContent::SPACE_AROUND
        );

        let between: Node<TestMsg> = Node::container().content_between();
        assert_eq!(
            between.style.unwrap().align_content,
            taffy::AlignContent::SPACE_BETWEEN
        );

        let evenly: Node<TestMsg> = Node::container().content_evenly();
        assert_eq!(
            evenly.style.unwrap().align_content,
            taffy::AlignContent::SPACE_EVENLY
        );

        let stretch: Node<TestMsg> = Node::container().content_stretch();
        assert_eq!(
            stretch.style.unwrap().align_content,
            taffy::AlignContent::STRETCH
        );
    }

    #[test]
    fn test_align_self_methods() {
        let start: Node<TestMsg> = Node::container().self_start();
        assert_eq!(
            start.style.unwrap().align_self,
            Some(taffy::AlignSelf::FLEX_START)
        );

        let end: Node<TestMsg> = Node::container().self_end();
        assert_eq!(
            end.style.unwrap().align_self,
            Some(taffy::AlignSelf::FLEX_END)
        );

        let center: Node<TestMsg> = Node::container().self_center();
        assert_eq!(
            center.style.unwrap().align_self,
            Some(taffy::AlignSelf::CENTER)
        );

        let stretch: Node<TestMsg> = Node::container().self_stretch();
        assert_eq!(
            stretch.style.unwrap().align_self,
            Some(taffy::AlignSelf::STRETCH)
        );
    }

    #[test]
    fn test_margin_methods() {
        let m: Node<TestMsg> = Node::container().m(5.0);
        let style = m.style.unwrap();
        assert_eq!(style.margin, taffy::Rect::length(5.0_f32));

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
        assert_eq!(style.padding, taffy::Rect::length(5.0_f32));

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
        assert_eq!(style.size.width, taffy::Dimension::fit_content());
        assert_eq!(style.size.height, taffy::Dimension::fit_content());
    }

    #[test]
    fn with_style_preserves_existing_properties_and_allows_later_overrides() {
        let spacing = 3.0;
        let node: Node<TestMsg> = Node::container()
            .w(50.0)
            .with_style(|style| {
                style.display = taffy::Display::Grid;
                style.gap = taffy::Size::length(spacing);
            })
            .gap_x(5.0)
            .h(20.0);
        let style = node.style.unwrap();
        assert_eq!(style.display, taffy::Display::Grid);
        assert_eq!(style.size.width, taffy::Dimension::length(50.0));
        assert_eq!(style.size.height, taffy::Dimension::length(20.0));
        assert_eq!(style.gap.width, taffy::LengthPercentage::length(5.0));
        assert_eq!(style.gap.height, taffy::LengthPercentage::length(3.0));
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
        assert_eq!(style.padding, taffy::Rect::length(5.0_f32));
        assert_eq!(style.margin, taffy::Rect::length(10.0_f32));
        assert_eq!(style.align_items, taffy::AlignItems::CENTER);
        assert_eq!(style.justify_content, taffy::JustifyContent::SPACE_BETWEEN);
    }
}
