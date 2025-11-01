//! Chat commands

// Just derive and you're done!
#[derive(Debug, PartialEq, kitcar::chat::ChatCommands)]
#[allow(missing_docs)]
pub enum MyChatCommands {
    Echo { message: String },
    Quit,
    Start,
    Rules,
    Motd,
    Help,
}
