use itertools::Itertools;

use super::MARKER;

pub const COLOUR_SEQUENCES: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
const COLOUR_SEQUENCES_BYTES: &[u8] = &[b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9'];

/// Strip LFS colours
pub fn strip(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut iter = input.chars();

    while let Some(i) = iter.next() {
        if i == (MARKER as char) {
            if let Some(j) = iter.next() {
                if COLOUR_SEQUENCES.contains(&j) {
                    continue;
                }

                output.push(i);
                output.push(j);
                continue;
            }
        }

        output.push(i);
    }

    output
}

/// Replace LFS colours with ANSI colours.
/// Assumes that the input has already been unescaped. This usually happens automatically when
/// de-serialising from the wire.
pub fn to_ansi(input: &str) -> String {
    let mut has_colours = false;

    let mut output = String::with_capacity(input.len());

    // FIXME this should be peekable
    let mut iter = input.chars();

    while let Some(i) = iter.next() {
        if i == (MARKER as char) {
            if let Some(j) = iter.next() {
                if COLOUR_SEQUENCES.contains(&j) {
                    if j == '9' || j == '8' {
                        // 9 is reset to default (inc codepage), 8 is 'default' colour only
                        output += "\x1b[0m";
                        has_colours = false;
                    } else {
                        has_colours = true;
                        // conveniently the colour code + 30 are the same as the ANSI codes
                        output = format!("{output}\x1b[0;3{j}m");
                    }
                    continue;
                }

                output.push(i);
                output.push(j);
                continue;
            }
        }

        output.push(i);
    }

    if has_colours {
        output += "\x1b[0m";
    }

    output
}

pub fn to_html(input: &str) -> String {
    // FIXME this should be operating on characters, not bytes

    let input = input.as_bytes();

    let mut indices: Vec<usize> = input
        .iter()
        .tuple_windows()
        .positions(|(elem, next)| *elem == b'^' && COLOUR_SEQUENCES_BYTES.contains(next))
        .collect();

    // make sure we've got at least something in the indices
    if indices.first() != Some(&0) {
        indices.insert(0, 0);
    }

    // make sure we've got the last item in here as well
    match indices.last() {
        Some(last) => {
            if *last != input.len() {
                indices.push(input.len());
            }
        }
        None => indices.push(input.len()),
    };

    let mut output: Vec<String> = Vec::new();

    for pair in indices.windows(2) {
        if pair[0] == pair[1] {
            continue;
        }

        let range = &input[pair[0]..pair[1]];

        if range[0] != MARKER {
            output.push(std::str::from_utf8(range).unwrap().to_string());
            continue;
        }

        let colour = match &input[pair[0] + 1] {
            b'0' => "black",
            b'1' => "red",
            b'2' => "green",
            b'3' => "yellow",
            b'4' => "blue",
            b'5' => "purple",
            b'6' => "blue",
            b'7' => "white",
            b'8' => "default",
            b'9' => "default",
            _ => "black",
        };

        output.push(format!(
            "<span class='lfs-text-{}'>{}</span>",
            colour,
            // FIXME no unwraps, what if its <= 2 chars?
            std::str::from_utf8(&range[2..]).unwrap()
        ));
    }

    output.join("")
}
