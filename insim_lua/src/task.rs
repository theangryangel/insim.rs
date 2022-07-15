use crate::config::definition::Server;
use crate::script;
use bounded_vec_deque::BoundedVecDeque;
use convert_case::{Case, Casing};
use insim::client::prelude::*;
use insim::protocol::Packet;
use miette::{Context, IntoDiagnostic, Result};
use mlua::{Function, Lua, LuaSerdeExt};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;

pub(crate) type Task = (
    JoinHandle<Result<()>>,
    JoinHandle<Result<()>>,
    crate::state::State,
);

pub(crate) fn spawn(server: &Server) -> Result<Task> {
    let (lua_tx, mut lua_rx) = mpsc::unbounded_channel::<Event>();
    let (insim_tx, mut insim_rx) = mpsc::unbounded_channel::<Event>();

    let state = crate::state::State::new(insim_tx.clone());

    let lua = Lua::new();

    lua.globals()
        .set("tracing", script::tracing::Tracing {})
        .into_diagnostic()?;

    let events = script::insim::Insim::new(server.name.clone(), insim_tx, state.clone());
    lua.globals().set("insim", events).into_diagnostic()?;

    {
        // emitter helper - this is a hack to get around the fact that we can't easily call
        // insim:emit directly from rust, as far as as I can tell.
        // it is, however, helpful that we can used the named registry value
        let emit: Function = lua
            .load(
                r#"
                function(key, value)
                    insim:emit(key, value)
                end
            "#,
            )
            .eval()
            .into_diagnostic()?;

        lua.set_named_registry_value("insim_emit_helper", emit)
            .into_diagnostic()?;
    }

    for script in server.scripts.iter() {
        let content = fs::read_to_string(&script.path)
            .into_diagnostic()
            .wrap_err(format!("failed reading script: {}", script.path.display()))?;

        lua.load(&content)
            .exec()
            .into_diagnostic()
            .wrap_err(format!("failed loading script: {}", script.path.display()))?;
    }

    {
        let emit: Function = lua
            .named_registry_value("insim_emit_helper")
            .into_diagnostic()?;
        emit.call::<_, ()>(("startup",))
            .into_diagnostic()
            .wrap_err("failed calling startup")?;
    }

    let task_lua = tokio::task::spawn(async move {
        while let Some(msg) = lua_rx.recv().await {
            let emit: Function = lua
                .named_registry_value("insim_emit_helper")
                .into_diagnostic()?;

            match msg {
                Event::Data(frame) => {
                    emit.call::<_, ()>((
                        frame.name().to_case(Case::Snake),
                        lua.to_value(&frame).into_diagnostic()?,
                    ))
                    .into_diagnostic()?;
                }

                event => {
                    emit.call::<_, ()>((event.name().to_case(Case::Snake),))
                        .into_diagnostic()?;
                }
            }
        }

        Ok(())
    });

    let conf = server.as_insim_config().into_diagnostic()?;

    let task_insim = tokio::task::spawn({
        let state = state.clone();
        async move {
            let mut client = conf.into_client();

            loop {
                tokio::select! {
                    msg = client.next() => match msg {
                        Some(msg) => {
                            state.handle_event(msg.clone())?;
                            lua_tx.send(msg.clone()).into_diagnostic()?;
                        },
                        None => {
                            break;
                        }
                    },

                    msg = insim_rx.recv() => match msg {
                        Some(msg) => match msg {
                            Event::Shutdown => {
                                tracing::debug!("shuttingdown?");
                                client.shutdown();
                                break;
                            },
                            Event::Handshaking => unimplemented!(),
                            Event::Connected => unimplemented!(),
                            Event::Disconnected => unimplemented!(),
                            Event::Error(e) => {
                                panic!("{}", e)
                            },
                            Event::Data(data) => {
                                client.send(Event::Data(data)).await?;
                            },
                        },
                        None => {
                            break;
                        }
                    }
                }
            }

            tracing::debug!("loop ended?");

            Ok(())
        }
    });

    Ok((task_insim, task_lua, state))
}
