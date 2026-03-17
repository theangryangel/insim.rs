use insim::core::string::{colours::Colour, escaping::Escape};

#[askama::filter_fn]
pub fn format_time_ms(ms: &i64, _env: &dyn askama::Values) -> askama::Result<String> {
    let ms = *ms;
    let minutes = ms / 60_000;
    let seconds = (ms % 60_000) / 1000;
    let millis = ms % 1000;
    Ok(format!("{minutes}:{seconds:02}.{millis:03}"))
}

#[askama::filter_fn]
pub fn format_delta_ms(ms: &i64, _env: &dyn askama::Values) -> askama::Result<String> {
    let ms = *ms;
    let total = ms.abs();
    let sign = if ms >= 0 { "+" } else { "-" };
    let seconds = total / 1000;
    let millis = total % 1000;
    Ok(format!("{sign}{seconds}.{millis:03}s"))
}

/// Convert an LFS player name with colour markers into HTML `<span>` elements.
/// Must be used with `|safe` in templates.
#[askama::filter_fn]
pub fn colour_html(s: &str, _env: &dyn askama::Values) -> askama::Result<String> {
    fn lfs_colour_class(c: u8) -> &'static str {
        match c {
            0 => "text-gray-900",
            1 => "text-red-500",
            2 => "text-green-500",
            3 => "text-amber-500",
            4 => "text-blue-500",
            5 => "text-purple-500",
            6 => "text-cyan-500",
            _ => "", // 7 (white) and 8 (default): inherit parent colour
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
    Ok(out)
}
