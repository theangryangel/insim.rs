//! Maud port of `about.html`.

use maud::{Markup, html};

use super::{components, layout::layout};
use crate::web::state::User;

pub fn about(user: &User) -> Markup {
    layout(
        "About - Clockwork Carnage",
        user,
        html! {
            div class="max-w-screen-xl mx-auto px-6 lg:px-12 py-10" {
                div class="mb-8" {
                    h1 class="text-3xl md:text-4xl font-black tracking-tight" {
                        span class="text-on-surface" { "ABOUT " }
                        span class="text-primary-container" { "CLOCKWORK CARNAGE" }
                    }
                    p class="text-outline mt-2" { "A mini-game racing server for Live for Speed." }
                }
                div class="h-px bg-outline-variant/20 mb-10" {}
                section class="mb-12" {
                    h2 class="text-xl font-black tracking-tight mb-6" {
                        span class="text-on-surface" { "GAME " }
                        span class="text-primary-container" { "MODES" }
                    }
                    div class="h-px bg-outline-variant/20 mb-6" {}
                    div class="grid md:grid-cols-3 gap-6" {
                        (components::mode_card_shortcut())
                        (components::mode_card_metronome())
                        (components::mode_card_bomb())
                    }
                }
            }
        },
    )
}
