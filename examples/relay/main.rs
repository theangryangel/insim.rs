extern crate insim;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tracing::{debug, error, info};
use tracing_subscriber;

fn setup() {
    // setup tracing with some defaults if nothing is set
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}

#[derive(Default, Clone)]
struct State {
    pub counter: Arc<AtomicUsize>,
}

#[tokio::main]
pub async fn main() {
    setup();

    let mut client = insim::framework::Config::default()
        .relay()
        .build_with_state(State::default());

    client.on_connect(|ctx| {
        info!("🎉🎉🎉🎉🎉🎉 we've connected!");
        ctx.state.counter.store(0, Ordering::Relaxed);

        ctx.send(insim::protocol::relay::HostListRequest::default().into());

        ctx.send(
            insim::protocol::relay::HostSelect {
                hname: "Nubbins AU Demo".into(),
                ..Default::default()
            }
            .into(),
        );

        ctx.send(
            insim::protocol::insim::Tiny {
                reqi: 0,
                subtype: insim::protocol::insim::TinyType::Npl,
            }
            .into(),
        );
    });

    client.on_relay_host_list(|_ctx, list| {
        for i in list.hinfo.iter() {
            if i.numconns > 1 {
                tracing::info!(
                    "{} ({} / {}) {} {:?} {}",
                    insim::string::colours::to_ansi(i.hname.to_string()),
                    i.hname.to_string(),
                    i.numconns,
                    i.track.to_string(),
                    i.track.track_info(),
                    i.track.is_open_world(),
                );
            }
        }
    });

    client.on_tiny(|_ctx, tiny| {
        info!("⭐⭐⭐⭐⭐⭐: {:?}", tiny);
    });

    client.on_any(|ctx, packet| {
        let count = ctx.state.counter.fetch_add(1, Ordering::Relaxed);

        debug!("{:?} #={}", packet, count);
    });

    client.on_multi_car_info(|_ctx, data| {
        for i in data.info.iter() {
            info!(
                "{:?} {:?}mph, {:?}kph, {:?}mps, {:?}raw",
                i.plid,
                i.speed_as_mph(),
                i.speed_as_kmph(),
                i.speed_as_mps(),
                i.speed
            );
        }
    });

    client.on_new_player(|_ctx, data| {
        info!(
            "New player! {}, cname={} ismod={}",
            data.pname.to_string(),
            data.cname.to_string(),
            data.cname.is_mod()
        );
    });

    let res = client.run().await;

    match res {
        Ok(()) => {
            info!("Clean shutdown");
        }
        Err(e) => {
            error!("Unclean shutdown: {:?}", e);
        }
    }
}
