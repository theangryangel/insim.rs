use super::MARKER;

pub const COLOUR_SEQUENCES: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

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
