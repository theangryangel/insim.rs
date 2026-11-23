//! ZINC's rules: points for clean, confirmed results, yellow flags and
//! collisions counted while racing,
//! and cars unlocked by points. All in memory, so it resets on restart.
//!
//! Cars are checked in two layers. The host's `RaceDirector` restricts every
//! combo to its track's cars (rallycross cars on BL3, say), which greys the
//! rest out in the garage. ZINC answers LFS's join requests, letting a driver
//! out only in a car they've unlocked. Neither touches the other's job.
//!
//! Built as a `Router`: one handler per input.

use std::collections::{HashMap, HashSet};

use insim::{
    core::vehicle::Vehicle,
    identifiers::{ConnectionId, PlayerId, RequestId},
    insim::{
        Con, Flg, FlgType, Jrr, JrrAction, JrrStartPosition, Npl, PlayerType, RaceConfirmFlags, Res,
    },
};
use insim_extra::{
    chat::ChatCommand,
    kitcar::{
        Context, Effects,
        director::DirectorCommand,
        services::{
            motd::MotdCommand,
            router::{Caller, Router},
        },
    },
    ui::{self, Child, Node, Ui},
    world::{
        World,
        event::{Connected, Disconnected},
        race::FinishStatus,
    },
};

use crate::{rank::RankCard, topbar::Topbar};

/// A licence ladder. Rung `n` needs `n` points, and rungs stack. The ladder
/// itself is configuration, defined in `main.rs` as `LICENCES`.
pub type Ladder = &'static [&'static [Vehicle]];

/// Any of these disqualifies a result, so it earns nothing. LFS calls this set
/// `CONF_DISQ`.
const DISQUALIFIED: RaceConfirmFlags = RaceConfirmFlags::PENALTY_DT
    .union(RaceConfirmFlags::PENALTY_SG)
    .union(RaceConfirmFlags::DID_NOT_PIT);

/// One driver's record, keyed by LFS username.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Driver {
    /// One per confirmed, clean result.
    pub points: u32,
    /// Yellow flags caused while racing.
    pub yellows: u32,
    /// Contact with another car while racing, whoever's fault.
    pub collisions: u32,
}

impl Driver {
    /// The cars this driver has unlocked on `ladder`.
    pub fn cars(&self, ladder: Ladder) -> impl Iterator<Item = Vehicle> {
        ladder
            .iter()
            .take(self.points as usize + 1)
            .flat_map(|rung| rung.iter().copied())
    }

    /// The rung their next point unlocks, if any are left.
    pub fn next_rung(&self, ladder: Ladder) -> Option<&'static [Vehicle]> {
        ladder.get(self.points as usize + 1).copied()
    }

    pub(crate) fn car_list(&self, ladder: Ladder) -> String {
        self.cars(ladder)
            .map(|car| car.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Debug, ChatCommand)]
pub enum Cmd {
    /// Your points, yellow flags and cars.
    Stats,
    /// The drivers with the most points.
    Top,
    /// Your rank, in a panel. Again to close.
    Rank,
    /// List the commands.
    Help,
    /// Move on to the next track now.
    #[chat(admin)]
    Rotate,
}

/// Every driver's record, and the ladder they climb.
#[derive(Debug)]
pub struct Zinc {
    ladder: Ladder,
    drivers: HashMap<String, Driver>,
    /// Who has their rank card open.
    rank_open: HashSet<ConnectionId>,
}

#[derive(Debug, Clone)]
pub enum Msg {
    /// A player closed their rank card.
    CloseRank(ConnectionId),
}

impl Zinc {
    pub fn new(ladder: Ladder) -> Self {
        Self {
            ladder,
            drivers: HashMap::new(),
            rank_open: HashSet::new(),
        }
    }

    /// The licence ladder.
    pub fn ladder(&self) -> Ladder {
        self.ladder
    }

    /// A driver's record, or a fresh one.
    pub fn driver(&self, uname: &str) -> Driver {
        self.drivers.get(uname).copied().unwrap_or_default()
    }

    /// Every driver, most points first.
    pub(crate) fn ranked(&self) -> Vec<(&str, Driver)> {
        let mut drivers: Vec<_> = self
            .drivers
            .iter()
            .map(|(uname, driver)| (uname.as_str(), *driver))
            .collect();
        drivers.sort_by(|a, b| b.1.points.cmp(&a.1.points).then(a.0.cmp(b.0)));
        drivers
    }

    /// Set a driver's points, for tests.
    #[cfg(test)]
    pub(crate) fn set(&mut self, uname: &str, points: u32) {
        self.drivers.entry(uname.to_owned()).or_default().points = points;
    }

    /// The username of a human still racing as `plid`: in a race, and not
    /// yet finished. Yellows and collisions only count against them.
    fn racer(world: &World, plid: PlayerId) -> Option<&str> {
        let racing = world.session().is_some_and(|session| session.is_race())
            && world
                .entrant_by_plid(plid)
                .is_some_and(|entrant| matches!(entrant.status, FinishStatus::Racing));
        if !racing {
            return None;
        }
        Self::human(world, plid).map(|(_, uname)| uname)
    }

    /// The connection and username behind a human player.
    fn human(world: &World, plid: PlayerId) -> Option<(ConnectionId, &str)> {
        let player = world.player(plid)?;
        if player.ptype.contains(PlayerType::AI) {
            return None;
        }
        let conn = world.connection(player.ucid)?;
        Some((conn.ucid, conn.uname.as_str()))
    }
}

/// The ZINC service: a `Router` over the drivers' records.
///
/// A router fits because each rule reacts to one input on its own. A simple
/// guard (yellows only counting while racing) reads fine in a handler; once
/// rules depend on each other, a hand-written `Service::update` is clearer.
pub fn service(ladder: Ladder) -> Router<Zinc, Msg> {
    Router::<_, Msg>::with_messages(Zinc::new(ladder))
        .on(joined)
        .on(left)
        .on(join_request)
        .on(result)
        .on(flagged)
        .on(collided)
        .on_msg(message)
        .commands(command)
        .view(view)
}

/// Tell each driver where they stand as they join.
fn joined(zinc: &mut Zinc, event: &Connected, _: &Context<'_>) -> Effects<Msg> {
    let driver = zinc.driver(&event.info.uname);
    tracing::info!(
        uname = %event.info.uname,
        points = driver.points,
        cars = %driver.car_list(zinc.ladder),
        "driver connected"
    );
    if event.existing {
        return Effects::none();
    }
    Effects::new().message(
        format!(
            "^7ZINC: ^2{} point(s)^7, cars: ^3{}^7 - type !help",
            driver.points,
            driver.car_list(zinc.ladder)
        ),
        event.info.ucid,
    )
}

fn left(zinc: &mut Zinc, event: &Disconnected, _: &Context<'_>) {
    let _ = zinc.rank_open.remove(&event.ucid);
}

fn message(zinc: &mut Zinc, msg: Msg, _: &Context<'_>) {
    match msg {
        Msg::CloseRank(ucid) => {
            let _ = zinc.rank_open.remove(&ucid);
        },
    }
}

/// ZINC's commands, and the welcome message's.
fn help() -> String {
    Cmd::COMMANDS
        .iter()
        .chain(MotdCommand::COMMANDS)
        .map(|c| {
            let admin = if c.admin() { " ^1(admin)" } else { "" };
            format!("^7!{c} ^8- {}{admin}", c.help())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The topbar, always, and the rank card while open. The card is placed only
/// while open, so each opening mounts it afresh with a new snapshot.
fn view(
    zinc: &Zinc,
    ucid: ConnectionId,
    ui: &mut Ui<'_, Msg>,
    cx: &Context<'_>,
) -> Option<Node<Msg>> {
    let topbar = Topbar::of(zinc, ucid, cx, ui).render();
    let uname = cx.world().connection(ucid)?.uname.as_str();
    let rank_open = zinc.rank_open.contains(&ucid);
    Some(ui::container().with_child(topbar).when(rank_open, |root| {
        root.with_child(ui.child(
            Child::with_props((zinc, uname), RankCard::default).on(move |()| Msg::CloseRank(ucid)),
        ))
    }))
}

/// LFS asks before anyone takes the track (a join request: an `Npl` with no
/// players yet), and waits for the answer. Let them out only in a car they've
/// unlocked. The track's own cars are the director's job.
///
/// AI, and anyone the world doesn't know, are let through: blocking a join
/// for want of a record would leave it hanging.
fn join_request(zinc: &mut Zinc, npl: &Npl, cx: &Context<'_>) -> Effects<Msg> {
    if npl.nump != 0 {
        return Effects::none();
    }
    let reply = |jrraction| Jrr {
        reqi: RequestId(0),
        plid: PlayerId(0),
        ucid: npl.ucid,
        jrraction,
        startpos: JrrStartPosition::DefaultStartPosition,
    };
    let world = cx.world();
    let refuse = |why: String| {
        let uname = world
            .connection(npl.ucid)
            .map(|conn| conn.uname.as_str())
            .unwrap_or_default();
        tracing::info!(uname, car = %npl.cname, %why, "join refused");
        Effects::new()
            .packet(reply(JrrAction::Reject))
            .message(why, npl.ucid)
    };

    // The track's own cars. The director already greys the rest out in the
    // garage, mods included, so this is a backstop should that not have
    // reached LFS. It applies to everyone, AI too.
    if let Some(combo) = cx
        .director()
        .and_then(|director| director.combo)
        .filter(|combo| !combo.vehicles.is_empty())
        && !combo.vehicles.contains(&npl.cname)
    {
        let cars: Vec<String> = combo.vehicles.iter().map(ToString::to_string).collect();
        return refuse(format!(
            "^1{} only allows ^3{}",
            combo.track,
            cars.join(" ")
        ));
    }

    // The licence. AI, and anyone the world doesn't know, have none to check.
    let Some(conn) = world
        .connection(npl.ucid)
        .filter(|_| !npl.ptype.contains(PlayerType::AI))
    else {
        return Effects::new().packet(reply(JrrAction::Spawn));
    };
    let driver = zinc.driver(&conn.uname);
    if !driver.cars(zinc.ladder).any(|car| car == npl.cname) {
        return refuse(format!(
            "^1You haven't unlocked the {} yet ^8- your cars: ^3{}",
            npl.cname,
            driver.car_list(zinc.ladder)
        ));
    }
    Effects::new().packet(reply(JrrAction::Spawn))
}

/// A confirmed, clean result earns a point, which may unlock a car.
///
/// LFS sends `Res` unprompted (`reqi` 0) once a result is confirmed. Replies
/// to `TINY_RES` repeat old results, so only the unprompted one counts. `Res`
/// carries the username, so the point counts even if the driver has left.
fn result(zinc: &mut Zinc, res: &Res, cx: &Context<'_>) -> Effects<Msg> {
    let counts = res.reqi.0 == 0
        && res.confirm.contains(RaceConfirmFlags::CONFIRMED)
        && !res.confirm.intersects(DISQUALIFIED)
        && res.resultnum != 255
        && !res.uname.is_empty();
    let ai = cx
        .world()
        .player(res.plid)
        .is_some_and(|player| player.ptype.contains(PlayerType::AI));
    if !counts || ai {
        return Effects::none();
    }
    let before = zinc.driver(&res.uname);
    let driver = zinc.drivers.entry(res.uname.clone()).or_default();
    driver.points += 1;
    let after = *driver;
    tracing::info!(uname = %res.uname, points = after.points, "point awarded");

    // Tell them and update their cars, if they're still here.
    let Some(conn) = cx
        .world()
        .connections()
        .find(|conn| conn.uname == res.uname)
    else {
        return Effects::none();
    };
    let ucid = conn.ucid;
    let effects =
        Effects::new().message(format!("^2+1 point^7 - you have ^2{}", after.points), ucid);
    let unlocked: Vec<String> = after
        .cars(zinc.ladder)
        .filter(|car| !before.cars(zinc.ladder).any(|c| c == *car))
        .map(|car| car.to_string())
        .collect();
    if unlocked.is_empty() {
        return effects;
    }
    tracing::info!(uname = %conn.uname, cars = %unlocked.join(" "), "licence upgraded");
    // Announced to everyone, silently.
    let name = if conn.pname.is_empty() {
        &conn.uname
    } else {
        &conn.pname
    };
    effects.message(
        format!(
            "^3Licence up! ^7{name} ^7can now drive ^3{}",
            unlocked.join(" ")
        ),
        ConnectionId::ALL,
    )
}

/// Count each yellow flag a driver causes while racing.
fn flagged(zinc: &mut Zinc, flg: &Flg, cx: &Context<'_>) {
    if !(flg.offon && flg.flag == FlgType::Yellow) {
        return;
    }
    if let Some(uname) = Zinc::racer(cx.world(), flg.plid) {
        zinc.drivers.entry(uname.to_owned()).or_default().yellows += 1;
    }
}

/// Count contact between two cars against both drivers, while each is still
/// racing. LFS doesn't say whose fault it was.
fn collided(zinc: &mut Zinc, con: &Con, cx: &Context<'_>) {
    for plid in [con.a.plid, con.b.plid] {
        if let Some(uname) = Zinc::racer(cx.world(), plid) {
            zinc.drivers.entry(uname.to_owned()).or_default().collisions += 1;
        }
    }
}

fn command(zinc: &mut Zinc, caller: Caller, cmd: Cmd, _: &Context<'_>) -> Effects<Msg> {
    match cmd {
        Cmd::Stats => {
            let driver = zinc.driver(&caller.uname);
            caller.reply(format!(
                "^7{}: ^2{} point(s)^7, ^3{} yellow(s)^7, ^3{} collision(s)^7, cars: ^3{}",
                caller.uname,
                driver.points,
                driver.yellows,
                driver.collisions,
                driver.car_list(zinc.ladder)
            ))
        },
        Cmd::Rank => {
            if !zinc.rank_open.remove(&caller.ucid) {
                let _ = zinc.rank_open.insert(caller.ucid);
            }
            Effects::none()
        },
        Cmd::Top => {
            let lines: Vec<String> = zinc
                .ranked()
                .into_iter()
                .take(5)
                .enumerate()
                .map(|(i, (uname, d))| format!("^7{}. {uname} - ^2{}", i + 1, d.points))
                .collect();
            if lines.is_empty() {
                caller.reply("^7No points yet - finish a race cleanly!")
            } else {
                caller.reply(lines.join("\n"))
            }
        },
        Cmd::Help => caller.reply(help()),
        // The router refuses non-admins before it gets here.
        Cmd::Rotate => {
            tracing::info!(uname = %caller.uname, "admin asked to rotate");
            caller
                .reply("^3Moving on to the next track")
                .director(DirectorCommand::Rotate)
        },
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use insim::{
        Packet,
        insim::{Fin, Ncn, Npl},
    };
    use insim_extra::kitcar::{Input, Service};

    use super::*;

    /// A world with one human driver, `bob`, on track as player 5.
    fn world() -> World {
        let mut world = World::new();
        let _ = world.apply(
            &Ncn {
                ucid: ConnectionId(2),
                uname: "bob".into(),
                ..Default::default()
            }
            .into(),
        );
        let _ = world.apply(
            &Npl {
                ucid: ConnectionId(2),
                plid: PlayerId(5),
                nump: 1,
                ..Default::default()
            }
            .into(),
        );
        world
    }

    /// A confirmed result for `bob`, as LFS sends it unprompted.
    fn res(confirm: RaceConfirmFlags) -> Res {
        Res {
            plid: PlayerId(5),
            uname: "bob".into(),
            confirm: RaceConfirmFlags::CONFIRMED | confirm,
            ..Default::default()
        }
    }

    fn input(res: Res) -> Input<Msg> {
        Input::Packet(res.into())
    }

    /// ZINC never restricts cars itself: that's the director's job.
    fn restricts(effects: &Effects<Msg>) -> bool {
        effects
            .packets_sent()
            .any(|p| matches!(p, Packet::Plc(_) | Packet::Mal(_)))
    }

    /// The answer to a join request, if any.
    fn answer(effects: &Effects<Msg>) -> Option<JrrAction> {
        effects.packets_sent().find_map(|p| match p {
            Packet::Jrr(jrr) => Some(jrr.jrraction.clone()),
            _ => None,
        })
    }

    /// `ucid` asking to join the track in `car`.
    fn join_request(ucid: u8, car: Vehicle) -> Input<Msg> {
        Input::Packet(
            Npl {
                ucid: ConnectionId(ucid),
                nump: 0,
                cname: car,
                ..Default::default()
            }
            .into(),
        )
    }

    fn chat(effects: &Effects<Msg>) -> Vec<String> {
        effects
            .packets_sent()
            .filter_map(|p| match p {
                Packet::Mtc(mtc) => Some(mtc.text.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn each_point_unlocks_the_next_rung() {
        let world = world();
        let cx = Context::new(&world, Instant::now());
        let mut zinc = service(crate::LICENCES);

        for (points, rung) in (1..).zip(&crate::LICENCES[1..]) {
            let effects = zinc.update(input(res(RaceConfirmFlags::empty())), &cx);
            let cars: Vec<Vehicle> = zinc.state().driver("bob").cars(crate::LICENCES).collect();
            assert!(
                rung.iter().all(|car| cars.contains(car)),
                "{points}: {cars:?}"
            );
            // Announced to everyone, silently.
            let names: Vec<String> = rung.iter().map(ToString::to_string).collect();
            let announced = effects.packets_sent().any(|p| {
                matches!(p, Packet::Mtc(mtc)
                    if mtc.ucid == ConnectionId::ALL
                        && mtc.sound == insim::insim::SoundType::Silent
                        && mtc.text.contains(&format!("can now drive ^3{}", names.join(" "))))
            });
            assert!(announced, "{:?}", chat(&effects));
        }

        // Past the top of the ladder, points still count but unlock nothing.
        let effects = zinc.update(input(res(RaceConfirmFlags::empty())), &cx);
        assert_eq!(
            zinc.state().driver("bob").points,
            u32::try_from(crate::LICENCES.len()).unwrap()
        );
        assert!(chat(&effects).iter().all(|c| !c.contains("Licence up")));
    }

    #[test]
    fn only_new_clean_confirmed_results_count() {
        let world = world();
        let cx = Context::new(&world, Instant::now());
        let mut zinc = service(crate::LICENCES);

        let unconfirmed = Res {
            confirm: RaceConfirmFlags::MENTIONED,
            ..res(RaceConfirmFlags::empty())
        };
        // A reply to TINY_RES repeats a result we already counted.
        let requested = Res {
            reqi: insim::identifiers::RequestId(3),
            ..res(RaceConfirmFlags::empty())
        };
        let unclassified = Res {
            resultnum: 255,
            ..res(RaceConfirmFlags::empty())
        };
        let ignored = [
            unconfirmed,
            requested,
            unclassified,
            res(RaceConfirmFlags::PENALTY_DT),
            res(RaceConfirmFlags::PENALTY_SG),
            res(RaceConfirmFlags::DID_NOT_PIT),
        ];
        for res in ignored {
            assert!(zinc.update(input(res), &cx).is_empty());
        }
        assert_eq!(zinc.state().driver("bob").points, 0);
    }

    #[test]
    fn a_driver_who_has_left_still_gets_the_point() {
        let world = World::new();
        let cx = Context::new(&world, Instant::now());
        let mut zinc = service(crate::LICENCES);

        let effects = zinc.update(input(res(RaceConfirmFlags::empty())), &cx);
        assert!(effects.is_empty(), "nobody to tell");
        assert_eq!(zinc.state().driver("bob").points, 1);
    }

    #[test]
    fn yellows_and_collisions_only_count_while_racing() {
        use insim::insim::{Con, ConInfo, RaceLaps, Rst};

        let yellow = || {
            Input::Packet(
                Flg {
                    plid: PlayerId(5),
                    offon: true,
                    flag: FlgType::Yellow,
                    ..Default::default()
                }
                .into(),
            )
        };
        // Bob (player 5) touches player 6, an AI.
        let contact = || {
            Input::Packet(
                Con {
                    a: ConInfo {
                        plid: PlayerId(5),
                        ..Default::default()
                    },
                    b: ConInfo {
                        plid: PlayerId(6),
                        ..Default::default()
                    },
                    ..Default::default()
                }
                .into(),
            )
        };
        let mut zinc = service(crate::LICENCES);
        let tally = |zinc: &Router<Zinc, Msg>| {
            let bob = zinc.state().driver("bob");
            (bob.yellows, bob.collisions)
        };

        // Not in a race: nothing counts.
        let lobby = world();
        let cx = Context::new(&lobby, Instant::now());
        let _ = zinc.update(yellow(), &cx);
        let _ = zinc.update(contact(), &cx);
        assert_eq!(tally(&zinc), (0, 0));

        // Racing: both count, and a lowered or blue flag doesn't.
        let mut racing = World::new();
        for packet in [
            insim::Packet::from(Ncn {
                ucid: ConnectionId(2),
                uname: "bob".into(),
                ..Default::default()
            }),
            Rst {
                racelaps: RaceLaps::Laps(3),
                ..Default::default()
            }
            .into(),
            Npl {
                ucid: ConnectionId(2),
                plid: PlayerId(5),
                nump: 1,
                ..Default::default()
            }
            .into(),
        ] {
            let _ = racing.apply(&packet);
        }
        let cx = Context::new(&racing, Instant::now());
        let _ = zinc.update(yellow(), &cx);
        let _ = zinc.update(contact(), &cx);
        let lowered = Flg {
            plid: PlayerId(5),
            offon: false,
            flag: FlgType::Yellow,
            ..Default::default()
        };
        let _ = zinc.update(Input::Packet(lowered.into()), &cx);
        assert_eq!(tally(&zinc), (1, 1));

        // Once they've finished, neither counts.
        let _ = racing.apply(
            &Fin {
                plid: PlayerId(5),
                ..Default::default()
            }
            .into(),
        );
        let cx = Context::new(&racing, Instant::now());
        let _ = zinc.update(yellow(), &cx);
        let _ = zinc.update(contact(), &cx);
        assert_eq!(tally(&zinc), (1, 1));
    }

    #[test]
    fn joins_are_let_through_only_in_unlocked_cars() {
        let world = world();
        let cx = Context::new(&world, Instant::now());
        let mut zinc = service(crate::LICENCES);

        // A new driver has the UF1 and XFG, and nothing else.
        for car in [Vehicle::Uf1, Vehicle::Xfg] {
            let effects = zinc.update(join_request(2, car), &cx);
            assert!(matches!(answer(&effects), Some(JrrAction::Spawn)), "{car}");
        }
        let modded = zinc.update(join_request(2, Vehicle::Mod(0x1D_FF72)), &cx);
        assert!(matches!(answer(&modded), Some(JrrAction::Reject)));

        let xrg = zinc.update(join_request(2, Vehicle::Xrg), &cx);
        assert!(matches!(answer(&xrg), Some(JrrAction::Reject)), "{xrg:?}");
        assert_eq!(
            chat(&xrg),
            ["^1You haven't unlocked the XRG yet ^8- your cars: ^3UF1 XFG"]
        );

        // Three points later, the XRG is theirs.
        zinc.state_mut().set("bob", 3);
        let xrg = zinc.update(join_request(2, Vehicle::Xrg), &cx);
        assert!(matches!(answer(&xrg), Some(JrrAction::Spawn)), "{xrg:?}");
        assert!(!restricts(&xrg));
    }

    #[test]
    fn a_track_with_its_own_cars_refuses_the_rest_whatever_the_licence() {
        use insim_extra::kitcar::director::{Combo, RaceDirectorStatus};

        /// `world`, with the director running `combo`.
        fn on<'a>(world: &'a World, combo: &'a Combo) -> Context<'a> {
            Context::new(world, Instant::now()).with_director(RaceDirectorStatus {
                combo: Some(combo),
                ..Default::default()
            })
        }

        let world = world();
        let bl3 = Combo::untimed("BL3".parse().unwrap()).vehicles(vec![
            Vehicle::Uf1,
            Vehicle::Xfr,
            Vehicle::Xrg,
        ]);
        let fe1 = Combo::untimed("FE1".parse().unwrap());
        let mut zinc = service(crate::LICENCES);
        zinc.state_mut().set("bob", 5);

        // The garage restriction should stop these first; this is the backstop.
        let modded = zinc.update(join_request(2, Vehicle::Mod(0x1D_FF72)), &on(&world, &bl3));
        assert!(matches!(answer(&modded), Some(JrrAction::Reject)));
        assert_eq!(chat(&modded), ["^1BL3 only allows ^3UF1 XFR XRG"]);
        let ai = Input::Packet(
            Npl {
                ucid: ConnectionId(2),
                nump: 0,
                ptype: PlayerType::AI,
                cname: Vehicle::Fxo,
                ..Default::default()
            }
            .into(),
        );
        let ai = zinc.update(ai, &on(&world, &bl3));
        assert!(matches!(answer(&ai), Some(JrrAction::Reject)), "AI too");
        let xrg = zinc.update(join_request(2, Vehicle::Xrg), &on(&world, &bl3));
        assert!(matches!(answer(&xrg), Some(JrrAction::Spawn)));

        // On the track, the licence still counts.
        zinc.state_mut().set("bob", 0);
        let xrg = zinc.update(join_request(2, Vehicle::Xrg), &on(&world, &bl3));
        assert!(matches!(answer(&xrg), Some(JrrAction::Reject)));
        assert!(
            chat(&xrg)[0].contains("haven't unlocked"),
            "{:?}",
            chat(&xrg)
        );

        // An open track leaves it to the licence.
        zinc.state_mut().set("bob", 5);
        let modded = zinc.update(join_request(2, Vehicle::Mod(0x1D_FF72)), &on(&world, &fe1));
        assert!(matches!(answer(&modded), Some(JrrAction::Spawn)));
    }

    #[test]
    fn only_join_requests_are_answered() {
        let world = world();
        let cx = Context::new(&world, Instant::now());
        let mut zinc = service(crate::LICENCES);

        // A player already on track (`nump` set) is a join, not a request.
        let joined = Input::Packet(
            Npl {
                ucid: ConnectionId(2),
                plid: PlayerId(6),
                nump: 1,
                cname: Vehicle::Xrg,
                ..Default::default()
            }
            .into(),
        );
        assert!(answer(&zinc.update(joined, &cx)).is_none());

        // AI, and connections the world doesn't know, are let through.
        let ai = Input::Packet(
            Npl {
                ucid: ConnectionId(2),
                nump: 0,
                ptype: PlayerType::AI,
                cname: Vehicle::Fxo,
                ..Default::default()
            }
            .into(),
        );
        assert!(matches!(
            answer(&zinc.update(ai, &cx)),
            Some(JrrAction::Spawn)
        ));
        let stranger = zinc.update(join_request(9, Vehicle::Fxo), &cx);
        assert!(matches!(answer(&stranger), Some(JrrAction::Spawn)));
    }

    fn command_from(ucid: u8, text: &str) -> Input<Msg> {
        Input::Packet(
            insim::insim::Mso {
                ucid: ConnectionId(ucid),
                usertype: insim::insim::MsoUserType::Prefix,
                msg: text.into(),
                ..Default::default()
            }
            .into(),
        )
    }

    #[test]
    fn only_admins_can_rotate() {
        let mut world = world();
        let _ = world.apply(
            &Ncn {
                ucid: ConnectionId(3),
                uname: "admin".into(),
                admin: true,
                ..Default::default()
            }
            .into(),
        );
        let cx = Context::new(&world, Instant::now());
        let mut zinc = service(crate::LICENCES);

        let refused = zinc.update(command_from(2, "!rotate"), &cx);
        assert_eq!(refused.director_commands().count(), 0);
        assert_eq!(chat(&refused), ["^1!rotate is for admins only"]);

        let rotated = zinc.update(command_from(3, "!rotate"), &cx);
        assert_eq!(
            rotated.director_commands().collect::<Vec<_>>(),
            [&DirectorCommand::Rotate]
        );
    }

    #[test]
    fn rank_toggles_the_card_and_closing_or_leaving_clears_it() {
        let world = world();
        let cx = Context::new(&world, Instant::now());
        let mut zinc = service(crate::LICENCES);
        let bob = ConnectionId(2);

        let _ = zinc.update(command_from(2, "!rank"), &cx);
        assert!(zinc.state().rank_open.contains(&bob));
        let _ = zinc.update(command_from(2, "!rank"), &cx);
        assert!(!zinc.state().rank_open.contains(&bob));

        let _ = zinc.update(command_from(2, "!rank"), &cx);
        let _ = zinc.update(Input::Msg(Msg::CloseRank(bob)), &cx);
        assert!(!zinc.state().rank_open.contains(&bob));
    }

    #[test]
    fn standings_rank_by_points_then_name() {
        let mut zinc = Zinc::new(crate::LICENCES);
        for (uname, points) in [("bob", 2), ("amy", 2), ("cat", 5)] {
            let _ = zinc.drivers.insert(
                uname.into(),
                Driver {
                    points,
                    ..Default::default()
                },
            );
        }
        let standing = crate::rank::Standing::of(&zinc, "bob");
        assert_eq!(standing.position, Some(3));
        assert_eq!(standing.of, 3);
        assert_eq!(crate::rank::Standing::of(&zinc, "amy").position, Some(2));
        assert_eq!(crate::rank::Standing::of(&zinc, "new").position, None);
    }

    #[test]
    fn the_topbar_is_pinned_above_the_rank_card() {
        use insim_extra::ui::{Screen, id_pool::ClickIdPool};

        let world = world();
        let cx = Context::new(&world, Instant::now());
        let mut zinc = service(crate::LICENCES);
        let _ = zinc.update(command_from(2, "!rank"), &cx);

        let mut pool = ClickIdPool::new();
        let mut screen = Screen::<Msg>::new(ConnectionId(2));
        let buttons: Vec<_> = screen
            .render(&mut pool, cx.now(), |ui| {
                Service::view(&zinc, ConnectionId(2), ui, &cx)
            })
            .into_iter()
            .filter_map(|p| match p {
                Packet::Btn(btn) => Some(btn),
                _ => None,
            })
            .collect();
        let at = |text: &str| {
            buttons
                .iter()
                .find(|b| b.text.contains(text))
                .map(|b| b.t)
                .unwrap_or_else(|| panic!("no {text:?} button"))
        };
        assert_eq!(at("!help"), 0);
        assert_eq!(at("Your rank"), 60);
        // A clickable button shows the mouse cursor, which gets in the way of
        // mouse steering, so the always-on bar has none.
        assert!(
            buttons
                .iter()
                .filter(|b| b.t == 0)
                .all(|b| !b.bstyle.flags.contains(insim::insim::BtnStyleFlags::CLICK)),
            "{buttons:?}"
        );
    }

    #[test]
    fn the_card_snapshots_the_standing_when_opened() {
        use insim_extra::ui::{Screen, id_pool::ClickIdPool};

        let mut zinc = Zinc::new(crate::LICENCES);
        let _ = zinc.drivers.insert(
            "bob".into(),
            Driver {
                points: 1,
                ..Default::default()
            },
        );
        let mut pool = ClickIdPool::new();
        let mut screen = Screen::<Msg>::new(ConnectionId(2));
        let now = Instant::now();
        let texts = |packets: Vec<Packet>| -> Vec<String> {
            packets
                .into_iter()
                .filter_map(|p| match p {
                    Packet::Btn(btn) => Some(btn.text),
                    _ => None,
                })
                .collect()
        };
        fn card(zinc: &Zinc) -> impl FnOnce(&mut Ui<'_, Msg>) -> Option<Node<Msg>> + '_ {
            move |ui| Some(ui.child(Child::with_props((zinc, "bob"), RankCard::default)))
        }

        let opened = texts(screen.render(&mut pool, now, card(&zinc)));
        assert!(
            opened.iter().any(|t| t.contains("1 point(s)")),
            "{opened:?}"
        );

        // A point scored while it's open doesn't change the snapshot.
        zinc.drivers.get_mut("bob").unwrap().points = 2;
        let open = texts(screen.render(&mut pool, now, card(&zinc)));
        assert!(open.is_empty(), "nothing changed on screen: {open:?}");
    }

    /// The real host, director and ZINC against a fake LFS: once the only
    /// driver finishes, the topbar counts down to the next race each second.
    #[tokio::test]
    async fn the_topbar_counts_down_to_the_restart() {
        use std::time::Duration;

        use insim::{
            core::track::Track,
            insim::{RaceInProgress, RaceLaps, Rst, Sta},
            test::{Lfs, pair},
        };
        use insim_extra::kitcar::{
            Host,
            director::{Combo, Cycle, Every, RaceDirector},
        };

        /// Button texts until one contains `want`.
        async fn shown(lfs: &mut Lfs, want: &str) -> String {
            loop {
                if let Packet::Btn(btn) = lfs.recv().await.expect("host closed")
                    && btn.text.contains(want)
                {
                    return btn.text;
                }
            }
        }

        tokio::time::timeout(Duration::from_secs(10), async {
            let (session, mut lfs) = pair().await;
            let host = Host::spawn(session);
            let fe1: Track = "FE1".parse().unwrap();
            let combo = |track: &str| Combo::untimed(track.parse().unwrap());
            host.start_director(
                RaceDirector::new(Cycle::new([combo("FE1"), combo("AS1")]))
                    .every(Every::Races(1))
                    .auto_restart(Duration::from_secs(60)),
            )
            .await
            .unwrap();
            let _zinc = host.start(service(crate::LICENCES)).await.unwrap();

            lfs.send(Ncn {
                ucid: ConnectionId(2),
                uname: "bob".into(),
                ..Default::default()
            })
            .await;
            lfs.send(Sta {
                track: fe1,
                raceinprog: RaceInProgress::No,
                ..Default::default()
            })
            .await;
            lfs.send(Rst {
                track: fe1,
                racelaps: RaceLaps::Laps(1),
                ..Default::default()
            })
            .await;
            lfs.send(Npl {
                ucid: ConnectionId(2),
                plid: PlayerId(5),
                nump: 1,
                ..Default::default()
            })
            .await;
            let _ = shown(&mut lfs, "race 1 of 1").await;

            lfs.send(Fin {
                plid: PlayerId(5),
                ..Default::default()
            })
            .await;
            // The last race on FE1, so it names the next track.
            assert_eq!(
                shown(&mut lfs, "AS1 in").await,
                "^3Track change: AS1 in 10s"
            );
            assert_eq!(shown(&mut lfs, "AS1 in").await, "^3Track change: AS1 in 9s");

            host.shutdown().await.unwrap();
        })
        .await
        .expect("the countdown showed and ticked");
    }
}
