use crate::web::hooks::{use_insim, use_insim_player_future};

use dioxus::prelude::*;

#[inline_props]
pub(crate) fn list(cx: Scope, server: String) -> Element {
    let state = use_insim(&cx, server);
    use_insim_player_future(&cx, state.clone());

    cx.render(rsx! {
        div {
            state.get_players().iter().map(|(plid, player)| {
                rsx! {
                    div {
                        key: "{plid}",
                        "{player:?}"
                    }
                }
            })
        }
    })
}
