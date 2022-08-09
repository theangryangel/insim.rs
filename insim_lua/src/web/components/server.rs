use dioxus::{prelude::*, router::use_route};

use super::*;

pub(crate) fn show(cx: Scope) -> Element {
    let route = use_route(&cx);
    let key = route.segment("id").unwrap();

    cx.render(rsx! {

        players::list { server: key.to_string() }

        connections::list { server: key.to_string() }

        chat::chat { server: key.to_string() }

    })
}
