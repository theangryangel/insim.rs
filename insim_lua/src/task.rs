use crate::config::definition::Server;
use crate::script;
use convert_case::{Case, Casing};
use insim::client::prelude::*;
use miette::{Context, IntoDiagnostic, Result};
use mlua::{Function, Lua, LuaSerdeExt};
use std::fs;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

type TaskResult = (JoinHandle<Result<()>>, JoinHandle<Result<()>>);

pub(crate) fn spawn(server: &Server) -> Result<TaskResult> {
    let (lua_tx, mut lua_rx) = mpsc::unbounded_channel::<Event>();
    let (insim_tx, mut insim_rx) = mpsc::unbounded_channel::<Event>();

    let lua = Lua::new();

    lua.globals()
        .set("tracing", script::tracing::Tracing {})
        .into_diagnostic()?;

    let events = script::insim::Insim::new(server.name.clone(), insim_tx);
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

    let task_insim = tokio::task::spawn(async move {
        let mut client = conf.into_client();

        loop {
            tokio::select! {
                msg = client.next() => match msg {
                    Some(msg) => lua_tx.send(msg).into_diagnostic()?,
                    None => break
                },

                msg = insim_rx.recv() => match msg {
                    Some(msg) => match msg {
                        Event::Shutdown => {
                            client.shutdown();
                            break;
                        },
                        Event::Handshaking => unimplemented!(),
                        Event::Connected => unimplemented!(),
                        Event::Disconnected => unimplemented!(),
                        Event::Error(_) => unimplemented!(),
                        Event::Data(_) => todo!(),
                    },
                    None => break
                }
            }
        }

        Ok(())
    });

    Ok((task_insim, task_lua))
}
