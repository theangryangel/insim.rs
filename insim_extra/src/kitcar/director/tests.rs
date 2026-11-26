//! The round, rotations and auto-restart, driven input by input.

use std::time::Instant;

use insim::{
    Packet,
    core::track::Track,
    identifiers::PlayerId,
    insim::{Cnl, Fin, Mst, Ncn, Npl, Pll, RaceInProgress, RaceLaps, Rst, Sta},
};

use super::*;

/// A director whose combo has already loaded, to test the round alone.
fn loaded(combo: Combo) -> RaceDirector {
    let mut director = RaceDirector::new(combo.clone());
    director.current = Some(combo);
    director
}

/// Apply a packet to the world, then pass it and its events on, as the host
/// does. Returns the commands sent and transitions reported.
pub(super) fn feed(
    world: &mut World,
    director: &mut RaceDirector,
    packet: Packet,
) -> (Vec<String>, Vec<Transition>) {
    let events = world.apply(&packet);
    let world = &*world;
    let cx = Context::new(world, Instant::now());
    let mut commands = Vec::new();
    let mut transitions = Vec::new();
    let inputs =
        std::iter::once(Input::<()>::Packet(packet)).chain(events.into_iter().map(Input::World));
    for input in inputs {
        let (effects, transition) = director.round(&input, &cx);
        commands.extend(effects.packets_sent().filter_map(|p| match p {
            Packet::Mst(Mst { msg, .. }) => Some(msg.clone()),
            _ => None,
        }));
        transitions.extend(transition);
    }
    (commands, transitions)
}

#[test]
fn sets_up_in_the_lobby_then_races_and_ends_when_everyone_leaves() {
    let track: Track = "FE1X".parse().unwrap();
    let mut world = World::new();
    let mut director = loaded(Combo::untimed(track));

    // The lobby, already on the right track.
    let _ = feed(
        &mut world,
        &mut director,
        Sta {
            track,
            raceinprog: RaceInProgress::No,
            ..Default::default()
        }
        .into(),
    );
    let (commands, transitions) = feed(
        &mut world,
        &mut director,
        Ncn {
            ucid: ConnectionId(1),
            uname: "driver".into(),
            ..Default::default()
        }
        .into(),
    );
    assert_eq!(transitions, [Transition::SetupStarted]);
    assert!(commands.iter().any(|c| c == "/axclear"), "{commands:?}");
    assert!(!commands.iter().any(|c| c.starts_with("/track")));
    assert_eq!(director.phase(), Phase::SettingUp);

    // /ready starts the session.
    let (_, transitions) = feed(
        &mut world,
        &mut director,
        Rst {
            track,
            ..Default::default()
        }
        .into(),
    );
    assert_eq!(transitions, [Transition::Racing]);
    assert!(director.racing());

    // Everyone leaves mid-race: end it.
    let (commands, transitions) = feed(
        &mut world,
        &mut director,
        Cnl {
            ucid: ConnectionId(1),
            ..Default::default()
        }
        .into(),
    );
    assert_eq!(transitions, [Transition::Ended { shortfall: true }]);
    assert_eq!(commands, ["/end"]);
    assert_eq!(director.phase(), Phase::Waiting);
    // A rotation picks the next track; once LFS is back in the lobby (after
    // the /end), the next player to join triggers the change.
    director.current = Some(Combo::untimed("BL1".parse().unwrap()));
    director.configured = false;
    let _ = feed(
        &mut world,
        &mut director,
        Sta {
            track,
            raceinprog: RaceInProgress::No,
            ..Default::default()
        }
        .into(),
    );
    let (commands, _) = feed(
        &mut world,
        &mut director,
        Ncn {
            ucid: ConnectionId(2),
            ..Default::default()
        }
        .into(),
    );
    assert_eq!(commands, ["/clear", "/track BL1"]);
}

#[test]
fn changes_track_first_and_waits_for_it_to_load() {
    let mut world = World::new();
    let mut director = loaded(Combo::untimed("BL1".parse().unwrap()));
    let _ = feed(
        &mut world,
        &mut director,
        Sta {
            track: "FE1X".parse().unwrap(),
            ..Default::default()
        }
        .into(),
    );
    let (commands, transitions) = feed(
        &mut world,
        &mut director,
        Ncn {
            ucid: ConnectionId(1),
            ..Default::default()
        }
        .into(),
    );
    assert_eq!(transitions, [Transition::SetupStarted]);
    assert_eq!(commands, ["/clear", "/track BL1"]);
}

#[test]
fn races_whoever_starts_the_session_and_restarts_end_one_round() {
    let track: Track = "FE1X".parse().unwrap();
    let mut world = World::new();
    let mut director = loaded(Combo::untimed(track));
    let rst = || -> Packet {
        Rst {
            track,
            ..Default::default()
        }
        .into()
    };
    let _ = feed(
        &mut world,
        &mut director,
        Sta {
            track,
            raceinprog: RaceInProgress::No,
            ..Default::default()
        }
        .into(),
    );
    let _ = feed(
        &mut world,
        &mut director,
        Ncn {
            ucid: ConnectionId(1),
            uname: "driver".into(),
            ..Default::default()
        }
        .into(),
    );
    let (_, transitions) = feed(&mut world, &mut director, rst());
    assert_eq!(transitions, [Transition::Racing]);

    // An admin's /restart mid-race: that round ends and the next one races.
    let (commands, transitions) = feed(&mut world, &mut director, rst());
    assert_eq!(
        transitions,
        [Transition::Ended { shortfall: false }, Transition::Racing]
    );
    assert!(commands.is_empty(), "{commands:?}");
    assert!(director.racing());

    // An admin's /end: back to the lobby, which is set up again.
    let (_, transitions) = feed(
        &mut world,
        &mut director,
        Sta {
            track,
            raceinprog: RaceInProgress::No,
            ..Default::default()
        }
        .into(),
    );
    assert_eq!(
        transitions,
        [
            Transition::Ended { shortfall: false },
            Transition::SetupStarted
        ]
    );
    assert_eq!(director.phase(), Phase::SettingUp);
}

/// What the service did, with tasks run to completion and their messages
/// handled.
#[derive(Debug, Default)]
struct Seen {
    commands: Vec<String>,
    chat: Vec<String>,
    timers: usize,
}

/// Pass `input` in as the host would: packets hit the world first, then
/// their events. Tasks are awaited and their messages passed back in.
async fn serve(
    world: &mut World,
    director: &mut RaceDirector,
    input: Input<RaceDirectorMsg>,
) -> Seen {
    let mut queue: std::collections::VecDeque<_> = match input {
        Input::Packet(packet) => {
            let events = world.apply(&packet);
            std::iter::once(Input::Packet(packet))
                .chain(events.into_iter().map(Input::World))
                .collect()
        },
        input => [input].into(),
    };
    let mut seen = Seen::default();
    while let Some(input) = queue.pop_front() {
        let cx = Context::new(world, Instant::now());
        let mut effects = director.update(input, &cx);
        for task in effects.take_tasks() {
            queue.push_back(Input::Msg(task.await));
        }
        for packet in effects.packets_sent() {
            match packet {
                Packet::Mst(Mst { msg, .. }) => seen.commands.push(msg.clone()),
                Packet::Mtc(mtc) => seen.chat.push(mtc.text.clone()),
                _ => {},
            }
        }
        seen.timers += effects.timers().count();
    }
    seen
}

fn lobby(track: Track) -> Input<RaceDirectorMsg> {
    Input::Packet(
        Sta {
            track,
            raceinprog: RaceInProgress::No,
            ..Default::default()
        }
        .into(),
    )
}

fn start(track: Track) -> Input<RaceDirectorMsg> {
    Input::Packet(
        Rst {
            track,
            ..Default::default()
        }
        .into(),
    )
}

fn join() -> Input<RaceDirectorMsg> {
    Input::Packet(
        Ncn {
            ucid: ConnectionId(1),
            uname: "driver".into(),
            ..Default::default()
        }
        .into(),
    )
}

fn msg(msg: DirectorMsg) -> Input<RaceDirectorMsg> {
    Input::Msg(RaceDirectorMsg(msg))
}

/// Started, in a lobby on `track` with one player, and set up.
async fn ready(director: RaceDirector, track: Track) -> (World, RaceDirector) {
    let mut world = World::new();
    let mut director = director;
    let _ = serve(&mut world, &mut director, Input::Started).await;
    let _ = serve(&mut world, &mut director, lobby(track)).await;
    let _ = serve(&mut world, &mut director, join()).await;
    assert_eq!(director.phase(), Phase::SettingUp);
    (world, director)
}

pub(super) fn fe1x() -> Track {
    "FE1X".parse().unwrap()
}

pub(super) fn bl1() -> Track {
    "BL1".parse().unwrap()
}

fn cycle() -> Cycle {
    Cycle::new([Combo::untimed(fe1x()), Combo::untimed(bl1())])
}

/// FE1X then BL1, `races` each.
fn cycle_of(races: u32) -> Cycle {
    Cycle::new([
        Combo::untimed(fe1x()).races(races),
        Combo::untimed(bl1()).races(races),
    ])
}

#[tokio::test]
async fn every_races_moves_on_after_that_many_races() {
    let director = RaceDirector::new(cycle_of(2));
    let (mut world, mut director) = ready(director, fe1x()).await;

    let _ = serve(&mut world, &mut director, start(fe1x())).await;
    let first = serve(&mut world, &mut director, lobby(fe1x())).await;
    assert!(first.chat.iter().all(|c| !c.contains("Next track")));
    assert!(!first.commands.iter().any(|c| c.starts_with("/track")));

    let _ = serve(&mut world, &mut director, start(fe1x())).await;
    let second = serve(&mut world, &mut director, lobby(fe1x())).await;
    assert!(
        second.chat.iter().any(|c| c.contains("Next track: ^7BL1")),
        "{second:?}"
    );
    assert_eq!(second.commands, ["/clear", "/track BL1"]);
    assert_eq!(director.combo().unwrap().track, bl1());
}

#[tokio::test]
async fn everyone_leaving_does_not_count_as_a_race() {
    let director = RaceDirector::new(cycle_of(1));
    let (mut world, mut director) = ready(director, fe1x()).await;

    let _ = serve(&mut world, &mut director, start(fe1x())).await;
    let left = serve(
        &mut world,
        &mut director,
        Input::Packet(
            Cnl {
                ucid: ConnectionId(1),
                ..Default::default()
            }
            .into(),
        ),
    )
    .await;
    assert_eq!(left.commands, ["/end"]);
    assert_eq!(director.combo().unwrap().track, fe1x());
}

#[tokio::test]
async fn a_single_combo_never_moves_on() {
    let director = RaceDirector::new(Combo::untimed(fe1x()));
    let (mut world, mut director) = ready(director, fe1x()).await;

    for _ in 0..3 {
        let started = serve(&mut world, &mut director, start(fe1x())).await;
        assert_eq!(started.timers, 0);
        let ended = serve(&mut world, &mut director, lobby(fe1x())).await;
        assert!(ended.chat.iter().all(|c| !c.contains("Next track")));
        assert!(!ended.commands.iter().any(|c| c.starts_with("/track")));
    }
    assert_eq!(director.combo().unwrap().track, fe1x());
}

#[tokio::test]
async fn a_closure_picks_each_combo_from_the_world() {
    // The first combo depends on who is online when the director starts.
    let rotation = |current: Option<&Combo>, world: &World| {
        let track = match (current, world.connection_count()) {
            (None, 0) => fe1x(),
            _ => bl1(),
        };
        async move { Ok::<_, RotationError>(Combo::untimed(track).races(1)) }
    };
    let director = RaceDirector::new(rotation);
    let (mut world, mut director) = ready(director, fe1x()).await;
    assert_eq!(director.combo().unwrap().track, fe1x());

    let _ = serve(&mut world, &mut director, start(fe1x())).await;
    let ended = serve(&mut world, &mut director, lobby(fe1x())).await;
    assert_eq!(ended.commands, ["/clear", "/track BL1"]);
}

#[tokio::test]
async fn a_failed_rotation_stays_on_the_current_combo() {
    let mut calls = 0;
    let rotation = move |_: Option<&Combo>, _: &World| {
        calls += 1;
        let first = calls == 1;
        async move {
            if first {
                Ok(Combo::untimed(fe1x()).races(1))
            } else {
                Err(RotationError::from("layout file is missing"))
            }
        }
    };
    let director = RaceDirector::new(rotation);
    let (mut world, mut director) = ready(director, fe1x()).await;

    let _ = serve(&mut world, &mut director, start(fe1x())).await;
    let ended = serve(&mut world, &mut director, lobby(fe1x())).await;
    assert!(
        ended
            .chat
            .iter()
            .any(|c| c.contains("Couldn't load the next track - staying on ^7FE1X")),
        "{ended:?}"
    );
    assert!(!ended.commands.iter().any(|c| c.starts_with("/track")));
    assert_eq!(director.combo().unwrap().track, fe1x());
    assert_eq!(director.phase(), Phase::SettingUp);
}

#[tokio::test]
async fn the_lobby_waits_for_a_combo_still_loading() {
    let mut world = World::new();
    let mut director = RaceDirector::new(Combo::untimed(bl1()));
    // Start, but hold on to the load instead of finishing it.
    let cx = Context::new(&world, Instant::now());
    let mut started = director.update(Input::Started, &cx);
    let load = started.take_tasks().pop().expect("the first combo loads");

    let _ = serve(&mut world, &mut director, lobby(fe1x())).await;
    let waiting = serve(&mut world, &mut director, join()).await;
    assert!(waiting.commands.is_empty(), "{waiting:?}");
    assert!(
        waiting
            .chat
            .iter()
            .any(|c| c.contains("Loading the next track"))
    );

    // Once it arrives, the lobby is set up.
    let arrived = serve(&mut world, &mut director, Input::Msg(load.await)).await;
    assert_eq!(arrived.commands, ["/clear", "/track BL1"]);
}

/// A 5-lap race on FE1X with players 1 and 2 on track.
async fn two_racing(director: RaceDirector) -> (World, RaceDirector) {
    let (mut world, mut director) = ready(director, fe1x()).await;
    let _ = serve(
        &mut world,
        &mut director,
        Input::Packet(
            Ncn {
                ucid: ConnectionId(2),
                uname: "other".into(),
                ..Default::default()
            }
            .into(),
        ),
    )
    .await;
    // A lap race: untimed sessions have no finish.
    let race = Input::Packet(
        Rst {
            track: fe1x(),
            racelaps: RaceLaps::Laps(5),
            ..Default::default()
        }
        .into(),
    );
    let _ = serve(&mut world, &mut director, race).await;
    for (ucid, plid) in [(1, 1), (2, 2)] {
        let _ = serve(
            &mut world,
            &mut director,
            Input::Packet(
                Npl {
                    ucid: ConnectionId(ucid),
                    plid: PlayerId(plid),
                    nump: 1,
                    ..Default::default()
                }
                .into(),
            ),
        )
        .await;
    }
    assert_eq!(world.live_entrants().count(), 2);
    (world, director)
}

fn fin(plid: u8) -> Input<RaceDirectorMsg> {
    Input::Packet(
        Fin {
            plid: PlayerId(plid),
            ..Default::default()
        }
        .into(),
    )
}

#[tokio::test]
async fn auto_restart_restarts_once_the_field_is_home() {
    let director = RaceDirector::new(Combo::untimed(fe1x())).auto_restart(Duration::from_secs(60));
    let (mut world, mut director) = two_racing(director).await;

    let first = serve(&mut world, &mut director, fin(1)).await;
    assert_eq!(first.timers, 1, "the grace period");
    assert_eq!(first.chat, ["^3Next race in 60s"]);
    assert!(first.commands.is_empty());

    // Everyone's home: the wait is cut short, and players are told.
    let second = serve(&mut world, &mut director, fin(2)).await;
    assert_eq!(second.chat, ["^3All drivers finished - next race in 10s"]);
    assert_eq!(second.timers, 1, "the shortened grace period");
    assert!(second.commands.is_empty());
    let up = serve(&mut world, &mut director, msg(DirectorMsg::GraceUp)).await;
    assert_eq!(up.commands, ["/restart"]);

    // LFS restarts the race; its first finisher starts a new grace period.
    let restarted = Input::Packet(
        Rst {
            track: fe1x(),
            racelaps: RaceLaps::Laps(5),
            ..Default::default()
        }
        .into(),
    );
    let _ = serve(&mut world, &mut director, restarted).await;
    // Drivers rejoin the grid.
    for (ucid, plid) in [(1, 1), (2, 2)] {
        let _ = serve(
            &mut world,
            &mut director,
            Input::Packet(
                Npl {
                    ucid: ConnectionId(ucid),
                    plid: PlayerId(plid),
                    nump: 1,
                    ..Default::default()
                }
                .into(),
            ),
        )
        .await;
    }
    let next = serve(&mut world, &mut director, fin(1)).await;
    assert_eq!(next.chat, ["^3Next race in 60s"]);
}

#[tokio::test]
async fn auto_restart_stops_waiting_when_the_grace_period_ends() {
    let director = RaceDirector::new(Combo::untimed(fe1x())).auto_restart(Duration::from_secs(60));
    let (mut world, mut director) = two_racing(director).await;

    let _ = serve(&mut world, &mut director, fin(1)).await;
    let up = serve(&mut world, &mut director, msg(DirectorMsg::GraceUp)).await;
    assert_eq!(up.commands, ["/restart"]);
}

#[tokio::test]
async fn auto_restart_counts_drivers_who_leave_as_home() {
    let director = RaceDirector::new(Combo::untimed(fe1x())).auto_restart(Duration::from_secs(60));
    let (mut world, mut director) = two_racing(director).await;

    let _ = serve(&mut world, &mut director, fin(1)).await;
    let left = serve(
        &mut world,
        &mut director,
        Input::Packet(
            Pll {
                plid: PlayerId(2),
                ..Default::default()
            }
            .into(),
        ),
    )
    .await;
    assert_eq!(left.chat, ["^3All drivers finished - next race in 10s"]);
    assert!(left.commands.is_empty());
    let up = serve(&mut world, &mut director, msg(DirectorMsg::GraceUp)).await;
    assert_eq!(up.commands, ["/restart"]);
}

/// Player 1 alone on track, in a 5-lap race on FE1X.
async fn solo_racing(director: RaceDirector) -> (World, RaceDirector) {
    let (mut world, mut director) = ready(director, fe1x()).await;
    let race = Input::Packet(
        Rst {
            track: fe1x(),
            racelaps: RaceLaps::Laps(5),
            ..Default::default()
        }
        .into(),
    );
    let _ = serve(&mut world, &mut director, race).await;
    let joined = Input::Packet(
        Npl {
            ucid: ConnectionId(1),
            plid: PlayerId(1),
            nump: 1,
            ..Default::default()
        }
        .into(),
    );
    let _ = serve(&mut world, &mut director, joined).await;
    (world, director)
}

#[tokio::test]
async fn auto_restart_alone_waits_briefly_and_says_so() {
    let director = RaceDirector::new(Combo::untimed(fe1x())).auto_restart(Duration::from_secs(60));
    let (mut world, mut director) = solo_racing(director).await;

    // One honest announcement, not "60s" followed by an instant restart.
    let home = serve(&mut world, &mut director, fin(1)).await;
    assert_eq!(home.chat, ["^3All drivers finished - next race in 10s"]);
    assert_eq!(home.timers, 1);
    assert!(home.commands.is_empty(), "{home:?}");

    let up = serve(&mut world, &mut director, msg(DirectorMsg::GraceUp)).await;
    assert_eq!(up.commands, ["/restart"]);
}

#[tokio::test]
async fn auto_restart_keeps_a_grace_period_shorter_than_the_settle() {
    let director = RaceDirector::new(Combo::untimed(fe1x())).auto_restart(Duration::from_secs(5));
    let (mut world, mut director) = solo_racing(director).await;

    let home = serve(&mut world, &mut director, fin(1)).await;
    assert_eq!(home.chat, ["^3Next race in 5s"]);
    assert!(home.commands.is_empty());
}

#[tokio::test]
async fn auto_restart_returns_to_the_lobby_when_the_next_combo_is_due() {
    let director = RaceDirector::new(cycle_of(1)).auto_restart(Duration::from_secs(60));
    let (mut world, mut director) = two_racing(director).await;

    let _ = serve(&mut world, &mut director, fin(1)).await;
    let _ = serve(&mut world, &mut director, fin(2)).await;
    let up = serve(&mut world, &mut director, msg(DirectorMsg::GraceUp)).await;
    assert_eq!(up.commands, ["/end"]);

    // The lobby that follows sets the next combo up.
    let lobby = serve(&mut world, &mut director, lobby(fe1x())).await;
    assert_eq!(lobby.commands, ["/clear", "/track BL1"]);
}

#[tokio::test]
async fn without_auto_restart_finished_races_are_left_alone() {
    let director = RaceDirector::new(Combo::untimed(fe1x()));
    let (mut world, mut director) = two_racing(director).await;

    for plid in [1, 2] {
        let seen = serve(&mut world, &mut director, fin(plid)).await;
        assert!(seen.commands.is_empty() && seen.chat.is_empty() && seen.timers == 0);
    }
}

#[tokio::test]
async fn a_late_grace_period_does_nothing_once_the_race_is_over() {
    let director = RaceDirector::new(Combo::untimed(fe1x())).auto_restart(Duration::from_secs(60));
    let (mut world, mut director) = two_racing(director).await;

    let _ = serve(&mut world, &mut director, fin(1)).await;
    // An admin ends it first.
    let _ = serve(&mut world, &mut director, lobby(fe1x())).await;
    let up = serve(&mut world, &mut director, msg(DirectorMsg::GraceUp)).await;
    assert!(up.commands.is_empty(), "{up:?}");
}

#[tokio::test]
async fn a_rotate_command_ends_the_race_and_moves_on() {
    let (mut world, mut director) = two_racing(RaceDirector::new(cycle())).await;
    assert_eq!(director.combo().map(|c| c.track), Some(fe1x()));

    let cx = Context::new(&world, Instant::now());
    let mut effects = director.command(DirectorCommand::Rotate, &cx);
    let commands: Vec<_> = effects
        .packets_sent()
        .filter_map(|p| match p {
            Packet::Mst(Mst { msg, .. }) => Some(msg.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(commands, ["/end"]);
    assert!(director.loading());

    // A second request while it loads does nothing.
    assert!(director.command(DirectorCommand::Rotate, &cx).is_empty());

    // The next combo arrives and is announced.
    let [load] = effects.take_tasks().try_into().ok().expect("one load");
    let loaded = serve(&mut world, &mut director, Input::Msg(load.await)).await;
    assert_eq!(loaded.chat, ["^3Next track: ^7BL1"]);
    assert_eq!(director.combo().map(|c| c.track), Some(bl1()));
}

#[tokio::test]
async fn the_status_names_the_combo_on_the_server_until_the_next_is_set_up() {
    let (mut world, mut director) = two_racing(RaceDirector::new(cycle())).await;
    let on_server = |director: &RaceDirector| director.status().combo.map(|c| c.track);
    assert_eq!(on_server(&director), Some(fe1x()));
    assert_eq!(director.status().next.map(|c| c.track), Some(bl1()));

    // Rotating loads BL1 at once, but FE1X is still on the server.
    let cx = Context::new(&world, Instant::now());
    let mut effects = director.command(DirectorCommand::Rotate, &cx);
    let [load] = effects.take_tasks().try_into().ok().expect("one load");
    let _ = serve(&mut world, &mut director, Input::Msg(load.await)).await;
    assert_eq!(director.combo().map(|c| c.track), Some(bl1()));
    assert_eq!(on_server(&director), Some(fe1x()));

    // The lobby asks for BL1; once it's loaded, BL1 is set up.
    let _ = serve(&mut world, &mut director, lobby(fe1x())).await;
    assert_eq!(on_server(&director), Some(fe1x()));
    let _ = serve(&mut world, &mut director, lobby(bl1())).await;
    assert_eq!(on_server(&director), Some(bl1()));
}

#[tokio::test]
async fn a_race_ended_to_rotate_does_not_count_on_the_next_combo() {
    let director = RaceDirector::new(cycle_of(2));
    let (mut world, mut director) = two_racing(director).await;
    let cx = Context::new(&world, Instant::now());
    let mut effects = director.command(DirectorCommand::Rotate, &cx);
    let [load] = effects.take_tasks().try_into().ok().expect("one load");
    // The next combo loads before LFS reports the race over.
    let _ = serve(&mut world, &mut director, Input::Msg(load.await)).await;
    let _ = serve(&mut world, &mut director, lobby(fe1x())).await;
    assert_eq!(director.races(), Some((0, 2)));
}

pub(super) fn as1() -> Track {
    "AS1".parse().unwrap()
}

#[tokio::test]
async fn next_races_that_combo_then_the_rotation_carries_on() {
    let director = RaceDirector::new(cycle_of(2));
    let (mut world, mut director) = two_racing(director).await;
    let cx = Context::new(&world, Instant::now());
    let aston = Combo::untimed(as1()).name("Aston Cadet");
    let effects = director.command(DirectorCommand::Next(aston), &cx);
    let commands: Vec<_> = effects
        .packets_sent()
        .filter_map(|p| match p {
            Packet::Mst(Mst { msg, .. }) => Some(msg.as_str()),
            Packet::Mtc(mtc) => Some(mtc.text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(commands, ["/end", "^3Next track: ^7Aston Cadet"]);
    assert_eq!(director.combo().map(|c| c.track), Some(as1()));
    // The cycle still has BL1 up after FE1X.
    assert_eq!(director.status().next.map(|c| c.track), Some(bl1()));

    // The race it ended doesn't count, and the lobby sets AS1 up.
    let _ = serve(&mut world, &mut director, lobby(fe1x())).await;
    assert_eq!(director.races(), Some((0, 2)));
    let _ = serve(&mut world, &mut director, lobby(as1())).await;
    assert_eq!(director.status().combo.map(|c| c.track), Some(as1()));
}

#[tokio::test]
async fn next_replaces_a_combo_still_loading() {
    let (world, mut director) = two_racing(RaceDirector::new(cycle())).await;
    let cx = Context::new(&world, Instant::now());
    let _ = director.command(DirectorCommand::Rotate, &cx);
    assert!(director.loading());
    let _ = director.command(DirectorCommand::Next(Combo::untimed(as1())), &cx);
    assert!(!director.loading());
    assert_eq!(director.combo().map(|c| c.track), Some(as1()));
}

#[tokio::test]
async fn each_combo_gets_its_own_number_of_races() {
    let cycle = Cycle::new([
        Combo::untimed(fe1x()).races(1),
        Combo::untimed(bl1()).races(2),
    ]);
    let (mut world, mut director) = ready(RaceDirector::new(cycle), fe1x()).await;
    assert_eq!(director.status().races, Some((0, 1)));

    // One race on FE1X, then BL1 for two.
    let _ = serve(&mut world, &mut director, start(fe1x())).await;
    let _ = serve(&mut world, &mut director, lobby(fe1x())).await;
    let _ = serve(&mut world, &mut director, lobby(bl1())).await;
    assert_eq!(director.status().combo.map(|c| c.track), Some(bl1()));
    assert_eq!(director.status().races, Some((0, 2)));

    let _ = serve(&mut world, &mut director, start(bl1())).await;
    let _ = serve(&mut world, &mut director, lobby(bl1())).await;
    assert_eq!(director.status().races, Some((1, 2)));
    assert_eq!(director.combo().map(|c| c.track), Some(bl1()));
}
