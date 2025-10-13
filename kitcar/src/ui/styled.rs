pub trait Styled: Sized {
    fn style_mut(&mut self) -> &mut taffy::Style;
    fn style(&self) -> &taffy::Style;

    fn w(mut self, val: f32) -> Self {
        self.style_mut().size.width = taffy::Dimension::length(val);
        self
    }

    fn w_auto(mut self) -> Self {
        self.style_mut().size.width = taffy::Dimension::auto();
        self
    }

    fn h(mut self, val: f32) -> Self {
        self.style_mut().size.height = taffy::Dimension::length(val);
        self
    }

    fn h_auto(mut self) -> Self {
        self.style_mut().size.height = taffy::Dimension::auto();
        self
    }

    fn block(mut self) -> Self {
        self.style_mut().display = taffy::Display::Block;
        self
    }

    fn flex(mut self) -> Self {
        self.style_mut().display = taffy::Display::Flex;
        self
    }

    fn flex_col(mut self) -> Self {
        self.style_mut().flex_direction = taffy::FlexDirection::Column;
        self
    }

    fn flex_row(mut self) -> Self {
        self.style_mut().flex_direction = taffy::FlexDirection::Row;
        self
    }

    fn flex_col_reverse(mut self) -> Self {
        self.style_mut().flex_direction = taffy::FlexDirection::ColumnReverse;
        self
    }

    fn flex_row_reverse(mut self) -> Self {
        self.style_mut().flex_direction = taffy::FlexDirection::RowReverse;
        self
    }

    fn flex_grow(mut self, val: f32) -> Self {
        self.style_mut().flex_grow = val;
        self
    }

    fn flex_shrink(mut self, val: f32) -> Self {
        self.style_mut().flex_shrink = val;
        self
    }

    fn flex_wrap(mut self) -> Self {
        self.style_mut().flex_wrap = taffy::FlexWrap::Wrap;
        self
    }

    fn flex_nowrap(mut self) -> Self {
        self.style_mut().flex_wrap = taffy::FlexWrap::NoWrap;
        self
    }

    fn flex_wrap_reverse(mut self) -> Self {
        self.style_mut().flex_wrap = taffy::FlexWrap::WrapReverse;
        self
    }

    fn items_start(mut self) -> Self {
        self.style_mut().align_items = Some(taffy::AlignItems::Start);
        self
    }

    fn items_end(mut self) -> Self {
        self.style_mut().align_items = Some(taffy::AlignItems::End);
        self
    }

    fn items_center(mut self) -> Self {
        self.style_mut().align_items = Some(taffy::AlignItems::Center);
        self
    }

    fn items_baseline(mut self) -> Self {
        self.style_mut().align_items = Some(taffy::AlignItems::Baseline);
        self
    }

    fn justify_start(mut self) -> Self {
        self.style_mut().justify_content = Some(taffy::JustifyContent::Start);
        self
    }

    fn justify_end(mut self) -> Self {
        self.style_mut().justify_content = Some(taffy::JustifyContent::End);
        self
    }

    fn justify_center(mut self) -> Self {
        self.style_mut().justify_content = Some(taffy::JustifyContent::Center);
        self
    }

    fn justify_between(mut self) -> Self {
        self.style_mut().justify_content = Some(taffy::JustifyContent::SpaceBetween);
        self
    }

    fn justify_around(mut self) -> Self {
        self.style_mut().justify_content = Some(taffy::JustifyContent::SpaceAround);
        self
    }

    fn content_start(mut self) -> Self {
        self.style_mut().align_content = Some(taffy::AlignContent::Start);
        self
    }

    fn content_end(mut self) -> Self {
        self.style_mut().align_content = Some(taffy::AlignContent::End);
        self
    }

    fn content_around(mut self) -> Self {
        self.style_mut().align_content = Some(taffy::AlignContent::SpaceAround);
        self
    }

    fn content_between(mut self) -> Self {
        self.style_mut().align_content = Some(taffy::AlignContent::SpaceBetween);
        self
    }

    fn content_evenly(mut self) -> Self {
        self.style_mut().align_content = Some(taffy::AlignContent::SpaceEvenly);
        self
    }

    fn content_stretch(mut self) -> Self {
        self.style_mut().align_content = Some(taffy::AlignContent::Stretch);
        self
    }

    fn self_start(mut self) -> Self {
        self.style_mut().align_self = Some(taffy::AlignSelf::FlexStart);
        self
    }

    fn self_end(mut self) -> Self {
        self.style_mut().align_self = Some(taffy::AlignSelf::FlexEnd);
        self
    }

    fn self_center(mut self) -> Self {
        self.style_mut().align_self = Some(taffy::AlignSelf::Center);
        self
    }

    fn self_stretch(mut self) -> Self {
        self.style_mut().align_self = Some(taffy::AlignSelf::Stretch);
        self
    }

    fn m(mut self, val: f32) -> Self {
        self.style_mut().margin = taffy::Rect::length(val);
        self
    }

    fn mt_auto(mut self) -> Self {
        self.style_mut().margin.top = taffy::LengthPercentageAuto::auto();
        self
    }

    fn mt(mut self, val: f32) -> Self {
        self.style_mut().margin.top = taffy::LengthPercentageAuto::length(val);
        self
    }

    fn mb(mut self, val: f32) -> Self {
        self.style_mut().margin.bottom = taffy::LengthPercentageAuto::length(val);
        self
    }

    fn ml(mut self, val: f32) -> Self {
        self.style_mut().margin.left = taffy::LengthPercentageAuto::length(val);
        self
    }

    fn mr(mut self, val: f32) -> Self {
        self.style_mut().margin.right = taffy::LengthPercentageAuto::length(val);
        self
    }

    fn mx(mut self, val: f32) -> Self {
        self.style_mut().margin.left = taffy::LengthPercentageAuto::length(val);
        self.style_mut().margin.right = taffy::LengthPercentageAuto::length(val);
        self
    }

    fn my(mut self, val: f32) -> Self {
        self.style_mut().margin.top = taffy::LengthPercentageAuto::length(val);
        self.style_mut().margin.bottom = taffy::LengthPercentageAuto::length(val);
        self
    }

    fn mx_auto(mut self) -> Self {
        self.style_mut().margin.left = taffy::LengthPercentageAuto::auto();
        self.style_mut().margin.right = taffy::LengthPercentageAuto::auto();
        self
    }

    fn my_auto(mut self) -> Self {
        self.style_mut().margin.top = taffy::LengthPercentageAuto::auto();
        self.style_mut().margin.bottom = taffy::LengthPercentageAuto::auto();
        self
    }

    fn p(mut self, val: f32) -> Self {
        self.style_mut().padding = taffy::Rect::length(val);
        self
    }

    fn pt(mut self, val: f32) -> Self {
        self.style_mut().padding.top = taffy::LengthPercentage::length(val);
        self
    }

    fn pb(mut self, val: f32) -> Self {
        self.style_mut().padding.bottom = taffy::LengthPercentage::length(val);
        self
    }

    fn pl(mut self, val: f32) -> Self {
        self.style_mut().padding.left = taffy::LengthPercentage::length(val);
        self
    }

    fn pr(mut self, val: f32) -> Self {
        self.style_mut().padding.right = taffy::LengthPercentage::length(val);
        self
    }

    fn px(mut self, val: f32) -> Self {
        self.style_mut().padding.left = taffy::LengthPercentage::length(val);
        self.style_mut().padding.right = taffy::LengthPercentage::length(val);
        self
    }

    fn py(mut self, val: f32) -> Self {
        self.style_mut().padding.top = taffy::LengthPercentage::length(val);
        self.style_mut().padding.bottom = taffy::LengthPercentage::length(val);
        self
    }

    fn fit_content(mut self) -> Self {
        self = self.w_auto().h_auto();
        self
    }

    fn sized(mut self, width: f32, height: f32) -> Self {
        self = self.w(width).h(height);
        self
    }
}
