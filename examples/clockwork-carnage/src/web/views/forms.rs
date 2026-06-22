//! Form field components — Maud port of `templates/partials/form.html`.
//! The Askama `{% macro %}` + `caller()` pattern becomes plain functions; the
//! `{% call select_field %}…options…{% endcall %}` body becomes a `Markup` arg.

use maud::{Markup, html};

/// Shared input border, swapped on validation error.
fn border(error: &str) -> &'static str {
    if error.is_empty() {
        "border-outline-variant focus:border-primary-container"
    } else {
        "border-error"
    }
}

fn label(name: &str, text: &str) -> Markup {
    html! {
        @if !text.is_empty() {
            label for=(name) class="text-sm uppercase tracking-widest text-outline block mb-2" { (text) }
        }
    }
}

fn error_line(error: &str) -> Markup {
    html! {
        @if !error.is_empty() { p class="mt-1 text-error" { (error) } }
    }
}

/// `<form>` wrapper with hidden CSRF field. Body is the caller's markup.
pub fn form(action: &str, csrf_token: &str, body: Markup) -> Markup {
    html! {
        form method="post" action=(action) {
            input type="hidden" name="csrf_token" value=(csrf_token);
            (body)
        }
    }
}

pub fn text_field(name: &str, lbl: &str, value: &str, placeholder: &str, error: &str) -> Markup {
    html! {
        (label(name, lbl))
        input id=(name) type="text" name=(name) value=(value) placeholder=(placeholder)
            class=(format!("bg-surface-container-low w-full px-4 py-3 text-on-surface border {} focus:outline-none transition-colors placeholder:text-outline/40", border(error)));
        (error_line(error))
    }
}

pub fn textarea_field(
    name: &str,
    lbl: &str,
    value: &str,
    rows: u32,
    placeholder: &str,
    error: &str,
) -> Markup {
    html! {
        (label(name, lbl))
        textarea id=(name) name=(name) rows=(rows) placeholder=(placeholder)
            class=(format!("bg-surface-container-low w-full px-4 py-3 text-on-surface border {} focus:outline-none transition-colors placeholder:text-outline/40 resize-none", border(error))) { (value) }
        (error_line(error))
    }
}

pub fn number_field(
    name: &str,
    lbl: &str,
    value: impl std::fmt::Display,
    min: &str,
    error: &str,
) -> Markup {
    html! {
        (label(name, lbl))
        input id=(name) type="number" name=(name) value=(value.to_string()) min=(min)
            class=(format!("bg-surface-container-low w-full px-4 py-3 text-on-surface border {} focus:outline-none transition-colors", border(error)));
        (error_line(error))
    }
}

pub fn datetime_local_field(name: &str, lbl: &str, value: &str, error: &str) -> Markup {
    html! {
        (label(name, lbl))
        input id=(name) type="datetime-local" name=(name) value=(value)
            class=(format!("bg-surface-container-low w-full px-4 py-3 text-on-surface border {} focus:outline-none transition-colors", border(error)));
        (error_line(error))
    }
}

/// `<select>` whose `<option>`s are supplied by the caller as `Markup`.
pub fn select_field(name: &str, lbl: &str, error: &str, options: Markup) -> Markup {
    html! {
        (label(name, lbl))
        select id=(name) name=(name)
            class=(format!("bg-surface-container-low w-full px-4 py-3 text-on-surface border {} focus:outline-none transition-colors appearance-none cursor-pointer", border(error))) {
            (options)
        }
        (error_line(error))
    }
}
