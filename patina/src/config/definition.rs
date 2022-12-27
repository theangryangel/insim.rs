use insim::protocol::insim::InitFlags;
use std::default::Default;
use std::path::PathBuf;

use super::path::Path;

#[derive(knuffel::Decode, Clone, Debug, Default, PartialEq, Eq)]
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

// #[derive(knuffel::Decode, Debug)]
// pub(crate) struct Script {
//     #[knuffel(argument)]
//     pub(crate) path: ScriptPath,
// }

#[derive(knuffel::Decode, Debug)]
pub(crate) struct Web {
    #[knuffel(child, unwrap(argument, str))]
    pub(crate) listen: std::net::SocketAddr,

    #[knuffel(child, unwrap(argument))]
    pub(crate) templates: Option<Path>,
}

impl Web {
    pub fn templates_to_path_buf(&self) -> PathBuf {
        if let Some(p) = &self.templates {
            p.to_path_buf()
        } else {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates")
        }
    }
}

impl Default for Web {
    fn default() -> Self {
        Self {
            listen: ([0, 0, 0, 0], 3000).into(),
            templates: None,
        }
    }
}

#[derive(knuffel::Decode, Clone, Debug, PartialEq, Eq)]
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
    //#[knuffel(child, unwrap(children(name = "path")))]
    //pub(crate) scripts: Vec<Script>,
}

impl Server {
    pub(crate) fn as_insim_config(&self) -> Result<insim::client::Config, insim::error::Error> {
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
            if let Some(c) = prefix.chars().next() {
                builder = builder.prefix(c);
            }
        }

        builder = builder.set_flags(self.flags.as_init_flags());

        Ok(builder)
    }
}

#[derive(knuffel::Decode, Debug)]
pub(crate) struct Config {
    #[knuffel(child)]
    pub(crate) web: Web,

    #[knuffel(children(name = "insim"))]
    pub(crate) servers: Vec<Server>,
}
