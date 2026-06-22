//! Maud port of `templates/event_detail.html`.

use maud::{Markup, html};

use super::components;
use crate::{
    db::{Event, EventStatus},
    web::state::User,
};

pub fn event_detail(
    user: &User,
    event: &Event,
    results: &crate::web::handlers::EventResults,
) -> Markup {
    let title = match &event.name {
        Some(n) => format!("{n} - Clockwork Carnage"),
        None => format!("Event #{} - Clockwork Carnage", event.id),
    };

    let is_live = event.status == EventStatus::Live;

    let content = html! {
        // ── Event header ──
        div class="border-b border-outline-variant/20" {
            div class="max-w-screen-xl mx-auto px-6 lg:px-12 py-6" {
                a href="/events" class="inline-flex items-center gap-2 text-outline hover:text-primary-container transition-colors mb-6" {
                    "<- BACK TO EVENTS"
                }
                div class=(format!("flex items-start justify-between gap-6 border-l-4 border-[{}] pl-4", event.mode.colour())) {
                    div class="min-w-0" {
                        // Tags row
                        div class="flex flex-wrap items-center gap-1.5 mb-2" {
                            (components::mode_tag(&event.mode, true, is_live))
                            @if let Some(n) = &event.era_name {
                                span class="text-sm font-bold uppercase tracking-widest px-1.5 py-0.5 bg-surface-container-high text-outline" { (n) }
                            }
                        }
                        // Title
                        h1 class="text-4xl font-black tracking-tight mb-2" {
                            @match &event.name {
                                Some(n) => { (n) }
                                None => { span class="text-primary-container" { "Event #" (event.id) } }
                            }
                        }
                        // Track
                        div class="text-sm text-outline mb-2" {
                            (event.track.complete_name()) " " span class="text-outline/50" { "(" (event.track.to_string()) ")" }
                            @if !event.layout.is_empty() { span class="text-outline/50" { " / " } (event.layout) }
                        }
                        // Admin actions
                        (components::event_actions(event, user.admin))
                    }
                    // Status / date
                    div class="shrink-0 text-right" {
                        @match event.status {
                            EventStatus::Live => {
                                div class="inline-flex items-center gap-1.5 border border-primary-container/50 px-2 py-1" {
                                    div class="w-1.5 h-1.5 rounded-full bg-primary-container live-indicator shrink-0" {}
                                    span class="text-sm font-bold text-primary-container tracking-wider" { "LIVE" }
                                }
                            }
                            EventStatus::Pending => {
                                @match &event.scheduled_at {
                                    Some(s) => {
                                        div class="text-xs font-bold uppercase tracking-widest text-outline/40 mb-1" { "Starts" }
                                        (components::local_time(s, "text-on-surface font-semibold"))
                                    }
                                    None => { span class="text-sm font-bold uppercase tracking-widest text-outline/40" { "TBD" } }
                                }
                            }
                            EventStatus::Completed => {
                                @match &event.ended_at {
                                    Some(e) => {
                                        div class="text-xs font-bold uppercase tracking-widest text-outline/40 mb-1" { "Ended" }
                                        (components::local_time(e, "text-on-surface font-semibold"))
                                    }
                                    None => { span class="text-sm font-bold uppercase tracking-widest text-outline/40" { "Ended" } }
                                }
                            }
                        }
                    }
                }
            }
        }

        // ── Main content ──
        div class="max-w-screen-xl mx-auto px-6 lg:px-12 py-8" {
            div class="flex flex-col lg:flex-row gap-8" {

                // ── Leaderboard / Results ──
                div class="flex-1 min-w-0" {

                    // Description
                    @if let Some(d) = &event.description {
                        p class="text-outline mb-6" { (d) }
                    }

                    // Write-up
                    @if let Some(w) = &event.writeup {
                        div class="mb-8" {
                            div class="flex items-center gap-3 mb-4 border-l-4 border-primary-container pl-4" {
                                h2 class="text-xl font-black tracking-tight text-on-surface" { "RACE RECAP" }
                            }
                            div class="bg-surface-container-low p-6" {
                                p class=" text-on-surface-variant leading-relaxed whitespace-pre-line" { (w) }
                            }
                        }
                    }

                    // Results
                    div class="flex items-center justify-between mb-4" {
                        h2 class="text-xl font-black tracking-tight" {
                            span class="text-on-surface" { "FINAL " } span class="text-primary-container" { "RESULTS" }
                        }
                        @if is_live {
                            span class="text-sm text-outline uppercase tracking-widest" { "Live - updates every 5s" }
                        }
                    }
                    (crate::web::views::event_results(results))

                }

                // ── Sidebar ──
                div class="w-full lg:w-80 shrink-0 space-y-6" {
                    // Mode card
                    (components::mode_card_for(&event.mode))
                }
            }
        }
    };

    crate::web::views::layout::layout(&title, user, content)
}
