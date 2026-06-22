//! Maud port of `templates/profile.html` (the `{% block content %}` body).
//! Wrapped by the shared `layout()` shell.

use maud::{Markup, html};

use super::{forms, layout};
use crate::web::{Changeset, handlers::ProfileInput, state::User};

pub fn profile(user: &User, csrf: &str, cs: &Changeset<ProfileInput>) -> Markup {
    let body = html! {
        div class="space-y-8" {

            section {
                h2 class="text-sm font-bold text-outline uppercase tracking-widest mb-1" { "Streaming Links" }
                p class=" text-outline mt-2 mb-6 leading-relaxed" {
                    "Your handles appear alongside your results so spectators can find your stream."
                }

                div class="space-y-4" {

                    // Twitch
                    div class="bg-surface-container-low border-l-4 border-[#9146ff] p-5" {
                        div class="flex items-center gap-3 mb-4" {
                            img src="/assets/twitch-icon.svg" class="w-5 h-5 shrink-0" alt="";
                            span class=" font-bold text-on-surface uppercase tracking-wider" { "Twitch" }
                        }
                        (forms::text_field("twitch_username", "Username", &cs.params.twitch_username, "your_handle", cs.error("twitch_username")))
                        p class="mt-2 text-[10px] uppercase tracking-widest text-outline" { "Without the @ sign. Leave blank to remove." }
                    }

                    // YouTube
                    div class="bg-surface-container-low border-l-4 border-[#ff0000] p-5" {
                        div class="flex items-center gap-3 mb-4" {
                            img src="/assets/youtube-icon.svg" class="w-5 h-5 shrink-0" alt="";
                            span class=" font-bold text-on-surface uppercase tracking-wider" { "YouTube" }
                        }
                        (forms::text_field("youtube_username", "Username", &cs.params.youtube_username, "your_handle", cs.error("youtube_username")))
                        p class="mt-2 text-[10px] uppercase tracking-widest text-outline" { "Without the @ sign. Leave blank to remove." }
                    }

                }
            }

            div class="flex items-center justify-end pt-6 border-t border-outline-variant/20" {
                button type="submit" class="px-8 py-3 bg-primary-container text-white font-bold uppercase tracking-wider  hover:bg-primary-container/90 transition-colors" {
                    "Save"
                }
            }

        }
    };

    let content = html! {
        // ── Header ──
        div class="border-b border-outline-variant/20" {
            div class="max-w-screen-xl mx-auto px-6 lg:px-12 py-6" {
                @match &user.uname {
                    Some(uname) => {
                        h1 class="text-3xl md:text-4xl font-black tracking-tight" {
                            span class="text-on-surface" { "HELLO " } span class="text-primary-container" { (uname) }
                        }
                    }
                    None => {
                        h1 class="text-3xl md:text-4xl font-black tracking-tight text-on-surface" { "PROFILE" }
                    }
                }
            }
        }

        // ── Form ──
        div class="max-w-screen-xl mx-auto px-6 lg:px-12 py-8" {
            div class="max-w-2xl" {
                (forms::form("/profile", csrf, body))
            }
        }
    };

    layout::layout("Profile - Clockwork Carnage", user, content)
}
