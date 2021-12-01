extern crate insim;
use std::{
    io::Write,
    sync::atomic::{AtomicUsize, Ordering},
};
use tracing::{error, info};
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

// Example handler usage that counts the number of packets received and resets on each
// reconnection.
struct Party {}

#[allow(unused)]
impl insim::framework::EventHandler for Party {
    fn on_connect(&self, ctx: &insim::framework::Client) {
        info!("🎉🎉🎉🎉🎉🎉 we're connected!");
    }

    fn on_disconnect(&self, client: &insim::framework::Client) {
        info!("💩💩💩💩💩💩 we've lost connection!");
    }

    fn on_tiny(&self, client: &insim::framework::Client, data: &insim::protocol::insim::Tiny) {
        info!("✨✨✨✨✨✨ {:?}", data);
    }

    fn on_npl(&self, client: &insim::framework::Client, data: &insim::protocol::insim::Npl) {
        info!("{:?}", data);
    }

    fn on_mci(
        &self,
        client: &insim::framework::Client,
        _data: &insim::protocol::insim::MultiCarInfo,
    ) {
    }

    fn on_message(
        &self,
        client: &insim::framework::Client,
        data: &insim::protocol::insim::MessageOut,
    ) {
        info!("{:?}", data.msg);
    }
}

// Example handler usage that counts the number of packets received and resets on each
// reconnection.
struct Counter {
    i: AtomicUsize,
}

impl insim::framework::EventHandler for Counter {
    fn on_connect(&self, ctx: &insim::framework::Client) {
        // on connection reset our AtomicUsize back to 0.
        self.i.store(0, Ordering::Relaxed);

        ctx.send(insim::protocol::relay::HostListRequest::default().into());

        ctx.send(
            insim::protocol::relay::HostSelect {
                hname: "^1(^3FM^1) ^4GTi Thursday".into(),
                ..Default::default()
            }
            .into(),
        );
    }

    #[allow(unused)]
    fn on_raw(&self, ctx: &insim::framework::Client, data: &insim::protocol::Packet) {
        self.i.fetch_add(1, Ordering::Relaxed);

        match data {
            insim::protocol::Packet::RelayHostList(hostlist) => {
                for i in hostlist.hinfo.iter() {
                    if i.numconns > 1 {
                        info!("{:?}", i);
                    }
                }
            }
            _ => {
                //info!("got {:?} #={:?}", data, self.i);
            }
        }

        //* Auto shutdown on 5th packet.
        // if self.i.load(Ordering::Relaxed) > 5 {
        //     ctx.shutdown();
        // }
    }
}

#[tokio::main]
pub async fn main() {
    setup();

    let client = insim::framework::Config::default()
        .relay()
        .using_event_handler(Counter {
            i: AtomicUsize::new(0),
        })
        .using_event_handler(Party {})
        .build();

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
