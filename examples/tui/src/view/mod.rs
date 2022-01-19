pub mod servers;

use bounded_vec_deque::BoundedVecDeque;
use chrono;
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub(crate) const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub(crate) enum ViewState {
    Browsing,
    Selected,
}

// TODO: start spliting this up once I know where I'm going with this.
pub(crate) struct View {
    pub state: ViewState,
    pub servers: servers::ServersTable,

    pub chat: BoundedVecDeque<String>,
}

impl View {
    pub fn new() -> Self {
        Self {
            state: ViewState::Browsing,
            servers: servers::ServersTable::default(),

            chat: BoundedVecDeque::new(20),
        }
    }

    pub fn on_network(&mut self, e: &insim::client::Event) {
        match e {
            insim::client::Event::Disconnected => {
                self.servers.clear();
                self.state = ViewState::Browsing;
            }

            insim::client::Event::Connected => {
                self.servers.clear();

                self.push_chat("Connected to relay".into());
            }

            insim::client::Event::Packet(frame) => match frame {
                insim::protocol::Packet::MessageOut(data) => {
                    self.push_chat(insim::string::colours::strip(data.msg.to_string()));
                }

                insim::protocol::Packet::Lap(data) => {
                    self.push_chat(format!("lap plid={} lap={}", data.plid, data.lapsdone,));
                }

                insim::protocol::Packet::SplitX(data) => {
                    self.push_chat(format!(
                        "split plid={} split={} etime={}",
                        data.plid, data.split, data.etime,
                    ));
                }

                _ => {}
            },

            _ => {}
        };

        match self.state {
            ViewState::Browsing => {
                self.servers.on_network(e);
            }

            _ => {}
        }
    }

    pub fn push_chat(&mut self, data: String) {
        self.chat.push_front(format!(
            "{}: {}",
            chrono::Local::now().format(DATETIME_FORMAT),
            data,
        ));
    }

    pub fn next_server(&mut self) {
        self.servers.scroll_next();
    }

    pub fn previous_server(&mut self) {
        self.servers.scroll_prev();
    }

    pub fn render_chat<B: tui::backend::Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        // Chat
        let chat_items: Vec<ListItem> = self
            .chat
            .iter()
            .map(|item| ListItem::new(item.to_string()))
            .collect();

        let items = List::new(chat_items)
            .block(Block::default().borders(Borders::ALL).title("Chat"))
            .highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_widget(items, area);
    }

    pub fn render<B: tui::backend::Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let inner = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(10)].as_ref())
            .split(area);

        self.render_chat(f, inner[1]);

        match self.state {
            ViewState::Browsing => {
                self.servers.render(f, inner[0]);
            }

            ViewState::Selected => {
                // Controls
                /*
                let text = Text::from(Spans::from(vec![
                    Span::raw("controls: "),
                    bold("\u{2191}\u{2193}"), // up - down
                    Span::raw(" = scroll, "),
                    bold("\u{21B5}"), // enter
                    Span::raw(" = select player, "),
                    bold("q"),
                    Span::raw(" or "),
                    bold("\u{238B} esc"),
                    Span::raw(" = back"),
                ]));

                let help = Paragraph::new(text);
                f.render_widget(help, inner[0]);
                */

                // Players
                let t = Block::default().title("Players");
                f.render_widget(t, inner[0]);
            }
        }
    }
}
