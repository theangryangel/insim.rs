//! [`Combo`]: how the server is set up for a race.

use insim::{
    core::{object::ObjectInfo, track::Track, vehicle::Vehicle},
    insim::RaceLaps,
};

use crate::commands::time::TimeOfDay;

/// The layout a [`Combo`] loads.
#[derive(Debug, Clone)]
pub enum Layout {
    /// A layout saved on the server, loaded with `/axload`.
    Axload(String),
    /// Objects sent over InSim, such as from a `.lyt` file.
    Objects(Vec<ObjectInfo>),
}

/// How the grid is ordered at the start of a race (`/start`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum StartOrder {
    /// The order drivers joined in.
    Fixed,
    /// The last race's finishing order.
    #[default]
    Finish,
    /// The last race's finishing order, reversed.
    Reverse,
    /// Shuffled.
    Random,
}

impl StartOrder {
    /// The host command that sets it, such as `/start finish`.
    pub fn to_command(self) -> String {
        let order = match self {
            Self::Fixed => "fixed",
            Self::Finish => "finish",
            Self::Reverse => "reverse",
            Self::Random => "random",
        };
        format!("/start {order}")
    }
}

/// How the server is set up before each race.
#[derive(Debug, Clone)]
pub struct Combo {
    /// The track. Only changed if not already loaded.
    pub track: Track,
    /// The layout, or `None` to clear it.
    pub layout: Option<Layout>,
    /// Race length.
    pub laps: RaceLaps,
    /// Wind strength (0 for none).
    pub wind: u8,
    /// The grid order. Always sent, as LFS keeps it across track changes.
    pub start: StartOrder,
    /// Whether drivers must make a pit stop (`/mustpit`). Ignored where there's
    /// no room for one: untimed, practice, or a single lap. See
    /// [`pit_stop_required`](Self::pit_stop_required). Always sent, as LFS
    /// keeps it across track changes.
    pub must_pit: bool,
    /// Vehicles allowed for everyone; empty allows them all, mods included.
    /// Mods follow the list: listed mods only, or none (`/allowmods no`) if it
    /// names only standard cars. Always sent, as LFS keeps the last
    /// restriction across track changes. To narrow it per
    /// player, answer LFS's join requests (`IsiFlags::REQ_JOIN`) in a service.
    pub vehicles: Vec<Vehicle>,
    /// The time of day: one of these, picked at random each time this combo
    /// comes up. Empty leaves the server's clock alone; it keeps its settings
    /// across track changes. Repeat one to make it likelier.
    pub time: Vec<TimeOfDay>,
    /// What players are shown, such as "Blackwood GP", in place of the track
    /// code. See [`display_name`](Self::display_name).
    pub name: Option<String>,
    /// Races on this combo before the next comes up. A race counts when its
    /// session ends (finished, `/end` or `/restart`), not when everyone
    /// leaves. `None` stays until a
    /// [`DirectorCommand`](super::DirectorCommand) moves on, or for good with
    /// a single combo.
    pub races: Option<u32>,
}

impl Combo {
    /// An untimed session on `track`: no wind, no layout, every car allowed,
    /// the grid in finishing order, no pit stop required, and the time of day
    /// untouched.
    pub fn untimed(track: Track) -> Self {
        Self {
            track,
            layout: None,
            laps: RaceLaps::Untimed,
            wind: 0,
            start: StartOrder::default(),
            must_pit: false,
            vehicles: Vec::new(),
            time: Vec::new(),
            name: None,
            races: None,
        }
    }
    /// Move on after this many races. For a combo timed rather than counted,
    /// race in hours ([`RaceLaps::Hours`]), once.
    pub fn races(mut self, races: u32) -> Self {
        self.races = Some(races);
        self
    }
    /// Show players this name in place of the track code.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    /// The name players see: [`name`](Self::name), or the track code without
    /// one.
    pub fn display_name(&self) -> String {
        self.name.clone().unwrap_or_else(|| self.track.to_string())
    }
    /// Always this time of day.
    pub fn time(self, time: TimeOfDay) -> Self {
        self.times([time])
    }
    /// One of these times of day, at random each time this combo comes up.
    pub fn times(mut self, times: impl IntoIterator<Item = TimeOfDay>) -> Self {
        self.time = times.into_iter().collect();
        self
    }
    /// Load this layout, or clear it with `None`.
    pub fn layout(mut self, layout: impl Into<Option<Layout>>) -> Self {
        self.layout = layout.into();
        self
    }
    /// Allow only these vehicles, for everyone. Empty allows all.
    pub fn vehicles(mut self, vehicles: impl IntoIterator<Item = Vehicle>) -> Self {
        self.vehicles = vehicles.into_iter().collect();
        self
    }
    /// The grid order.
    pub fn start(mut self, start: StartOrder) -> Self {
        self.start = start;
        self
    }
    /// Require a pit stop, or not.
    pub fn must_pit(mut self, must_pit: bool) -> Self {
        self.must_pit = must_pit;
        self
    }
    /// Whether a pit stop is actually required: only if asked for, and only in
    /// a race of two or more laps, or one timed in hours.
    pub fn pit_stop_required(&self) -> bool {
        self.must_pit
            && match self.laps {
                RaceLaps::Laps(laps) => laps > 1,
                RaceLaps::Hours(_) => true,
                // Untimed, practice, and anything newer: no race to pit in.
                _ => false,
            }
    }
    /// Race length.
    pub fn laps(mut self, laps: RaceLaps) -> Self {
        self.laps = laps;
        self
    }
}
