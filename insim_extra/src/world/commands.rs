//! Stateless host-command builders for [`World`].
//!
//! Each method formats an LFS admin command (or the packets implementing one)
//! and returns it for the caller to send; none of them read or mutate world
//! state. They are split out from the state-mirror logic in the parent module
//! purely for readability.

use insim::{
    core::{track::Track, vehicle::Vehicle},
    identifiers::ConnectionId,
    insim::{Mal, Plc, PlcAllowedCarsSet, RaceLaps},
};

use super::{GridMode, TimeDemoPreset, TimeSet, World};
use crate::util::host_command;

impl World {
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

    /// `/grid real yes` / `/grid real no` - allow or disallow real players joining.
    pub fn change_grid_real(&self, allow: bool) -> insim::Packet {
        host_command(if allow {
            "/grid real yes"
        } else {
            "/grid real no"
        })
    }

    /// `/grid ai yes` / `/grid ai no` - allow or disallow AI players joining.
    pub fn change_grid_ai(&self, allow: bool) -> insim::Packet {
        host_command(if allow { "/grid ai yes" } else { "/grid ai no" })
    }

    /// `/flood yes` / `/flood no` - switch floodlights on or off.
    pub fn change_flood(&self, on: bool) -> insim::Packet {
        host_command(if on { "/flood yes" } else { "/flood no" })
    }

    /// Apply vehicle restrictions server-wide (ucid = `ConnectionId::ALL`).
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

    /// Returns an `/unban` packet.
    pub fn unban(&self, uname: impl Into<String>) -> insim::Packet {
        host_command(format!("/unban {}", uname.into()))
    }

    /// Returns the packets needed to set and display a Race Control Message for
    /// every connection (`/rcm` + `/rcm_all`). Send both packets.
    ///
    /// For a single connection use
    /// [`ConnectionInfo::send_rcm`](crate::world::ConnectionInfo::send_rcm).
    pub fn broadcast_rcm(&self, message: &str) -> Vec<insim::Packet> {
        vec![
            host_command(format!("/rcm {message}")),
            host_command("/rcm_all"),
        ]
    }

    /// Returns a packet to clear the Race Control Message for every connection
    /// (`/rcc_all`).
    ///
    /// For a single connection use
    /// [`ConnectionInfo::clear_rcm`](crate::world::ConnectionInfo::clear_rcm).
    pub fn clear_rcm_all(&self) -> insim::Packet {
        host_command("/rcc_all")
    }
}
