use std::{cell::Cell, marker::PhantomData};

use insim::insim::BtnStyle;
use insim_extras::ui;

pub const PAGE_SIZE: usize = 20;

#[derive(Debug, Default)]
pub struct ScrollList<Item> {
    offset: usize,
    filter: String,
    last_filtered_count: Cell<usize>,
    _marker: PhantomData<Item>,
}

#[derive(Debug, Clone)]
pub enum ScrollMsg {
    Up,
    Down,
    Filter(String),
    ItemClicked(usize),
}

pub struct ScrollListProps<'a, Item> {
    pub items: &'a [Item],
    pub render_item: Box<dyn Fn(&Item, usize) -> ui::Node<ScrollMsg>>,
    pub filter_item: Box<dyn Fn(&Item, &str) -> bool>,
}

impl<Item: 'static> ui::Component for ScrollList<Item> {
    type Message = ScrollMsg;
    type Props<'a> = ScrollListProps<'a, Item>;

    fn update(&mut self, msg: ScrollMsg) {
        match msg {
            ScrollMsg::Up => self.offset = self.offset.saturating_sub(1),
            ScrollMsg::Down => {
                if self.offset + PAGE_SIZE < self.last_filtered_count.get() {
                    self.offset += 1;
                }
            },
            ScrollMsg::Filter(text) => {
                self.filter = text;
                self.offset = 0;
            },
            ScrollMsg::ItemClicked(_) => {},
        }
    }

    fn render(&self, props: Self::Props<'_>) -> ui::Node<ScrollMsg> {
        let filtered: Vec<(usize, &Item)> = props
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| (props.filter_item)(item, &self.filter))
            .collect();

        let len = filtered.len();
        self.last_filtered_count.set(len);

        let end = (self.offset + PAGE_SIZE).min(len);
        let visible = &filtered[self.offset..end];
        let needs_scroll = len > PAGE_SIZE;

        let filter_label = if self.filter.is_empty() {
            "Filter...".to_string()
        } else {
            format!("Filter: {}", self.filter)
        };

        let filter_input = ui::typein(
            filter_label,
            BtnStyle::style_interactive(),
            64,
            ScrollMsg::Filter,
        )
        .block()
        .h(5.);

        let item_col = visible.iter().fold(
            ui::container().flex().flex_col().flex_grow(1.0),
            |col, (orig_idx, item)| col.with_child((props.render_item)(item, *orig_idx)),
        );

        let items_row = if !needs_scroll {
            item_col
        } else {
            let up_btn = if self.offset > 0 {
                ui::clickable("\u{25b2}", BtnStyle::style_interactive(), ScrollMsg::Up)
                    .key("scroll-up")
                    .h(5.)
            } else {
                ui::text("\u{25b2}", BtnStyle::style_unavailable())
                    .key("scroll-up")
                    .h(5.)
            };

            let indicator = ui::text(
                format!("{}/{}", self.offset + 1, len.saturating_sub(PAGE_SIZE) + 1),
                BtnStyle::style_readonly(),
            )
            .key("scroll-indicator")
            .h(5.);

            let down_btn = if self.offset + PAGE_SIZE < len {
                ui::clickable("\u{25bc}", BtnStyle::style_interactive(), ScrollMsg::Down)
                    .key("scroll-down")
                    .h(5.)
            } else {
                ui::text("\u{25bc}", BtnStyle::style_unavailable())
                    .key("scroll-down")
                    .h(5.)
            };

            let sidebar = ui::container()
                .flex()
                .flex_col()
                .w(8.)
                .with_child(up_btn)
                .with_child(indicator)
                .with_child(down_btn);

            ui::container()
                .flex()
                .flex_row()
                .with_child(item_col)
                .with_child(sidebar)
        };

        ui::container()
            .flex()
            .flex_col()
            .with_child(filter_input)
            .with_child(items_row)
    }
}
