//! Utilities for working with colours from Insim.

use std::borrow::Cow;

use if_chain::if_chain;

use super::control::ControlCharacter;

/// Trait to help identify colour markers/identifiers
/// Left public to allow users to implement their own variation on colour stripping or replacement.
/// i.e. ASCII or HTML.
pub trait Colour {
    /// Is this a supported colour control character within LFS
    fn is_lfs_colour(&self) -> bool;
}

impl Colour for char {
    fn is_lfs_colour(&self) -> bool {
        matches!(
            self,
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'
        )
    }
}

impl Colour for u8 {
    fn is_lfs_colour(&self) -> bool {
        (*self as char).is_lfs_colour()
    }
}

/// Trait to help build coloured strings. API is heavily inspired by colored-rs/colored.
pub trait Colourify {
    /// Make this black
    fn black(self) -> String;
    /// Make this red
    fn red(self) -> String;
    /// Make this light green
    fn light_green(self) -> String;
    /// Make this yellow
    fn yellow(self) -> String;
    /// Make this blue
    fn blue(self) -> String;
    /// Make this purple
    fn purple(self) -> String;
    /// Make this light blue
    fn light_blue(self) -> String;
    /// Make this white
    fn white(self) -> String;
    /// Make this dark green (default colour)
    fn dark_green(self) -> String;
}

impl Colourify for &str {
    fn black(self) -> String {
        format!("^0{}", self)
    }

    fn red(self) -> String {
        format!("^1{}", self)
    }

    fn light_green(self) -> String {
        format!("^2{}", self)
    }

    fn yellow(self) -> String {
        format!("^3{}", self)
    }

    fn blue(self) -> String {
        format!("^4{}", self)
    }

    fn purple(self) -> String {
        format!("^5{}", self)
    }

    fn light_blue(self) -> String {
        format!("^6{}", self)
    }

    fn white(self) -> String {
        format!("^7{}", self)
    }

    fn dark_green(self) -> String {
        format!("^9{}", self)
    }
}

/// Strip LFS colours
pub fn strip(input: &str) -> Cow<str> {
    if !input.chars().any(|c| c.is_lfs_control_char()) {
        return input.into();
    }

    let mut iter = input.chars().peekable();
    let mut output = String::with_capacity(input.len());

    while let Some(i) = iter.next() {
        let j = iter.peek();

        if_chain! {
            if i.is_lfs_control_char();
            if let Some(k) = j;
            if k.is_lfs_control_char();
            then {
                // Special case, ignore escaped markers (AKA ^^)
                // If we don't do this now, and just fall through the next check, something like ^^1
                // wont be handled correctly!

                output.push(i);
                output.push(*k);
                let _ = iter.next();
                continue;
            }
        }

        if_chain! {
            if i.is_lfs_control_char();
            if let Some(k) = j;
            if k.is_lfs_colour();
            then {
                let _ = iter.next();
                continue;
            }
        }

        output.push(i);
    }

    Cow::Owned(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_colours_only() {
        assert_eq!(strip("^1^2^3^4^5^6^7^8^9"), "");
    }

    #[test]
    fn test_strip_colours() {
        assert_eq!(strip("^1234^56789"), "2346789");
    }

    #[test]
    fn test_strip_colours_escaped() {
        assert_eq!(strip("^^1234^56789"), "^^12346789");
    }

    #[test]
    fn test_colourify() {
        assert_eq!(
            "^9The ^0quick ^1brown ^2fox ^3jumps ^4over ^5the ^6lazy ^7dog",
            format!(
                "{} {} {} {} {} {} {} {} {}",
                "The".dark_green(),
                "quick".black(),
                "brown".red(),
                "fox".light_green(),
                "jumps".yellow(),
                "over".blue(),
                "the".purple(),
                "lazy".light_blue(),
                "dog".white()
            )
        );
    }

    #[test]
    fn test_colourify_deref() {
        assert_eq!("^4Test", String::from("Test").blue());
    }
}
