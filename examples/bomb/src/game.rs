//! The Bomb service: one update function over the game's state.
//!
//! Bomb doesn't set up the server or start races; the `RaceDirector` (or an
//! admin) does. Bomb follows the session and asks LFS for the layout when a
//! race starts.

use std::time::{Duration, Instant};

use insim::{
    Colour, Packet,
    core::object::{
        ObjectInfo,
        insim::{InsimCheckpoint, InsimCircle},
    },
    identifiers::{ConnectionId, PlayerId, RequestId},
    insim::{Axm, Con, PmoAction, PmoFlags, Tiny, TinyType, Uco, UcoAction},
};
use insim_extra::{
    chat::{ChatCommand, ChatParser},
    kitcar::{Context, Effects, Input, Service},
    world::{Connected, PlayerLeft, PlayerTeleportedToPits, WorldEvent},
};

use crate::{
    rules::{Config, Outcome, Penalty, Rules, Run},
    view::{ABOUT, DialogMsg, HELP},
};

/// How often countdowns and the marquee redraw.
const REPAINT: Duration = Duration::from_millis(200);
/// Leaderboard length.
const BOARD_SIZE: usize = 10;
/// Tags our layout request, so we know the reply.
const LAYOUT_REQUEST: RequestId = RequestId(2);

#[derive(Debug, Clone)]
pub enum Msg {
    /// A run's deadline passed.
    Boom(PlayerId),
    /// Redraw, so countdowns tick.
    Repaint,
    /// A player clicked the `?` button.
    OpenHelp(ConnectionId),
}

/// One timer per run: rescheduling with the same key moves the deadline.
#[derive(Debug, PartialEq)]
struct Fuse(PlayerId);

#[derive(Debug, PartialEq)]
struct Repaint;

#[derive(Debug, Clone, Copy, ChatCommand)]
enum Cmd {
    Help,
    About,
}

/// A leaderboard entry.
#[derive(Debug, Clone)]
pub struct Standing {
    pub uname: String,
    pub pname: String,
    pub checkpoints: u32,
    pub survival: Duration,
}

#[derive(Debug, Clone, Copy)]
enum End {
    Boom,
    Left,
    TeleportedToPits,
    Pitted,
    SessionOver,
    /// The game is stopping, such as for a rotation.
    Closing,
}

pub struct Bomb {
    pub(crate) rules: Rules,
    pub(crate) board: Vec<Standing>,
    pub(crate) started: Instant,
    chat: ChatParser<Cmd>,
}

impl Bomb {
    pub fn new(config: Config) -> Self {
        Self {
            rules: Rules::new(config),
            board: Vec::new(),
            started: Instant::now(),
            chat: ChatParser::new(&['!']),
        }
    }

    /// Forget the circles and ask LFS for the full layout, since the director
    /// or an admin may have loaded it. It arrives as `Axm` packets; the last
    /// has `FILE_END`.
    fn request_layout(&mut self) -> Effects<Msg> {
        self.rules.circles.clear();
        Effects::new().packet(Tiny {
            reqi: LAYOUT_REQUEST,
            subt: TinyType::Axm,
        })
    }

    fn race_started(&mut self) -> Effects<Msg> {
        let secs = self.rules.config.checkpoint_timeout.as_secs();
        self.request_layout().message(
            format!("Bomb - hit checkpoints before the {secs}s timer expires!"),
            ConnectionId::ALL,
        )
    }

    /// End every run in progress.
    fn end_all(&mut self, end: End, cx: &Context<'_>) -> Effects<Msg> {
        let runs: Vec<PlayerId> = self.rules.runs.keys().copied().collect();
        runs.into_iter().fold(Effects::new(), |fx, plid| {
            fx.and(self.end_run(plid, end, cx))
        })
    }

    /// Score a run, tell its driver, and disarm its timer.
    fn end_run(&mut self, plid: PlayerId, end: End, cx: &Context<'_>) -> Effects<Msg> {
        let Some(run) = self.rules.runs.remove(&plid) else {
            return Effects::none();
        };
        let survival = run.survival(cx.now());
        self.record(&run, survival);
        let cps = run.checkpoints;
        let text = match end {
            End::Boom => format!("BOOM - {cps} cps, {:.1}s", survival.as_secs_f64()).red(),
            End::Left => format!("Run ended - left race after {cps} cps").red(),
            End::TeleportedToPits => {
                format!("TELE-PITTED - run ended after {cps} cps. No shortcuts.").red()
            },
            End::Pitted => {
                format!("PITTED - run ended after {cps} cps. Commit to your fuel.").red()
            },
            End::SessionOver => format!("Run ended - race finished after {cps} cps").yellow(),
            End::Closing => format!("Run ended - Bomb is closing after {cps} cps").yellow(),
        };
        let effects = Effects::new().cancel(Fuse(plid));
        match cx.world().player(plid) {
            Some(player) => effects.message(text, player.ucid),
            None => effects,
        }
    }

    fn record(&mut self, run: &Run, survival: Duration) {
        self.board.push(Standing {
            uname: run.uname.clone(),
            pname: run.pname.clone(),
            checkpoints: run.checkpoints,
            survival,
        });
        self.board.sort_by(|a, b| {
            b.checkpoints
                .cmp(&a.checkpoints)
                .then(b.survival.cmp(&a.survival))
        });
        self.board.truncate(BOARD_SIZE);
    }

    fn uco(&mut self, uco: &Uco, cx: &Context<'_>) -> Effects<Msg> {
        let now = cx.now();
        let Some(player) = cx.world().player(uco.plid) else {
            return Effects::none();
        };
        let outcome = match (&uco.ucoaction, &uco.info) {
            (UcoAction::CircleEnter, ObjectInfo::InsimCircle(InsimCircle { index, .. })) => {
                let uname = cx
                    .world()
                    .connection(player.ucid)
                    .map(|c| c.uname.as_str())
                    .unwrap_or_default();
                self.rules.checkpoint(uname, player, *index, now)
            },
            (_, ObjectInfo::InsimCheckpoint(InsimCheckpoint { .. })) => {
                self.rules.finish(uco.plid, now)
            },
            _ => None,
        };
        let Some(outcome) = outcome else {
            return Effects::none();
        };
        let text = match outcome {
            Outcome::Started => "Run started - hit every checkpoint!".light_green(),
            Outcome::Refreshed {
                checkpoints,
                window,
            } => format!(
                "FINISH - cp {checkpoints} - REFRESHED {:.1}s",
                window.as_secs_f64()
            )
            .yellow(),
            Outcome::Extended {
                checkpoints,
                time_left,
            } => format!("cp {checkpoints} - {:.1}s left", time_left.as_secs_f64()).light_green(),
        };
        let deadline = self.rules.runs[&uco.plid].deadline;
        Effects::new().message(text, player.ucid).schedule(
            Fuse(uco.plid),
            deadline,
            Msg::Boom(uco.plid),
        )
    }

    fn penalised(
        &self,
        plid: PlayerId,
        penalty: Option<Penalty>,
        cx: &Context<'_>,
    ) -> Effects<Msg> {
        let Some(Penalty {
            cost,
            time_left,
            deadline,
        }) = penalty
        else {
            return Effects::none();
        };
        let effects = Effects::new().schedule(Fuse(plid), deadline, Msg::Boom(plid));
        match cx.world().player(plid) {
            Some(player) => effects.message(
                format!(
                    "PENALTY -{:.2}s - {:.1}s left",
                    cost.as_secs_f64(),
                    time_left.as_secs_f64()
                )
                .red(),
                player.ucid,
            ),
            None => effects,
        }
    }

    fn collision(&mut self, con: &Con, cx: &Context<'_>) -> Effects<Msg> {
        let closing = con.spclose.to_metres_per_sec();
        [con.a.plid, con.b.plid]
            .into_iter()
            .fold(Effects::new(), |effects, plid| {
                let penalty = self.rules.collision(plid, closing, cx.now());
                effects.and(self.penalised(plid, penalty, cx))
            })
    }

    fn layout(&mut self, axm: &Axm) {
        match &axm.action {
            PmoAction::ClearAll => self.rules.circles.clear(),
            PmoAction::LoadingFile(objects)
            | PmoAction::TinyAxm(objects)
            | PmoAction::AddObjects(objects) => {
                self.rules
                    .circles
                    .accumulate(objects.iter().filter_map(circle_index));
                if axm.flags.contains(PmoFlags::FILE_END) {
                    self.rules.circles.finalize();
                }
            },
            _ => {},
        }
    }
}

fn circle_index(object: &ObjectInfo) -> Option<u8> {
    match object {
        ObjectInfo::InsimCircle(InsimCircle { index, .. }) => Some(*index),
        _ => None,
    }
}

impl Service for Bomb {
    type Msg = Msg;

    fn update(&mut self, input: Input<Msg>, cx: &Context<'_>) -> Effects<Msg> {
        // Started by the `RaceDirector`, an admin, or before we connected.
        let racing = cx.world().session().is_some();

        match input {
            Input::Started => Effects::new().every(Repaint, REPAINT, Msg::Repaint),

            // A race starts, or ends (finish, /end, /restart, or everyone leaving).
            Input::World(WorldEvent::SessionStarted(_)) => self.race_started(),
            Input::World(WorldEvent::SessionEnded(_)) => self.end_all(End::SessionOver, cx),
            // Connected mid-race: learn the layout now.
            Input::World(WorldEvent::Ready(_)) if racing => self.request_layout(),

            // Greet newcomers, not those already here, with the About dialog.
            Input::World(WorldEvent::Connected(Connected {
                info,
                existing: false,
            })) => Effects::new().send(info.ucid, "about", DialogMsg::Show),

            Input::World(WorldEvent::PlayerLeft(PlayerLeft(player))) if racing => {
                self.end_run(player.plid, End::Left, cx)
            },
            Input::World(WorldEvent::PlayerTeleportedToPits(PlayerTeleportedToPits(player)))
                if racing =>
            {
                self.end_run(player.plid, End::TeleportedToPits, cx)
            },

            Input::Packet(packet) => match &packet {
                Packet::Axm(axm) => {
                    self.layout(axm);
                    Effects::none()
                },
                Packet::Uco(uco) if racing => self.uco(uco, cx),
                Packet::Con(con) if racing => self.collision(con, cx),
                Packet::Crs(crs) if racing => {
                    let penalty = self.rules.reset(crs.plid, cx.now());
                    self.penalised(crs.plid, penalty, cx)
                },
                Packet::Pit(pit) if racing => self.end_run(pit.plid, End::Pitted, cx),
                _ => match self.chat.parse(&packet) {
                    Some(event) => {
                        let key = match event.parsed {
                            Cmd::Help => HELP,
                            Cmd::About => ABOUT,
                        };
                        Effects::new().send(event.ucid, key, DialogMsg::Show)
                    },
                    None => Effects::none(),
                },
            },

            Input::Msg(Msg::Boom(plid)) => self.end_run(plid, End::Boom, cx),
            Input::Msg(Msg::OpenHelp(ucid)) => Effects::new().send(ucid, HELP, DialogMsg::Show),
            // Every update redraws, so the countdowns tick by themselves.
            Input::Msg(Msg::Repaint) => Effects::none(),
            // Score runs in progress. The host still sends these packets, then
            // clears our buttons and timers.
            Input::Stopping => self
                .end_all(End::Closing, cx)
                .message("Bomb is closing - thanks for playing!", ConnectionId::ALL),
            _ => Effects::none(),
        }
    }

    fn view(
        &self,
        ucid: ConnectionId,
        ui: &mut insim_extra::ui::Ui<'_, Msg>,
        cx: &Context<'_>,
    ) -> Option<insim_extra::ui::Node<Msg>> {
        Some(crate::view::view(self, ucid, ui, cx))
    }
}

#[cfg(test)]
mod tests {
    use insim::{
        identifiers::{ClickId, ConnectionId},
        insim::{Bfn, BfnType, Btc, Btn, Mso, MsoUserType, Ncn, Tiny},
        test::{Lfs, pair},
    };
    use insim_extra::kitcar::Host;

    use super::*;

    /// Button packets from the host, skipping its startup requests.
    async fn buttons_until(lfs: &mut Lfs, done: impl Fn(&[Packet]) -> bool) -> Vec<Packet> {
        let mut seen = Vec::new();
        while !done(&seen) {
            match lfs.recv().await.expect("host closed the connection") {
                Packet::Tiny(Tiny { reqi, .. }) if reqi.0 >= 128 => {},
                packet @ (Packet::Btn(_) | Packet::Bfn(_)) => seen.push(packet),
                _ => {},
            }
        }
        seen
    }
    fn button<'a>(packets: &'a [Packet], text: &str) -> Option<&'a Btn> {
        packets.iter().find_map(|p| match p {
            Packet::Btn(btn) if btn.text == text => Some(btn),
            _ => None,
        })
    }

    /// Apply a packet to the world, then feed it and its events to Bomb, as
    /// the host does.
    fn feed(world: &mut insim_extra::world::World, bomb: &mut Bomb, packet: Packet) -> Vec<Packet> {
        let events = world.apply_packet(&packet);
        let cx = Context::new(world, Instant::now());
        let inputs =
            std::iter::once(Input::Packet(packet)).chain(events.into_iter().map(Input::World));
        let mut sent = Vec::new();
        for input in inputs {
            sent.extend(bomb.update(input, &cx).packets_sent().cloned());
        }
        sent
    }

    fn circles(indices: &[u8], file_end: bool) -> Packet {
        Axm {
            flags: if file_end {
                PmoFlags::FILE_END
            } else {
                PmoFlags::empty()
            },
            action: PmoAction::TinyAxm(
                indices
                    .iter()
                    .map(|&index| {
                        ObjectInfo::InsimCircle(InsimCircle {
                            index,
                            ..Default::default()
                        })
                    })
                    .collect(),
            ),
            ..Default::default()
        }
        .into()
    }

    #[test]
    fn a_race_starting_asks_for_the_layout_and_ending_scores_runs() {
        use insim::insim::{Npl, RaceInProgress, Rst, Sta};
        use insim_extra::world::World;

        let track = "FE1X".parse().unwrap();
        let mut world = World::new();
        let mut bomb = Bomb::new(Config::default());
        // Circles left over from an earlier layout.
        bomb.rules.circles.accumulate([9].into_iter());
        let _ = feed(
            &mut world,
            &mut bomb,
            Ncn {
                ucid: ConnectionId(2),
                uname: "driver".into(),
                ..Default::default()
            }
            .into(),
        );
        let _ = feed(
            &mut world,
            &mut bomb,
            Npl {
                ucid: ConnectionId(2),
                plid: PlayerId(5),
                nump: 1,
                ..Default::default()
            }
            .into(),
        );

        // Someone else started the race: Bomb drops the old circles and asks.
        let sent = feed(
            &mut world,
            &mut bomb,
            Rst {
                track,
                ..Default::default()
            }
            .into(),
        );
        assert!(
            sent.iter().any(|p| matches!(
                p,
                Packet::Tiny(Tiny {
                    subt: TinyType::Axm,
                    ..
                })
            )),
            "{sent:?}"
        );
        assert!(bomb.rules.circles.indices().is_empty());

        // The reply spans two packets; the last has FILE_END.
        let _ = feed(&mut world, &mut bomb, circles(&[3, 1], false));
        let _ = feed(&mut world, &mut bomb, circles(&[2], true));
        assert_eq!(bomb.rules.circles.indices(), [1, 2, 3]);

        // A run starts, then an admin's /end scores it.
        let now = Instant::now();
        let player = world.player(PlayerId(5)).unwrap().clone();
        assert!(bomb.rules.checkpoint("driver", &player, 1, now).is_some());
        let sent = feed(
            &mut world,
            &mut bomb,
            Sta {
                track,
                raceinprog: RaceInProgress::No,
                ..Default::default()
            }
            .into(),
        );
        assert!(bomb.rules.runs.is_empty());
        assert_eq!(bomb.board.len(), 1);
        assert!(
            sent.iter()
                .any(|p| matches!(p, Packet::Mtc(m) if m.text.contains("race finished"))),
            "{sent:?}"
        );
    }

    #[test]
    fn stopping_scores_runs_in_progress_and_says_goodbye() {
        use insim::insim::Npl;
        use insim_extra::world::World;

        let mut world = World::new();
        let _ = world.apply_packet(
            &Ncn {
                ucid: ConnectionId(2),
                uname: "driver".into(),
                ..Default::default()
            }
            .into(),
        );
        let _ = world.apply_packet(
            &Npl {
                ucid: ConnectionId(2),
                plid: PlayerId(5),
                pname: "Driver".into(),
                nump: 1,
                ..Default::default()
            }
            .into(),
        );
        let now = Instant::now();
        let cx = Context::new(&world, now);
        let mut bomb = Bomb::new(Config::default());
        let player = world.player(PlayerId(5)).unwrap();
        assert!(bomb.rules.checkpoint("driver", player, 1, now).is_some());

        let effects = bomb.update(Input::Stopping, &cx);
        assert!(bomb.rules.runs.is_empty());
        assert_eq!(bomb.board.len(), 1);
        assert_eq!(bomb.board[0].uname, "driver");
        let texts: Vec<&str> = effects
            .packets_sent()
            .filter_map(|p| match p {
                Packet::Mtc(mtc) => Some(mtc.text.as_str()),
                _ => None,
            })
            .collect();
        assert!(
            texts
                .iter()
                .any(|t| t.contains("Bomb is closing after 0 cps")),
            "{texts:?}"
        );
        assert!(
            texts.iter().any(|t| t.contains("thanks for playing")),
            "{texts:?}"
        );
    }

    #[tokio::test]
    async fn a_player_sees_the_hud_and_can_open_and_close_help() {
        tokio::time::timeout(Duration::from_secs(10), async {
            let (session, mut lfs) = pair().await;
            let host = Host::spawn(session);
            let _bomb = host
                .start(Bomb::new(Config::default()))
                .await
                .unwrap();
            let player = ConnectionId(2);
            lfs.send(Ncn {
                ucid: player,
                uname: "driver".into(),
                pname: "Driver".into(),
                ..Default::default()
            })
            .await;

            let hud = buttons_until(&mut lfs, |p| {
                ["Bomb - waiting for a race", "?", "Cross any checkpoint to start"]
                    .iter()
                    .all(|text| button(p, text).is_some())
            })
            .await;
            assert!(button(&hud, "?").is_some());
            assert!(button(&hud, "Cross any checkpoint to start").is_some());

            lfs.send(Mso {
                ucid: player,
                usertype: MsoUserType::User,
                msg: "!help".into(),
                ..Default::default()
            })
            .await;
            let opened = buttons_until(&mut lfs, |p| button(p, "Close").is_some()).await;
            assert!(button(&opened, "Bomb - Help").is_some());
            let close: ClickId = button(&opened, "Close").unwrap().clickid;

            lfs.send(Btc {
                ucid: player,
                clickid: close,
                ..Default::default()
            })
            .await;
            let closed = buttons_until(&mut lfs, |p| {
                p.iter().any(|p| matches!(p, Packet::Bfn(Bfn { subt: BfnType::DelBtn, clickid, .. }) if *clickid == close))
            })
            .await;
            assert!(closed.iter().all(|p| !matches!(p, Packet::Btn(_))));
        })
        .await
        .unwrap();
    }
}
