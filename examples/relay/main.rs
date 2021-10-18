extern crate insim;
use std::sync::atomic::{AtomicUsize, Ordering};
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
struct Counter {
    i: AtomicUsize,
}

impl insim::client::EventHandler for Counter {
    fn on_connect(&self, ctx: insim::client::Ctx) {
        // on connection reset our AtomicUsize back to 0.
        self.i.store(0, Ordering::Relaxed);

        info!("CONNECTED! {:?}", self.i);

        // TODO: we need a better way to create packets. Impl default probably?
        // Or maybe some kind of factory?
        //

        let hlr = insim::protocol::Packet::RelayHostListRequest(
            insim::protocol::relay::HostListRequest { reqi: 0 },
        );

        ctx.send(hlr);

        let hs = insim::protocol::Packet::RelayHostSelect(insim::protocol::relay::HostSelect {
            reqi: 0,

            hname: "^0[^7MR^0c] ^7Beginner ^0BMW".into(),
            admin: "".into(),
            spec: "".into(),
        });

        ctx.send(hs);
    }

    #[allow(unused)]
    fn on_raw(&self, ctx: insim::client::Ctx, data: insim::protocol::Packet) {
        self.i.fetch_add(1, Ordering::Relaxed);

        /*
         * Auto shutdown on 5th packet.
        if self.i.load(Ordering::Relaxed) > 5 {
            ctx.shutdown();
        }
        */
        info!("got {:?} #={:?}", data, self.i);
    }

    fn on_disconnect(&self) {
        info!("DISCONNECTED!");
    }
}

use std::sync::Arc;

#[tokio::main]
pub async fn main() {
    setup();

    let client = insim::client::Config::default()
        .relay()
        // TODO: Do we even care if this is an Arc really?
        .event_handler(Arc::new(Counter {
            i: AtomicUsize::new(0),
        }))
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
