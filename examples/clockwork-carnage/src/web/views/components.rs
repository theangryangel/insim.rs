//! Shared Maud components — ports of the reusable `partials/*.html`.

use jiff_sqlx::Timestamp;
use maud::{Markup, PreEscaped, html};

use crate::{
    db::{Event, EventMode, EventStatus},
    web::filters,
};

/// LFS-coloured player name + optional Twitch/YouTube links.
pub fn player_name(
    pname: &str,
    uname: &str,
    twitch: &Option<String>,
    youtube: &Option<String>,
) -> Markup {
    html! {
        span class="font-bold text-primary-container" { (PreEscaped(filters::colour_spans_html(pname))) }
        span class="text-sm text-outline ml-1.5" { (uname) }
        @if let Some(tw) = twitch {
            a href=(format!("https://twitch.tv/{tw}")) target="_blank" class="ml-1.5" {
                img src="/assets/twitch-icon.svg" class="h-4 w-4 inline" alt="Twitch";
            }
        }
        @if let Some(yt) = youtube {
            a href=(format!("https://youtube.com/@{yt}")) target="_blank" class="ml-1" {
                img src="/assets/youtube-icon.svg" class="h-4 w-4 inline" alt="YouTube";
            }
        }
    }
}

/// `<time>` element rendered by the Alpine `x-localtime` directive.
pub fn local_time(ts: &Timestamp, class: &str) -> Markup {
    html! {
        time class=(class) datetime=(filters::iso8601(ts)) x-data="" x-localtime="" {
            (filters::timestamp_human(ts))
        }
    }
}

/// Mode pill. `large` renders the header variant (with optional LIVE marker).
pub fn mode_tag(mode: &EventMode, large: bool, is_live: bool) -> Markup {
    if large {
        html! {
            div class=(format!("inline-flex items-center gap-2 px-3 py-1 mb-4 text-sm font-bold uppercase tracking-wider {}", mode.tag_classes())) {
                span { (mode.label()) }
                @if is_live {
                    span class="opacity-60" { "|" }
                    div class="flex items-center gap-1.5" {
                        div class="w-1.5 h-1.5 rounded-full bg-primary-container live-indicator" {}
                        span class="text-primary-container" { "LIVE" }
                    }
                }
            }
        }
    } else {
        html! {
            span class=(format!("text-sm font-bold uppercase tracking-widest px-1.5 py-0.5 shrink-0 {}", mode.tag_classes())) { (mode.label()) }
        }
    }
}

/// Admin-only edit action for an event.
pub fn event_actions(event: &Event, admin: bool) -> Markup {
    html! {
        div id="event-actions" {
            @if admin {
                div class="flex items-center gap-2 flex-wrap mt-4" {
                    a href=(format!("/events/{}/edit", event.id))
                        class="text-sm font-bold uppercase tracking-widest text-outline border border-outline-variant px-4 py-2 hover:text-on-surface hover:border-outline transition-colors" { "Edit" }
                }
            }
        }
    }
}

fn event_name(event: &Event) -> Markup {
    html! {
        @match &event.name {
            Some(n) => { (n) }
            None => { "Event #" (event.id) }
        }
    }
}

fn event_status_cell(event: &Event) -> Markup {
    html! {
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
                        div class="text-right" {
                            div class="text-xs font-bold uppercase tracking-widest text-outline/40" { "Starts" }
                            (local_time(s, "text-outline"))
                        }
                    }
                    None => { span class="text-sm font-bold uppercase tracking-widest text-outline/40" { "TBD" } }
                }
            }
            EventStatus::Completed => {
                @match &event.ended_at {
                    Some(e) => {
                        div class="text-right" {
                            div class="text-xs font-bold uppercase tracking-widest text-outline/40" { "Ended" }
                            (local_time(e, "text-outline"))
                        }
                    }
                    None => { span class="text-sm font-bold uppercase tracking-widest text-outline/40" { "Completed" } }
                }
            }
        }
    }
}

/// Table of events (home + events list).
pub fn event_list(events: &[Event]) -> Markup {
    html! {
        table class="w-full border-separate border-spacing-y-2.5" {
            tbody {
                @for event in events {
                    tr class="bg-surface-container-low hover:bg-surface-container-high transition-colors group" {
                        td class=(format!("py-4 pl-4 pr-3 border-l-4 border-[{}]", event.mode.colour())) {
                            div class="flex flex-wrap items-center gap-1.5 mb-1" {
                                a href=(format!("/events/{}", event.id))
                                    class="font-bold text-on-surface group-hover:text-primary-container transition-colors" { (event_name(event)) }
                                (mode_tag(&event.mode, false, false))
                                @if let Some(n) = &event.era_name {
                                    span class="text-sm font-bold uppercase tracking-widest px-1.5 py-0.5 bg-surface-container-high text-outline" { (n) }
                                }
                            }
                            div class="text-sm text-outline" {
                                (event.track.complete_name()) " " span class="text-outline/50" { "(" (event.track.to_string()) ")" }
                                @if !event.layout.is_empty() { span class="text-outline/50" { " / " } (event.layout) }
                            }
                        }
                        td class="py-4 px-3 text-right whitespace-nowrap" { (event_status_cell(event)) }
                    }
                }
            }
        }
    }
}

// ── Static game-mode info cards (about page + event detail sidebar) ──

pub fn mode_card_metronome() -> Markup {
    html! {
        div class="bg-surface-container-low border-l-4 border-[#3b82f6]" {
            div class="p-5" {
                div class="mb-4" {
                    h3 class="font-bold text-on-surface" { "Metronome" }
                    p class="text-sm text-outline uppercase tracking-wider" { "Game Mode" }
                }
                div class="space-y-4" {
                    (card_point("Match the Time", "Drive between start and finish and hit the target time as closely as possible."))
                    (card_point("Retry Freely", "Attempt as many runs as you like. Only your best delta counts."))
                    (card_point("Earn a Tier", "Platinum · Gold · Silver · Bronze - awarded by your best delta margin."))
                    (card_meta("Format", "Open"))
                    (card_meta("Scoring", "Best Delta"))
                }
            }
        }
    }
}

pub fn mode_card_bomb() -> Markup {
    html! {
        div class="bg-surface-container-low border-l-4 border-[#f59e0b]" {
            div class="p-5" {
                div class="mb-4" {
                    h3 class="font-bold text-on-surface" { "Bomb Run" }
                    p class="text-sm text-outline uppercase tracking-wider" { "Game Mode" }
                }
                div class="space-y-4" {
                    (card_point("Survive", "Reach each checkpoint before the countdown expires or your run ends - BOOM."))
                    (card_point("Reset the Clock", "Every checkpoint buys you more time. Miss one and it's over."))
                    (card_point("Score by Checkpoints", "Most checkpoints wins. Survival time breaks ties."))
                    (card_meta("Format", "Open"))
                    (card_meta("Scoring", "Checkpoints"))
                }
            }
        }
    }
}

pub fn mode_card_shortcut() -> Markup {
    html! {
        div class="bg-surface-container-low border-l-4 border-[#22c55e]" {
            div class="p-5" {
                div class="mb-4" {
                    h3 class="font-bold text-on-surface" { "Shortcut" }
                    p class="text-sm text-outline uppercase tracking-wider" { "Game Mode" }
                }
                div class="space-y-4" {
                    (card_point("Hit the Checkpoints", "Race checkpoint-to-finish segments and post the fastest time to the persistent leaderboard."))
                    (card_point("Always On", "No rounds, no admin - drive whenever you want and climb the board."))
                    (card_meta("Format", "Open"))
                    (card_meta("Scoring", "Fastest"))
                }
            }
        }
    }
}

fn card_point(heading: &str, body: &str) -> Markup {
    html! {
        div {
            h4 class="font-bold uppercase text-on-surface mb-1" { (heading) }
            p class="text-sm text-on-surface-variant leading-relaxed" { (body) }
        }
    }
}

fn card_meta(label: &str, value: &str) -> Markup {
    html! {
        div {
            p class="text-sm uppercase text-outline block mb-1" { (label) }
            p class="font-bold uppercase text-on-surface" { (value) }
        }
    }
}

/// Mode info card matching an event's mode (detail sidebar).
pub fn mode_card_for(mode: &EventMode) -> Markup {
    match mode {
        EventMode::Metronome { .. } => mode_card_metronome(),
        EventMode::Shortcut => mode_card_shortcut(),
        EventMode::Bomb { .. } => mode_card_bomb(),
    }
}
