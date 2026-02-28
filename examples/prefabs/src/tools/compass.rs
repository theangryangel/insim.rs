use insim::{Colour, core::heading::Heading};

pub fn generate(heading: Heading, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    const LABELS: [&str; 8] = ["N", "NW", "W", "SW", "S", "SE", "E", "NE"];
    const DASHES_BETWEEN: usize = 4; // consistent spacing

    // build a uniform tape
    // each entry is a "point": [label] -> [dash] -> [dash] -> [dash] -> [dash]
    let mut tape: Vec<String> = Vec::new();
    for label in LABELS {
        tape.push(label.to_string());
        for _ in 0..DASHES_BETWEEN {
            tape.push("-".to_string());
        }
    }

    let tape_len = tape.len();
    let degrees = heading.normalize().to_degrees();

    // map degrees to the tape index
    // total indices = 8 labels + (8 * 4 dashes) = 40 total points
    let degrees_per_tick = 360.0 / tape_len as f64;
    let focus_idx = (degrees / degrees_per_tick).round() as usize % tape_len;

    // 3. Construct the window
    let mut output = String::new();
    let half_width = width / 2;

    for i in 0..width {
        // calculate which tick falls into this screen slot
        // we use a floating point offset if you want smooth sub-character scrolling,
        // but for a simple tape, index-based works best:
        let tape_idx = (focus_idx + tape_len + i).saturating_sub(half_width) % tape_len;
        let item = &tape[tape_idx];

        // highlight if it's the center item
        if i == half_width {
            output.push_str(&item.red().to_string());
        } else {
            output.push_str(&item.white().to_string());
        }
    }

    output
}
