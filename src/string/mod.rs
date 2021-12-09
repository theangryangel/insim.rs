//! Utilities for working with various strings from Insim.

mod escaping;
pub use escaping::*;

pub mod colours;

mod codepage_string;
pub use codepage_string::CodepageString;

pub mod istring;
