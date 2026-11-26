//! Setting up the lobby for a [`Combo`]: track, race settings, time of day,
//! cars and layout.

use insim::{
    core::vehicle::Vehicle,
    identifiers::ConnectionId,
    insim::{Axm, Mal, Mst, Plc, PlcAllowedCarsSet, PmoAction, PmoFlags},
};

use super::{Combo, LOADING, Layout, READY, RaceDirector};
use crate::{commands::time::TimeOfDay, kitcar::Effects, world::World};

impl RaceDirector {
    /// Set up the lobby, or just prompt for `/ready` if it's done. While a
    /// combo loads, ask players to wait.
    pub(super) fn set_up<M: Send + 'static>(&mut self, world: &World) -> Effects<M> {
        let combo = match &self.current {
            Some(combo) if !self.loading => combo,
            _ => return Effects::new().message(LOADING, ConnectionId::ALL),
        };
        if self.configured {
            return Effects::new().message(READY, ConnectionId::ALL);
        }
        let time = (!combo.time.is_empty()).then(|| combo.time[self.rng.usize(..combo.time.len())]);
        let (effects, done) = self.configure(combo, time, world);
        self.configured = done;
        if done {
            tracing::info!(track = %combo.track, "lobby set up, waiting for /ready");
            self.set_up = Some(combo.clone());
            effects.message(READY, ConnectionId::ALL)
        } else {
            effects
        }
    }

    /// Returns the packets and whether setup is complete. LFS changes track
    /// asynchronously, so nothing else is sent until `TrackChanged`.
    pub(super) fn configure<M: Send + 'static>(
        &self,
        combo: &Combo,
        time: Option<TimeOfDay>,
        world: &World,
    ) -> (Effects<M>, bool) {
        if world.track() != Some(combo.track) {
            tracing::info!(track = %combo.track, "changing track");
            return (
                Effects::new()
                    .packet(Mst::new("/clear"))
                    .packet(Mst::new(format!("/track {}", combo.track))),
                false,
            );
        }
        if let Some(time) = time {
            tracing::info!(track = %combo.track, ?time, "time of day");
        }
        let effects = Effects::new()
            .packet(Mst::new(combo.laps.to_command()))
            .packet(Mst::new(format!("/wind {}", combo.wind)))
            .packet(Mst::new(combo.start.to_command()))
            .packet(Mst::new(if combo.pit_stop_required() {
                "/mustpit yes"
            } else {
                "/mustpit no"
            }))
            .packets(time.iter().flat_map(TimeOfDay::to_commands).map(Mst::new))
            .packets(restrict_everyone(&combo.vehicles))
            .packet(Mst::new("/axclear"));
        let effects = match &combo.layout {
            None => effects,
            Some(Layout::Axload(name)) => effects
                .message(
                    "^3Setting up - loading layout, please wait...",
                    ConnectionId::ALL,
                )
                .packet(Mst::new(format!("/axload {name}"))),
            Some(Layout::Objects(objects)) => {
                let chunks = objects.chunks(60);
                let last = chunks.len().saturating_sub(1);
                effects.packets(chunks.enumerate().map(|(index, chunk)| Axm {
                    ucid: ConnectionId::LOCAL,
                    flags: if index == last {
                        PmoFlags::FILE_END
                    } else {
                        PmoFlags::empty()
                    },
                    action: PmoAction::AddObjects(chunk.to_vec()),
                    ..Default::default()
                }))
            },
        };
        (effects, true)
    }
}

/// Restrict everyone to `vehicles`, or allow everything if empty. Standard
/// cars go in a `Plc` to every connection, mods in a server-wide `Mal`. An
/// empty `Mal` allows any mod, so a list of only standard cars turns mods off
/// with `/allowmods no` instead. Every `Mal` goes with `/unapproved yes`: the
/// list is the authority, whether or not LFS has approved the mods on it.
fn restrict_everyone(vehicles: &[Vehicle]) -> Vec<insim::Packet> {
    if vehicles.is_empty() {
        return vec![
            Mst::new("/allowmods yes").into(),
            Mst::new("/unapproved yes").into(),
            Plc {
                ucid: ConnectionId::ALL,
                cars: PlcAllowedCarsSet::all(),
                ..Default::default()
            }
            .into(),
            Mal::default().into(),
        ];
    }
    let mut cars = PlcAllowedCarsSet::default();
    let mut mods = Mal::default();
    for vehicle in vehicles {
        match vehicle {
            Vehicle::Mod(_) => {
                let _ = mods.insert(*vehicle);
            },
            _ => {
                let _ = cars.insert(*vehicle);
            },
        }
    }
    let plc = Plc {
        ucid: ConnectionId::ALL,
        cars,
        ..Default::default()
    };
    if mods.is_empty() {
        vec![Mst::new("/allowmods no").into(), plc.into()]
    } else {
        vec![
            Mst::new("/allowmods yes").into(),
            Mst::new("/unapproved yes").into(),
            plc.into(),
            mods.into(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use insim::{
        Packet,
        insim::{Ncn, RaceInProgress, RaceLaps, Sta},
    };

    use super::*;
    use crate::kitcar::director::{
        StartOrder,
        tests::{bl1, fe1x, feed},
    };

    /// The commands sent setting up a lobby already on `combo`'s track.
    fn set_up_commands(combo: Combo) -> Vec<String> {
        set_up_commands_with(RaceDirector::new(combo.clone()), combo)
    }

    /// Like [`set_up_commands`], with `director` already configured.
    fn set_up_commands_with(mut director: RaceDirector, combo: Combo) -> Vec<String> {
        let track = combo.track;
        let mut world = World::new();
        director.current = Some(combo);
        let lobby = Sta {
            track,
            raceinprog: RaceInProgress::No,
            ..Default::default()
        };
        let _ = feed(&mut world, &mut director, lobby.into());
        let joined = Ncn {
            ucid: ConnectionId(1),
            uname: "driver".into(),
            ..Default::default()
        };
        feed(&mut world, &mut director, joined.into()).0
    }

    #[test]
    fn sets_the_time_of_day_with_the_lobby() {
        use crate::commands::time::TimeOfDay;

        let commands = set_up_commands(Combo::untimed(fe1x()).time(TimeOfDay::fixed(17, 30)));
        let wind = commands.iter().position(|c| c.starts_with("/wind"));
        let set = commands.iter().position(|c| c == "/time set 17:30");
        let mul = commands.iter().position(|c| c == "/time mul 0");
        assert!(wind < set && set < mul && wind.is_some(), "{commands:?}");

        // Without one, the server's clock is left alone.
        let untouched = set_up_commands(Combo::untimed(fe1x()));
        assert!(
            !untouched.iter().any(|c| c.starts_with("/time")),
            "{untouched:?}"
        );
    }

    #[test]
    fn picks_one_of_several_times_each_time_a_combo_comes_up() {
        use crate::commands::time::TimeOfDay;

        let night = "/time set 00:00";
        let noon = "/time set 12:00";
        let mut seen = std::collections::HashSet::new();
        for seed in 0..20 {
            let combo =
                Combo::untimed(fe1x()).times([TimeOfDay::fixed(0, 0), TimeOfDay::fixed(12, 0)]);
            let commands = set_up_commands_with(RaceDirector::new(combo.clone()).seed(seed), combo);
            let picked: Vec<_> = commands
                .iter()
                .filter(|c| c.starts_with("/time set"))
                .collect();
            assert_eq!(picked.len(), 1, "exactly one time: {commands:?}");
            assert!(picked[0] == night || picked[0] == noon, "{picked:?}");
            let _ = seen.insert(picked[0].clone());
        }
        assert_eq!(seen.len(), 2, "both come up over 20 seeds");
    }

    /// The car restrictions sent setting up a lobby already on `combo`'s track:
    /// `/allowmods` and `/unapproved`, the `Plc`s and the `Mal`s.
    fn restrictions(combo: Combo) -> (Vec<String>, Vec<Plc>, Vec<Mal>) {
        let mut world = World::new();
        let lobby = Sta {
            track: combo.track,
            raceinprog: RaceInProgress::No,
            ..Default::default()
        };
        let _ = world.apply(&lobby.into());
        let director = RaceDirector::new(combo.clone());
        let (effects, done) = director.configure::<()>(&combo, None, &world);
        assert!(done, "already on the track");
        let mut seen = (Vec::new(), Vec::new(), Vec::new());
        for packet in effects.packets_sent() {
            match packet {
                Packet::Mst(Mst { msg, .. })
                    if msg.starts_with("/allowmods") || msg.starts_with("/unapproved") =>
                {
                    seen.0.push(msg.clone());
                },
                Packet::Plc(plc) => seen.1.push(plc.clone()),
                Packet::Mal(mal) => seen.2.push(mal.clone()),
                _ => {},
            }
        }
        seen
    }

    #[test]
    fn every_combo_states_its_cars() {
        // By default, everything: whatever the last track restricted is lifted.
        let (allowmods, plcs, mals) = restrictions(Combo::untimed(fe1x()));
        assert_eq!(allowmods, ["/allowmods yes", "/unapproved yes"]);
        let [open] = plcs.try_into().expect("one Plc");
        assert_eq!(open.ucid, ConnectionId::ALL);
        assert_eq!(open.cars, PlcAllowedCarsSet::all());
        let [any] = mals.try_into().expect("one Mal");
        assert!(any.is_empty(), "an empty Mal allows any mod");

        // Standard cars only: those, and no mods at all.
        let rallycross = Combo::untimed(bl1()).vehicles([Vehicle::Uf1, Vehicle::Xfr]);
        let (allowmods, plcs, mals) = restrictions(rallycross);
        // No mod list, so nothing to say about unapproved mods.
        assert_eq!(allowmods, ["/allowmods no"]);
        let [narrow] = plcs.try_into().expect("one Plc");
        assert!(narrow.cars.contains(&Vehicle::Uf1) && narrow.cars.contains(&Vehicle::Xfr));
        assert!(!narrow.cars.contains(&Vehicle::Fxo));
        assert!(mals.is_empty(), "{mals:?}");

        // A mod in the list: mods on, just that one.
        let modded = Combo::untimed(bl1()).vehicles([Vehicle::Uf1, Vehicle::Mod(0x1D_FF72)]);
        let (allowmods, plcs, mals) = restrictions(modded);
        assert_eq!(allowmods, ["/allowmods yes", "/unapproved yes"]);
        let [cars] = plcs.try_into().expect("one Plc");
        assert!(cars.cars.contains(&Vehicle::Uf1) && !cars.cars.contains(&Vehicle::Xfr));
        let [mods] = mals.try_into().expect("one Mal");
        assert!(mods.contains(&Vehicle::Mod(0x1D_FF72)));
        assert_eq!(mods.iter().count(), 1);
    }

    #[test]
    fn a_pit_stop_is_only_required_where_there_is_room_for_one() {
        let asking = |laps| {
            Combo::untimed(fe1x())
                .laps(laps)
                .must_pit(true)
                .pit_stop_required()
        };
        assert!(!asking(RaceLaps::Untimed));
        assert!(!asking(RaceLaps::Practice));
        assert!(!asking(RaceLaps::Laps(1)));
        assert!(asking(RaceLaps::Laps(2)));
        assert!(asking(RaceLaps::Hours(1)));
        // Never when not asked for.
        assert!(
            !Combo::untimed(fe1x())
                .laps(RaceLaps::Laps(5))
                .pit_stop_required()
        );

        // An untimed combo that asks still sends "no".
        let untimed = set_up_commands(Combo::untimed(fe1x()).must_pit(true));
        assert!(untimed.iter().any(|c| c == "/mustpit no"), "{untimed:?}");
    }

    #[test]
    fn every_combo_restates_its_race_settings() {
        // Defaults are sent too, so nothing carries over from the last track.
        let plain = set_up_commands(Combo::untimed(fe1x()));
        for want in ["/start finish", "/mustpit no"] {
            assert!(plain.iter().any(|c| c == want), "{want}: {plain:?}");
        }
        let set = set_up_commands(
            Combo::untimed(fe1x())
                .laps(RaceLaps::Laps(5))
                .start(StartOrder::Reverse)
                .must_pit(true),
        );
        for want in ["/start reverse", "/mustpit yes"] {
            assert!(set.iter().any(|c| c == want), "{want}: {set:?}");
        }
    }
}
