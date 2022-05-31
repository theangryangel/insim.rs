use crate::script_path::ScriptPath;
use insim::protocol::insim::InitFlags;
use std::default::Default;
use std::{fs, path};

#[derive(knuffel::Decode, Debug, Default)]
pub(crate) struct ServerFlags {
    #[knuffel(child)]
    pub(crate) multicar_info: bool,

    #[knuffel(child)]
    pub(crate) collisions: bool,

    #[knuffel(child)]
    pub(crate) object_collisions: bool,
}

impl ServerFlags {
    pub(crate) fn as_init_flags(&self) -> InitFlags {
        let mut flags = InitFlags::default();
        if self.multicar_info {
            flags |= InitFlags::MCI;
        }

        if self.collisions {
            flags |= InitFlags::CON;
        }

        if self.object_collisions {
            flags |= InitFlags::OBH;
        }

        flags
    }
}

#[derive(knuffel::Decode, Debug)]
pub(crate) struct Script {
    #[knuffel(argument)]
    pub(crate) path: ScriptPath,
}

#[derive(knuffel::Decode, Debug)]
pub(crate) struct Server {
    #[knuffel(argument)]
    pub(crate) name: String,

    #[knuffel(child, unwrap(argument))]
    pub(crate) hostname: String,

    #[knuffel(child, unwrap(argument))]
    pub(crate) password: Option<String>,

    #[knuffel(child)]
    pub(crate) relay: bool,

    #[knuffel(child)]
    pub(crate) uncompressed: bool,

    #[knuffel(child)]
    pub(crate) no_reconnect: bool,

    #[knuffel(child)]
    pub(crate) no_verify_version: bool,

    #[knuffel(child, default)]
    pub(crate) flags: ServerFlags,

    #[knuffel(child, unwrap(argument))]
    pub(crate) prefix: Option<String>,

    #[knuffel(child, unwrap(argument))]
    pub(crate) interval: Option<u16>,

    #[knuffel(child, unwrap(children(name = "path")))]
    pub(crate) scripts: Vec<Script>,
}

impl Server {
    pub(crate) fn as_insim_config(&self) -> insim::client::Config {
        let mut builder = insim::client::Config::default().tcp(self.hostname.clone());

        if let Some(password) = &self.password {
            builder = builder.password(password.to_string());
        }

        if self.uncompressed {
            builder = builder.use_uncompressed_header_byte();
        }

        if self.no_verify_version {
            builder = builder.verify_version(false);
        }

        if self.relay {
            builder = builder.relay(Some(self.hostname.clone()));
        }

        if self.no_reconnect {
            builder = builder.try_reconnect(false);
        }

        if let Some(interval) = &self.interval {
            builder = builder.interval(*interval);
        }

        if let Some(prefix) = &self.prefix {
            // TODO: Use let_chains when it's stable
            if !prefix.is_empty() {
                let mut chars: [u8; 1] = [0; 1];
                prefix.chars().next().unwrap().encode_utf8(&mut chars);
                builder = builder.prefix(chars[0]);
            }
        }

        builder = builder.set_flags(self.flags.as_init_flags());

        builder
    }
}

#[derive(knuffel::Decode, Debug)]
pub(crate) struct Config {
    #[knuffel(children(name = "server"))]
    pub(crate) servers: Vec<Server>,
}

pub(crate) fn read(config_path: &path::PathBuf) -> Config {
    // TODO: do not just process exit here. handle that in the caller.

    if !config_path.exists() {
        panic!("config file does not exist: {}", config_path.display());
    }

    let config_content = fs::read_to_string(config_path).unwrap();

    match knuffel::parse::<Config>(config_path.to_str().unwrap(), &config_content) {
        Ok(config) => config,
        Err(e) => {
            println!("{:?}", miette::Report::new(e));
            std::process::exit(1);
        }
    }
}
