#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
use fltk::window::DoubleWindow;
use fltk::{prelude::*, *};
use log::{debug, error, info, trace, warn};
use redis::{Client, Commands, Connection, RedisResult};
use serde_yaml::Value;
use simplelog::SimpleLogger;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
mod limero;

// use the extension you require!
const PATH: &str = "src/mqtt.yaml";

fn load_yaml_file(path: &str) -> BTreeMap<String, Value> {
    let mut file = File::open(path).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");
    let v: BTreeMap<String, Value> = serde_yaml::from_str(&contents).expect("Unable to parse YAML");
    v
}

fn redis_config(config: &BTreeMap<String, Value>) -> RedisResult<Connection> {
    let redis_entry = config["redis"].as_mapping();
    match redis_entry {
        Some(_) => {
            let host = config["redis"]["host"].as_str().unwrap_or("localhost");
            let port = config["redis"]["port"].as_i64().unwrap_or(6379);
            let url = format!("redis://{}:{}/", host, port);
            info!("Redis url: {}", url);
            let client = redis::Client::open(url)?;
            let con = client.get_connection()?;
            Ok(con)
        }
        None => {
            error!("Redis config not found");
            Err(redis::RedisError::from((
                redis::ErrorKind::InvalidClientConfig,
                "Redis config not found",
            )))
        }
    }
}

fn window_fill(win: & mut DoubleWindow, config: BTreeMap<String, Value>) {
    for (config_key, config_value) in config.iter() {
        let mut position = (0, 0);
        match config_key.as_str() {
            "redis" => {
                info!("Redis config found");
            }
            "mqtt" => {
                info!("MQTT config found");
            }
            _ => {
                info!("Unknown config found");
            }
        }
    }
}

fn main() -> Result<(), Option<String> > {
    let _ = SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());
    info!("Starting up. Reading config file {}.", PATH);

    let config = load_yaml_file(PATH);

    let redis_connection = redis_config(&config);

    info!("Starting up fltk");

    let mut _app = app::App::default();
    let mut win = window::Window::default()
        .with_size(400, 300)
        .center_screen()
        .with_label("Hello from rust");
    window_fill(& mut  win, config);
    let mut _but = button::Button::default()
        .with_size(160, 30)
        .center_of(&win)
        .with_label("Click me!");
    win.end();
    win.show();
    // sleep
    _app.run().expect("app failed to run");
    Ok(())
}
