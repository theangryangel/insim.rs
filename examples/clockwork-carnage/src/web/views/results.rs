//! Maud views — HTML as type-checked Rust functions.
//!
//! Spike: a Maud port of `templates/partials/event_results.html`. The same
//! `event_results` function is called from the Askama page (embedded) and from
//! the `/events/{id}/results` fragment route (standalone) — no `#[derive(Template)]`
//! struct, no separate file, no include/standalone split.

use maud::{Markup, html};

use super::components::player_name;
use crate::web::{filters, handlers::EventResults};

/// Metronome tier pill, by best delta in ms.
fn tier_badge(delta_ms: i64) -> Markup {
    html! {
        @if delta_ms <= 100 {
            span class="text-[10px] uppercase tracking-widest bg-primary-container/10 text-primary-container px-3 py-1 border border-primary-container/20" { "Platinum" }
        } @else if delta_ms <= 500 {
            span class="text-[10px] uppercase tracking-widest bg-[#fbbf24]/10 text-[#fbbf24] px-3 py-1 border border-[#fbbf24]/20" { "Gold" }
        } @else if delta_ms <= 2000 {
            span class="text-[10px] uppercase tracking-widest bg-surface-variant/40 text-outline px-3 py-1" { "Silver" }
        } @else if delta_ms <= 5000 {
            span class="text-[10px] uppercase tracking-widest bg-[#b45309]/10 text-[#b45309] px-3 py-1 border border-[#b45309]/20" { "Bronze" }
        }
    }
}

/// Shared rank `<td>` — primary colour for P1, muted otherwise.
fn rank_td(i: usize) -> Markup {
    let cls = if i == 0 {
        "px-4 py-3 font-black text-xl text-primary-container"
    } else {
        "px-4 py-3 font-black text-xl text-outline"
    };
    html! { td class=(cls) { (i + 1) } }
}

fn row_cls(i: usize) -> &'static str {
    if i == 0 {
        "border-t border-outline-variant/20 hover:bg-surface-container-high transition-colors bg-primary-container/5"
    } else {
        "border-t border-outline-variant/20 hover:bg-surface-container-high transition-colors"
    }
}

const TH: &str = "text-left px-4 py-3 text-[10px] font-bold text-outline uppercase tracking-widest";

/// The whole `#event-results` section. Call site decides embed vs fragment.
pub fn event_results(results: &EventResults) -> Markup {
    html! {
        section id="event-results" {
            @match results {
                EventResults::Metronome { standings, target_ms } => {
                    div class="bg-surface-container-high px-5 py-4 flex items-center justify-between border-l-4 border-[#3b82f6] mb-4" {
                        span class="text-sm uppercase tracking-[0.3em] text-outline" { "Target Time" }
                        span class="font-black tracking-tighter text-3xl text-[#3b82f6]" { (filters::time_ms(*target_ms)) }
                    }
                    div class="border-l-4 border-primary-container" {
                        @if standings.is_empty() {
                            div class="px-4 py-8 text-center text-outline" { "No results yet" }
                        } @else {
                            table class="w-full border-collapse" {
                                thead {
                                    tr class="bg-surface-container-high" {
                                        th class=(format!("{TH} w-16")) { "Rank" }
                                        th class=(TH) { "Driver" }
                                        th class=(format!("{TH} w-36")) style="text-align:right" { "Best Delta" }
                                        th class=(format!("{TH} w-28")) style="text-align:right" { "Tier" }
                                    }
                                }
                                tbody {
                                    @for (i, s) in standings.iter().enumerate() {
                                        tr class=(row_cls(i)) {
                                            (rank_td(i))
                                            td class="px-4 py-3" { (player_name(&s.pname, &s.uname, &s.twitch_username, &s.youtube_username)) }
                                            td class="px-4 py-3 text-right font-black tracking-tighter text-xl" { (filters::delta_ms(s.best_delta_ms)) }
                                            td class="px-4 py-3 text-right" { (tier_badge(s.best_delta_ms)) }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                EventResults::Shortcut { best, all } => {
                    div x-data="{ tab: 'best' }" {
                        div class="flex items-center border-b border-outline-variant/20 mb-4" {
                            button x-on:click="tab = 'best'"
                                x-bind:class="tab === 'best' ? 'text-primary-container border-b-2 border-primary-container -mb-px' : 'text-outline hover:text-on-surface'"
                                class="px-4 py-2 text-sm font-bold uppercase tracking-widest transition-colors" { "Best Times" }
                            button x-on:click="tab = 'all'"
                                x-bind:class="tab === 'all' ? 'text-primary-container border-b-2 border-primary-container -mb-px' : 'text-outline hover:text-on-surface'"
                                class="px-4 py-2 text-sm font-bold uppercase tracking-widest transition-colors" { "All Times" }
                        }
                        div x-show="tab === 'best'" {
                            (shortcut_table(best, false))
                        }
                        div x-show="tab === 'all'" x-cloak="" {
                            (shortcut_table(all, true))
                        }
                    }
                }

                EventResults::Bomb { best, all } => {
                    div x-data="{ tab: 'best' }" {
                        div class="flex items-center border-b border-outline-variant/20 mb-4" {
                            button x-on:click="tab = 'best'"
                                x-bind:class="tab === 'best' ? 'text-primary-container border-b-2 border-primary-container -mb-px' : 'text-outline hover:text-on-surface'"
                                class="px-4 py-2 text-sm font-bold uppercase tracking-widest transition-colors" { "Best Runs" }
                            button x-on:click="tab = 'all'"
                                x-bind:class="tab === 'all' ? 'text-primary-container border-b-2 border-primary-container -mb-px' : 'text-outline hover:text-on-surface'"
                                class="px-4 py-2 text-sm font-bold uppercase tracking-widest transition-colors" { "All Runs" }
                        }
                        div x-show="tab === 'best'" {
                            (bomb_table(best, false))
                        }
                        div x-show="tab === 'all'" x-cloak="" {
                            (bomb_table(all, true))
                        }
                    }
                }
            }
        }
    }
}

fn shortcut_table(rows: &[crate::db::ShortcutTime], show_set_at: bool) -> Markup {
    html! {
        div class="border-l-4 border-[#22c55e]" {
            @if rows.is_empty() {
                div class="px-4 py-8 text-center text-outline" { "No times recorded yet" }
            } @else {
                table class="w-full border-collapse" {
                    thead {
                        tr class="bg-surface-container-high" {
                            th class=(format!("{TH} w-16")) { "Rank" }
                            th class=(TH) { "Driver" }
                            th class=(format!("{TH} w-28")) { "Vehicle" }
                            th class=(format!("{TH} w-32")) style="text-align:right" { @if show_set_at { "Time" } @else { "Best Time" } }
                            @if show_set_at { th class=(format!("{TH} w-32")) style="text-align:right" { "Set At" } }
                        }
                    }
                    tbody {
                        @for (i, e) in rows.iter().enumerate() {
                            tr class=(row_cls(i)) {
                                (rank_td(i))
                                td class="px-4 py-3" { (player_name(&e.pname, &e.uname, &e.twitch_username, &e.youtube_username)) }
                                td class="px-4 py-3 text-sm uppercase tracking-widest text-outline" { (e.vehicle) }
                                td class="px-4 py-3 text-right font-black tracking-tighter text-xl" { (filters::time_ms(e.time_ms)) }
                                @if show_set_at {
                                    td class="px-4 py-3 text-right text-sm uppercase tracking-widest text-outline" {
                                        time datetime=(filters::iso8601(&e.set_at)) x-data="" x-localtime="" { (filters::timestamp_human(&e.set_at)) }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::MetronomeStanding;

    #[test]
    fn renders_metronome() {
        let results = EventResults::Metronome {
            target_ms: 83_400,
            standings: vec![
                MetronomeStanding {
                    pname: "^1Karl".into(),
                    uname: "karl".into(),
                    best_delta_ms: 87,
                    twitch_username: Some("karltv".into()),
                    youtube_username: None,
                },
                MetronomeStanding {
                    pname: "Bob".into(),
                    uname: "bob".into(),
                    best_delta_ms: 1_240,
                    twitch_username: None,
                    youtube_username: None,
                },
            ],
        };
        // Run with: cargo test -p clockwork-carnage renders_metronome -- --nocapture
        println!("{}", event_results(&results).into_string());
    }
}

fn bomb_table(rows: &[crate::db::BombRun], show_recorded: bool) -> Markup {
    html! {
        div class="border-l-4 border-[#f59e0b]" {
            @if rows.is_empty() {
                div class="px-4 py-8 text-center text-outline" { "No runs recorded yet" }
            } @else {
                table class="w-full border-collapse" {
                    thead {
                        tr class="bg-surface-container-high" {
                            th class=(format!("{TH} w-16")) { "Rank" }
                            th class=(TH) { "Driver" }
                            th class=(format!("{TH} w-28")) { "Vehicle" }
                            th class=(format!("{TH} w-32")) style="text-align:right" { "Checkpoints" }
                            th class=(format!("{TH} w-32")) style="text-align:right" { "Survival" }
                            @if show_recorded { th class=(format!("{TH} w-32")) style="text-align:right" { "Recorded" } }
                        }
                    }
                    tbody {
                        @for (i, e) in rows.iter().enumerate() {
                            tr class=(row_cls(i)) {
                                (rank_td(i))
                                td class="px-4 py-3" { (player_name(&e.pname, &e.uname, &e.twitch_username, &e.youtube_username)) }
                                td class="px-4 py-3 text-sm uppercase tracking-widest text-outline" { (e.vehicle) }
                                td class="px-4 py-3 text-right font-black tracking-tighter text-xl" { (e.checkpoint_count) }
                                td class="px-4 py-3 text-right font-black tracking-tighter text-xl" { (filters::time_ms(e.survival_ms)) }
                                @if show_recorded {
                                    td class="px-4 py-3 text-right text-sm uppercase tracking-widest text-outline" {
                                        time datetime=(filters::iso8601(&e.recorded_at)) x-data="" x-localtime="" { (filters::timestamp_human(&e.recorded_at)) }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
