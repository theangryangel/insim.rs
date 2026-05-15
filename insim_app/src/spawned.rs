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
//!
//! ## Cancellation
//!
//! When the runtime's [`tokio_util::sync::CancellationToken`] fires, an
//! internal forwarder task exits and drops the sender feeding the spawned
//! task's receiver. A task whose body is just `while let Some(d) =
//! rx.recv().await { ... }` therefore exits naturally on shutdown - no
//! explicit cancel check required. Tasks that need a `select!` on cancel
//! alongside other futures can still grab a clone of the token from
//! elsewhere; both patterns work.
//!
//! Crucially, the recv-loop pattern also gives the user a place to do
//! **async cleanup** on shutdown: anything after the loop body runs *while
//! the task is still alive*, before it exits, so you can persist state,
//! flush a DB write, or send a farewell packet:
//!
//! ```ignore
//! spawned(|mut rx, sender| async move {
//!     while let Some(d) = rx.recv().await {
//!         // handle d
//!     }
//!     // runs once on cancel, before the task ends
//!     persist_round_result().await;
//!     let _ = sender.packet(goodbye_mtc());
//! })
//! ```
//!
//! ### Why a forwarder rather than wrapping the user future in `select!`?
//!
//! The naive "kill the task on cancel" implementation would be a single
//! `tokio::select! { _ = cancel.cancelled() => {}, _ = user_fut => {} }`
//! wrapper - simpler, no extra task, no extra channel. We chose the
//! forwarder because it leaves room for the async-cleanup pattern above.
//! Wrapping in `select!` drops the user future at its current await point,
//! so any code after the recv-loop never runs and only sync `Drop` impls
//! can do cleanup. The forwarder closes the *channel* on cancel; the user
//! task continues running and decides for itself when to exit.
//!
//! The mechanic is forced by [`tokio::sync::mpsc`]: an
//! [`mpsc::UnboundedSender`] has no `close()` method, so the only way to
//! close the channel from the "outside" is to drop every sender that feeds
//! it. The forwarder owns the sole sender feeding the user-facing
//! [`mpsc::UnboundedReceiver`]; when cancel fires, the forwarder exits,
//! the sender drops, the channel closes, and the user's `rx.recv()` returns
//! `None`.

use std::{future::Future, sync::Arc};

use tokio::sync::{Mutex, mpsc};
use tokio_util::sync::CancellationToken;

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
        // First call only: take the receiver + factory, spawn the task plus
        // a cancellation-aware forwarder. The forwarder owns the only tx
        // feeding the user's rx, so when the runtime cancels we drop that
        // tx and the user's `rx.recv()` resolves to `None`.
        if let Some((f, inner_rx)) = self.inner.spawn_state.lock().await.take() {
            let (user_tx, user_rx) = mpsc::unbounded_channel::<Dispatch>();
            let cancel = cx.cancel.clone();
            #[allow(clippy::let_underscore_future)]
            let _ = tokio::spawn(forwarder(inner_rx, user_tx, cancel));
            #[allow(clippy::let_underscore_future)]
            let _ = tokio::spawn(f(user_rx, cx.sender.clone()));
        }
        // Every call: feed this dispatch into the internal channel; the
        // forwarder picks it up and either passes it on or, if cancel has
        // fired, drops out and lets the user task wind down.
        let _ = self.inner.tx.send(cx.dispatch.clone());
        Ok(())
    }
}

/// Forwards dispatches from the handler-side channel to the user-task channel.
///
/// Exists so that cancellation closes the user-facing channel rather than
/// killing the user future mid-await: see the module-level docs for the full
/// rationale. The forwarder is the sole owner of the sender feeding the
/// user's [`mpsc::UnboundedReceiver`], so exiting here drops that sender and
/// the user's `rx.recv()` resolves to `None`.
///
/// Exits on cancellation, when the inbound channel closes, or when the user
/// task has already dropped its receiver.
async fn forwarder(
    mut inner_rx: mpsc::UnboundedReceiver<Dispatch>,
    user_tx: mpsc::UnboundedSender<Dispatch>,
    cancel: CancellationToken,
) {
    loop {
        tokio::select! {
            biased;
            _ = cancel.cancelled() => return,
            maybe = inner_rx.recv() => {
                let Some(d) = maybe else { return; };
                if user_tx.send(d).is_err() { return; }
            }
        }
    }
}
