//! The `!clock`: a clock in the middle of the screen, ticking each second.
//! Click it to close.
//!
//! A demo of a component's own timer: it starts ticking as it mounts, and the
//! timer stops when it unmounts. A clock that only shows the time could
//! instead redraw itself with `Ui::redraw_at`; a timer is for when something
//! should change.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use insim_extra::ui::{self, Component, Effects, Node, Ui};

const WIDTH: u16 = 40;
const HEIGHT: u16 = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockMsg {
    Tick,
    Close,
}

/// Names the clock's tick.
#[derive(Debug, PartialEq)]
struct Tick;

/// Shows the time it last ticked. Reports `()` when clicked.
#[derive(Debug)]
pub struct Clock {
    time: SystemTime,
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            time: SystemTime::now(),
        }
    }
}

/// `time` as `HH:MM:SS`, in UTC.
fn hms(time: SystemTime) -> String {
    let secs = time
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        % 86_400;
    format!("{:02}:{:02}:{:02}", secs / 3600, secs / 60 % 60, secs % 60)
}

impl Component for Clock {
    type Msg = ClockMsg;
    type Out = ();
    type Props<'a> = ();

    fn mounted(&mut self, (): &()) -> Effects<ClockMsg> {
        Effects::new().every(Tick, Duration::from_secs(1), ClockMsg::Tick)
    }

    fn update(&mut self, msg: ClockMsg) -> Effects<ClockMsg> {
        match msg {
            ClockMsg::Tick => {
                self.time = SystemTime::now();
                Effects::none()
            },
            ClockMsg::Close => Effects::new().emit(()),
        }
    }

    fn render(&self, (): (), _: &mut Ui<'_, ClockMsg>) -> Node<ClockMsg> {
        ui::clickable(
            format!("^7{} ^8UTC", hms(self.time)),
            ui::components::style::panel(),
            ClockMsg::Close,
        )
        .w(WIDTH)
        .h(HEIGHT)
        .position_absolute()
        .top((200 - HEIGHT) / 2)
        .left((200 - WIDTH) / 2)
    }
}

#[cfg(test)]
mod tests {
    use insim::{
        Packet,
        identifiers::ConnectionId,
        insim::{Btc, Btn},
    };
    use insim_extra::ui::{Child, Outbox, Screen, id_pool::ClickIdPool};

    use super::*;

    #[test]
    fn formats_the_time_of_day_in_utc() {
        let at = |secs| UNIX_EPOCH + Duration::from_secs(secs);
        assert_eq!(hms(at(0)), "00:00:00");
        // A day and a bit later.
        assert_eq!(hms(at(86_400 + 13 * 3600 + 4 * 60 + 5)), "13:04:05");
    }

    #[tokio::test(start_paused = true)]
    async fn ticks_each_second_and_closes_when_clicked() {
        #[derive(Debug, Clone, PartialEq)]
        struct Closed;

        let (outbox, mut rx) = Outbox::channel();
        let mut pool = ClickIdPool::new();
        let mut screen = Screen::<Closed>::with_outbox(ConnectionId(2), outbox);
        let view =
            |ui: &mut Ui<'_, Closed>| Some(ui.child(Child::new(Clock::default).on(|()| Closed)));
        let now = || tokio::time::Instant::now().into_std();

        let clickid = screen
            .render(&mut pool, now(), view)
            .into_iter()
            .find_map(|p| match p {
                Packet::Btn(Btn { clickid, text, .. }) if text.ends_with("UTC") => Some(clickid),
                _ => None,
            })
            .expect("the clock is drawn");

        let start = tokio::time::Instant::now();
        for n in 1..=3 {
            let tick = rx.recv().await.expect("a tick");
            assert_eq!(start.elapsed(), Duration::from_secs(n));
            assert!(screen.deliver(tick).is_empty());
            assert!(screen.is_dirty());
            let _ = screen.render(&mut pool, now(), view);
        }

        let click = Btc {
            ucid: ConnectionId(2),
            clickid,
            ..Default::default()
        };
        assert_eq!(screen.handle(&click.into(), &mut pool), [Closed]);

        // Unplaced, it stops ticking.
        let _ = screen.render(&mut pool, now(), |_| None);
        let waited = tokio::time::timeout(Duration::from_secs(60), rx.recv()).await;
        assert!(waited.is_err(), "a closed clock ticked");
    }
}
