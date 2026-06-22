//! Maud port of `event_new.html`, with phx-change live re-render.
//!
//! The form is an Alpine `liveForm` (Livewire-style): any change re-POSTs to
//! `/events/new` with `_event=change`, the server re-renders [`event_new_form`]
//! from the changeset, and `Alpine.morph` patches it back in. The server is the
//! single source of truth for which fields each mode exposes (no `x-show`).

use insim::core::track::Track;
use maud::{Markup, html};

use super::{forms, layout::layout};
use crate::{
    db::{self, EventStatus},
    web::{Changeset, handlers::NewEventInput, state::User},
};

/// A selectable game-mode radio card.
fn mode_card(
    cs: &Changeset<NewEventInput>,
    value: &str,
    title: &str,
    accent: &str,
    tagline: &str,
) -> Markup {
    let checked = cs.params.mode == value;
    html! {
        label class="cursor-pointer" {
            input type="radio" name="mode" value=(value) class="sr-only peer" checked[checked];
            div class=(format!("relative p-4 text-left border-l-4 border-transparent peer-checked:border-[{accent}] peer-checked:bg-surface-container-high bg-surface-container-low hover:bg-surface-container-high/50 transition-all")) {
                div class=(format!("inline-flex items-center justify-center w-10 h-10 bg-[{accent}]/20 text-[{accent}] font-black mb-3")) { (title.to_uppercase()) }
                h3 class="font-bold text-on-surface mb-1" { (title) }
                p class="text-sm text-outline leading-relaxed" { (tagline) }
            }
        }
    }
}

/// Mode-specific settings, rendered inline from the changeset. Because the whole
/// form re-renders on change, this is just a `match` — the single source of
/// truth for which fields a mode has.
fn mode_settings_fields(cs: &Changeset<NewEventInput>) -> Markup {
    html! {
        @match cs.params.mode.as_str() {
            "metronome" => {
                div class="border-t border-outline-variant/20 pt-6" {
                    h2 class="text-sm font-bold text-primary-container uppercase tracking-widest mb-4" { "Metronome Settings" }
                    div class="grid grid-cols-2 gap-6" {
                        div { (forms::number_field("target", "Target (s)", cs.params.target, "1", cs.error("target"))) }
                    }
                }
            }
            "bomb" => {
                div class="border-t border-outline-variant/20 pt-6" {
                    h2 class="text-sm font-bold text-[#f59e0b] uppercase tracking-widest mb-4" { "Bomb Run Settings" }
                    div class="grid grid-cols-2 gap-6" {
                        div { (forms::number_field("checkpoint_timeout", "Checkpoint Timeout (s)", cs.params.checkpoint_timeout, "1", cs.error("checkpoint_timeout"))) }
                    }
                }
            }
            _ => {}
        }
    }
}

/// The `<form>` itself — the morph target's content. Re-rendered on every change.
pub fn event_new_form(
    csrf_token: &str,
    tracks: &[Track],
    eras: &[db::Era],
    cs: &Changeset<NewEventInput>,
) -> Markup {
    let mode_err = cs.error("mode");
    let name_val = cs.params.name.as_deref().unwrap_or("");
    let desc_val = cs.params.description.as_deref().unwrap_or("");
    let sched_val = cs.params.scheduled_at.as_deref().unwrap_or("");

    html! {
        form id="event-form" method="post" action="/events/new" x-data="liveForm" x-on:change="refresh" {
            input type="hidden" name="csrf_token" value=(csrf_token);
            div class="space-y-10" {

                // ── Game Mode ──
                section {
                    h2 class="text-sm font-bold text-outline uppercase tracking-widest mb-4" { "Game Mode *" }
                    div class="grid grid-cols-3 gap-4" {
                        (mode_card(cs, "metronome", "Metronome", "#3b82f6", "Maintain consistent lap times. Precision over raw speed."))
                        (mode_card(cs, "shortcut", "Shortcut", "#22c55e", "Find and exploit every possible shortcut. Fastest route wins."))
                        (mode_card(cs, "bomb", "Bomb Run", "#f59e0b", "Race against the clock. Checkpoints add time. Miss one and you're out."))
                    }
                    @if !mode_err.is_empty() { p class="mt-2 text-error" { (mode_err) } }
                }

                // ── Track + Layout ──
                section {
                    h2 class="text-sm font-bold text-outline uppercase tracking-widest mb-4" { "Track" }
                    div class="grid grid-cols-2 gap-6" {
                        div {
                            (forms::select_field("track", "Track *", cs.error("track"), html! {
                                option value="" disabled selected[cs.params.track.is_none()] { "Select a track…" }
                                @for track in tracks {
                                    option value=(track.to_string()) selected[cs.params.track.as_ref() == Some(track)] {
                                        (track.to_string()) " - " (track.complete_name())
                                    }
                                }
                            }))
                        }
                        div { (forms::text_field("layout", "Layout", &cs.params.layout, "blank for default", cs.error("layout"))) }
                    }
                }

                // ── Basic Info ──
                section {
                    h2 class="text-sm font-bold text-outline uppercase tracking-widest mb-4" { "Basic Information" }
                    div class="space-y-4" {
                        div { (forms::text_field("name", "Name", name_val, "Optional display name", cs.error("name"))) }
                        div { (forms::textarea_field("description", "Description", desc_val, 3, "Optional description", cs.error("description"))) }
                    }
                }

                // ── Schedule ──
                section {
                    h2 class="text-sm font-bold text-outline uppercase tracking-widest mb-4" { "Schedule" }
                    div { (forms::datetime_local_field("scheduled_at", "Scheduled Start", sched_val, cs.error("scheduled_at"))) }
                }

                // ── Status ──
                section {
                    h2 class="text-sm font-bold text-outline uppercase tracking-widest mb-4" { "Status" }
                    (forms::select_field("status", "Status", cs.error("status"), html! {
                        option value="pending" selected[cs.params.status == EventStatus::Pending] { "Pending" }
                        option value="live" selected[cs.params.status == EventStatus::Live] { "Live" }
                        option value="completed" selected[cs.params.status == EventStatus::Completed] { "Completed" }
                    }))
                }

                // ── Era / Season ── (Alpine: ephemeral select-existing vs create-new toggle)
                section x-data="{ creating: false }" {
                    h2 class="text-sm font-bold text-outline uppercase tracking-widest mb-4" { "Era / Season" }
                    div class="flex items-center gap-4 mb-3" {
                        button type="button" x-on:click="creating = false"
                            x-bind:class="!creating ? 'text-primary-container border-b border-primary-container' : 'text-outline hover:text-on-surface'"
                            class="text-xs font-bold uppercase tracking-widest pb-0.5 transition-colors" { "Select Existing" }
                        button type="button" x-on:click="creating = true"
                            x-bind:class="creating ? 'text-primary-container border-b border-primary-container' : 'text-outline hover:text-on-surface'"
                            class="text-xs font-bold uppercase tracking-widest pb-0.5 transition-colors" { "+ Create New" }
                    }
                    div x-show="!creating" {
                        label for="era_id" class="text-sm uppercase tracking-widest text-outline block mb-2" { "Era" }
                        select id="era_id" name="era_id" x-bind:disabled="creating"
                            class="bg-surface-container-low w-full px-4 py-3 text-on-surface border border-outline-variant focus:border-primary-container focus:outline-none transition-colors appearance-none cursor-pointer" {
                            option value="" { "- None -" }
                            @for era in eras {
                                option value=(era.id.to_string()) selected[cs.params.era_id == Some(era.id)] { (era.name) }
                            }
                        }
                    }
                    div x-show="creating" x-cloak="" {
                        label for="era_name" class="text-sm uppercase tracking-widest text-outline block mb-2" { "New Era Name" }
                        input id="era_name" type="text" name="era_name" x-bind:disabled="!creating" placeholder="e.g. Season 1"
                            value=(cs.params.era_name.as_deref().unwrap_or(""))
                            class="bg-surface-container-low w-full px-4 py-3 text-on-surface border border-outline-variant focus:border-primary-container focus:outline-none transition-colors placeholder:text-outline/40";
                    }
                }

                // ── Mode-specific settings (server decides; re-rendered on change) ──
                (mode_settings_fields(cs))

                // ── Vehicle restrictions ──
                section class="border-t border-outline-variant/20 pt-6" {
                    h2 class="text-sm font-bold text-outline uppercase tracking-widest mb-2" { "Vehicle Restrictions" }
                    p class="text-sm text-outline/60 mb-3" {
                        "Comma-separated codes (e.g. " span class="font-mono text-on-surface-variant" { "XFG, XRG" } "). Leave blank to allow all."
                    }
                    (forms::text_field("allowed_vehicles", "", &cs.params.allowed_vehicles, "blank = all allowed", cs.error("allowed_vehicles")))
                }

                // ── Actions ──
                div class="flex items-center justify-between pt-6 border-t border-outline-variant/20" {
                    a href="/events" class="font-bold uppercase tracking-wider text-outline hover:text-on-surface transition-colors px-6 py-3" { "Cancel" }
                    button type="submit" class="inline-flex items-center gap-2 px-8 py-3 bg-primary-container text-white font-bold uppercase tracking-wider hover:bg-primary-container/90 transition-colors" { "Create Event" }
                }
            }
        }
    }
}

/// Full create-event page.
pub fn event_new(
    user: &User,
    csrf_token: &str,
    tracks: &[Track],
    eras: &[db::Era],
    cs: &Changeset<NewEventInput>,
) -> Markup {
    layout(
        "New Event - Clockwork Carnage",
        user,
        html! {
            div class="border-b border-outline-variant/20" {
                div class="max-w-screen-xl mx-auto px-6 lg:px-12 py-6" {
                    a href="/events" class="inline-flex items-center gap-2 text-outline hover:text-primary-container transition-colors mb-6 block" { "<- BACK TO EVENTS" }
                    h1 class="text-3xl font-black tracking-tight" {
                        span class="text-on-surface" { "CREATE " } span class="text-primary-container" { "EVENT" }
                    }
                    p class="text-outline mt-2" { "Set up a new mini-game event for players to compete in." }
                }
            }
            div class="max-w-screen-xl mx-auto px-6 lg:px-12 py-8" {
                (event_new_form(csrf_token, tracks, eras, cs))
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_event_new_form() {
        let cs = Changeset::<NewEventInput>::empty();
        let html = event_new_form("csrf123", Track::ALL, &[], &cs).into_string();
        assert!(html.contains(r#"x-data="liveForm""#));
        assert!(html.contains(r#"action="/events/new""#));
        // metronome is the default mode → its settings field is present
        assert!(html.contains(r#"name="target""#));
    }
}
