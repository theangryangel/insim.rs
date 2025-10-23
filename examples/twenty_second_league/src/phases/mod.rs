//! Phases
mod game;
mod idle;
mod lobby;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub(crate) enum Transition {
    Idle,
    Lobby,
    Game,
    Shutdown,
}

pub(crate) use game::PhaseGame;
pub(crate) use idle::PhaseIdle;
pub(crate) use lobby::PhaseLobby;
