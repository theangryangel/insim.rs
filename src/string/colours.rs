const COLOUR_SEQUENCES: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

/// Strip LFS colours
pub fn strip_colours(input: String) -> String {
    // FIXME: probably should make this a Cow and then return the Cow if nothing needs to be
    // changed?

    let mut output = String::with_capacity(input.len());
    let mut iter = input.chars();

    while let Some(i) = iter.next() {
        if i == '^' {
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

/// Americanised version of [strip_colours].
pub fn strip_colors(input: String) -> String {
    strip_colours(input)
}

/// Replace LFS colours with ANSI colours.
pub fn ansi_colours(input: String) -> String {
    let mut has_colours = false;

    let mut output = String::with_capacity(input.len());
    let mut iter = input.chars();

    while let Some(i) = iter.next() {
        if i == '^' {
            if let Some(j) = iter.next() {
                if COLOUR_SEQUENCES.contains(&j) {
                    if j == '9' || j == '8' {
                        // 9 is reset to default (inc codepage), 8 is 'default' colour only
                        output += "\x1b[0m";
                        has_colours = false;
                    } else {
                        has_colours = true;
                        // conveniently the colour code + 30 are the same as the ANSI codes
                        output += &format!("\x1b[0;3{}m", j);
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
