//! Twenty Second League
use std::{
    fmt::Debug,
    fs,
    time::{Duration, Instant},
};

use eyre::Context as _;
use insim::{
    identifiers::ConnectionId,
    insim::{Mso, Mst},
    Packet,
};
use kitcar::{Framework, Plugin, PluginContext};
use tokio::time::interval;
use tracing::info;

use crate::{combo::ComboCollection, components::countdown};

mod combo;
mod components;
mod cpa;
mod dictator;
mod league;

/// Config
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    /// Insim IName
    pub iname: String,
    /// Server address
    pub addr: String,
    /// admin password
    pub admin: Option<String>,
    /// Combinations
    pub combo: combo::ComboCollection,
    /// tick rate (hz)
    pub tick_rate: Option<u64>,
    /// Warmup Duration
    pub warmup_duration_minutes: u64,
    /// Game duration
    pub game_duration_minutes: u64,
    /// Rounds
    pub total_rounds: u32,
}

struct AnnouncerPlugin;

#[async_trait::async_trait]
impl<S> Plugin<S> for AnnouncerPlugin
where
    S: Send + Sync + Clone + Debug + 'static,
{
    async fn run(mut self: Box<Self>, _ctx: PluginContext<S>) -> Result<(), ()> {
        info!("Announcer Plugin started and finished its job!");
        Ok(())
    }
}

pub(crate) struct CountdownView;

async fn chatterbox<S: Send + Sync + Clone + Debug + 'static>(mut ctx: PluginContext<S>) -> Result<(), ()> {
    info!("Chatterbox plugin started!");
    let mut packets = ctx.subscribe_to_packets();

    let mut timer = interval(Duration::from_secs(1));

    let target = Instant::now() + Duration::from_secs(100);

    loop {
        tokio::select! {

            _ = timer.tick() => {

                let connections = ctx.get_connections().await;

                for ucid in connections.keys() {
                    ctx.set_ui::<CountdownView>(*ucid, countdown(target - Instant::now())).await;
                }
            },

            packet = packets.recv() => {
                println!("{:?}", packet);
                // FIXME
                let packet = packet.unwrap();
                match packet {
                    Packet::Ncn(ncn) => {
                        let conn = ctx.get_connection(ncn.ucid).await;

                        println!("conn = {:?}", conn);

                        ctx.send_packet(Mst {
                            msg: format!("A big welcome to {:?} ({:?})", ncn.pname, ncn.uname),
                            ..Default::default()
                        }).await;
                    },
                    _ => {},
                }


            }

        }
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    let config: Config = serde_norway::from_str(
        &fs::read_to_string("config.yaml").wrap_err("could not read config.yaml")?,
    )
    .wrap_err("Could not parse config.yaml")?;

    let framework = Framework::new()
        .with_plugin(cpa::cpa)
        .with_plugin(dictator::dictator)
        .with_plugin(AnnouncerPlugin)
        .with_plugin(league::League::default())
        .with_plugin(chatterbox);
    // .with_chat_command("!test", |_ctx: TaskContext<()>| {
    //      info!("Woot!");
    // });

    info!("Framework built. Running application...");

    let net = insim::tcp(config.addr.clone())
        .isi_iname(config.iname.clone())
        .isi_admin_password(config.admin.clone())
        .isi_prefix('!')
        .set_non_blocking(true)
        .connect_async()
        .await?;

    framework.run(config, net).await?;

    Ok(())
}
