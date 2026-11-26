//! Stopping services and the director.

use super::{Loop, Target};
use crate::kitcar::service::Input;

impl Loop {
    pub(super) async fn stop(&mut self, index: usize) {
        tracing::info!(
            service = self.services[index].service.name(),
            "service stopped"
        );
        self.services[index].slot.stopping = true;
        self.deliver(Target::Service(index), Input::Stopping).await;
        let buttons = self.services[index].service.clear(&mut self.pools);
        self.send_all(buttons).await;
        drop(self.services.remove(index));
    }

    pub(super) async fn stop_director(&mut self) {
        let Some(directing) = self.director.as_mut() else {
            return;
        };
        tracing::info!("director stopped");
        directing.slot.stopping = true;
        self.deliver(Target::Director, Input::Stopping).await;
        self.director = None;
    }

    /// Services first, then the director they read.
    pub(super) async fn stop_all(&mut self) {
        while !self.services.is_empty() {
            self.stop(self.services.len() - 1).await;
        }
        self.stop_director().await;
    }

    pub(super) async fn remove_stopped(&mut self) {
        self.remove_stopped_services().await;
        if self.director.as_ref().is_some_and(|d| d.slot.stopping) {
            self.stop_director().await;
        }
    }

    pub(super) async fn remove_stopped_services(&mut self) {
        loop {
            let Some(index) = self.services.iter().position(|m| m.slot.stopping) else {
                return;
            };
            self.stop(index).await;
        }
    }
}
