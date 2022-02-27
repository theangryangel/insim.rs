use std::borrow::Cow;

use crate::view::colourify;
use bounded_vec_deque::BoundedVecDeque;
use tui::widgets::{StatefulWidget, Widget};
use tui::{
    layout::Rect,
    widgets::{Block, Borders, List, ListItem},
};

pub(crate) const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub struct ChatState {
    inner: BoundedVecDeque<String>,
}

impl std::default::Default for ChatState {
    fn default() -> Self {
        Self {
            inner: BoundedVecDeque::new(20),
        }
    }
}

impl ChatState {
    pub fn push(&mut self, data: String) {
        self.inner.push_front(format!(
            "{}: {}",
            chrono::Local::now().format(DATETIME_FORMAT).to_string(),
            data,
        ));
    }
}

#[derive(Default)]
pub struct ChatWidget {}

impl StatefulWidget for ChatWidget {
    type State = ChatState;

    fn render(self, area: Rect, buf: &mut tui::buffer::Buffer, state: &mut Self::State) {
        // Chat
        let chat_items: Vec<ListItem> = state
            .inner
            .iter()
            .map(|item| ListItem::new(colourify(Cow::Borrowed(item))))
            .collect();

        let items =
            List::new(chat_items).block(Block::default().borders(Borders::ALL).title("Chat"));
        Widget::render(items, area, buf);
    }
}
