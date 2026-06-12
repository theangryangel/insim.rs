//! Host command methods for [`Game`] and their associated parameter types.

use insim::{
    core::{track::Track, vehicle::Vehicle},
    identifiers::ConnectionId,
    insim::{Mal, Plc, PlcAllowedCarsSet, RaceLaps},
};

use super::Game;
use crate::util::host_command;

/// Grid access mode for [`Game::change_grid`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridMode {
    /// Non-admins can freely join, leave, and move others on the grid.
    Open,
    /// Non-admins can only move themselves on the grid.
    Slf,
    /// Only admins can modify the grid.
    Lock,
}

impl std::fmt::Display for GridMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            GridMode::Open => "open",
            GridMode::Slf => "self",
            GridMode::Lock => "lock",
        };
        write!(f, "{s}")
    }
}

/// Month of the year, used in [`TimeSet`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Month {
    /// January
    Jan,
    /// February
    Feb,
    /// March
    Mar,
    /// April
    Apr,
    /// May
    May,
    /// June
    Jun,
    /// July
    Jul,
    /// August
    Aug,
    /// September
    Sep,
    /// October
    Oct,
    /// November
    Nov,
    /// December
    Dec,
}

impl std::fmt::Display for Month {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Month::Jan => "Jan",
            Month::Feb => "Feb",
            Month::Mar => "Mar",
            Month::Apr => "Apr",
            Month::May => "May",
            Month::Jun => "Jun",
            Month::Jul => "Jul",
            Month::Aug => "Aug",
            Month::Sep => "Sep",
            Month::Oct => "Oct",
            Month::Nov => "Nov",
            Month::Dec => "Dec",
        };
        write!(f, "{s}")
    }
}

/// Parameters for [`Game::time_set`].
///
/// All fields are optional; only the parts that are `Some` are included in the
/// `/time set` command string.
///
/// # Example
/// ```rust,ignore
/// game.time_set(TimeSet {
///     date: Some((23, Month::Jan)),
///     time: Some((16, 0)),
///     utc_offset: Some(5),
/// })
/// ```
#[derive(Debug, Default, Clone)]
pub struct TimeSet {
    /// Day and month `(1..=31, Month)`. Both must be provided together.
    pub date: Option<(u8, Month)>,
    /// Hour and minute `(0..=23, 0..=59)`.
    pub time: Option<(u8, u8)>,
    /// UTC offset in whole hours (e.g. `5` -> `utc+5`, `-3` -> `utc-3`).
    pub utc_offset: Option<i8>,
}

/// Preset time-of-day for [`Game::time_demo`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeDemoPreset {
    /// Morning preset.
    Morning,
    /// Afternoon preset.
    Afternoon,
    /// Sunset preset.
    Sunset,
}

impl std::fmt::Display for TimeDemoPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TimeDemoPreset::Morning => "morning",
            TimeDemoPreset::Afternoon => "afternoon",
            TimeDemoPreset::Sunset => "sunset",
        };
        write!(f, "{s}")
    }
}

impl Game {
    /// `/end` - finish the current race.
    pub fn end(&self) -> insim::Packet {
        host_command("/end")
    }

    /// `/clear` - remove all connections from the server.
    pub fn clear(&self) -> insim::Packet {
        host_command("/clear")
    }

    /// `/track {track}` - load a different track.
    pub fn change_track(&self, track: Track) -> insim::Packet {
        host_command(format!("/track {track}"))
    }

    /// Change race length. Maps onto `/laps`, `/hours`, or `/laps no`.
    pub fn change_laps(&self, laps: RaceLaps) -> insim::Packet {
        let cmd = match laps {
            RaceLaps::Untimed => "/laps no".to_string(),
            RaceLaps::Hours(h) => format!("/hours {h}"),
            other => format!("/laps {}", Into::<u8>::into(other)),
        };
        host_command(cmd)
    }

    /// `/wind {wind}` - set wind strength (0..=2 typically).
    pub fn change_wind(&self, wind: u8) -> insim::Packet {
        host_command(format!("/wind {wind}"))
    }

    /// `/axclear` - clear the autocross layout.
    pub fn ax_clear(&self) -> insim::Packet {
        host_command("/axclear")
    }

    /// `/axload {layout}` - load an autocross layout by name.
    pub fn ax_load(&self, layout: impl Into<String>) -> insim::Packet {
        host_command(format!("/axload {}", layout.into()))
    }

    /// `/restart` - start a race.
    pub fn restart(&self) -> insim::Packet {
        host_command("/restart")
    }

    /// `/qualify` - start qualifying.
    pub fn qualify(&self) -> insim::Packet {
        host_command("/qualify")
    }

    /// `/reinit` - full restart, kicks all connections.
    pub fn reinit(&self) -> insim::Packet {
        host_command("/reinit")
    }

    /// `/weather {weather}` - set weather/lighting.
    pub fn change_weather(&self, weather: u8) -> insim::Packet {
        host_command(format!("/weather {weather}"))
    }

    /// `/qual {minutes}` - set qualifying duration. `0` = no qualifying.
    pub fn change_qual(&self, minutes: u8) -> insim::Packet {
        host_command(format!("/qual {minutes}"))
    }

    /// `/time` - report the current in-game time status.
    pub fn time_status(&self) -> insim::Packet {
        host_command("/time")
    }

    /// `/time live` - switch to live (real-world) time.
    pub fn time_live(&self) -> insim::Packet {
        host_command("/time live")
    }

    /// `/time offset [days] [HH:MM]` - shift in-game time by an offset.
    ///
    /// `days` is a signed day count. `minutes` is a signed total-minute count
    /// formatted as `HH:MM` (e.g. `270` -> `+4:30`, `-90` -> `-1:30`).
    /// At least one argument should be `Some`.
    pub fn time_offset(&self, days: Option<i32>, minutes: Option<i32>) -> insim::Packet {
        let mut cmd = String::from("/time offset");
        if let Some(d) = days {
            let sign = if d < 0 { '-' } else { '+' };
            cmd.push_str(&format!(" {sign}{}", d.unsigned_abs()));
        }
        if let Some(m) = minutes {
            let sign = if m < 0 { '-' } else { '+' };
            let abs = m.unsigned_abs();
            cmd.push_str(&format!(" {sign}{}:{:02}", abs / 60, abs % 60));
        }
        host_command(cmd)
    }

    /// `/time set [DD Mon] [HH:MM] [utc±offset]` - set in-game time explicitly.
    ///
    /// Only the `Some` fields of `params` are appended to the command.
    pub fn time_set(&self, params: TimeSet) -> insim::Packet {
        let mut cmd = String::from("/time set");
        if let Some((day, month)) = params.date {
            cmd.push_str(&format!(" {day} {month}"));
        }
        if let Some((hour, minute)) = params.time {
            cmd.push_str(&format!(" {hour:02}:{minute:02}"));
        }
        if let Some(off) = params.utc_offset {
            let sign = if off < 0 { '-' } else { '+' };
            cmd.push_str(&format!(" utc{sign}{}", off.unsigned_abs()));
        }
        host_command(cmd)
    }

    /// `/time mul {0..=240}` - set the time multiplier (set-time mode only).
    pub fn time_multiplier(&self, factor: u8) -> insim::Packet {
        host_command(format!("/time mul {factor}"))
    }

    /// `/time demo {preset}` - activate a demo time-of-day preset.
    pub fn time_demo(&self, preset: TimeDemoPreset) -> insim::Packet {
        host_command(format!("/time demo {preset}"))
    }

    /// `/pit_all` - send every player to the pits.
    pub fn pit_all(&self) -> insim::Packet {
        host_command("/pit_all")
    }

    /// `/spec_all` - spectate all players.
    pub fn spec_all(&self) -> insim::Packet {
        host_command("/spec_all")
    }

    /// `/grid open|self|lock` - set who can modify the grid in the game setup screen.
    pub fn change_grid(&self, mode: GridMode) -> insim::Packet {
        host_command(format!("/grid {mode}"))
    }

    /// `/grid real yes` / `/grid real no` - allow or disallow real players joining in-game or on the setup screen.
    pub fn change_grid_real(&self, allow: bool) -> insim::Packet {
        host_command(if allow {
            "/grid real yes"
        } else {
            "/grid real no"
        })
    }

    /// `/grid ai yes` / `/grid ai no` - allow or disallow AI players joining in-game or on the setup screen.
    pub fn change_grid_ai(&self, allow: bool) -> insim::Packet {
        host_command(if allow { "/grid ai yes" } else { "/grid ai no" })
    }

    /// `/flood yes` / `/flood no` - switch floodlights on or off.
    pub fn change_flood(&self, on: bool) -> insim::Packet {
        host_command(if on { "/flood yes" } else { "/flood no" })
    }

    /// Apply vehicle restrictions server-wide (ucid = `ConnectionId::ALL`).
    ///
    /// Sends a `Plc` packet for standard cars and a `Mal` packet for mods.
    /// Pass an empty slice to clear all restrictions.
    pub fn restrict_vehicles(&self, vehicles: &[Vehicle]) -> Vec<insim::Packet> {
        let mut mal = Mal::default();
        let cars = if vehicles.is_empty() {
            PlcAllowedCarsSet::all()
        } else {
            let mut cars = PlcAllowedCarsSet::default();
            for v in vehicles {
                match v {
                    Vehicle::Mod(_) => {
                        let _ = mal.insert(*v);
                    },
                    _ => {
                        let _ = cars.insert(*v);
                    },
                }
            }
            cars
        };
        vec![
            insim::Packet::from(Plc {
                cars,
                ucid: ConnectionId::ALL,
                ..Plc::default()
            }),
            insim::Packet::from(mal),
        ]
    }
}
