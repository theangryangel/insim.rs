pub use kitcar_macros::ParseChat as Parse;

pub trait Parse: Sized {
    fn parse(input: &str) -> Result<Self, ParseError>;
    fn help() -> Vec<&'static str>;
    fn prefix() -> Option<char>;
}

pub trait FromArg: Sized {
    fn from_arg(s: &str) -> Result<Self, String>;
}

impl FromArg for String {
    fn from_arg(s: &str) -> Result<Self, String> {
        Ok(s.to_string())
    }
}

impl FromArg for i32 {
    fn from_arg(s: &str) -> Result<Self, String> {
        s.parse()
            .map_err(|_| format!("'{}' is not a valid number", s))
    }
}

impl FromArg for f32 {
    fn from_arg(s: &str) -> Result<Self, String> {
        s.parse()
            .map_err(|_| format!("'{}' is not a valid number", s))
    }
}

impl FromArg for bool {
    fn from_arg(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "true" | "yes" | "1" => Ok(true),
            "false" | "no" | "0" => Ok(false),
            _ => Err(format!("'{}' is not a valid boolean", s)),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    EmptyInput,
    UnknownCommand(String),
    MissingRequiredArg(String),
    InvalidArg(String, String),
    MissingPrefix(char),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::EmptyInput => write!(f, "Empty input"),
            ParseError::UnknownCommand(cmd) => write!(f, "Unknown command: {}", cmd),
            ParseError::MissingRequiredArg(arg) => write!(f, "Missing required argument: {}", arg),
            ParseError::InvalidArg(arg, msg) => write!(f, "Invalid argument '{}': {}", arg, msg),
            ParseError::MissingPrefix(prefix) => write!(f, "Command must start with '{}'", prefix),
        }
    }
}
