use crate::state::State;
use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

pub(crate) fn use_insim(cx: &ScopeState, key: &str) -> Arc<State> {
    let state = cx
        .use_hook(|_| {
            cx.consume_context::<HashMap<String, Arc<State>>>()
                .unwrap()
                .get(key)
                .unwrap()
                .clone()
        })
        .clone();

    state
}

pub(crate) fn use_insim_player_future(cx: &ScopeState, state: Arc<State>) {
    let update = cx.schedule_update();

    use_future(cx, (), |_| async move {
        loop {
            state.notify_on_player().notified().await;
            update();
        }
    });
}

pub fn use_insim_chat_future(cx: &ScopeState, state: Arc<State>) {
    let update = cx.schedule_update();

    use_future(cx, (), |_| async move {
        loop {
            state.notify_on_chat().notified().await;
            update();
        }
    });
}

pub fn use_insim_connection_future(cx: &ScopeState, state: Arc<State>) {
    let update = cx.schedule_update();

    use_future(cx, (), |_| async move {
        loop {
            state.notify_on_connection().notified().await;
            update();
        }
    });
}
