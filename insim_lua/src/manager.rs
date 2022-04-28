use crate::config::{Config, Server};
use convert_case::{Case, Casing};
use mlua::{Function, Lua, LuaSerdeExt, Table};
use std::{collections::HashMap, fs};
use tokio::task::JoinHandle;

pub(crate) struct Manager {
    pub(crate) instances: HashMap<String, JoinHandle<()>>,
}

impl Manager {
    pub(crate) fn new(config: Config) -> Self {
        let mut manager = Self {
            instances: HashMap::new(),
        };

        for server in config.servers.iter() {
            manager.add_instance(server);
        }

        manager
    }

    pub(crate) fn add_instance(&mut self, server: &Server) {
        // TODO: add miette integration and error handling

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
                return;
            }

            let res = lua.load(&content.unwrap()).exec();

            if res.is_err() {
                println!("Error loading script: {}", res.unwrap_err());
                return;
            }
        }

        let insim_client = server.as_insim_client_config().build();

        let handle = tokio::spawn(async move {
            while let Some(m) = insim_client.next().await {
                println!("{:?}", m);

                match m {
                    insim::client::Event::Frame(frame) => {
                        let table: Table = lua.globals().get("insim").unwrap();
                        let emit: Function = table.get("_emit").unwrap();

                        emit.call::<_, ()>((
                            frame.name().to_case(Case::Snake),
                            lua.to_value(&frame).unwrap(),
                        ))
                        .unwrap();
                    }

                    _ => {}
                }
            }
        });

        self.instances.insert(server.name.clone(), handle);
    }

    pub(crate) async fn run(&mut self) {
        futures::future::join_all(self.instances.values_mut()).await;
    }
}
