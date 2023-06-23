#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
use fltk::window::DoubleWindow;
use fltk::{prelude::*, *};
use log::{debug, error, info, trace, warn};
use serde_yaml::Value;
use simplelog::SimpleLogger;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;
use std::sync::mpsc;
//use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
mod limero;
use limero::limero::Thread;
use limero::limero::ThreadCommand;
use limero::limero::Timer;
use limero::redis::Redis;
use crossbeam::channel::{bounded, unbounded, Receiver, Sender};

use crate::limero::redis::RedisCommand;


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

fn window_fill(win: &mut DoubleWindow, config: BTreeMap<String, Value>) {
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

fn main() -> Result<(), Option<String>> {
    let _ = SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());
    info!("Starting up. Reading config file {}.", PATH);

    let config = Box::new(load_yaml_file(PATH));

    let (redisSender, redisReceiver) = unbounded::<RedisCommand>();

    let redisConfig = config["redis"].clone();

    thread::spawn( move || {
        let mut thr = Arc::new(RefCell::new(Thread::new()));

        let mut redis = Redis::new(thr.clone(),redisReceiver);
        redis.connect();

        let tim = thr
            .borrow_mut()
            .new_timer(true, Duration::from_millis(1000));
        redis.config(redisConfig);
        thr.borrow_mut().run();
    });

    info!("Starting up fltk");

    let mut _app = app::App::default();
    let mut win = window::Window::default()
        .with_size(400, 300)
        .center_screen()
        .with_label("Hello from rust");
    window_fill(&mut win, config.as_ref().clone());
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
