use crate::state::State;
use insim::client::Event;
use mlua::{AnyUserData, Function, Lua, Table, UserData, Value};
use std::collections::HashMap;
use tokio::sync::mpsc;

pub(crate) struct Insim {
    callbacks: HashMap<String, Vec<i32>>,
    instance: String,
    writer: mpsc::UnboundedSender<Event>,
    state: State,
}

impl Insim {
    pub(crate) fn new(
        instance: String,
        writer: mpsc::UnboundedSender<Event>,
        state: State,
    ) -> Self {
        Insim {
            callbacks: HashMap::new(),
            instance,
            writer,
            state,
        }
    }
}

impl UserData for Insim {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("instance", |_, this| Ok(this.instance.clone()));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("shutdown", |_, router: AnyUserData| {
            let router = router.borrow::<Insim>()?;
            router.writer.send(Event::Shutdown).unwrap();
            Ok(())
        });

        methods.add_function(
            "emit",
            |lua: &Lua, (router, key, value): (AnyUserData, String, Value)| {
                let callback_table = match router.get_user_value::<Option<Table>>()? {
                    Some(t) => t,
                    None => {
                        let table = lua.create_table()?;
                        router.set_user_value(table.clone())?;
                        table
                    }
                };

                // nothing to do
                if !callback_table.contains_key(&*key).unwrap() {
                    return Ok(());
                }

                let keyed_callback: Table = callback_table.get(&*key).unwrap();
                let router = router.borrow::<Insim>()?;

                if router.callbacks.contains_key(&key) {
                    let callbacks = router.callbacks.get(&key).unwrap();
                    for callback in callbacks {
                        keyed_callback
                            .get::<_, Function>(*callback)?
                            .call(value.clone())?;
                    }
                }

                Ok(())
            },
        );

        methods.add_function(
            "off",
            |_lua: &Lua, (router, key, idx): (AnyUserData, String, i32)| {
                let mut router = router.borrow_mut::<Insim>()?;

                match router.callbacks.get_mut(&key) {
                    Some(vec) => {
                        vec.retain(|&i| i != idx);
                    }
                    None => {}
                };

                Ok(())
            },
        );

        methods.add_function(
            "on",
            |lua: &Lua, (router, key, callback): (AnyUserData, String, Function)| {
                let callback_table = match router.get_user_value::<Option<Table>>()? {
                    Some(t) => t,
                    None => {
                        let table = lua.create_table()?;
                        router.set_user_value(table.clone())?;
                        table
                    }
                };

                let keyed_callback = match callback_table.get(&*key)? {
                    Some(f) => f,
                    None => {
                        let table = lua.create_table()?;
                        callback_table.set(&*key, table.clone())?;
                        table
                    }
                };

                let idx = keyed_callback.raw_len() + 1;
                keyed_callback.raw_insert(idx, callback)?;

                let mut router = router.borrow_mut::<Insim>()?;

                match router.callbacks.get_mut(&key) {
                    Some(vec) => vec.push(idx),
                    None => {
                        router.callbacks.insert(key, vec![idx]);
                    }
                };

                Ok(idx)
            },
        );
    }
}
