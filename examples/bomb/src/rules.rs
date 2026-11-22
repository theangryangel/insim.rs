//! Bomb's rules: runs, checkpoints and penalties. Plain state and methods; time
//! is passed in, so they are easy to test.

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use insim::{identifiers::PlayerId, insim::PlayerType};
use insim_extra::world::PlayerInfo;

/// Collisions at or above this closing speed cost the full collision penalty.
const COLLISION_THRESHOLD_MPS: f32 = 30.0;

#[derive(Debug, Clone)]
pub struct Config {
    /// The window to reach the first checkpoint, and after each finish.
    pub checkpoint_timeout: Duration,
    /// How much each checkpoint shrinks the window, and what a reset costs.
    pub checkpoint_penalty: Duration,
    /// The most a collision can cost.
    pub collision_max_penalty: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            checkpoint_timeout: Duration::from_secs(30),
            checkpoint_penalty: Duration::from_millis(250),
            collision_max_penalty: Duration::from_millis(500),
        }
    }
}

/// The order of the layout's InSim circles. Crossing them out of order does not
/// count. An empty sequence (no layout loaded) accepts any order.
#[derive(Debug, Default)]
pub struct CircleSequence(Vec<u8>);

impl CircleSequence {
    pub fn accumulate(&mut self, indices: impl Iterator<Item = u8>) {
        self.0.extend(indices);
    }

    pub fn finalize(&mut self) {
        self.0.sort_unstable();
        self.0.dedup();
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    fn is_next(&self, last: u8, candidate: u8) -> bool {
        if self.0.is_empty() {
            return true;
        }
        self.0
            .iter()
            .position(|&i| i == last)
            .map(|pos| self.0[(pos + 1) % self.0.len()])
            == Some(candidate)
    }
}

/// A run in progress.
#[derive(Debug, Clone)]
pub struct Run {
    pub started_at: Instant,
    pub deadline: Instant,
    pub window: Duration,
    pub checkpoints: u32,
    pub uname: String,
    pub pname: String,
    last_circle: u8,
}

impl Run {
    pub fn survival(&self, now: Instant) -> Duration {
        self.deadline
            .min(now)
            .saturating_duration_since(self.started_at)
    }

    pub fn time_left(&self, now: Instant) -> Duration {
        self.deadline.saturating_duration_since(now)
    }
}

/// What crossing a circle or checkpoint did.
#[derive(Debug)]
pub enum Outcome {
    Started,
    Extended {
        checkpoints: u32,
        time_left: Duration,
    },
    Refreshed {
        checkpoints: u32,
        window: Duration,
    },
}

/// A time penalty taken from a run.
#[derive(Debug)]
pub struct Penalty {
    pub cost: Duration,
    pub time_left: Duration,
    pub deadline: Instant,
}

#[derive(Debug, Default)]
pub struct Rules {
    pub config: Config,
    pub circles: CircleSequence,
    pub runs: HashMap<PlayerId, Run>,
}

impl Rules {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    /// Advance a run, or start one, when `player` crosses circle `circle`.
    pub fn checkpoint(
        &mut self,
        uname: &str,
        player: &PlayerInfo,
        circle: u8,
        now: Instant,
    ) -> Option<Outcome> {
        if let Some(run) = self.runs.get_mut(&player.plid) {
            if run.deadline < now || !self.circles.is_next(run.last_circle, circle) {
                return None;
            }
            run.last_circle = circle;
            run.deadline = now + run.window;
            run.window = run.window.saturating_sub(self.config.checkpoint_penalty);
            run.checkpoints += 1;
            return Some(Outcome::Extended {
                checkpoints: run.checkpoints,
                time_left: run.time_left(now),
            });
        }

        // Only humans can start runs.
        if player.ptype.contains(PlayerType::AI) {
            return None;
        }
        let _ = self.runs.insert(
            player.plid,
            Run {
                started_at: now,
                deadline: now + self.config.checkpoint_timeout,
                window: self.config.checkpoint_timeout,
                checkpoints: 0,
                uname: uname.to_owned(),
                pname: player.pname.clone(),
                last_circle: circle,
            },
        );
        Some(Outcome::Started)
    }

    /// A finish object resets the window to the full timeout.
    pub fn finish(&mut self, plid: PlayerId, now: Instant) -> Option<Outcome> {
        let run = self.runs.get_mut(&plid)?;
        if run.deadline < now {
            return None;
        }
        run.window = self.config.checkpoint_timeout;
        run.deadline = now + self.config.checkpoint_timeout;
        run.checkpoints += 1;
        Some(Outcome::Refreshed {
            checkpoints: run.checkpoints,
            window: run.window,
        })
    }

    /// Harder impacts cost more, up to the collision maximum.
    pub fn collision(&mut self, plid: PlayerId, closing_mps: f32, now: Instant) -> Option<Penalty> {
        let fraction = (closing_mps / COLLISION_THRESHOLD_MPS).clamp(0.0, 1.0);
        let cost = self.config.collision_max_penalty.mul_f32(fraction);
        if cost.is_zero() {
            return None;
        }
        self.penalise(plid, cost, now)
    }

    /// Resetting the car costs the checkpoint penalty.
    pub fn reset(&mut self, plid: PlayerId, now: Instant) -> Option<Penalty> {
        self.penalise(plid, self.config.checkpoint_penalty, now)
    }

    fn penalise(&mut self, plid: PlayerId, cost: Duration, now: Instant) -> Option<Penalty> {
        let run = self.runs.get_mut(&plid)?;
        run.deadline = run.deadline.checked_sub(cost).unwrap_or(now).max(now);
        Some(Penalty {
            cost,
            time_left: run.time_left(now),
            deadline: run.deadline,
        })
    }
}

#[cfg(test)]
mod tests {
    use insim::identifiers::ConnectionId;

    use super::*;

    fn player(plid: u8) -> PlayerInfo {
        PlayerInfo {
            plid: PlayerId(plid),
            ucid: ConnectionId(1),
            vehicle: Default::default(),
            ptype: PlayerType::default(),
            flags: Default::default(),
            in_pitlane: false,
            pname: "driver".into(),
        }
    }

    #[test]
    fn checkpoints_must_follow_the_circle_order_and_shrink_the_window() {
        let now = Instant::now();
        let mut rules = Rules::new(Config::default());
        rules.circles.accumulate([1, 2, 3].into_iter());
        rules.circles.finalize();

        assert!(matches!(
            rules.checkpoint("u", &player(1), 1, now),
            Some(Outcome::Started)
        ));
        // Out of order: ignored.
        assert!(rules.checkpoint("u", &player(1), 3, now).is_none());
        let next = now + Duration::from_secs(5);
        assert!(matches!(
            rules.checkpoint("u", &player(1), 2, next),
            Some(Outcome::Extended { checkpoints: 1, .. })
        ));
        let run = &rules.runs[&PlayerId(1)];
        assert_eq!(run.deadline, next + Duration::from_secs(30));
        assert_eq!(run.window, Duration::from_millis(29_750));
    }

    #[test]
    fn penalties_shorten_the_deadline_but_not_past_now() {
        let now = Instant::now();
        let mut rules = Rules::new(Config::default());
        let _ = rules.checkpoint("u", &player(1), 1, now);
        let hit = rules.collision(PlayerId(1), 15.0, now).unwrap();
        assert_eq!(hit.cost, Duration::from_millis(250));
        assert!(rules.collision(PlayerId(1), 0.0, now).is_none());
        let late = now + Duration::from_secs(40);
        assert_eq!(rules.reset(PlayerId(1), late).unwrap().deadline, late);
    }
}
