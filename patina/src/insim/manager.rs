use std::collections::HashMap;

use crate::{
    config::definition::{Config, Server},
    insim::InsimHandle,
};

pub(crate) struct InsimManagerInfo {
    pub(crate) config: Server,
    pub(crate) handle: InsimHandle,
}

#[derive(Default)]
pub(crate) struct InsimManager {
    pub(crate) instances: HashMap<String, InsimManagerInfo>,
}

impl InsimManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn update_from_config(&mut self, config: &Config) {
        for server in config.servers.iter() {
            let existing = self.instances.get(&server.name);
            match existing {
                None => {
                    self.instances.insert(
                        server.name.to_owned(),
                        InsimManagerInfo {
                            config: server.clone(),
                            handle: server.into(),
                        },
                    );
                }
                Some(existing) => {
                    if existing.config != *server {
                        unimplemented!("reloading config isnt supported.. yet")
                    }
                }
            };
        }
    }
}
