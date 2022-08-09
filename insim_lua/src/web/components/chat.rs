use crate::web::hooks::{use_insim, use_insim_chat_future};

use dioxus::prelude::*;

#[inline_props]
pub(crate) fn chat(cx: Scope, server: String) -> Element {
    let state = use_insim(&cx, server);
    use_insim_chat_future(&cx, state.clone());

    let messages = state.chat();

    cx.render(if messages.is_empty() {
        rsx!("It's too quiet.")
    } else {
        rsx! {
            ul {
                messages.iter().map(|c| {
                    rsx! {
                        li {
                            key: "{c.at}-{c.ucid}",
                            div {
                                "{c.at}"
                            }
                            div {
                                "{c.body}"
                            }
                        }
                    }
                })

            }
        }
    })
}
