//! The topbar: a strip pinned to the top of every player's screen, saying
//! where the race is and where they stand.
//!
//! Drawn straight from ZINC's records, the [`World`] and the host's
//! `RaceDirector`, so it never needs syncing. Countdowns ask the UI to redraw
//! each second while they run.

use insim::{
    identifiers::ConnectionId,
    insim::{BtnStyle, RaceFlags, RaceLaps},
};
use insim_extra::{
    kitcar::{Context, director::RaceDirectorStatus},
    ui::{self, Node, Ui},
    world::{
        World,
        game::SessionKind,
        race::{EntrantState, FinishStatus},
    },
};

use crate::{
    rank::Standing,
    zinc::{Driver, Ladder, Msg, Zinc},
};

/// What one player's topbar says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Topbar {
    /// Track and race progress.
    pub race: String,
    /// What the director does next. Empty without one.
    pub round: String,
    /// Points.
    pub points: String,
    /// Rank among everyone with a record.
    pub rank: String,
    /// The next licence rung, if any are left.
    pub next: String,
}

impl Topbar {
    /// Countdowns ask `ui` to redraw when their numbers change.
    pub fn of(zinc: &Zinc, ucid: ConnectionId, cx: &Context<'_>, ui: &mut Ui<'_, Msg>) -> Self {
        let world = cx.world();
        let uname = world
            .connection(ucid)
            .map(|conn| conn.uname.as_str())
            .unwrap_or_default();
        let standing = Standing::of(zinc, uname);
        Self {
            race: race(ucid, cx),
            round: cx
                .director()
                .map(|director| round(director, ui))
                .unwrap_or_default(),
            points: format!("^2{} pt{}", standing.points, plural(standing.points)),
            rank: match standing.position {
                Some(position) => format!("^7#{position} ^8of {}", standing.of),
                None => "^8Unranked".to_string(),
            },
            next: next_unlock(zinc.ladder(), &zinc.driver(uname)),
        }
    }

    pub fn render(self) -> Node<Msg> {
        let style = BtnStyle::default().dark();
        ui::row([ui::text(self.race, style).w(30).h(5)])
            .when(!self.round.is_empty(), |bar| {
                bar.with_child(ui::text(self.round, style).w(30).h(5))
            })
            .with_children([
                ui::text(self.points, style).w(12).h(5),
                ui::text(self.rank, style).w(14).h(5),
                ui::text(self.next, style).w(42).h(5),
                // Nothing here is clickable: a clickable button makes LFS show
                // the mouse cursor, which gets in the way of mouse steering.
                ui::text("^8!help", style).w(10).h(5),
            ])
            // Pinned to the window, whatever else the view draws.
            .position_fixed()
            .top(0)
            .left(0)
            .w(200)
            .justify_center()
    }
}

/// What the director does next. Countdowns ask `ui` to redraw as they tick.
fn round(director: RaceDirectorStatus<'_>, ui: &mut Ui<'_, Msg>) -> String {
    if director.loading {
        return "^3loading the next track".to_string();
    }
    let track = director.combo.map(|combo| combo.track);
    let upcoming = director.next.map(|combo| combo.track);
    // Only worth naming if it's a different track.
    let next = upcoming.filter(|next| Some(*next) != track);
    let last_race = director.races.is_some_and(|(done, of)| done + 1 >= of);
    let restart_in = director.restart_at.map(|at| ui.countdown(at));
    let rotates_in = director.rotates_at.map(|at| ui.countdown(at));
    match (restart_in, director.races, rotates_in) {
        // The same track again unless this was its last race, and the
        // rotation names a different one (or can't say).
        (Some(secs), ..) => match (last_race, upcoming, next) {
            (true, _, Some(next)) => format!("^3Track change: {next} in {secs}s"),
            (true, None, None) => format!("^3Track change in {secs}s"),
            _ => format!("^3Restart in {secs}s"),
        },
        (None, Some((done, of)), _) => {
            let race = format!("^7race {} of {of}", (done + 1).min(of));
            match next {
                Some(next) if last_race => format!("{race} ^8- then ^7{next}"),
                _ => race,
            }
        },
        (None, None, Some(secs)) => {
            let clock = format!("{}:{:02}", secs / 60, secs % 60);
            match next {
                Some(next) => format!("^7Track change: {next} in {clock}"),
                None => format!("^7Track change in {clock}"),
            }
        },
        (None, None, None) => String::new(),
    }
}

fn plural(n: u32) -> &'static str {
    if n == 1 { "" } else { "s" }
}

/// The next rung's cars. Each rung needs one more point.
fn next_unlock(ladder: Ladder, driver: &Driver) -> String {
    match driver.next_rung(ladder) {
        Some(rung) => {
            let cars: Vec<String> = rung.iter().map(ToString::to_string).collect();
            format!("^8Next point: ^3{}", cars.join(" "))
        },
        None => "^2Every car unlocked".to_string(),
    }
}

/// This player's entry in the current race: the one they're driving, or one
/// they finished before spectating.
fn entry(ucid: ConnectionId, world: &World) -> Option<&EntrantState> {
    world
        .players_of(ucid)
        .find_map(|player| world.entrant_by_plid(player.plid))
        .or_else(|| {
            world.entrants().find(|entrant| {
                matches!(entrant.status, FinishStatus::Finished { .. })
                    && entrant.drivers.last().is_some_and(|d| d.ucid == ucid)
            })
        })
}

fn race(ucid: ConnectionId, cx: &Context<'_>) -> String {
    let world = cx.world();
    let Some(track) = world.track() else {
        return "^8Waiting for the server".to_string();
    };
    let (laps, flags) = match world.session() {
        // Only the director knows whether the lobby is ready yet.
        None if cx.director().is_some_and(|director| !director.ready) => {
            return format!("^7{track} ^8- ^3setting up");
        },
        None => return format!("^7{track} ^8- ^7lobby, type ^3/ready"),
        Some(SessionKind::Race { laps, flags }) => (laps, flags),
        Some(_) => return format!("^7{track} ^8- ^7open session"),
    };
    let progress = match entry(ucid, world) {
        None => "^8spectating".to_string(),
        Some(entrant) => match &entrant.status {
            FinishStatus::Finished {
                result_num: Some(n),
                ..
            } => format!("^2finished P{}", u32::from(*n) + 1),
            FinishStatus::Finished { .. } => "^2finished".to_string(),
            FinishStatus::Dnf => "^1retired".to_string(),
            FinishStatus::Racing => {
                let lap = entrant.laps_done + 1;
                let lap = match laps {
                    RaceLaps::Laps(total) => {
                        format!("^7lap {}/{total}", usize::from(lap).min(total))
                    },
                    _ => format!("^7lap {lap}"),
                };
                // As LFS reports it, however it was set. Whether they've
                // stopped yet isn't shown.
                if flags.contains(RaceFlags::MUST_PIT) {
                    format!("{lap} ^8- ^3must pit")
                } else {
                    lap
                }
            },
        },
    };
    format!("^7{track} ^8- {progress}")
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use insim::{
        identifiers::PlayerId,
        insim::{Fin, Ncn, Npl, Res, Rst, Sta},
    };
    use insim_extra::kitcar::director::Combo;

    use super::*;

    const BOB: ConnectionId = ConnectionId(2);

    fn apply(world: &mut World, packet: impl Into<insim::Packet>) {
        let _ = world.apply(&packet.into());
    }

    /// Bob, connected, with the server on BL1.
    fn lobby() -> World {
        let mut world = World::new();
        apply(
            &mut world,
            Sta {
                track: "BL1".parse().unwrap(),
                ..Default::default()
            },
        );
        apply(
            &mut world,
            Ncn {
                ucid: BOB,
                uname: "bob".into(),
                ..Default::default()
            },
        );
        world
    }

    /// Bob on the grid of a 5-lap race.
    fn racing() -> World {
        let mut world = lobby();
        apply(
            &mut world,
            Rst {
                track: "BL1".parse().unwrap(),
                racelaps: RaceLaps::Laps(5),
                ..Default::default()
            },
        );
        apply(
            &mut world,
            Npl {
                ucid: BOB,
                plid: PlayerId(5),
                nump: 1,
                ..Default::default()
            },
        );
        world
    }

    /// Bob's topbar at `now`, reading `director` if given.
    fn topbar_at(
        zinc: &Zinc,
        world: &World,
        director: Option<RaceDirectorStatus<'_>>,
        now: Instant,
    ) -> Topbar {
        use insim_extra::ui::{Screen, id_pool::ClickIdPool};

        let cx = Context::new(world, now);
        let cx = match director {
            Some(director) => cx.with_director(director),
            None => cx,
        };
        let mut bar = None;
        let _ = Screen::<Msg>::new(BOB).render(&mut ClickIdPool::new(), now, |ui| {
            bar = Some(Topbar::of(zinc, BOB, &cx, ui));
            None
        });
        bar.expect("rendered")
    }

    /// Bob's topbar, with no director.
    fn topbar(zinc: &Zinc, world: &World) -> Topbar {
        topbar_at(zinc, world, None, Instant::now())
    }

    #[test]
    fn follows_the_race() {
        let zinc = Zinc::new(crate::LICENCES);
        let bar = |world: &World| topbar(&zinc, world).race;

        assert_eq!(bar(&lobby()), "^7BL1 ^8- ^7lobby, type ^3/ready");
        // While the director is still setting the lobby up, don't say /ready.
        let setting_up = topbar_at(
            &zinc,
            &lobby(),
            Some(RaceDirectorStatus::default()),
            Instant::now(),
        );
        assert_eq!(setting_up.race, "^7BL1 ^8- ^3setting up");

        let mut world = racing();
        assert_eq!(bar(&world), "^7BL1 ^8- ^7lap 1/5");

        apply(
            &mut world,
            Fin {
                plid: PlayerId(5),
                ..Default::default()
            },
        );
        assert_eq!(bar(&world), "^7BL1 ^8- ^2finished");

        apply(
            &mut world,
            Res {
                plid: PlayerId(5),
                uname: "bob".into(),
                resultnum: 0,
                ..Default::default()
            },
        );
        assert_eq!(bar(&world), "^7BL1 ^8- ^2finished P1");

        // No director, nothing to say about the round.
        assert_eq!(topbar(&zinc, &world).round, "");
    }

    #[test]
    fn says_when_a_pit_stop_is_required() {
        let zinc = Zinc::new(crate::LICENCES);
        let mut world = lobby();
        apply(
            &mut world,
            Rst {
                track: "BL1".parse().unwrap(),
                racelaps: RaceLaps::Laps(3),
                flags: RaceFlags::MUST_PIT,
                ..Default::default()
            },
        );
        apply(
            &mut world,
            Npl {
                ucid: BOB,
                plid: PlayerId(5),
                nump: 1,
                ..Default::default()
            },
        );
        assert_eq!(
            topbar(&zinc, &world).race,
            "^7BL1 ^8- ^7lap 1/3 ^8- ^3must pit"
        );

        // LFS sends `Sta` all through the race, without the race flags.
        apply(
            &mut world,
            Sta {
                track: "BL1".parse().unwrap(),
                racelaps: RaceLaps::Laps(3),
                raceinprog: insim::insim::RaceInProgress::Racing,
                ..Default::default()
            },
        );
        assert_eq!(
            topbar(&zinc, &world).race,
            "^7BL1 ^8- ^7lap 1/3 ^8- ^3must pit"
        );

        // Once finished, it's no longer worth saying.
        apply(
            &mut world,
            Fin {
                plid: PlayerId(5),
                ..Default::default()
            },
        );
        assert_eq!(topbar(&zinc, &world).race, "^7BL1 ^8- ^2finished");
    }

    #[test]
    fn shows_points_rank_and_the_next_car() {
        let mut zinc = Zinc::new(crate::LICENCES);
        let world = racing();

        let fresh = topbar(&zinc, &world);
        assert_eq!(fresh.points, "^20 pts");
        assert_eq!(fresh.rank, "^8Unranked");
        assert_eq!(fresh.next, "^8Next point: ^31DFF72");

        zinc.set("bob", 1);
        let one = topbar(&zinc, &world);
        assert_eq!(one.points, "^21 pt");
        assert_eq!(one.rank, "^7#1 ^8of 1");
        assert_eq!(one.next, "^8Next point: ^3XFR");

        zinc.set("bob", 5);
        assert_eq!(topbar(&zinc, &world).next, "^2Every car unlocked");
    }

    #[test]
    fn says_what_the_director_does_next() {
        let zinc = Zinc::new(crate::LICENCES);
        let world = racing();
        let now = Instant::now();
        let round = |director| topbar_at(&zinc, &world, Some(director), now).round;
        let fe1 = Combo::untimed("FE1".parse().unwrap());
        let as1 = Combo::untimed("AS1".parse().unwrap());
        let secs = |n| Some(now + Duration::from_secs(n));

        let racing = RaceDirectorStatus {
            combo: Some(&fe1),
            next: Some(&as1),
            races: Some((0, 2)),
            ..Default::default()
        };
        assert_eq!(round(racing), "^7race 1 of 2");

        let last = RaceDirectorStatus {
            races: Some((1, 2)),
            ..racing
        };
        assert_eq!(round(last), "^7race 2 of 2 ^8- then ^7AS1");

        // Counting down: a restart, or the next track on the last race.
        let restarting = RaceDirectorStatus {
            restart_at: secs(8),
            ..racing
        };
        assert_eq!(round(restarting), "^3Restart in 8s");
        let rotating = RaceDirectorStatus {
            restart_at: secs(8),
            ..last
        };
        assert_eq!(round(rotating), "^3Track change: AS1 in 8s");

        // The last race, but the rotation can't say what's next.
        let unknown = RaceDirectorStatus {
            next: None,
            ..rotating
        };
        assert_eq!(round(unknown), "^3Track change in 8s");

        // One combo forever: its last race restarts on the same track.
        let same = RaceDirectorStatus {
            next: Some(&fe1),
            ..rotating
        };
        assert_eq!(round(same), "^3Restart in 8s");

        let timed = RaceDirectorStatus {
            combo: Some(&fe1),
            next: Some(&as1),
            rotates_at: secs(272),
            ..Default::default()
        };
        assert_eq!(round(timed), "^7Track change: AS1 in 4:32");

        // One combo forever: nothing to rotate to.
        let forever = RaceDirectorStatus {
            combo: Some(&fe1),
            next: Some(&fe1),
            ..Default::default()
        };
        assert_eq!(round(forever), "");

        let loading = RaceDirectorStatus {
            loading: true,
            ..racing
        };
        assert_eq!(round(loading), "^3loading the next track");
    }
}
