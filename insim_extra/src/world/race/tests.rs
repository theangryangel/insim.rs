//! Characterization tests for the race state machine.
//!
//! These drive a [`World`] through synthesized packets and assert on the
//! emitted [`RaceEvent`]s and the queryable entrant state. They pin the
//! *current* behavior so that refactors of the dispatch/handler layer can be
//! shown to be behavior-preserving. They assert only on the public surface
//! (`apply_packet` output + query methods), so they remain valid across
//! internal restructuring.

use std::time::Duration;

use insim::{
    identifiers::{ConnectionId, PlayerId},
    insim::{
        Cnl, Fin, Lap, Ncn, Npl, Pen, PenaltyInfo, PenaltyReason, Pit, PitStopWorkFlags, Pll, Plp,
        Psf, RaceConfirmFlags, RaceLaps, Reo, Res, Rst, Spx, Toc,
    },
};

use crate::world::{FinishStatus, RaceEvent, World, WorldEvent};

// ── packet builders ───────────────────────────────────────────────────────

fn ncn(ucid: u8, uname: &str, pname: &str) -> insim::Packet {
    Ncn {
        ucid: ConnectionId(ucid),
        uname: uname.into(),
        pname: pname.into(),
        ..Default::default()
    }
    .into()
}

fn npl(plid: u8, ucid: u8, pname: &str) -> insim::Packet {
    Npl {
        plid: PlayerId(plid),
        ucid: ConnectionId(ucid),
        nump: 1,
        pname: pname.into(),
        ..Default::default()
    }
    .into()
}

fn pll(plid: u8) -> insim::Packet {
    Pll {
        plid: PlayerId(plid),
        ..Default::default()
    }
    .into()
}

fn cnl(ucid: u8) -> insim::Packet {
    Cnl {
        ucid: ConnectionId(ucid),
        ..Default::default()
    }
    .into()
}

fn toc(plid: u8, olducid: u8, newucid: u8) -> insim::Packet {
    Toc {
        plid: PlayerId(plid),
        olducid: ConnectionId(olducid),
        newucid: ConnectionId(newucid),
        ..Default::default()
    }
    .into()
}

fn lap(plid: u8, lapsdone: u16, ltime_ms: u64) -> insim::Packet {
    Lap {
        plid: PlayerId(plid),
        ltime: Duration::from_millis(ltime_ms),
        lapsdone,
        numstops: 0,
        ..Default::default()
    }
    .into()
}

fn spx(plid: u8, split: u8, stime_ms: u64) -> insim::Packet {
    Spx {
        plid: PlayerId(plid),
        split,
        stime: Duration::from_millis(stime_ms),
        ..Default::default()
    }
    .into()
}

fn fin(plid: u8, ttime_ms: u64, btime_ms: u64) -> insim::Packet {
    Fin {
        plid: PlayerId(plid),
        ttime: Duration::from_millis(ttime_ms),
        btime: Duration::from_millis(btime_ms),
        numstops: 0,
        confirm: RaceConfirmFlags::empty(),
        ..Default::default()
    }
    .into()
}

fn res(plid: u8, resultnum: u8, numres: u8) -> insim::Packet {
    Res {
        plid: PlayerId(plid),
        resultnum,
        numres,
        ..Default::default()
    }
    .into()
}

fn pit(plid: u8, numstops: u8) -> insim::Packet {
    Pit {
        plid: PlayerId(plid),
        numstops,
        work: PitStopWorkFlags::STOP,
        ..Default::default()
    }
    .into()
}

fn psf(plid: u8, stime_ms: u64) -> insim::Packet {
    Psf {
        plid: PlayerId(plid),
        stime: Duration::from_millis(stime_ms),
        ..Default::default()
    }
    .into()
}

fn pen(plid: u8, newpen: PenaltyInfo, reason: PenaltyReason) -> insim::Packet {
    Pen {
        plid: PlayerId(plid),
        oldpen: PenaltyInfo::None,
        newpen,
        reason,
        ..Default::default()
    }
    .into()
}

fn plp(plid: u8) -> insim::Packet {
    Plp {
        plid: PlayerId(plid),
        ..Default::default()
    }
    .into()
}

fn reo(order: &[u8]) -> insim::Packet {
    let mut reo = Reo {
        nump: order.len() as u8,
        ..Default::default()
    };
    for (i, &plid) in order.iter().enumerate() {
        reo.plid[i] = PlayerId(plid);
    }
    reo.into()
}

fn rst_race(laps: usize) -> insim::Packet {
    Rst {
        racelaps: RaceLaps::Laps(laps),
        qualmins: 0,
        ..Default::default()
    }
    .into()
}

fn rst_qualifying(mins: u8) -> insim::Packet {
    Rst {
        racelaps: RaceLaps::Practice,
        qualmins: mins,
        ..Default::default()
    }
    .into()
}

fn rst_practice() -> insim::Packet {
    Rst {
        racelaps: RaceLaps::Practice,
        qualmins: 0,
        ..Default::default()
    }
    .into()
}

fn rst_untimed() -> insim::Packet {
    Rst {
        racelaps: RaceLaps::Untimed,
        qualmins: 0,
        ..Default::default()
    }
    .into()
}

// ── helpers ───────────────────────────────────────────────────────────────

/// Apply a packet and return only the race events it produced.
fn race_events(world: &World, packet: insim::Packet) -> Vec<RaceEvent> {
    world
        .apply_packet(&packet)
        .into_iter()
        .filter_map(|e| match e {
            WorldEvent::Race(r) => Some(r),
            _ => None,
        })
        .collect()
}

/// Apply a packet, discarding all events.
fn apply(world: &World, packet: insim::Packet) {
    let _ = world.apply_packet(&packet);
}

// ── session gating ──────────────────────────────────────────────────────────

#[test]
fn no_session_means_no_race_tracking() {
    let world = World::new();
    apply(&world, ncn(1, "alice", "Alice"));
    // No Rst: session_kind is None.
    assert!(race_events(&world, npl(1, 1, "Alice")).is_empty());
    assert!(race_events(&world, spx(1, 1, 30_000)).is_empty());
    assert!(race_events(&world, lap(1, 1, 90_000)).is_empty());
    assert!(race_events(&world, pit(1, 1)).is_empty());
    assert!(race_events(&world, pen(1, PenaltyInfo::Dt, PenaltyReason::Speeding)).is_empty());
    assert!(world.entrants().is_empty());
}

#[test]
fn practice_session_tracks_nothing() {
    let world = World::new();
    apply(&world, rst_practice());
    apply(&world, ncn(1, "alice", "Alice"));
    assert!(race_events(&world, npl(1, 1, "Alice")).is_empty());
    assert!(race_events(&world, lap(1, 1, 90_000)).is_empty());
    assert!(world.entrants().is_empty());
}

#[test]
fn untimed_session_tracks_nothing() {
    let world = World::new();
    apply(&world, rst_untimed());
    apply(&world, ncn(1, "alice", "Alice"));
    assert!(race_events(&world, npl(1, 1, "Alice")).is_empty());
    assert!(race_events(&world, lap(1, 1, 90_000)).is_empty());
    assert!(world.entrants().is_empty());
}

#[test]
fn qualifying_tracks_laps_but_fin_does_not_finish() {
    let world = World::new();
    apply(&world, rst_qualifying(10));
    apply(&world, ncn(1, "alice", "Alice"));
    assert!(matches!(
        race_events(&world, npl(1, 1, "Alice")).as_slice(),
        [RaceEvent::EntrantJoined { .. }]
    ));
    // Laps are tracked in qualifying.
    assert!(!race_events(&world, lap(1, 1, 90_000)).is_empty());
    // Fin does NOT finish in qualifying.
    assert!(race_events(&world, fin(1, 90_000, 90_000)).is_empty());
    let e = world.entrant_by_plid(PlayerId(1)).unwrap();
    assert!(matches!(e.status, FinishStatus::Racing));
}

// ── join / lap / split / personal-best / fastest ────────────────────────────

#[test]
fn join_split_lap_emits_ordered_events_and_records_splits() {
    let world = World::new();
    apply(&world, rst_race(5));
    apply(&world, ncn(1, "alice", "Alice"));
    assert!(matches!(
        race_events(&world, npl(1, 1, "Alice")).as_slice(),
        [RaceEvent::EntrantJoined { .. }]
    ));

    assert!(matches!(
        race_events(&world, spx(1, 1, 30_000)).as_slice(),
        [RaceEvent::SplitCompleted { split: 1, .. }]
    ));
    assert!(matches!(
        race_events(&world, spx(1, 2, 33_000)).as_slice(),
        [RaceEvent::SplitCompleted { split: 2, .. }]
    ));

    let events = race_events(&world, lap(1, 1, 90_000));
    // First completed lap: LapCompleted, then PersonalBest, then FastestLap.
    match events.as_slice() {
        [
            RaceEvent::LapCompleted { record, .. },
            RaceEvent::PersonalBest {
                previous: None,
                time: pb,
                ..
            },
            RaceEvent::FastestLap { time: fl, .. },
        ] => {
            assert_eq!(record.lap, 1);
            assert_eq!(record.time, Duration::from_millis(90_000));
            assert_eq!(
                record.splits,
                vec![Duration::from_millis(30_000), Duration::from_millis(33_000)]
            );
            assert_eq!(*pb, Duration::from_millis(90_000));
            assert_eq!(*fl, Duration::from_millis(90_000));
        },
        other => panic!("unexpected events: {other:?}"),
    }

    let e = world.entrant_by_plid(PlayerId(1)).unwrap();
    assert_eq!(e.laps_done, 1);
    assert_eq!(e.best_lap, Some(Duration::from_millis(90_000)));
    assert!(
        e.current_splits.is_empty(),
        "splits drained into the lap record"
    );
    assert_eq!(
        world.fastest_lap().map(|(_, plid, t)| (plid, t)),
        Some((PlayerId(1), Duration::from_millis(90_000)))
    );
}

#[test]
fn slower_second_lap_is_not_a_personal_best() {
    let world = World::new();
    apply(&world, rst_race(5));
    apply(&world, npl(1, 1, "Alice"));
    let _ = race_events(&world, lap(1, 1, 90_000));
    let events = race_events(&world, lap(1, 2, 95_000));
    assert!(
        matches!(events.as_slice(), [RaceEvent::LapCompleted { .. }]),
        "a slower lap only emits LapCompleted: {events:?}"
    );
}

#[test]
fn fastest_lap_tracks_across_entrants() {
    let world = World::new();
    apply(&world, rst_race(5));
    apply(&world, npl(1, 1, "Alice"));
    apply(&world, npl(2, 2, "Bob"));
    let _ = race_events(&world, lap(1, 1, 90_000));
    // Bob goes faster -> new FastestLap.
    let events = race_events(&world, lap(2, 1, 89_000));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, RaceEvent::FastestLap { .. }))
    );
    assert_eq!(
        world.fastest_lap().map(|(_, plid, _)| plid),
        Some(PlayerId(2))
    );
}

// ── DNF ──────────────────────────────────────────────────────────────────────

#[test]
fn leaving_during_a_race_is_a_dnf() {
    let world = World::new();
    apply(&world, rst_race(5));
    apply(&world, npl(1, 1, "Alice"));
    assert!(matches!(
        race_events(&world, pll(1)).as_slice(),
        [RaceEvent::Dnf { .. }]
    ));
    let e = world.entrants().into_iter().next().unwrap();
    assert!(matches!(e.status, FinishStatus::Dnf));
}

#[test]
fn disconnect_during_a_race_is_a_dnf() {
    let world = World::new();
    apply(&world, rst_race(5));
    apply(&world, ncn(1, "alice", "Alice"));
    apply(&world, npl(1, 1, "Alice"));
    assert!(matches!(
        race_events(&world, cnl(1)).as_slice(),
        [RaceEvent::Dnf { .. }]
    ));
}

#[test]
fn leaving_during_qualifying_is_not_a_dnf() {
    let world = World::new();
    apply(&world, rst_qualifying(10));
    apply(&world, npl(1, 1, "Alice"));
    assert!(race_events(&world, pll(1)).is_empty());
}

// ── idempotency / grid ───────────────────────────────────────────────────────

#[test]
fn duplicate_npl_does_not_panic_or_duplicate_entrant() {
    // LFS re-announces existing players in reply to TINY_NPL. The repeated Npl
    // for an already-live plid must be a no-op at the race layer (and must not
    // panic on the unique-plid players index).
    let world = World::new();
    apply(&world, rst_race(5));
    assert!(matches!(
        race_events(&world, npl(1, 1, "Alice")).as_slice(),
        [RaceEvent::EntrantJoined { .. }]
    ));
    assert!(
        race_events(&world, npl(1, 1, "Alice")).is_empty(),
        "re-announced plid produces no fresh race event"
    );
    assert_eq!(world.entrants().len(), 1);
    assert_eq!(world.player_count(), 1, "no duplicate player entry");
}

#[test]
fn npl_after_restart_recreates_entrant() {
    // The players map persists across a session restart while race state is
    // cleared; the re-announced Npl must recreate the entrant for the new
    // session.
    let world = World::new();
    apply(&world, rst_race(5));
    apply(&world, npl(1, 1, "Alice"));
    apply(&world, lap(1, 1, 90_000));
    apply(&world, rst_race(5)); // new session: race state cleared
    assert!(world.entrants().is_empty());
    assert!(matches!(
        race_events(&world, npl(1, 1, "Alice")).as_slice(),
        [RaceEvent::EntrantJoined { .. }]
    ));
    assert_eq!(world.entrants().len(), 1);
}

#[test]
fn pending_grid_is_applied_on_join() {
    let world = World::new();
    apply(&world, rst_race(5));
    // Reo arrives before the player joins: positions are buffered.
    apply(&world, reo(&[2, 1]));
    apply(&world, npl(1, 1, "Alice")); // grid pos 2
    apply(&world, npl(2, 2, "Bob")); // grid pos 1
    assert_eq!(
        world.entrant_by_plid(PlayerId(1)).unwrap().grid_position,
        Some(2)
    );
    assert_eq!(
        world.entrant_by_plid(PlayerId(2)).unwrap().grid_position,
        Some(1)
    );
}

// ── takeover ─────────────────────────────────────────────────────────────────

#[test]
fn takeover_appends_driver_record_with_from_lap() {
    let world = World::new();
    apply(&world, rst_race(5));
    apply(&world, ncn(1, "alice", "Alice"));
    apply(&world, ncn(2, "bob", "Bob"));
    apply(&world, npl(1, 1, "Alice"));
    apply(&world, lap(1, 1, 90_000)); // laps_done = 1
    assert!(matches!(
        race_events(&world, toc(1, 1, 2)).as_slice(),
        [RaceEvent::DriverSwap {
            new_ucid: ConnectionId(2),
            ..
        }]
    ));
    let e = world.entrant_by_plid(PlayerId(1)).unwrap();
    assert_eq!(e.drivers.len(), 2);
    let last = e.drivers.last().unwrap();
    assert_eq!(last.ucid, ConnectionId(2));
    assert_eq!(last.from_lap, 1);
    assert_eq!(last.uname.as_deref(), Some("bob"));
}

// ── rejoin ───────────────────────────────────────────────────────────────────

#[test]
fn rejoin_resumes_same_entrant_with_lap_offset() {
    let world = World::with_rejoin();
    apply(&world, ncn(1, "alice", "Alice"));
    apply(&world, rst_race(20));
    let joined = race_events(&world, npl(1, 1, "Alice"));
    let original_id = match joined.as_slice() {
        [RaceEvent::EntrantJoined { id, .. }] => *id,
        other => panic!("expected EntrantJoined: {other:?}"),
    };
    apply(&world, lap(1, 1, 90_000)); // laps_done = 1
    // Disconnect.
    apply(&world, cnl(1));
    // Reconnect under a fresh connection + player id, same uname.
    apply(&world, ncn(2, "alice", "Alice"));
    let rejoined = race_events(&world, npl(2, 2, "Alice"));
    match rejoined.as_slice() {
        [
            RaceEvent::EntrantRejoined {
                id,
                plid: PlayerId(2),
            },
        ] => {
            assert_eq!(*id, original_id, "same EntrantId reused");
        },
        other => panic!("expected EntrantRejoined: {other:?}"),
    }
    let e = world.entrant_by_plid(PlayerId(2)).unwrap();
    assert_eq!(e.id, original_id);
    assert_eq!(e.lap_offset, 1, "offset bumped by prior laps_done");
    // A fresh lap (LFS counter reset to 1) now lands at effective lap 2.
    let _ = race_events(&world, lap(2, 1, 88_000));
    let e = world.entrant_by_plid(PlayerId(2)).unwrap();
    assert_eq!(e.laps.last().unwrap().lap, 2);
}

#[test]
fn rejoin_with_no_match_creates_fresh_entrant() {
    let world = World::with_rejoin();
    apply(&world, rst_race(20));
    apply(&world, ncn(1, "alice", "Alice"));
    let events = race_events(&world, npl(1, 1, "Alice"));
    assert!(
        matches!(events.as_slice(), [RaceEvent::EntrantJoined { .. }]),
        "no prior entrant to match -> fresh join: {events:?}"
    );
}

// ── pit stops ────────────────────────────────────────────────────────────────

#[test]
fn pit_then_psf_pairs_into_a_pit_record() {
    let world = World::new();
    apply(&world, rst_race(5));
    apply(&world, npl(1, 1, "Alice"));
    // Pit alone produces no event (waiting for Psf).
    assert!(race_events(&world, pit(1, 1)).is_empty());
    match race_events(&world, psf(1, 25_000)).as_slice() {
        [RaceEvent::PitStopComplete { record, .. }] => {
            assert_eq!(record.stop_time, Some(Duration::from_millis(25_000)));
            assert_eq!(record.stop_number, 1);
        },
        other => panic!("expected PitStopComplete: {other:?}"),
    }
    assert_eq!(
        world.entrant_by_plid(PlayerId(1)).unwrap().pit_stops.len(),
        1
    );
}

#[test]
fn result_arriving_after_player_left_is_still_applied() {
    let world = World::new();
    apply(&world, rst_race(5));
    apply(&world, npl(1, 1, "Alice"));
    apply(&world, fin(1, 450_000, 88_000)); // status -> Finished
    apply(&world, pll(1)); // leaves track; no longer live
    match race_events(&world, res(1, 0, 1)).as_slice() {
        [
            RaceEvent::ResultConfirmed {
                result_num: 0,
                num_results: 1,
                ..
            },
        ] => {},
        other => panic!("expected ResultConfirmed: {other:?}"),
    }
    let e = world
        .entrants()
        .into_iter()
        .find(|e| e.plid == PlayerId(1))
        .unwrap();
    match e.status {
        FinishStatus::Finished { result_num, .. } => assert_eq!(result_num, Some(0)),
        other => panic!("expected Finished status: {other:?}"),
    }
}

#[test]
fn penalty_change_is_recorded() {
    let world = World::new();
    apply(&world, rst_race(5));
    apply(&world, npl(1, 1, "Alice"));
    match race_events(&world, pen(1, PenaltyInfo::Dt, PenaltyReason::Speeding)).as_slice() {
        [
            RaceEvent::PenaltyChanged {
                newpen: PenaltyInfo::Dt,
                ..
            },
        ] => {},
        other => panic!("expected PenaltyChanged: {other:?}"),
    }
    assert_eq!(
        world.entrant_by_plid(PlayerId(1)).unwrap().penalty,
        PenaltyInfo::Dt
    );
}

#[test]
fn telepit_discards_in_progress_lap() {
    let world = World::new();
    apply(&world, rst_race(5));
    apply(&world, npl(1, 1, "Alice"));
    apply(&world, spx(1, 1, 30_000));
    match race_events(&world, plp(1)).as_slice() {
        [RaceEvent::TeleportedToPits { .. }] => {},
        other => panic!("expected TeleportedToPits: {other:?}"),
    }
    assert!(
        world
            .entrant_by_plid(PlayerId(1))
            .unwrap()
            .current_splits
            .is_empty(),
        "the in-progress lap's splits are discarded"
    );
}

// ── session reset ────────────────────────────────────────────────────────────

#[test]
fn rst_clears_prior_session_state() {
    let world = World::new();
    apply(&world, rst_race(5));
    apply(&world, npl(1, 1, "Alice"));
    apply(&world, lap(1, 1, 90_000));
    assert!(!world.entrants().is_empty());
    assert!(world.fastest_lap().is_some());
    // A new session resets everything.
    apply(&world, rst_race(5));
    assert!(world.entrants().is_empty());
    assert!(world.fastest_lap().is_none());
    assert!(world.entrant_by_plid(PlayerId(1)).is_none());
}
