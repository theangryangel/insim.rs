//! Page shell — Maud port of `base.html` (head + header/nav).

use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::web::state::User;

const TAILWIND_CONFIG: &str = r##"
tailwind.config = {
  theme: {
    extend: {
      colors: {
        "background":"#121212","on-background":"#ffffff","surface":"#121212",
        "surface-dim":"#0d0d0d","surface-bright":"#2a2a2a",
        "surface-container-lowest":"#0a0a0a","surface-container-low":"#1f1f1f",
        "surface-container":"#1f1f1f","surface-container-high":"#2a2a2a",
        "surface-container-highest":"#333333","surface-variant":"#333333",
        "primary":"rgb(255, 87, 23)","on-primary":"#ffffff",
        "primary-container":"rgb(255, 87, 23)","on-primary-container":"#ffffff",
        "primary-fixed-dim":"rgb(255, 87, 23)","secondary":"#a0a0a0",
        "secondary-container":"#2a2a2a","on-secondary-container":"#a0a0a0",
        "tertiary":"#a9c7ff","tertiary-container":"#3b82f6","on-surface":"#ffffff",
        "on-surface-variant":"#a0a0a0","outline":"#a0a0a0","outline-variant":"#333333",
        "error":"#dc2626","inverse-surface":"#ffffff","inverse-primary":"rgb(255, 87, 23)",
        "surface-tint":"rgb(255, 87, 23)",
      },
      fontFamily: {
        "headline":["Space Grotesk","sans-serif"],
        "body":["Inter","sans-serif"],
        "label":["Space Grotesk","sans-serif"],
      },
      borderRadius: {"DEFAULT":"0px","sm":"0px","md":"0px","lg":"0px","xl":"0px","2xl":"0px","full":"9999px"},
    },
  },
}
"##;

const STYLES: &str = r#"
[x-cloak] { display: none !important; }
@keyframes live-pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.4; } }
.live-indicator { animation: live-pulse 1.2s ease-in-out infinite; }
.racing-italic { font-style: italic; transform: skewX(-6deg); display: inline-block; }
.event-card { transition: transform 0.2s ease, box-shadow 0.2s ease; }
.event-card:hover { transform: translateY(-2px); box-shadow: 0 8px 25px rgba(0,0,0,0.4); }
.position-1 { color: #fbbf24; }
.position-2 { color: #94a3b8; }
.position-3 { color: #b45309; }
"#;

// Alpine `x-localtime` directive: renders timestamps in the viewer's locale and
// counts down events starting within 3h.
const LOCALTIME_DIRECTIVE: &str = r#"
document.addEventListener("alpine:init", () => {
  Alpine.directive("localtime", (el, _bindings, { cleanup }) => {
    const raw = el.getAttribute("datetime");
    if (!raw) return;
    const date = new Date(raw);
    if (isNaN(date.getTime())) return;
    if (!el.hasAttribute("title")) el.setAttribute("title", raw);
    const THREE_HOURS = 3 * 60 * 60 * 1000;
    let timer = null;
    function render() {
      const diffMs = date - Date.now();
      if (diffMs > 0 && diffMs <= THREE_HOURS) {
        const totalSecs = Math.ceil(diffMs / 1000);
        const h = Math.floor(totalSecs / 3600);
        const m = Math.floor((totalSecs % 3600) / 60);
        const s = totalSecs % 60;
        el.textContent = h > 0
          ? `in ${h}h ${String(m).padStart(2,"0")}m ${String(s).padStart(2,"0")}s`
          : m > 0 ? `in ${m}m ${String(s).padStart(2,"0")}s` : `in ${s}s`;
      } else {
        el.textContent = date.toLocaleString(undefined, { dateStyle: "short", timeStyle: "short" });
        if (diffMs <= 0 && timer) { clearInterval(timer); timer = null; }
      }
    }
    render();
    if (date > new Date()) timer = setInterval(render, 1000);
    cleanup(() => { if (timer) clearInterval(timer); });
  });
});
"#;

// phx-change via Alpine (Livewire-style): a `liveForm` component re-POSTs the
// whole form with `_event=change` on any change, then `Alpine.morph`es the
// server-rendered form back in — preserving focus and nested Alpine state. CSRF
// rides in the form's hidden field, so the urlencoded body passes csrf_protect.
const LIVE_FORM_COMPONENT: &str = r#"
document.addEventListener("alpine:init", () => {
  Alpine.data("liveForm", () => ({
    async refresh() {
      const body = new URLSearchParams(new FormData(this.$el));
      body.set("_event", "change");
      const res = await fetch(this.$el.action, {
        method: "POST",
        headers: { "Accept": "text/html" },
        body: body,
      });
      if (res.ok) Alpine.morph(this.$el, await res.text());
    },
  }));
});
"#;

fn nav(user: &User) -> Markup {
    html! {
        header class="sticky top-0 z-50 bg-surface" x-data="{ open: false }" {
            div class="flex items-center justify-between h-12 px-6 lg:px-12 max-w-screen-xl mx-auto" {
                nav class="hidden md:flex items-center gap-8" {
                    a href="/" class="text-sm font-semibold text-on-surface hover:text-primary-container transition-colors tracking-wider" { "HOME" }
                    a href="/events" class="text-sm font-semibold text-on-surface hover:text-primary-container transition-colors tracking-wider" { "EVENTS" }
                    a href="/about" class="text-sm font-semibold text-on-surface hover:text-primary-container transition-colors tracking-wider" { "ABOUT" }
                }
                button x-on:click="open = !open" class="md:hidden p-2 text-on-surface" aria-label="Toggle menu" {
                    svg x-show="!open" xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" { path stroke-linecap="square" stroke-linejoin="square" d="M4 6h16M4 12h16M4 18h16" {} }
                    svg x-show="open" x-cloak="" xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" { path stroke-linecap="square" stroke-linejoin="square" d="M6 18L18 6M6 6l12 12" {} }
                }
                div class="flex items-center gap-6" {
                    @match &user.uname {
                        Some(uname) => {
                            a href="/profile" class="text-sm font-semibold text-outline hover:text-on-surface transition-colors tracking-wider" { (uname) }
                            a href="/logout" class="text-sm font-semibold text-outline hover:text-on-surface transition-colors tracking-wider" { "LOGOUT" }
                        }
                        None => {
                            a href="/login" class="text-sm font-semibold text-outline hover:text-on-surface transition-colors tracking-wider" { "LOGIN" }
                        }
                    }
                }
            }
            div class="h-0.5 bg-primary-container" {}
            div x-show="open" x-cloak="" class="md:hidden bg-surface-container-low border-b border-outline-variant/20" {
                nav class="flex flex-col py-2" {
                    a href="/" x-on:click="open = false" class="px-6 py-3 text-sm font-semibold text-on-surface hover:bg-surface-container-high tracking-wider" { "HOME" }
                    a href="/events" x-on:click="open = false" class="px-6 py-3 text-sm font-semibold text-on-surface hover:bg-surface-container-high tracking-wider" { "EVENTS" }
                    a href="/about" x-on:click="open = false" class="px-6 py-3 text-sm font-semibold text-on-surface hover:bg-surface-container-high tracking-wider" { "ABOUT" }
                }
            }
        }
    }
}

/// Full HTML document. `content` is the page body (everything inside `<main>`).
pub fn layout(title: &str, user: &User, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) }
                link href="https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700;800;900&family=Inter:wght@300;400;500;600;700&display=swap" rel="stylesheet";
                script src="https://cdn.tailwindcss.com?plugins=forms,container-queries" {}
                script { (PreEscaped(TAILWIND_CONFIG)) }
                style { (PreEscaped(STYLES)) }
                script { (PreEscaped(LOCALTIME_DIRECTIVE)) }
                script { (PreEscaped(LIVE_FORM_COMPONENT)) }
                // Alpine morph plugin — must load before Alpine core. Powers the
                // Livewire-style `liveForm` re-render (server renders, Alpine morphs).
                script src="https://cdn.jsdelivr.net/npm/@alpinejs/morph@3/dist/cdn.min.js" defer {}
                script src="https://cdn.jsdelivr.net/npm/alpinejs@3/dist/cdn.min.js" defer {}
            }
            body class="bg-surface text-on-background font-headline antialiased min-h-screen flex flex-col" {
                (nav(user))
                main class="flex-1" { (content) }
            }
        }
    }
}
