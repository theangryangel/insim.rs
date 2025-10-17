use std::fmt::Debug;

use insim::insim::{BtnStyle, BtnStyleColour, BtnStyleFlags};

pub type ElementId = usize;

pub(crate) struct Button {
    pub(crate) id: ElementId,
    pub(crate) text: String,
    pub(crate) style: taffy::Style,
    pub(crate) btnstyle: BtnStyle,
    pub(crate) on_click: Option<Box<dyn Fn()>>,
}

pub(crate) struct Container {
    pub(crate) children: Option<Vec<Element>>,
    pub(crate) style: taffy::Style,
}

/// Concrete Element - i.e. not a Component
pub enum Element {
    Button(Button),
    Container(Container),
}

impl Debug for Element {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

// impl PartialEq for Element {
//     fn eq(&self, other: &Self) -> bool {
//         match (self, other) {
//             (
//                 Element::Button {
//                     text,
//                     style,
//                     children,
//                     btnstyle,
//                     ..
//                 },
//                 Element::Button {
//                     text: other_text,
//                     style: other_style,
//                     children: other_children,
//                     btnstyle: other_btnstyle,
//                     ..
//                 },
//             ) => {
//                 text == other_text
//                     && style == other_style
//                     && children == other_children
//                     && btnstyle == other_btnstyle
//             },
//             (
//                 Element::Container { children, style },
//                 Element::Container {
//                     children: other_children,
//                     style: other_style,
//                 },
//             ) => children == other_children && style == other_style,
//             _ => false,
//         }
//     }
// }

impl Element {
    pub fn on_click(mut self, f: Option<Box<dyn Fn()>>) -> Self {
        if let Element::Button(Button {
            ref mut btnstyle,
            ref mut on_click,
            ..
        }) = self
        {
            btnstyle.flags.set(BtnStyleFlags::CLICK, f.is_some());
            *on_click = f;
        }
        self
    }

    pub fn dark(mut self) -> Self {
        if let Element::Button(Button {
            ref mut btnstyle, ..
        }) = self
        {
            btnstyle.flags.set(BtnStyleFlags::DARK, true);
            btnstyle.flags.set(BtnStyleFlags::LIGHT, false);
        }
        self
    }

    pub fn light(mut self) -> Self {
        if let Element::Button(Button {
            ref mut btnstyle, ..
        }) = self
        {
            btnstyle.flags.set(BtnStyleFlags::LIGHT, true);
            btnstyle.flags.set(BtnStyleFlags::DARK, false);
        }
        self
    }

    pub fn green(mut self) -> Self {
        if let Element::Button(Button {
            ref mut btnstyle, ..
        }) = self
        {
            btnstyle.colour = BtnStyleColour::Ok;
        }
        self
    }

    /// Align text left/start
    pub fn text_align_start(mut self) -> Self {
        if let Element::Button(Button {
            ref mut btnstyle, ..
        }) = self
        {
            btnstyle.flags.set(BtnStyleFlags::LEFT, true);
            btnstyle.flags.set(BtnStyleFlags::RIGHT, false);
        }
        self
    }

    /// Align text right/end
    pub fn text_align_end(mut self) -> Self {
        if let Element::Button(Button {
            ref mut btnstyle, ..
        }) = self
        {
            btnstyle.flags.set(BtnStyleFlags::RIGHT, true);
            btnstyle.flags.set(BtnStyleFlags::LEFT, false);
        }
        self
    }

    /// Align text center
    pub fn text_align_center(mut self) -> Self {
        if let Element::Button(Button {
            ref mut btnstyle, ..
        }) = self
        {
            btnstyle.flags.set(BtnStyleFlags::RIGHT, false);
            btnstyle.flags.set(BtnStyleFlags::LEFT, false);
        }
        self
    }

    pub fn with_child<E: Into<Option<Element>>>(mut self, val: E) -> Self {
        let val = val.into();
        if val.is_none() {
            return self;
        }
        match self {
            Self::Container(Container {
                ref mut children, ..
            }) => {
                children.get_or_insert_default().push(val.unwrap());
            },
            _ => {},
        }

        self
    }

    pub fn try_with_child(mut self, val: Option<Element>) -> Self {
        if let Some(inner) = val {
            self = self.with_child(inner);
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

    // Styling

    pub fn style(&self) -> &taffy::Style {
        match self {
            Element::Button(Button { style, .. }) => style,
            Element::Container(Container { style, .. }) => style,
        }
    }

    pub fn style_mut(&mut self) -> &mut taffy::Style {
        match self {
            Element::Button(Button { ref mut style, .. }) => style,
            Element::Container(Container { ref mut style, .. }) => style,
        }
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
}
