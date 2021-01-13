mod db;
mod fs;
mod handlers;
mod routes;

#[macro_use]
extern crate log;

use crate::db::Db;
use crate::fs::mount;
use anyhow::{Context, Result};
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use simplelog::{Config as SimplelogConfig, TermLogger, TerminalMode};
use std::thread;
use structopt::StructOpt;

#[derive(StructOpt)]
// #[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Cli {
    #[structopt(default_value = "./config.yaml")]
    config_path: String,
    #[structopt(default_value = "./plevy.db")]
    db_path: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    bevy_url: String,
    plex_url: String,
    plex_token: String,
    pub mount_point: String,
}

pub type Ino = u64;

#[tokio::main]
async fn main() -> Result<()> {
    TermLogger::init(
        if cfg!(debug_assertions) {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        },
        SimplelogConfig::default(),
        TerminalMode::Mixed,
    );

    info!("Plevy {} init", env!("TARGET"));

    let args = Cli::from_args();
    let config = load_config(args.config_path.as_str())?;
    let db = Db::new(args.db_path.as_str())?;

    // don't block main thread; we've got a filesystem to mount, samurai!
    let db2 = db.clone();
    tokio::spawn(async move {
        routes::serve(db2).await;
    });

    let mount_point = config.mount_point.clone();
    fs::mount(mount_point, db.clone());

    Ok(())
}

pub fn load_config(config_path: &str) -> Result<Config> {
    let config_contents = std::fs::read_to_string(config_path)
        .with_context(|| format!("Failed to load config file `{}`", config_path))?;

    let config: Config = serde_yaml::from_str(config_contents.as_str())
        .with_context(|| format!("Failed to parse config file `{}`", config_path))?;
    debug!("Loaded config: {:?}", config);

    Ok(config)
}
