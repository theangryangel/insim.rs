//! Maud port of `index.html`.

use maud::{Markup, html};

use super::{components, layout::layout};
use crate::{db::Event, web::state::User};

pub fn index(user: &User, events: &[Event]) -> Markup {
    layout(
        "Clockwork Carnage",
        user,
        html! {
            section class="relative min-h-screen overflow-hidden" {
                div class="absolute inset-0 bg-cover bg-center bg-no-repeat" style="background-image: url('/assets/track.jpg')" {
                    div class="absolute inset-0 bg-gradient-to-l from-surface from-30% via-surface/90 via-60% to-surface/60" {}
                }
                div class="relative z-10 grid lg:grid-cols-2 gap-12 px-6 lg:px-12 py-12 max-w-screen-xl mx-auto" {
                    div class="flex flex-col justify-center" {
                        img src="/assets/logo.svg" alt="Clockwork Carnage" class="h-[180px] w-auto mr-auto";
                        p class="text-base text-outline max-w-md leading-relaxed mt-8" {
                            "Mini-game server for Live for Speed. Three challenge modes. No mercy."
                        }
                    }
                    div class="flex flex-col justify-start pt-4" {
                        div class="flex items-center justify-between mb-4" {
                            h2 class="font-bold text-outline uppercase tracking-widest" { "Current & Upcoming" }
                            a href="/events" class="font-bold text-primary-container hover:underline tracking-wider uppercase" { "View All" }
                        }
                        div class="h-px bg-outline-variant/20 mb-4" {}
                        @if events.is_empty() {
                            p class="py-6 text-center text-outline" { "No events currently scheduled" }
                        } @else {
                            (components::event_list(events))
                        }
                    }
                }
            }
        },
    )
}
