use clap::Parser;
use convert_case::{Case, Casing};
use mlua::{Function, Lua, LuaSerdeExt, Table, UserData};
use std::{fs, path};

use serde::{Deserialize, Serialize};

mod config;

/// insim_lua does stuff
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    config: path::PathBuf,

    /// Script(s) to load
    #[clap(short, long, multiple_values = true)]
    script: Vec<path::PathBuf>,
}

#[tokio::main]
pub async fn main() {
    let args = Args::parse();

    let config = config::read(&args.config);

    println!("{:?}", config);

    std::process::exit(0);

    let lua = Lua::new();
    lua.load(include_str!("insim.lua"))
        .exec()
        .expect("Error loading core insim.lua");

    for script in &args.script {
        if !script.exists() {
            println!("{} does not exist", script.display());
            return;
        }

        let init = fs::read_to_string(script);

        if init.is_err() {
            println!("Could not read file: {}", init.unwrap_err());
            return;
        }
        let res = lua.load(&init.unwrap()).exec();

        if res.is_err() {
            println!("Error loading script: {}", res.unwrap_err());
            return;
        }
    }

    let table: Table = lua.globals().get("insim").unwrap();
    let emit: Function = table.get("_emit").unwrap();

    let client = insim::client::Config::default()
        .relay()
        .try_reconnect(true)
        .try_reconnect_attempts(2000)
        .build();

    emit.call::<_, ()>("hello_world").unwrap();

    while let Some(m) = client.next().await {
        println!("{:?}", m);

        match m {
            insim::client::Event::State(insim::client::State::Connected) => {
                let _ = client
                    .send(
                        insim::protocol::relay::HostSelect {
                            hname: "(FM) GTi Thursday".into(),
                            ..Default::default()
                        }
                        .into(),
                    )
                    .await;
            }

            insim::client::Event::Frame(frame) => {
                emit.call::<_, ()>((
                    frame.name().to_case(Case::Snake),
                    lua.to_value(&frame).unwrap(),
                ))
                .unwrap();
            }

            _ => {}
        }
    }
}
