use insim::core::string::{colours::Colour, escaping::Escape};
use jiff_sqlx::Timestamp;

// ── Plain helpers ──────────────────────────────────────────────────────────
// Single source of truth for formatting. Callable directly from Maud views
// (`filters::time_ms(x)`) and wrapped as Askama filters below for `.html`
// templates (`{{ x|format_time_ms }}`).

pub fn timestamp_human(ts: &Timestamp) -> String {
    ts.to_jiff().strftime("%Y-%m-%d %H:%M UTC").to_string()
}

/// ISO 8601 string at second precision for use in `<time datetime="...">`.
pub fn iso8601(ts: &Timestamp) -> String {
    ts.to_jiff().strftime("%Y-%m-%dT%H:%M:%SZ").to_string()
}

pub fn time_ms(ms: i64) -> String {
    let minutes = ms / 60_000;
    let seconds = (ms % 60_000) / 1000;
    let millis = ms % 1000;
    format!("{minutes}:{seconds:02}.{millis:03}")
}

pub fn delta_ms(ms: i64) -> String {
    let total = ms.abs();
    let sign = if ms >= 0 { "+" } else { "-" };
    let seconds = total / 1000;
    let millis = total % 1000;
    format!("{sign}{seconds}.{millis:03}s")
}

/// Convert an LFS player name with colour markers into HTML `<span>` elements.
/// Returns raw HTML (already escaped per-chunk); splice into Maud with
/// `PreEscaped(..)` or into Askama with `|safe`.
pub fn colour_spans_html(s: &str) -> String {
    fn lfs_colour_class(c: u8) -> &'static str {
        match c {
            0 => "text-gray-900",
            1 => "text-red-500",
            2 => "text-green-500",
            3 => "text-amber-500",
            4 => "text-blue-500",
            5 => "text-purple-500",
            6 => "text-cyan-500",
            _ => "",
        }
    }

    fn html_escape(s: &str) -> String {
        s.chars()
            .fold(String::with_capacity(s.len()), |mut out, c| {
                match c {
                    '&' => out.push_str("&amp;"),
                    '<' => out.push_str("&lt;"),
                    '>' => out.push_str("&gt;"),
                    '"' => out.push_str("&quot;"),
                    '\'' => out.push_str("&#39;"),
                    c => out.push(c),
                }
                out
            })
    }

    let mut out = String::new();
    for (colour, chunk) in s.colour_spans() {
        let text = html_escape(&chunk.unescape());
        let class = lfs_colour_class(colour);
        if class.is_empty() {
            out.push_str(&text);
        } else {
            out.push_str(&format!("<span class=\"{class}\">{text}</span>"));
        }
    }
    out
}
