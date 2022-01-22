use std::borrow::Cow;

use itertools::Itertools;
use tui::{
    style::{Color, Style},
    text::{Span, Spans},
};

use crate::widgets::{ChatState, PlayerListState, ServersState};

pub(crate) enum ViewState {
    Browsing,
    Selected,
}

// TODO: start spliting this up once I know where I'm going with this.
pub(crate) struct View {
    pub state: ViewState,
    pub servers: ServersState,
    pub players: PlayerListState,
    pub chat: ChatState,
}

impl View {
    pub fn new() -> Self {
        Self {
            state: ViewState::Browsing,
            servers: ServersState::default(),
            players: PlayerListState::default(),
            chat: ChatState::default(),
        }
    }

    pub fn on_network(&mut self, e: &insim::client::Event) {
        match e {
            insim::client::Event::Disconnected => {
                self.servers.clear();
                self.players.clear();
                self.state = ViewState::Browsing;

                self.chat.push("Connection to relay lost".into());
            }

            insim::client::Event::Connected => {
                self.servers.clear();
                self.players.clear();

                self.chat.push("Connected to relay".into());
            }

            insim::client::Event::Packet(frame) => match frame {
                insim::protocol::Packet::MessageOut(data) => {
                    self.chat.push(data.msg.to_lossy_string());
                }

                _ => {}
            },

            _ => {}
        };

        self.servers.on_network(e);
        self.players.on_network(e);
    }
}

const COLOUR_SEQUENCES: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

pub fn colourify(input: Cow<String>) -> Spans {
    // find the positions in the input for each ^L, ^B...
    let mut indices: Vec<usize> = input
        .chars()
        .into_iter()
        .tuple_windows()
        .positions(|(elem, next)| elem == '^' && COLOUR_SEQUENCES.contains(&next))
        .collect();

    // make sure we've got at least something in the indices
    if indices.get(0) != Some(&0) {
        indices.insert(0, 0);
    }

    // make sure we've got the last item in here as well
    if *indices.last().unwrap() != input.len() {
        indices.push(input.len());
    }

    let mut res = Vec::new();

    for pair in indices.windows(2) {
        if input.chars().nth(pair[0]) != Some('^') {
            res.push(Span::styled(
                input
                    .chars()
                    .skip(pair[0])
                    .take(pair[1] - pair[0])
                    .collect::<String>(),
                Style::default(),
            ));
            continue;
        }

        let style = match input.chars().nth(pair[0] + 1) {
            Some('0') => Style::default(), // should be black, leaving as default
            Some('1') => Style::default().fg(Color::Red),
            Some('2') => Style::default().fg(Color::Green),
            Some('3') => Style::default().fg(Color::Yellow),
            Some('4') => Style::default().fg(Color::Blue),
            Some('5') => Style::default().fg(Color::Blue), // should be purple
            Some('6') => Style::default().fg(Color::LightBlue),
            Some('7') => Style::default(), // should be white, leaving as default
            Some('8') => Style::default(), // default
            Some('9') => Style::default(), // default
            _ => Style::default(),
        };

        res.push(Span::styled(
            input
                .chars()
                .skip(pair[0] + 2)
                .take(pair[1] - pair[0] - 2)
                .collect::<String>(),
            style,
        ))
    }

    Spans::from(res)
}
