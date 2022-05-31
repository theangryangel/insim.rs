use crate::config::Server;
use convert_case::{Case, Casing};
use insim::client::prelude::*;
use mlua::{Function, Lua, LuaSerdeExt, Table};
use std::fs;
use tokio::task::JoinHandle;

pub(crate) fn spawn(server: &Server) -> Result<(JoinHandle<()>, JoinHandle<()>), ()> {
    let lua = Lua::new();

    lua.load(include_str!("stdlib.lua"))
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

    {
        let table: Table = lua.globals().get("insim").unwrap();
        let emit: Function = table.get("_emit").unwrap();
        emit.call::<_, ()>(("startup", lua.to_value(&()).unwrap()))
            .unwrap();
    }

    let (lua_tx, mut lua_rx) = tokio::sync::mpsc::unbounded_channel();
    let (insim_tx, mut insim_rx) = tokio::sync::mpsc::unbounded_channel();

    let task_lua = tokio::task::spawn(async move {
        while let Some(msg) = lua_rx.recv().await {
            let table: Table = lua.globals().get("insim").unwrap();
            let emit: Function = table.get("_emit").unwrap();

            match msg {
                Event::Data(frame) => {
                    emit.call::<_, ()>((
                        frame.name().to_case(Case::Snake),
                        lua.to_value(&frame).unwrap(),
                    ))
                    .unwrap();

                    // echo it back to insim, for the lols
                    insim_tx.send(Event::Data(frame)).unwrap();
                }
                event => {
                    emit.call::<_, ()>((
                        event.name().to_case(Case::Snake),
                        lua.to_value(&()).unwrap(),
                    ))
                    .unwrap();
                }
            }
        }
    });

    let conf = server.as_insim_config();

    let task_insim = tokio::task::spawn(async move {
        let mut client = conf.into_client();

        loop {
            tokio::select! {
                msg = client.next() => match msg {
                    Some(msg) => lua_tx.send(msg).unwrap(),
                    None => break
                },

                msg = insim_rx.recv() => match msg {
                    Some(msg) => {
                        println!("PONG: {:?}", msg);
                    },
                    None => break
                }
            }
        }
    });

    Ok((task_insim, task_lua))
}
