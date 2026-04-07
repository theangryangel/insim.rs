use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use insim_extras::ui;
use tokio::{task::JoinHandle, time::Instant as TokioInstant};

use super::theme::hud_text;

pub struct Marquee {
    wait_ticks: u64,
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
    pub fn new(invalidator: ui::InvalidateHandle) -> Self {
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

        Self {
            wait_ticks,
            tick_count,
            handle,
        }
    }
}

pub struct MarqueeProps<'a> {
    pub text: &'a str,
    pub width: usize,
}

impl ui::Component for Marquee {
    type Message = ();
    type Props<'a> = MarqueeProps<'a>;

    fn render(&self, props: Self::Props<'_>) -> ui::Node<Self::Message> {
        let mut canvas = Vec::new();
        canvas.extend(std::iter::repeat_n(' ', props.width));
        canvas.extend(props.text.chars());
        canvas.extend(std::iter::repeat_n(' ', props.width));
        let scroll_limit = canvas.len().saturating_sub(props.width);

        let w = props.width as f32;

        if scroll_limit == 0 {
            let end = props.width.min(canvas.len());
            let visible: String = canvas[..end].iter().collect();
            return ui::text(visible, hud_text().align_left())
                .w(w)
                .h(5.)
                .key("marquee");
        }

        let cycle_ticks = scroll_limit as u64 + self.wait_ticks;
        let phase = self.tick_count.load(Ordering::Relaxed) % cycle_ticks;

        if phase < scroll_limit as u64 {
            let offset = phase as usize;
            let end = (offset + props.width).min(canvas.len());
            let visible: String = canvas[offset..end].iter().collect();
            return ui::text(visible, hud_text()).w(w).h(5.).key("marquee");
        }

        // waiting phase at the end of each loop
        ui::text("", hud_text()).w(w).h(5.)
    }
}
