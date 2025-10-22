//! An implementation of a retained immediate mode UI that looks somewhat like React.
//! We diff the output to minimise the the button updates via insim.
//! The manager and player connection tasks run on it's own thread in the background, and each player
//! connection task is on the same tokio localset, allowing for !Send states within components, etc.
pub mod component;
pub mod component_state;
pub mod id_pool;
pub mod manager;
pub mod runtime;
pub mod scope;
pub mod vdom;

pub use component::{Component, ComponentPath};
pub use id_pool::ClickIdPool;
pub use kitcar_macros::component;
pub use manager::{Manager, ManagerHandle};
pub use runtime::{RenderDiff, Runtime};
pub use scope::Scope;
pub use vdom::Element;

const MAGIC_TEXT_RATIO: f32 = 0.2;

#[derive(Debug, Clone)]
pub struct WrapTextIter<'a> {
    remaining_text: &'a str,
    line_width: usize,
    lines_yielded: usize,
    max_lines: usize,
}

impl<'a> Iterator for WrapTextIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_text.is_empty() || self.lines_yielded >= self.max_lines {
            return None;
        }

        // Check for explicit newlines
        let mut scan_end_byte = self.remaining_text.len();
        if self.remaining_text.chars().count() > self.line_width {
            // This unwrap is safe because we've confirmed the character count is > line_width
            scan_end_byte = self
                .remaining_text
                .char_indices()
                .nth(self.line_width)
                .unwrap()
                .0;
        }

        if let Some(newline_byte_index) = &self.remaining_text[..scan_end_byte].find('\n') {
            self.lines_yielded += 1;
            let (line, rest) = self.remaining_text.split_at(*newline_byte_index);
            // Update remaining text to start *after* the newline character.
            self.remaining_text = &rest[1..];
            return Some(line);
        }

        self.lines_yielded += 1;

        // If the rest of the text fits on one line (and we know it has no early newline), this is the last piece.
        if self.remaining_text.chars().count() <= self.line_width {
            let line = self.remaining_text;
            self.remaining_text = ""; // Exhaust the iterator for the next call.
            return Some(line);
        }

        // Find whitespace to wrap at
        // Find the potential split point. We look for a space *before* the line_width limit.
        // We check a slice that is up to `line_width` characters long.
        let mut potential_end_byte = self.remaining_text.len();
        if let Some((byte_index, _)) = self.remaining_text.char_indices().nth(self.line_width) {
            // This is the byte index right at the character limit.
            potential_end_byte = byte_index;
        }

        let candidate_slice = &self.remaining_text[..potential_end_byte];

        // Find the last space within our candidate slice.
        let split_byte_index = if let Some(space_byte_index) = candidate_slice.rfind(' ') {
            // Found a space! This is our preferred split point.
            space_byte_index
        } else {
            // No space found, so we are forced to split the word at the character limit.
            potential_end_byte
        };

        // Create the line slice and update the remaining text state.
        let (line, rest) = self.remaining_text.split_at(split_byte_index);

        // The rest of the string needs to be trimmed of leading whitespace for the next iteration.
        self.remaining_text = rest.trim_start();

        // Return the current line, trimmed of any trailing space from the split.
        Some(line.trim_end())
    }
}

impl<'a> WrapTextIter<'a> {
    pub fn has_remaining_text(&self) -> bool {
        !self.remaining_text.is_empty()
    }
}

pub fn wrap_text<'a>(input: &'a str, height: u8, width: u8, max_lines: usize) -> WrapTextIter<'a> {
    let max_per_button = (width as f32 / (height as f32 * MAGIC_TEXT_RATIO)).floor();

    WrapTextIter {
        remaining_text: input,
        line_width: max_per_button as usize,
        lines_yielded: 0,
        max_lines,
    }
}
