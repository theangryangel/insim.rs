//! Tests for `#[derive(ChatCommand)]`.

use insim_extra::chat::{ChatCommand, ChatParseError};

#[derive(Debug, PartialEq, ChatCommand)]
enum Cmd {
    /// Show the help dialog.
    Help,
    /// Kick a player
    /// by username.
    Kick {
        uname: String,
    },
    #[chat(alias = "b", alias = "kb")]
    Ban {
        uname: String,
        mins: Option<u32>,
    },
    Echo {
        msg: Option<String>,
    },
    Say {
        to: String,
        msg: String,
    },
    #[chat(rename = "tp")]
    Teleport {
        x: i32,
        y: i32,
    },
    ListPlayers,
    // Fields named like the generated locals must not break parsing.
    Shadow {
        args: u8,
        name: u8,
    },
}

fn parse(s: &str) -> Result<Cmd, ChatParseError> {
    s.parse()
}

#[test]
fn unit_variants() {
    assert_eq!(parse("help"), Ok(Cmd::Help));
    assert_eq!(parse("  HeLp  "), Ok(Cmd::Help));
    assert_eq!(parse("list_players"), Ok(Cmd::ListPlayers));
    assert_eq!(
        parse("help me"),
        Err(ChatParseError::TooMany { command: "help" })
    );
}

#[test]
fn required_and_rest() {
    assert_eq!(
        parse("kick bob"),
        Ok(Cmd::Kick {
            uname: "bob".into()
        })
    );
    // A last `String` takes the rest of the line.
    assert_eq!(
        parse("kick bob smith"),
        Ok(Cmd::Kick {
            uname: "bob smith".into()
        })
    );
    assert_eq!(
        parse("kick"),
        Err(ChatParseError::Missing {
            command: "kick",
            arg: "uname"
        })
    );
    assert_eq!(
        parse("say  bob   hello  there "),
        Ok(Cmd::Say {
            to: "bob".into(),
            msg: "hello  there".into()
        })
    );
}

#[test]
fn optional_and_aliases() {
    assert_eq!(
        parse("ban bob"),
        Ok(Cmd::Ban {
            uname: "bob".into(),
            mins: None
        })
    );
    assert_eq!(
        parse("KB bob 5"),
        Ok(Cmd::Ban {
            uname: "bob".into(),
            mins: Some(5)
        })
    );
    assert_eq!(
        parse("b bob soon"),
        Err(ChatParseError::Invalid {
            command: "ban",
            arg: "mins",
            value: "soon".into()
        })
    );
    assert_eq!(
        parse("ban bob 5 6"),
        Err(ChatParseError::TooMany { command: "ban" })
    );
    assert_eq!(parse("echo"), Ok(Cmd::Echo { msg: None }));
    assert_eq!(
        parse("echo  hi  all"),
        Ok(Cmd::Echo {
            msg: Some("hi  all".into())
        })
    );
}

#[test]
fn rename_and_shadowing() {
    assert_eq!(parse("tp 1 -2"), Ok(Cmd::Teleport { x: 1, y: -2 }));
    assert_eq!(
        parse("teleport 1 2"),
        Err(ChatParseError::Unknown("teleport".into()))
    );
    assert_eq!(parse("shadow 1 2"), Ok(Cmd::Shadow { args: 1, name: 2 }));
}

#[test]
fn errors() {
    assert_eq!(parse(""), Err(ChatParseError::Empty));
    assert_eq!(parse("   "), Err(ChatParseError::Empty));
    assert_eq!(parse("nope"), Err(ChatParseError::Unknown("nope".into())));
}

#[test]
fn usage() {
    let names: Vec<_> = Cmd::COMMANDS.iter().map(|c| c.name()).collect();
    assert_eq!(
        names,
        [
            "help",
            "kick",
            "ban",
            "echo",
            "say",
            "tp",
            "list_players",
            "shadow"
        ]
    );
    let ban = Cmd::usage("KB").expect("alias resolves");
    assert_eq!(ban.name(), "ban");
    assert_eq!(ban.aliases(), ["b", "kb"]);
    assert_eq!(ban.to_string(), "ban <uname> [mins]");
    assert_eq!(Cmd::usage("echo").unwrap().to_string(), "echo [msg...]");
    assert_eq!(Cmd::usage("say").unwrap().to_string(), "say <to> <msg...>");
    assert_eq!(Cmd::usage("help").unwrap().to_string(), "help");
    assert_eq!(Cmd::usage("help").unwrap().help(), "Show the help dialog.");
    assert_eq!(
        Cmd::usage("kick").unwrap().help(),
        "Kick a player by username."
    );
    assert_eq!(Cmd::usage("echo").unwrap().help(), "");
    assert!(Cmd::usage("nope").is_none());
}
