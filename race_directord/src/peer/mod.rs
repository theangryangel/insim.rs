pub mod message;

use std::{collections::HashMap, time::Duration, marker::PhantomData};
use tokio::{sync::{mpsc, broadcast}, task::JoinHandle, time::sleep};
use insim::{connection::{Connection, Event}, codec::Frame, error::Error};

pub struct Manager {

}

pub struct Peer<T: Frame + 'static> {
    sender: mpsc::Sender<message::Message<T>>,
}

impl<T: Frame + 'static> Peer<T> {
    pub fn new(client: Connection<T>) -> Self {
        let (sender, _receiver) = mpsc::channel(8);
        tokio::spawn(run_peer(client));

        Self { sender }
    }
}

async fn run_peer<T: Frame>(mut client: Connection<T>) {
    let mut delay: u64 = 1;
    let (broadcast_tx, _) = broadcast::channel::<Event<T>>(32);

    loop {
        match client.poll().await {
            Ok(Event::Connected(id)) => {
                delay = 1;
                broadcast_tx.send(Event::Connected(id));
            },
            Ok(e) => {
                tracing::info!("{:?}", e);
                broadcast_tx.send(e);
            },
            Err(Error::Shutdown) => {
                break;
            },
            Err(Error::IncompatibleVersion(expected)) => {
                tracing::error!("{:?}", expected);
                break;
            },
            Err(Error::Timeout(_)) => {
                delay = delay.wrapping_mul(2);
                sleep(Duration::from_secs(delay)).await;
            },
            Err(e) => {
                tracing::error!("{:?}", e);
                sleep(Duration::from_secs(5)).await;
            } 
        }
    }
}
