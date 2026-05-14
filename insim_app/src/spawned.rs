//! [`Spawned<F>`] - wraps a generic async function as a long-running spawned
//! task plus a dispatch forwarder, in one handler.
//!
//! On its **first** invocation, [`Spawned`] spawns `f(rx, sender)` as a tokio
//! task, where `rx` is an [`mpsc::UnboundedReceiver<Dispatch>`] that receives
//! every [`Dispatch`] flowing through the runtime. On **every** invocation -
//! including the first - it forwards the current dispatch into that channel
//! so the spawned task sees it.
//!
//! This collapses the common "pumped task" pattern (a separate forwarder
//! handler + an `Event<Startup>` launcher + an `Arc<Mutex<Option<…>>>` receiver
//! hand-off) into one call:
//!
//! ```ignore
//! .handler(Spawned::new(move |rx, sender| run_my_game(rx, sender, presence.clone(), ui.clone())))
//! ```
//!
//! Other extension handles ([`crate::Presence`], [`crate::ui::Ui`], …) are
//! wired into the spawned task by ordinary closure capture - they're all
//! cloneable, so build them in `main` before `App` registration and move clones
//! into the closure.
//!
//! ## Spawn timing
//!
//! The task is spawned on the *first dispatch*, not at some special lifecycle
//! event. In practice that's still nearly instant - the runtime emits
//! [`crate::Startup`] as the very first dispatch - but it means by the time
//! the spawned task starts polling `rx.recv()`, the `Startup` event is already
//! buffered in the channel. Tasks that need to perform setup *before* seeing
//! any dispatches should do that setup at the top of their async body, before
//! the first `rx.recv().await`.

use std::{future::Future, sync::Arc};

use tokio::sync::{Mutex, mpsc};

use crate::{
    AppError, Dispatch, Handler,
    extract::{ExtractCx, Sender},
};

/// Handler that spawns a long-running async task on its first invocation and
/// forwards every dispatch into it.
///
/// See the [module docs](crate::spawned) for the rationale and usage pattern.
pub struct Spawned<F> {
    inner: Arc<SpawnedInner<F>>,
}

impl<F> Clone for Spawned<F> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<F> std::fmt::Debug for Spawned<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Spawned").finish_non_exhaustive()
    }
}

struct SpawnedInner<F> {
    /// `Some` until the first dispatch arrives; then taken and `f` runs in a
    /// spawned task.
    spawn_state: Mutex<Option<(F, mpsc::UnboundedReceiver<Dispatch>)>>,
    /// Tx half of the channel the spawned task drains. Every dispatch
    /// (including the very first, which triggered the spawn) goes here.
    tx: mpsc::UnboundedSender<Dispatch>,
}

impl<F> Spawned<F> {
    /// Wrap an async function as a spawn-on-first-dispatch handler.
    ///
    /// `f` is called once, with two arguments:
    /// 1. An [`mpsc::UnboundedReceiver<Dispatch>`] carrying every dispatch the
    ///    runtime processes (wire packets *and* synthetic events).
    /// 2. A [`Sender`] for sending packets back out or emitting synthetic events.
    ///
    /// Other extension handles should be cloned into the closure from outside;
    /// they're all `Clone`.
    pub fn new(f: F) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            inner: Arc::new(SpawnedInner {
                spawn_state: Mutex::new(Some((f, rx))),
                tx,
            }),
        }
    }
}

/// Convenience constructor for [`Spawned`].
///
/// Equivalent to [`Spawned::new`], but reads as a verb at the call site:
///
/// ```ignore
/// .handler(spawned(|rx, sender| run_my_game(rx, sender, presence.clone(), ui.clone())))
/// ```
///
/// Mirrors the shape of `tokio::spawn`, `axum::routing::get`, etc.
pub fn spawned<F, Fut>(f: F) -> Spawned<F>
where
    F: FnOnce(mpsc::UnboundedReceiver<Dispatch>, Sender) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    Spawned::new(f)
}

impl<S, F, Fut> Handler<S, (Sender,)> for Spawned<F>
where
    F: FnOnce(mpsc::UnboundedReceiver<Dispatch>, Sender) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
    S: Send + Sync + 'static,
{
    async fn call(self, cx: &ExtractCx<'_, S>) -> Result<(), AppError> {
        // First call only: take the receiver + factory, spawn the task.
        if let Some((f, rx)) = self.inner.spawn_state.lock().await.take() {
            #[allow(clippy::let_underscore_future)]
            let _ = tokio::spawn(f(rx, cx.sender.clone()));
        }
        // Every call: forward this dispatch to the running task.
        let _ = self.inner.tx.send(cx.dispatch.clone());
        Ok(())
    }
}
