//! Maud port of `templates/events.html` — the events list page.

use maud::{Markup, html};

use super::{components, layout::layout};
use crate::{
    db::{Era, Event},
    web::state::User,
};

// ── Filters + URL builders (ported from the old `EventsTemplate` methods) ──

/// The active list filters. Knows how to build `/events?…` URLs that toggle a
/// single facet while preserving the others.
pub struct Filters<'a> {
    pub status: &'a str,
    pub mode: &'a str,
    pub era: Option<i64>,
}

impl Filters<'_> {
    fn base(&self) -> Vec<String> {
        let mut params = vec![];
        if !self.status.is_empty() {
            params.push(format!("status={}", self.status));
        }
        if !self.mode.is_empty() {
            params.push(format!("mode={}", self.mode));
        }
        if let Some(era) = self.era {
            params.push(format!("era={era}"));
        }
        params
    }

    fn status_url(&self, s: &str) -> String {
        let mut params = self.base();
        params.retain(|p| !p.starts_with("status="));
        if !s.is_empty() {
            params.insert(0, format!("status={s}"));
        }
        build_url(params)
    }

    fn mode_url(&self, m: &str) -> String {
        let mut params = self.base();
        params.retain(|p| !p.starts_with("mode="));
        if !m.is_empty() {
            params.push(format!("mode={m}"));
        }
        build_url(params)
    }

    fn era_url(&self, era_id: Option<i64>) -> String {
        let mut params = self.base();
        params.retain(|p| !p.starts_with("era="));
        if let Some(id) = era_id {
            params.push(format!("era={id}"));
        }
        build_url(params)
    }

    fn page_url(&self, page: i64) -> String {
        let mut params = self.base();
        if page > 1 {
            params.push(format!("page={page}"));
        }
        build_url(params)
    }
}

fn build_url(params: Vec<String>) -> String {
    if params.is_empty() {
        "/events".to_string()
    } else {
        format!("/events?{}", params.join("&"))
    }
}

// ── Tailwind class fragments for the status tabs / mode pills ──

const STATUS_ACTIVE: &str = "text-primary-container border-b-2 border-primary-container -mb-px";
const STATUS_INACTIVE: &str = "text-outline hover:text-on-surface";

pub fn events(
    user: &User,
    events: &[Event],
    eras: &[Era],
    f: &Filters,
    page: i64,
    total_pages: i64,
) -> Markup {
    let status_tab = |label: &str, value: &str| -> Markup {
        let active = f.status == value;
        let cls = format!(
            "px-4 py-2 text-sm font-bold uppercase tracking-widest transition-colors {}",
            if active {
                STATUS_ACTIVE
            } else {
                STATUS_INACTIVE
            }
        );
        html! {
            a href=(f.status_url(value)) class=(cls) { (label) }
        }
    };

    let mode_pill = |label: &str, value: &str, active_cls: &str, inactive_cls: &str| -> Markup {
        let active = f.mode == value;
        let cls = format!(
            "px-3 py-1.5 text-sm font-bold uppercase tracking-widest transition-colors {}",
            if active { active_cls } else { inactive_cls }
        );
        html! {
            a href=(f.mode_url(value)) class=(cls) { (label) }
        }
    };

    let body = html! {
        main class="px-6 lg:px-12 py-10 max-w-screen-xl mx-auto" {

            // ── Page header ──
            div class="flex items-end justify-between mb-8 gap-4" {
                div {
                    h1 class="text-3xl md:text-4xl font-black tracking-tight mb-1" {
                        span class="text-on-surface" { "ALL " } span class="text-primary-container" { "EVENTS" }
                    }
                    p class=" text-outline" { "Every race, every result, every era." }
                }
                @if user.admin {
                    a href="/events/new" class="text-sm font-bold uppercase tracking-widest bg-primary-container text-white px-6 py-3 hover:bg-primary-container/90 transition-colors flex-shrink-0" {
                        "NEW EVENT"
                    }
                }
            }

            // ── Filters ──
            div class="flex flex-wrap items-center justify-between gap-y-3 mb-6" {

                // Status filter tabs
                div class="flex items-center border-b border-outline-variant/20" {
                    (status_tab("ALL", ""))
                    (status_tab("LIVE", "live"))
                    (status_tab("PENDING", "pending"))
                    (status_tab("COMPLETED", "completed"))
                }

                // Mode filter pills
                div class="flex items-center" {
                    (mode_pill("All", "", "bg-primary-container text-white", "bg-surface-container-low text-outline hover:text-on-surface hover:bg-surface-container-high"))
                    (mode_pill("Shortcut", "shortcut", "bg-[#22c55e] text-white", "bg-surface-container-low text-[#22c55e] hover:bg-surface-container-high"))
                    (mode_pill("Metronome", "metronome", "bg-[#3b82f6] text-white", "bg-surface-container-low text-[#3b82f6] hover:bg-surface-container-high"))
                    (mode_pill("Bomb", "bomb", "bg-[#f59e0b] text-white", "bg-surface-container-low text-[#f59e0b] hover:bg-surface-container-high"))

                    // Era filter
                    @if !eras.is_empty() {
                        select onchange="window.location.href=this.value"
                            class="bg-surface-container-low text-sm ml-2 px-3 py-1.5 font-bold uppercase tracking-widest text-outline focus:outline-none cursor-pointer appearance-none border-none" {
                            option value=(f.era_url(None)) selected[f.era.is_none()] { "All Eras" }
                            @for era in eras {
                                option value=(f.era_url(Some(era.id))) selected[f.era == Some(era.id)] { (era.name) }
                            }
                        }
                    }
                }
            }

            // ── Events table ──
            @if events.is_empty() {
                div class="flex flex-col items-center justify-center py-24 text-center" {
                    p class="text-lg font-bold text-on-surface mb-1" { "No events found" }
                    p class=" text-outline" { "Try adjusting the filters above." }
                }
            } @else {
                (components::event_list(events))
            }

            // ── Pagination ──
            @if total_pages > 1 {
                div class="flex items-center justify-center gap-3 mt-8 pt-6 border-t border-outline-variant/20" {
                    @if page > 1 {
                        a href=(f.page_url(page - 1))
                            class="px-5 py-2 text-sm font-bold uppercase tracking-widest text-outline border border-outline-variant/30 hover:border-primary-container hover:text-on-surface transition-colors" {
                            "<- PREV"
                        }
                    } @else {
                        span class="px-5 py-2 text-sm font-bold uppercase tracking-widest text-outline/30 border border-outline-variant/10" { "<- PREV" }
                    }

                    span class="text-sm text-outline uppercase tracking-widest" { "Page " (page) " of " (total_pages) }

                    @if page < total_pages {
                        a href=(f.page_url(page + 1))
                            class="px-5 py-2 text-sm font-bold uppercase tracking-widest text-outline border border-outline-variant/30 hover:border-primary-container hover:text-on-surface transition-colors" {
                            "NEXT ->"
                        }
                    } @else {
                        span class="px-5 py-2 text-sm font-bold uppercase tracking-widest text-outline/30 border border-outline-variant/10" { "NEXT ->" }
                    }
                }
            }
        }
    };

    layout("Events - Clockwork Carnage", user, body)
}
