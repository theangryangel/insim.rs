//! Maud port of `templates/event_edit.html` (the `{% block content %}` body).
//! Wrapped by `layout()`. Sibling of `event_new` — the edit variant of the
//! same form (mode is read-only here; status + era + write-up are editable).

use insim::core::track::Track;
use maud::{Markup, html};

use super::{forms, layout};
use crate::{
    db::{self, EventMode, EventStatus},
    web::{Changeset, handlers::EditEventInput, state::User},
};

/// The `<form>` — morph target content for phx-change live re-render.
pub fn event_edit_form(
    csrf: &str,
    event: &db::Event,
    tracks: &[Track],
    eras: &[db::Era],
    cs: &Changeset<EditEventInput>,
) -> Markup {
    let action = format!("/events/{}/edit", event.id);

    let name_val = cs.params.name.as_deref().unwrap_or("");
    let desc_val = cs.params.description.as_deref().unwrap_or("");
    let sched_val = cs.params.scheduled_at.as_deref().unwrap_or("");
    let writeup_val = cs.params.writeup.as_deref().unwrap_or("");

    html! {
      form id="event-form" method="post" action=(action) x-data="liveForm" x-on:change="refresh" {
        input type="hidden" name="csrf_token" value=(csrf);
        div class="space-y-8" {

            // Mode (read-only)
            div {
                p class="text-sm uppercase tracking-widest text-outline mb-2" { "Mode" }
                div class=(format!("inline-flex items-center justify-center px-3 py-1.5 {}", event.mode.tag_classes())) {
                    span class="text-sm font-bold tracking-wider uppercase" { (event.mode.label()) }
                }
            }

            div class="grid grid-cols-2 gap-6" {
                div {
                    (forms::select_field("track", "Track *", cs.error("track"), html! {
                        @for track in tracks {
                            option value=(track.to_string()) selected[cs.params.track == *track] {
                                (track.to_string()) " - " (track.complete_name())
                            }
                        }
                    }))
                }
                div { (forms::text_field("layout", "Layout", &cs.params.layout, "", cs.error("layout"))) }
            }

            div { (forms::text_field("name", "Name", name_val, "Optional display name", cs.error("name"))) }

            div { (forms::textarea_field("description", "Description", desc_val, 2, "Optional description", cs.error("description"))) }

            div { (forms::datetime_local_field("scheduled_at", "Scheduled Start", sched_val, cs.error("scheduled_at"))) }

            div {
                (forms::select_field("status", "Status", cs.error("status"), html! {
                    option value="pending" selected[cs.params.status == EventStatus::Pending] { "Pending" }
                    option value="live" selected[cs.params.status == EventStatus::Live] { "Live" }
                    option value="completed" selected[cs.params.status == EventStatus::Completed] { "Completed" }
                }))
            }

            // Era / Season (Alpine: toggle select-existing vs create-new)
            div x-data="{ creating: false }" {
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
                            option value=(era.id) selected[cs.params.era_id == Some(era.id)] { (era.name) }
                        }
                    }
                }
                div x-show="creating" x-cloak="" {
                    label for="era_name" class="text-sm uppercase tracking-widest text-outline block mb-2" { "New Era Name" }
                    input id="era_name" type="text" name="era_name" x-bind:disabled="!creating" placeholder="e.g. Season 2"
                        value=(cs.params.era_name.as_deref().unwrap_or(""))
                        class="bg-surface-container-low w-full px-4 py-3 text-on-surface border border-outline-variant focus:border-primary-container focus:outline-none transition-colors placeholder:text-outline/40";
                }
            }

            div { (forms::textarea_field("writeup", "Write-up", writeup_val, 6, "Post-event write-up (optional)", cs.error("writeup"))) }

            // Mode-specific settings (mode is fixed on edit)
            @match &*event.mode {
                EventMode::Metronome { .. } => {
                    div class="border-t border-outline-variant/20 pt-6" {
                        p class=(format!("text-sm font-bold text-[{}] uppercase tracking-widest mb-4", event.mode.colour())) { "Metronome Settings" }
                        div class="grid grid-cols-2 gap-6" {
                            div { (forms::number_field("target", "Target (s)", cs.params.target.unwrap_or(20), "1", cs.error("target"))) }
                        }
                    }
                }
                EventMode::Shortcut => {}
                EventMode::Bomb { .. } => {
                    div class="border-t border-outline-variant/20 pt-6" {
                        p class=(format!("text-sm font-bold text-[{}] uppercase tracking-widest mb-4", event.mode.colour())) { "Bomb Run Settings" }
                        div class="grid grid-cols-2 gap-6" {
                            div { (forms::number_field("checkpoint_timeout", "Checkpoint Timeout (s)", cs.params.checkpoint_timeout.unwrap_or(30), "1", cs.error("checkpoint_timeout"))) }
                        }
                    }
                }
            }

            // Vehicle restrictions
            div class="border-t border-outline-variant/20 pt-6" {
                p class="text-sm font-bold text-outline uppercase tracking-widest mb-2" { "Vehicle Restrictions" }
                p class="text-sm text-outline/60 mb-3" {
                    "Comma-separated codes (e.g. " span class="font-mono text-on-surface-variant" { "XFG, XRG" } "). Leave blank to allow all."
                }
                (forms::text_field("allowed_vehicles", "", &cs.params.allowed_vehicles, "blank = all allowed", cs.error("allowed_vehicles")))
            }

            // Actions
            div class="flex items-center justify-between pt-6 border-t border-outline-variant/20" {
                a href=(format!("/events/{}", event.id)) class="font-bold uppercase tracking-wider text-outline hover:text-on-surface transition-colors px-6 py-3" { "Cancel" }
                button type="submit" class="px-8 py-3 bg-primary-container text-white font-bold uppercase tracking-wider hover:bg-primary-container/90 transition-colors" { "Save Changes" }
            }
        }
      }
    }
}

/// Full edit-event page.
pub fn event_edit(
    user: &User,
    csrf: &str,
    event: &db::Event,
    tracks: &[Track],
    eras: &[db::Era],
    cs: &Changeset<EditEventInput>,
) -> Markup {
    let title = format!("Edit Event #{} - Clockwork Carnage", event.id);
    layout::layout(
        &title,
        user,
        html! {
            div class="border-b border-outline-variant/20" {
                div class="max-w-screen-xl mx-auto px-6 lg:px-12 py-6" {
                    a href=(format!("/events/{}", event.id)) class="inline-flex items-center gap-2 text-outline hover:text-primary-container transition-colors mb-6 block" {
                        "<- EVENT #" (event.id)
                    }
                    h1 class="text-3xl font-black tracking-tight" {
                        span class="text-on-surface" { "EDIT " } span class="text-primary-container" { "EVENT" }
                    }
                }
            }
            div class="max-w-screen-xl mx-auto px-6 lg:px-12 py-8" {
                (event_edit_form(csrf, event, tracks, eras, cs))
            }
        },
    )
}
