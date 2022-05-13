use crate::config::{Config, Server};
use convert_case::{Case, Casing};
use mlua::{Function, Lua, LuaSerdeExt, Table};
use pin_project::pin_project;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, fs};
use tokio::task::JoinHandle;

use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug, Default)]
pub(crate) struct Connection {
    pub(crate) uname: String,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct State {
    pub(crate) connections: Arc<Mutex<Vec<Connection>>>,
}

pub(crate) fn spawn(server: &Server) -> Result<(State, insim::client::Client), ()> {
    let lua = Lua::new();

    lua.load(include_str!("insim.lua"))
        .exec()
        .expect("Error loading core insim.lua");

    {
        let table: Table = lua.globals().get("insim").unwrap();
        table.set("instance", server.name.clone()).unwrap();
    }

    for script in server.scripts.iter() {
        let content = fs::read_to_string(&script.path);

        if content.is_err() {
            println!("Could not read file: {}", content.unwrap_err());
            return Err(());
        }

        let res = lua.load(&content.unwrap()).exec();

        if res.is_err() {
            println!("Error loading script: {}", res.unwrap_err());
            return Err(());
        }
    }

    let state = State::default();

    let insim_client = server.as_insim_client_builder().build();

    return Ok((state, insim_client));

    // let handle = tokio::spawn(async move {
    //     while let Some(m) = insim_client.next().await {
    //         println!("{:?}", m);
    //
    //         match m {
    //
    //             insim::client::Event::State(insim::client::State::Connected) => {
    //                 insim_client.send(insim::protocol::insim::Tiny{
    //                     subtype: insim::protocol::insim::TinyType::Ncn,
    //                     ..Default::default()
    //                 }.into()).await;
    //             },
    //
    //             insim::client::Event::Frame(frame) => {
    //                 let table: Table = lua.globals().get("insim").unwrap();
    //                 let emit: Function = table.get("_emit").unwrap();
    //
    //                 emit.call::<_, ()>((
    //                     frame.name().to_case(Case::Snake),
    //                     lua.to_value(&frame).unwrap(),
    //                 ))
    //                 .unwrap();
    //
    //                 match frame {
    //                     insim::protocol::Packet::NewConnection(data) => {
    //                         let mut connections = inner_state.connections.lock().unwrap();
    //                         connections.push(Connection {
    //                             uname: data.uname.clone(),
    //                         });
    //                     },
    //                     _ => {}
    //                 };
    //
    //             }
    //
    //             _ => {}
    //         }
    //     }
    // });

    // Ok((state, handle))
}
