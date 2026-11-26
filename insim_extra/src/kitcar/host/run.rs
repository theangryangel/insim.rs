//! The host's loop, and finding who an input is for.

use tokio::sync::mpsc;

use super::{Command, Directing, Fired, Loop, Running, Slot, Target};
use crate::{
    kitcar::{error::HostError, service::Input},
    timers,
    ui::Outbox,
    world::World,
};

impl Loop {
    pub(super) async fn run(
        mut self,
        mut commands: mpsc::Receiver<Command>,
        mut results: mpsc::UnboundedReceiver<Fired>,
    ) -> Result<(), HostError> {
        // What was there before we connected. The world reports Ready once
        // every reply is in.
        for request in World::startup() {
            if let Err(error) = self.connection.send(request).await {
                tracing::warn!(%error, "failed to send a startup request");
            }
        }
        loop {
            let wake = self.render_at();
            tokio::select! {
                biased;
                command = commands.recv() => match command {
                    Some(Command::Start { mut service, done, reply }) => {
                        let id = self.next_id;
                        tracing::info!(service = service.name(), "service started");
                        self.next_id += 1;
                        let results = self.results_tx.clone();
                        service.set_outbox(Outbox::new(move |delivery| {
                            results.send(Fired::Ui { owner: id, delivery }).is_ok()
                        }));
                        self.services.push(Running {
                            slot: Slot::new(id, &self.results_tx),
                            service,
                            done: Some(done),
                        });
                        let _ = reply.send(id);
                        self.deliver(Target::Service(self.services.len() - 1), Input::Started).await;
                    },
                    Some(Command::Stop { id, reply }) => {
                        if let Some(index) = self.index_of(id) {
                            self.stop(index).await;
                        }
                        let _ = reply.send(());
                    },
                    Some(Command::Direct { director, reply }) => {
                        tracing::info!("director started");
                        self.stop_director().await;
                        let id = self.next_id;
                        self.next_id += 1;
                        self.director = Some(Directing {
                            slot: Slot::new(id, &self.results_tx),
                            director: *director,
                        });
                        let _ = reply.send(());
                        self.deliver(Target::Director, Input::Started).await;
                    },
                    Some(Command::StopDirector { reply }) => {
                        self.stop_director().await;
                        let _ = reply.send(());
                    },
                    Some(Command::Shutdown { reply }) => {
                        self.stop_all().await;
                        let _ = reply.send(self.connection.shutdown().await);
                        return Ok(());
                    },
                    // Every `Host` handle is gone, so nothing can start or stop services.
                    None => {
                        self.stop_all().await;
                        let _ = self.connection.shutdown().await;
                        return Ok(());
                    },
                },
                item = self.receiver.recv() => match item {
                    Some(Ok(packet)) => self.on_packet(packet).await,
                    Some(Err(error @ insim::Error::IncompleteDecode { .. })) => {
                        tracing::warn!(%error, "skipping a partially decoded packet");
                    },
                    Some(Err(error)) => {
                        self.stop_all().await;
                        return Err(error.into());
                    },
                    None => {
                        tracing::info!("connection closed");
                        self.stop_all().await;
                        return Ok(());
                    },
                },
                Some(fired) = results.recv() => match fired {
                    Fired::Job { owner, job, msg } => {
                        if let Some(target) = self.target_of(owner).filter(|&t| self.live(t))
                            && self.slot_mut(target).timers.accept(job)
                        {
                            self.deliver(target, Input::Msg(msg)).await;
                        }
                    },
                    Fired::Ui { owner, delivery } => {
                        if let Some(index) = self.index_of(owner)
                            && self.live(Target::Service(index))
                        {
                            self.deliver_ui(index, delivery).await;
                        }
                    },
                },
                // Time for a frame; the render follows below.
                () = tokio::time::sleep_until(
                    tokio::time::Instant::from_std(wake.unwrap_or_else(timers::now)),
                ), if wake.is_some() => {},
            }
            self.remove_stopped().await;
            if self.render_at().is_some_and(|at| timers::now() >= at) {
                self.render().await;
            }
        }
    }

    // By index, so no borrow of `services` is held across an await.
    pub(super) fn index_of(&self, id: u64) -> Option<usize> {
        self.services.iter().position(|m| m.slot.id == id)
    }

    pub(super) fn target_of(&self, id: u64) -> Option<Target> {
        if self.director.as_ref().is_some_and(|d| d.slot.id == id) {
            return Some(Target::Director);
        }
        self.index_of(id).map(Target::Service)
    }

    /// The director first, then every service in start order.
    pub(super) fn targets(&self) -> Vec<Target> {
        self.director
            .as_ref()
            .map(|_| Target::Director)
            .into_iter()
            .chain((0..self.services.len()).map(Target::Service))
            .collect()
    }

    pub(super) fn slot_mut(&mut self, target: Target) -> &mut Slot {
        match target {
            Target::Director => {
                &mut self
                    .director
                    .as_mut()
                    .expect("targets only name a director that is running")
                    .slot
            },
            Target::Service(index) => &mut self.services[index].slot,
        }
    }

    /// Running and not stopping.
    pub(super) fn live(&self, target: Target) -> bool {
        match target {
            Target::Director => self.director.as_ref().is_some_and(|d| !d.slot.stopping),
            Target::Service(index) => self.services.get(index).is_some_and(|s| !s.slot.stopping),
        }
    }
}
