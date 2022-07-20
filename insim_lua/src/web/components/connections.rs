use crate::web::hooks::{use_insim, use_insim_connection_future};

use dioxus::prelude::*;

#[inline_props]
pub(crate) fn list(cx: Scope, server: String) -> Element {
    let state = use_insim(&cx, server);
    use_insim_connection_future(&cx, state.clone());

    cx.render(rsx! {
        div {
            state.get_connections().iter().map(|(ucid, player)| {
                rsx! {
                    div {
                        key: "{ucid}",
                        "{player:?}"
                    }
                }
            })
        }
    })
}
