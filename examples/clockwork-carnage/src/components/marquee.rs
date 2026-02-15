use std::time::{Duration, Instant};

use kitcar::ui;
use tokio::{sync::mpsc, task::JoinHandle};

use super::hud_text;

#[derive(Clone)]
enum MarqueeState {
    Scrolling,
    Waiting(Instant),
}

pub struct Marquee {
    width: usize,
    offset: usize,
    scroll_limit: usize,
    canvas: Vec<char>,
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

        let mut canvas = Vec::new();
        canvas.extend(std::iter::repeat_n(' ', width));
        canvas.extend(text.chars());
        canvas.extend(std::iter::repeat_n(' ', width));
        let scroll_limit = canvas.len().saturating_sub(width);

        Self {
            width,
            offset: 0,
            scroll_limit,
            canvas,
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
            MarqueeMsg::Tick => match self.state {
                MarqueeState::Scrolling => {
                    if self.scroll_limit == 0 {
                        return;
                    }

                    self.offset += 1;

                    if self.offset >= self.scroll_limit {
                        let deadline = Instant::now() + self.wait_duration;
                        self.state = MarqueeState::Waiting(deadline);
                        self.offset = 0;
                    }
                },

                MarqueeState::Waiting(deadline) => {
                    if Instant::now() >= deadline {
                        self.state = MarqueeState::Scrolling;
                    }
                },
            },
        }
    }

    fn render(&self, _props: Self::Props) -> ui::Node<Self::Message> {
        match self.state {
            MarqueeState::Scrolling => {
                let end = (self.offset + self.width).min(self.canvas.len());
                let visible: String = self.canvas[self.offset..end].iter().collect();

                ui::text(visible, hud_text().align_left()).key("marquee")
            },

            // If waiting, render an empty button
            MarqueeState::Waiting(_) => ui::text("", hud_text()),
        }
    }
}
