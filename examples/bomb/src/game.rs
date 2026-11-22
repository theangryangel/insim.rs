//! The Bomb service: one update function over the game's state.

use std::time::{Duration, Instant};

use insim::{
    Colour, Packet,
    core::object::{
        ObjectInfo,
        insim::{InsimCheckpoint, InsimCircle},
    },
    identifiers::{ConnectionId, PlayerId},
    insim::{Axm, Con, PmoAction, PmoFlags, Uco, UcoAction},
};
use kitcar::{
    ChatParser, ClearPenalty, Context, Effects, Input, Layout, Lifecycle, PenaltyClearer, Phase,
    Service, Setup, Transition, WorldEvent,
    world::{Connected, PlayerLeft, PlayerTeleportedToPits},
};

use crate::{
    rules::{Config, Outcome, Penalty, Rules, Run},
    view::{ABOUT, DialogMsg, HELP},
};

/// How often the countdowns and marquee are redrawn.
const REPAINT: Duration = Duration::from_millis(200);
/// How long after a penalty it is cleared.
const PENALTY_CLEAR_DELAY: Duration = Duration::from_secs(15);
/// How many results the leaderboard keeps.
const BOARD_SIZE: usize = 10;

#[derive(Debug, Clone)]
pub enum Msg {
    /// A run's deadline passed.
    Boom(PlayerId),
    /// Redraw, so countdowns tick.
    Repaint,
    /// A player clicked the `?` button.
    OpenHelp(ConnectionId),
    Penalty(ClearPenalty),
}

/// One timer per run: rescheduling with the same key moves the deadline.
#[derive(Debug, PartialEq)]
struct Fuse(PlayerId);

#[derive(Debug, PartialEq)]
struct Repaint;

#[derive(Debug, Clone, Copy)]
enum Cmd {
    Help,
    About,
}

impl std::str::FromStr for Cmd {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_whitespace().next() {
            Some("help") => Ok(Self::Help),
            Some("about") => Ok(Self::About),
            _ => Err(()),
        }
    }
}

/// A finished run, as shown on the leaderboard.
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
    SessionOver { shortfall: bool },
}

pub struct Bomb {
    pub(crate) lifecycle: Lifecycle,
    pub(crate) rules: Rules,
    pub(crate) board: Vec<Standing>,
    pub(crate) started: Instant,
    chat: ChatParser<Cmd>,
    clearer: PenaltyClearer,
}

impl Bomb {
    pub fn new(config: Config, setup: Setup) -> Self {
        let mut rules = Rules::new(config);
        // Objects we send ourselves are not echoed back, so learn their order now.
        if let Some(Layout::Objects(objects)) = &setup.layout {
            rules
                .circles
                .accumulate(objects.iter().filter_map(circle_index));
            rules.circles.finalize();
        }
        Self {
            lifecycle: Lifecycle::new(setup),
            rules,
            board: Vec::new(),
            started: Instant::now(),
            chat: ChatParser::new(&['!']),
            clearer: PenaltyClearer::new(PENALTY_CLEAR_DELAY),
        }
    }

    pub(crate) fn phase(&self) -> Phase {
        self.lifecycle.phase()
    }

    fn transition(&mut self, transition: Transition, cx: &Context<'_>) -> Effects<Msg> {
        match transition {
            Transition::SetupStarted => Effects::none(),
            Transition::Racing => {
                let secs = self.rules.config.checkpoint_timeout.as_secs();
                Effects::new().message(
                    format!("Bomb - hit checkpoints before the {secs}s timer expires!"),
                    ConnectionId::ALL,
                )
            },
            Transition::Ended { shortfall } => {
                let runs: Vec<PlayerId> = self.rules.runs.keys().copied().collect();
                let mut effects = runs.into_iter().fold(Effects::new(), |fx, plid| {
                    fx.and(self.end_run(plid, End::SessionOver { shortfall }, cx))
                });
                if shortfall {
                    effects = effects
                        .message("Bomb - not enough players, restarting.", ConnectionId::ALL);
                }
                effects
            },
        }
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
            End::SessionOver { shortfall: true } => {
                format!("Run ended - not enough players after {cps} cps").yellow()
            },
            End::SessionOver { shortfall: false } => {
                format!("Run ended - race finished after {cps} cps").yellow()
            },
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
        // Components first: they see every input.
        let mut effects = self.clearer.update(&input, cx).map(Msg::Penalty);
        let (setup, transition) = self.lifecycle.update(&input, cx);
        effects = effects.and(setup);
        if let Some(transition) = transition {
            effects = effects.and(self.transition(transition, cx));
        }
        let racing = self.lifecycle.racing();

        effects.and(match input {
            Input::Started => Effects::new().every(Repaint, REPAINT, Msg::Repaint),

            // Greet people who join after we connected with the About dialog.
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
            Input::Msg(Msg::Penalty(clear)) => self.clearer.clear(clear, cx),
            // Nothing to do: any update redraws, which moves the countdowns on.
            Input::Msg(Msg::Repaint) => Effects::none(),
            _ => Effects::none(),
        })
    }

    fn view(
        &self,
        ucid: ConnectionId,
        ui: &mut kitcar::ui::Ui<'_, Msg>,
        cx: &Context<'_>,
    ) -> Option<kitcar::ui::Node<Msg>> {
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
    use kitcar::Host;

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

    #[tokio::test]
    async fn a_player_sees_the_hud_and_can_open_and_close_help() {
        tokio::time::timeout(Duration::from_secs(10), async {
            let (session, mut lfs) = pair().await;
            let host = Host::spawn(session);
            let _bomb = host
                .start(Bomb::new(Config::default(), Setup::untimed(Default::default())))
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
                ["Bomb - waiting for players", "?", "Cross any checkpoint to start"]
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
