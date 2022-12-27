use std::collections::HashMap;

use crate::{config::definition::Config, insim_instance::Instance};

#[derive(Default)]
pub(crate) struct Manager {
    pub(crate) instances: HashMap<String, Instance>,
}

impl Manager {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn update_from_config(&mut self, config: &Config) {
        for server in config.servers.iter() {
            let existing = self.instances.get(&server.name);
            match existing {
                None => {
                    self.instances.insert(server.name.to_owned(), server.into());
                }
                Some(existing) => {
                    if &existing.config != server {
                        // FIXME implement config reloading
                        unimplemented!("reloading config isnt supported.. yet")
                    }
                }
            };
        }
    }
}
