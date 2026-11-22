//! ZINC's rules: points for clean, confirmed results, yellow flags counted,
//! and cars unlocked by points. All in memory, so it resets on restart.
//!
//! Built as a `Router`: one handler per input.

use std::collections::HashMap;

use insim::{
    core::vehicle::Vehicle,
    identifiers::{ConnectionId, PlayerId},
    insim::{Flg, FlgType, PlayerType, Plc, PlcAllowedCarsSet, RaceConfirmFlags, Res},
};
use insim_extra::{
    chat::ChatCommand,
    kitcar::{
        Context, Effects,
        services::router::{Caller, Router},
    },
    world::{Connected, TrackChanged, World},
};

/// The licence ladder. Rung `n` needs `n` points, and rungs stack: 3 points
/// unlocks the UF1, XFR, XRG, XRT, RB4 and FXO. Add rungs to extend it.
pub const LICENCES: &[&[Vehicle]] = &[
    &[Vehicle::Uf1],
    &[Vehicle::Xfr],
    &[Vehicle::Xrg],
    // The TBO class.
    &[Vehicle::Xrt, Vehicle::Rb4, Vehicle::Fxo],
    &[Vehicle::Lx4],
];

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
    /// Yellow flags caused.
    pub yellows: u32,
}

impl Driver {
    /// The cars this driver has unlocked.
    pub fn cars(&self) -> impl Iterator<Item = Vehicle> + '_ {
        LICENCES
            .iter()
            .take(self.points as usize + 1)
            .flat_map(|rung| rung.iter().copied())
    }

    fn allowed(&self) -> PlcAllowedCarsSet {
        let mut cars = PlcAllowedCarsSet::default();
        for car in self.cars() {
            let _ = cars.insert(car);
        }
        cars
    }

    fn car_list(&self) -> String {
        self.cars()
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
    /// List the commands.
    Help,
}

/// Every driver's record.
#[derive(Debug, Default)]
pub struct Zinc {
    drivers: HashMap<String, Driver>,
}

impl Zinc {
    /// A driver's record, or a fresh one.
    pub fn driver(&self, uname: &str) -> Driver {
        self.drivers.get(uname).copied().unwrap_or_default()
    }

    /// Restrict a connection to its driver's unlocked cars.
    fn licence(&self, ucid: ConnectionId, uname: &str) -> Effects<()> {
        Effects::new().packet(Plc {
            ucid,
            cars: self.driver(uname).allowed(),
            ..Default::default()
        })
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
/// A router fits because each rule reacts to one input on its own, with no
/// ordering or race-phase guards. Once rules depend on each other or on the
/// race phase (say, yellows only counting while racing), a hand-written
/// `Service::update` is clearer.
pub fn service() -> Router<Zinc> {
    Router::new(Zinc::default())
        .on(joined)
        .on(track_changed)
        .on(result)
        .on(flagged)
        .commands(command)
}

/// Licence each driver as they join, including those already connected.
fn joined(zinc: &mut Zinc, event: &Connected, _: &Context<'_>) -> Effects<()> {
    let driver = zinc.driver(&event.info.uname);
    let effects = zinc.licence(event.info.ucid, &event.info.uname);
    if event.existing {
        return effects;
    }
    effects.message(
        format!(
            "^7ZINC: ^2{} point(s)^7, cars: ^3{}^7 - type !help",
            driver.points,
            driver.car_list()
        ),
        event.info.ucid,
    )
}

/// Reapply every licence on a track change, in case anything reset them.
fn track_changed(zinc: &mut Zinc, _: &TrackChanged, cx: &Context<'_>) -> Effects<()> {
    cx.world()
        .connections()
        .fold(Effects::new(), |effects, conn| {
            effects.and(zinc.licence(conn.ucid, &conn.uname))
        })
}

/// A confirmed, clean result earns a point, which may unlock a car.
///
/// LFS sends `Res` unprompted (`reqi` 0) once a result is confirmed. Replies
/// to `TINY_RES` repeat old results, so only the unprompted one counts. `Res`
/// carries the username, so the point counts even if the driver has left.
fn result(zinc: &mut Zinc, res: &Res, cx: &Context<'_>) -> Effects<()> {
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

    // Tell them and update their cars, if they're still here.
    let Some(ucid) = cx
        .world()
        .connections()
        .find(|conn| conn.uname == res.uname)
        .map(|conn| conn.ucid)
    else {
        return Effects::none();
    };
    let effects =
        Effects::new().message(format!("^2+1 point^7 - you have ^2{}", after.points), ucid);
    let unlocked: Vec<String> = after
        .cars()
        .filter(|car| !before.cars().any(|c| c == *car))
        .map(|car| car.to_string())
        .collect();
    if unlocked.is_empty() {
        return effects;
    }
    effects.and(zinc.licence(ucid, &res.uname)).message(
        format!(
            "^3Licence upgraded^7 - you can now drive ^3{}",
            unlocked.join(" ")
        ),
        ucid,
    )
}

/// Count each yellow flag a driver causes.
fn flagged(zinc: &mut Zinc, flg: &Flg, cx: &Context<'_>) {
    if !(flg.offon && flg.flag == FlgType::Yellow) {
        return;
    }
    if let Some((_, uname)) = Zinc::human(cx.world(), flg.plid) {
        zinc.drivers.entry(uname.to_owned()).or_default().yellows += 1;
    }
}

fn command(zinc: &mut Zinc, caller: Caller, cmd: Cmd, _: &Context<'_>) -> Effects<()> {
    match cmd {
        Cmd::Stats => {
            let driver = zinc.driver(&caller.uname);
            caller.reply(format!(
                "^7{}: ^2{} point(s)^7, ^3{} yellow(s)^7, cars: ^3{}",
                caller.uname,
                driver.points,
                driver.yellows,
                driver.car_list()
            ))
        },
        Cmd::Top => {
            let mut drivers: Vec<_> = zinc.drivers.iter().collect();
            drivers.sort_by(|a, b| b.1.points.cmp(&a.1.points).then(a.0.cmp(b.0)));
            let lines: Vec<String> = drivers
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
        Cmd::Help => caller.reply(
            Cmd::COMMANDS
                .iter()
                .map(|c| format!("^7!{c} ^8- {}", c.help()))
                .collect::<Vec<_>>()
                .join("\n"),
        ),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use insim::{
        Packet,
        insim::{Ncn, Npl},
    };
    use insim_extra::kitcar::{Input, Service};

    use super::*;

    /// A world with one human driver, `bob`, on track as player 5.
    fn world() -> World {
        let mut world = World::new();
        let _ = world.apply_packet(
            &Ncn {
                ucid: ConnectionId(2),
                uname: "bob".into(),
                ..Default::default()
            }
            .into(),
        );
        let _ = world.apply_packet(
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

    fn input(res: Res) -> Input<()> {
        Input::Packet(res.into())
    }

    fn licences(effects: &Effects<()>) -> Vec<Vec<Vehicle>> {
        effects
            .packets_sent()
            .filter_map(|p| match p {
                Packet::Plc(plc) => Some(plc.cars.iter().copied().collect()),
                _ => None,
            })
            .collect()
    }

    fn chat(effects: &Effects<()>) -> Vec<String> {
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
        let mut zinc = service();
        let rungs: [&[Vehicle]; 4] = [
            &[Vehicle::Xfr],
            &[Vehicle::Xrg],
            &[Vehicle::Xrt, Vehicle::Rb4, Vehicle::Fxo],
            &[Vehicle::Lx4],
        ];

        let mut unlocked = vec![Vehicle::Uf1];
        for (points, rung) in (1..).zip(rungs) {
            let effects = zinc.update(input(res(RaceConfirmFlags::empty())), &cx);
            assert_eq!(zinc.state().driver("bob").points, points);
            unlocked.extend_from_slice(rung);

            // Their licence now allows every car so far, and nothing else.
            let [licence] = licences(&effects).try_into().expect("one licence");
            assert_eq!(licence.len(), unlocked.len(), "{licence:?}");
            assert!(
                unlocked.iter().all(|car| licence.contains(car)),
                "{licence:?}"
            );
            let names: Vec<String> = rung.iter().map(ToString::to_string).collect();
            assert!(
                chat(&effects)
                    .iter()
                    .any(|c| c.contains(&format!("you can now drive ^3{}", names.join(" ")))),
                "{:?}",
                chat(&effects)
            );
        }

        // Past the top of the ladder, points still count but unlock nothing.
        let effects = zinc.update(input(res(RaceConfirmFlags::empty())), &cx);
        assert_eq!(zinc.state().driver("bob").points, 5);
        assert!(licences(&effects).is_empty());
    }

    #[test]
    fn only_new_clean_confirmed_results_count() {
        let world = world();
        let cx = Context::new(&world, Instant::now());
        let mut zinc = service();

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
        let mut zinc = service();

        let effects = zinc.update(input(res(RaceConfirmFlags::empty())), &cx);
        assert!(effects.is_empty(), "nobody to tell");
        assert_eq!(zinc.state().driver("bob").points, 1);
    }

    #[test]
    fn yellow_flags_are_counted_when_raised() {
        let world = world();
        let cx = Context::new(&world, Instant::now());
        let mut zinc = service();
        let flag = |offon, flag| {
            Input::Packet(
                Flg {
                    plid: PlayerId(5),
                    offon,
                    flag,
                    ..Default::default()
                }
                .into(),
            )
        };

        let _ = zinc.update(flag(true, FlgType::Yellow), &cx);
        let _ = zinc.update(flag(false, FlgType::Yellow), &cx);
        let _ = zinc.update(flag(true, FlgType::Blue), &cx);
        assert_eq!(zinc.state().driver("bob").yellows, 1);
    }

    #[test]
    fn drivers_are_licensed_when_they_join() {
        let world = world();
        let cx = Context::new(&world, Instant::now());
        let mut zinc = service();
        let info = world.connection(ConnectionId(2)).unwrap().clone();

        let effects = zinc.update(
            Input::World(insim_extra::world::WorldEvent::Connected(Connected {
                info,
                existing: false,
            })),
            &cx,
        );
        assert_eq!(licences(&effects), [vec![Vehicle::Uf1]]);
    }
}
