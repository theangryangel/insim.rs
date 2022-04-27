use std::{fs, path};

#[derive(knuffel::Decode, Debug, Default)]
pub(crate) struct ServerFlags {
    #[knuffel(child)]
    multicar_info: bool,

    #[knuffel(child)]
    collisions: bool,

    #[knuffel(child)]
    object_collisions: bool,
}

#[derive(knuffel::Decode, Debug)]
pub(crate) struct Script {
    #[knuffel(argument)]
    path: path::PathBuf,
}

#[derive(knuffel::Decode, Debug)]
pub(crate) struct Server {
    #[knuffel(argument)]
    name: String,

    #[knuffel(child, unwrap(argument))]
    hostname: String,

    #[knuffel(child, unwrap(argument))]
    password: Option<String>,

    #[knuffel(child)]
    relay: bool,

    #[knuffel(child)]
    uncompressed: bool,

    #[knuffel(child)]
    no_reconnect: bool,

    #[knuffel(child)]
    no_verify_version: bool,

    #[knuffel(child, default)]
    flags: ServerFlags,

    #[knuffel(child, unwrap(argument))]
    prefix: Option<String>,

    #[knuffel(child, unwrap(argument))]
    interval: Option<u16>,

    #[knuffel(child, unwrap(children(name = "path")))]
    scripts: Vec<Script>,
}

#[derive(knuffel::Decode, Debug)]
pub(crate) struct Config {
    #[knuffel(children(name = "server"))]
    servers: Vec<Server>,
}

pub(crate) fn read(config_path: &path::PathBuf) -> Config {
    if !config_path.exists() {
        panic!("config file does not exist: {}", config_path.display());
    }

    let config_content = fs::read_to_string(config_path).unwrap();

    match knuffel::parse::<Config>(&config_path.to_str().unwrap(), &config_content) {
        Ok(config) => config,
        Err(e) => {
            println!("{:?}", miette::Report::new(e));
            std::process::exit(1);
        }
    }
}
