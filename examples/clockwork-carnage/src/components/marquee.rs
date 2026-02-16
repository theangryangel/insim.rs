use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use kitcar::ui;
use tokio::{task::JoinHandle, time::Instant as TokioInstant};

use super::hud_text;

pub struct Marquee {
    width: usize,
    scroll_limit: usize,
    wait_ticks: u64,
    canvas: Vec<char>,
    tick_count: Arc<AtomicU64>,
    handle: JoinHandle<()>,
}

impl Drop for Marquee {
    fn drop(&mut self) {
        // probably not required since we break out the loop on rx error
        self.handle.abort();
    }
}

impl Marquee {
    pub fn new(text: &str, width: usize, invalidator: ui::InvalidateHandle) -> Self {
        let period = Duration::from_millis(150);
        let wait_duration = Duration::from_secs(3);
        let wait_ticks = (wait_duration.as_millis() as u64).div_ceil(period.as_millis() as u64);

        let tick_count = Arc::new(AtomicU64::new(0));
        let tick_counter = tick_count.clone();
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval_at(TokioInstant::now() + period, period);
            loop {
                let _ = interval.tick().await;
                let _ = tick_counter.fetch_add(1, Ordering::Relaxed);
                invalidator.invalidate();
            }
        });

        let mut canvas = Vec::new();
        canvas.extend(std::iter::repeat_n(' ', width));
        canvas.extend(text.chars());
        canvas.extend(std::iter::repeat_n(' ', width));
        let scroll_limit = canvas.len().saturating_sub(width);

        Self {
            width,
            scroll_limit,
            wait_ticks,
            canvas,
            tick_count,
            handle,
        }
    }
}

impl ui::Component for Marquee {
    type Message = ();
    type Props = ();

    fn render(&self, _props: Self::Props) -> ui::Node<Self::Message> {
        if self.scroll_limit == 0 {
            let end = self.width.min(self.canvas.len());
            let visible: String = self.canvas[..end].iter().collect();
            return ui::text(visible, hud_text().align_left()).key("marquee");
        }

        let cycle_ticks = self.scroll_limit as u64 + self.wait_ticks;
        let phase = self.tick_count.load(Ordering::Relaxed) % cycle_ticks;

        if phase < self.scroll_limit as u64 {
            let offset = phase as usize;
            let end = (offset + self.width).min(self.canvas.len());
            let visible: String = self.canvas[offset..end].iter().collect();
            return ui::text(visible, hud_text().align_left()).key("marquee");
        }

        // waiting phase at the end of each loop
        ui::text("", hud_text())
    }
}
