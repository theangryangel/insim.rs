//! Chat commands

// Just derive and you're done!
#[derive(Debug, PartialEq, kitcar::chat::Parse)]
#[chat(prefix = '!')]
/// Chat Commands
pub enum Chat {
    /// Echo a string back from the server
    Echo { message: String },
    /// Quit the insim application
    Quit,
    /// Only valid during Idle scene - start the game
    Start,
    /// Display the rules
    Rules,
    /// Display the Message of the Day
    Motd,
    /// Help
    Help,
}
