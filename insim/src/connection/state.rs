use std::fmt::Display;

/// Internal Connection State.
#[derive(Debug, Default, Eq, PartialEq, Copy, Clone)]
pub enum State {
    #[default]
    Disconnected,
    Connected,
    Shutdown,
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Connected => write!(f, "Connected"),
            Self::Shutdown => write!(f, "Shutdown"),
        }
    }
}
