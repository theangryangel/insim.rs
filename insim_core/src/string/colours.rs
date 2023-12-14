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
