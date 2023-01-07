use std::sync::Arc;

use crate::config::definition::Server as ServerConfig;
use crate::state::chat::Chat;
use crate::state::{Connection, Game, Notifiers};
use miette::Result;
use tokio::sync::mpsc;
use tokio::{sync::oneshot, task::JoinHandle};

use super::actor::{InsimActor, InsimActorMessage};

#[derive(Clone)]
pub(crate) struct InsimHandle {
    pub(crate) tx: mpsc::Sender<InsimActorMessage>,

    pub(crate) handle: Arc<JoinHandle<Result<()>>>, // FIXME, can we get rid of the Arc?
}

impl InsimHandle {
    pub(crate) fn new(config: &ServerConfig) -> Self {
        let (tx, rx) = mpsc::channel(8);
        let actor = InsimActor::new(rx, config.as_insim_config().unwrap());
        let handle = tokio::spawn(super::actor::run(actor));

        Self {
            tx,
            handle: Arc::new(handle),
        }
    }

    #[allow(dead_code)]
    async fn shutdown(&self) {
        // FIXME this should request a clean shutdown first
        self.handle.abort()
    }

    async fn request_connections(&self, players_only: bool, flipped: bool) -> Vec<Connection> {
        let (send, recv) = oneshot::channel();
        let msg = InsimActorMessage::Connections {
            players_only,
            flipped,
            respond_to: send,
        };

        // Ignore send errors. If this send fails, so does the
        // recv.await below. There's no reason to check for the
        // same failure twice.
        let _ = self.tx.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }

    pub(crate) async fn get_players(&self) -> Vec<Connection> {
        self.request_connections(true, true).await
    }

    pub(crate) async fn get_connections(&self) -> Vec<Connection> {
        self.request_connections(false, true).await
    }

    pub(crate) async fn get_game(&self) -> Game {
        let (send, recv) = oneshot::channel();
        let msg = InsimActorMessage::Game { respond_to: send };

        // Ignore send errors. If this send fails, so does the
        // recv.await below. There's no reason to check for the
        // same failure twice.
        let _ = self.tx.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }

    pub(crate) async fn get_chat(&self) -> Vec<Chat> {
        let (send, recv) = oneshot::channel();
        let msg = InsimActorMessage::Chat { respond_to: send };

        // Ignore send errors. If this send fails, so does the
        // recv.await below. There's no reason to check for the
        // same failure twice.
        let _ = self.tx.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }

    pub(crate) async fn get_notifiers(&self) -> Notifiers {
        let (send, recv) = oneshot::channel();
        let msg = InsimActorMessage::Notifiers { respond_to: send };

        // Ignore send errors. If this send fails, so does the
        // recv.await below. There's no reason to check for the
        // same failure twice.
        let _ = self.tx.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }
}

impl From<&ServerConfig> for InsimHandle {
    fn from(value: &ServerConfig) -> Self {
        Self::new(value)
    }
}
