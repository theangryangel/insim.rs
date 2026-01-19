use std::time::{Duration, Instant};

use insim::{core::string::colours::Colourify, insim::BtnStyle};
use kitcar::ui;
use tokio::{sync::mpsc, task::JoinHandle};

#[derive(Clone)]
enum MarqueeState {
    Scrolling,
    Waiting(Instant),
}

pub struct Marquee {
    text: String,
    width: usize,
    offset: usize,
    state: MarqueeState,
    wait_duration: Duration,
    handle: JoinHandle<()>,
}

impl Drop for Marquee {
    fn drop(&mut self) {
        // probably not required since we break out the loop on rx error
        self.handle.abort();
    }
}

#[derive(Clone, Debug)]
pub enum MarqueeMsg {
    Tick,
}

impl Marquee {
    pub fn new<P: Send + Sync + 'static>(
        text: &str,
        width: usize,
        tx: mpsc::UnboundedSender<P>,
        map: impl Fn(MarqueeMsg) -> P + Send + 'static,
    ) -> Self {
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(150));
            loop {
                let _ = interval.tick().await;
                if tx.send(map(MarqueeMsg::Tick)).is_err() {
                    break;
                }
            }
        });

        Self {
            text: text.to_string(),
            width,
            offset: 0,
            state: MarqueeState::Scrolling,
            wait_duration: Duration::from_secs(3),
            handle,
        }
    }
}

impl ui::Component for Marquee {
    type Message = MarqueeMsg;
    type Props = ();

    fn update(&mut self, msg: Self::Message) {
        match msg {
            MarqueeMsg::Tick => {
                match self.state {
                    MarqueeState::Scrolling => {
                        // pad the text with spaces equal to width on both sides
                        // i.e. [   spaces   ][ TEXT ][   spaces   ]
                        let total_len = self.width + self.text.chars().count() + self.width;

                        self.offset += 1;

                        // have we scrolled past everything?
                        if self.offset >= total_len - self.width {
                            let deadline = Instant::now() + self.wait_duration;
                            self.state = MarqueeState::Waiting(deadline);
                            self.offset = 0; // reset for next time
                        }
                    },

                    MarqueeState::Waiting(deadline) => {
                        if Instant::now() >= deadline {
                            self.state = MarqueeState::Scrolling;
                        }
                    },
                }
            },
        }
    }

    fn render(&self, _props: Self::Props) -> ui::Node<Self::Message> {
        match self.state {
            MarqueeState::Scrolling => {
                let padding = " ".repeat(self.width);
                let full_canvas = format!("{}{}{}", padding, self.text, padding);
                let visible: String = full_canvas
                    .chars()
                    .skip(self.offset)
                    .take(self.width)
                    .collect();

                ui::text(visible.white(), BtnStyle::default().dark()).key("marquee")
            },

            // If waiting, render an empty button
            MarqueeState::Waiting(_) => ui::text("", BtnStyle::default().dark()),
        }
    }
}
