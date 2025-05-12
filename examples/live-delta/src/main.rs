//! Live deltas
use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use clap::Parser;
use insim::{
    core::point::Point,
    identifiers::{ClickId, PlayerId, RequestId},
    insim::{Btn, BtnStyle, LapTimingInfo, TinyType},
    Packet, Result, WithRequestId,
};

#[derive(Clone, Debug)]
pub struct RefPoint {
    pub position: Point<i32>,
    pub time: Instant,
}

#[derive(Debug)]
pub struct DeltaTracker {
    reference_lap: Option<Vec<RefPoint>>,
    current_lap: Vec<RefPoint>,
}

impl DeltaTracker {
    pub fn clear(&mut self) {
        self.reference_lap = None;
        self.current_lap.clear();
    }

    pub fn new() -> Self {
        Self {
            reference_lap: None,
            current_lap: Vec::new(),
        }
    }

    pub fn record(&mut self, pos: Point<i32>, time: Instant) {
        let last = self.current_lap.last().map(|p| &p.position);
        // did we move at least 1m? if not, do nothing
        // could probably remove this with the speed check tbh.
        if last.map_or(true, |last_pos| last_pos.distance(&pos) >= 1.0) {
            self.current_lap.push(RefPoint {
                position: pos,
                time,
            });
        }
    }

    /// We just did a lap
    pub fn lap(&mut self) {
        self.reference_lap = Some(self.current_lap.clone());
        self.current_lap.clear();
    }

    /// Delta in seconds
    pub fn delta(&self, current_pos: Point<i32>) -> Option<f32> {
        let reference_lap = self.reference_lap.as_ref()?;
        let lap_start = self.current_lap.first()?;

        let current_time = Instant::now();

        if reference_lap.len() < 2 {
            return None;
        }

        // get delta against our reference lap (i.e. the last lap, since we don't necesarily have a
        // pth file to use)
        //
        // The missile knows where it is at all times...
        // https://www.youtube.com/watch?v=bZe5J8SVCYQ

        let mut best_dist = f32::MAX;
        let mut best_index = 0;

        for i in 0..reference_lap.len() - 1 {
            let p1 = reference_lap[i].position;
            let p2 = reference_lap[i + 1].position;

            let proj = current_pos.project_onto_segment(&p1, &p2);
            let dist = current_pos.distance(&proj);

            if dist < best_dist {
                best_dist = dist;
                best_index = i;
            }
        }

        let r1 = &reference_lap[best_index];
        let r2 = &reference_lap[best_index + 1];

        let t = current_pos.project_onto_segment_ratio(&r1.position, &r2.position);

        let segment_duration = r2.time.duration_since(r1.time);
        let ref_time = r1.time + segment_duration.mul_f32(t);

        let current_duration = current_time.duration_since(lap_start.time).as_secs_f32();
        let ref_duration = ref_time.duration_since(reference_lap[0].time).as_secs_f32();

        Some(current_duration - ref_duration)
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// host:port of LFS to connect to
    addr: SocketAddr,
}

pub fn main() -> Result<()> {
    // Setup tracing_subcriber with some sane defaults
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Parse our command line arguments, using clap
    let cli = Cli::parse();

    let mut builder = insim::tcp(cli.addr);

    // set our IsiFlags
    builder = builder
        .isi_flag_local(true)
        .isi_flag_mci(true)
        .isi_iname(Some("insim.rs/btns".to_string()))
        .isi_interval(Duration::from_millis(100));

    // Establish a connection
    let mut connection = builder.connect_blocking()?;
    tracing::info!("Connected!");

    connection.write(TinyType::Npl.with_request_id(1))?;
    connection.write(TinyType::Sst.with_request_id(2))?;
    connection.write(TinyType::Rst.with_request_id(3))?;

    // FIXME: tidy this all up.
    // We need a state machine
    let mut plid: Option<PlayerId> = None;
    let mut pos: Point<i32> = Point::default();

    let mut deltas = DeltaTracker::new();
    let mut recording = false;

    while let Ok(packet) = connection.read() {
        match packet {
            Packet::Rst(rst) => {
                // FIXME
                if matches!(
                    rst.timing,
                    LapTimingInfo::Standard(_) | LapTimingInfo::Custom(_)
                ) {
                    if !recording {
                        deltas.clear();
                        recording = true;
                    }
                } else {
                    println!("{:?}", rst);
                    recording = false;
                    deltas.clear();
                }
            },

            Packet::Npl(npl) => {
                if !npl.ptype.is_remote() && !npl.ptype.is_ai() {
                    plid = Some(npl.plid);
                    tracing::info!("Woot! local player joined! {:?}", plid);

                    connection.write(TinyType::Rst.with_request_id(3))?;
                }
            },

            Packet::Plp(plp) => {
                if recording && plid.is_some_and(|id| id == plp.plid) {
                    recording = false;
                    deltas.clear();
                }
            },

            Packet::Pll(pll) => {
                if plid.map_or(false, |p| p == pll.plid) {
                    plid = None;

                    tracing::info!("Local player left!");
                    recording = false;
                    deltas.clear();
                }
            },

            Packet::Mci(mci) => {
                if !recording || plid.is_none() {
                    continue;
                }

                let info = mci.info.iter().find(|i| i.plid == plid.unwrap());
                if let Some(i) = info {
                    // deal with the fact that we cant precisely know when the race starts with the
                    // RST, so we should record only when we've started moving.
                    if i.speed.to_meters_per_sec() < 1.0 {
                        continue;
                    }
                    pos.x = i.xyz.x / 65536;
                    pos.y = i.xyz.y / 65536;
                    pos.z = i.xyz.z / 65536;
                    deltas.record(pos.clone(), Instant::now());
                }
            },

            Packet::Lap(lap) => {
                if recording && plid.is_some_and(|id| id == lap.plid) {
                    deltas.lap();
                }
            },

            Packet::Fin(fin) => {
                if plid.is_some_and(|id| id == fin.plid) {
                    recording = false;
                    //deltas.clear();
                }
            },

            _ => {},
        }

        // FIXME: see state machine comment above
        if plid.is_some() {
            let bstyle = if recording {
                BtnStyle::default().red()
            } else {
                BtnStyle::default().light_grey()
            }
            .dark();

            let text = if let Some(delta) = deltas.delta(pos) {
                format!("{:.2}", delta)
            } else {
                "No ref lap".to_string()
            };

            let btn = Btn {
                reqi: RequestId(1),
                clickid: ClickId(0),
                bstyle,
                l: 130,
                t: 10,
                w: 25,
                h: 10,

                text,
                ..Default::default()
            };

            connection.write(btn)?;
        }
    }

    Ok(())
}
