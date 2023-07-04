#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
use pretty_env_logger;
#[macro_use]
extern crate log;
use fltk::window::DoubleWindow;
use fltk::{prelude::*, *};
use log::{debug, error, info, trace, warn};
use serde_yaml::Value;
use simplelog::SimpleLogger;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;
//use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

mod limero;
use crossbeam::channel::{bounded, unbounded, Receiver, Sender};
use limero::limero::Thread as LimeroThread;
use limero::limero::ThreadCommand;
use limero::limero::Timer;
use limero::redis::Redis;
use std::fmt::Error;
use std::thread::{self, sleep, Thread};
use tokio::sync::broadcast;
use tokio::time::{self, Duration};
use tokio::{sync::mpsc, task};
use tokio_stream::StreamExt;

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

#[derive(Debug, Clone)]
struct PublishMessage {
    topic: String,
    message: String,
}

#[derive(Debug, Clone)]
enum Event {
    Publish(PublishMessage),
}

#[tokio::main]
async fn main() {
    //    pretty_env_logger::init();

    let _ = SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());
    info!("Starting up. Reading config file {}.", PATH);

    let config = Box::new(load_yaml_file(PATH));

    let (tx, rx) = broadcast::channel::<Event>(16);
    //    let (tx_publish, rx_publish) = mpsc::channel::<Event>();
    // let rx1 = tx.subscribe();
    // let rx2 = tx.subscribe();

    let redis_config = config["redis"].clone();
    tokio::spawn(async move {
        redis(redis_config, tx).await;
    });
    info!("Starting up fltk");

    loop {
        let config = config.clone();
        let mut _app = app::App::default();
        let mut win = window::Window::default()
            .with_size(400, 300)
            .center_screen()
            .with_label("Hello from rust");
        window_fill(&mut win, *config);
        let mut _but = button::Button::default()
            .with_size(160, 30)
            .center_of(&win)
            .with_label("Click me!")
            .handle(|button,event| {
                info!("Button clicked");
                true
            })
            ;
        win.end();
        win.show();
        // sleep
        while _app.wait() {
            info!("app wait");
        }
        _app.run().expect("app failed to run");
    }
}

async fn redis(config: Value, tx_broadcast: broadcast::Sender<Event>) {
    loop {
        let url = format!(
            "redis://{}:{}/",
            config["host"].as_str().or(Some("localhost")).unwrap(),
            config["port"].as_str().or(Some("6379")).unwrap()
        );
        let client = redis::Client::open(url.clone()).unwrap();
        info!(
            "|{:>20}| Redis connecting {} ...  ",
            thread::current().name().unwrap(),
            url
        );
        let connection = client.get_async_connection().await.unwrap();
        let mut pubsub = connection.into_pubsub();
        pubsub.psubscribe("*").await.unwrap();

        let mut pubsub_stream = pubsub.into_on_message();

        while let Some(msg) = pubsub_stream.next().await {
            info!(
                "|{:>20}| Redis topic: {}",
                thread::current().name().unwrap(),
                msg.get_channel_name().to_string(),
            );
            match tx_broadcast.send(Event::Publish(PublishMessage {
                topic: msg.get_channel_name().to_string(),
                message: msg.get_payload().unwrap(),
            })) {
                Ok(_) => {}
                Err(e) => {
                    error!("|{:>20}| error: {}", thread::current().name().unwrap(), e);
                    break;
                }
            }
        }
    }
}

// async channel receiver
async fn receiver(mut rx: broadcast::Receiver<Event>, pattern: &str) {
    let mut duration: Duration;
    const MAX_TIME: std::time::Duration = std::time::Duration::from_secs(10);
    let mut _time_last = std::time::Instant::now();
    let mut _alive: bool;

    loop {
        if _time_last.elapsed() > MAX_TIME {
            warn!(
                "|{:>20}| Widget pattern : {} timeout ==========> ",
                thread::current().name().unwrap(),
                pattern
            );
            _alive = false;
            duration = Duration::from_millis(1000);
        } else {
            _alive = true;
            duration = MAX_TIME - _time_last.elapsed()
        }
        let event = time::timeout(duration, rx.recv()).await;
        match event {
            Ok(Ok(Event::Publish(msg))) => {
                if msg.topic.starts_with(pattern) {
                    _time_last = std::time::Instant::now();
                    info!(
                        "|{:>20}| Widget pattern : {} topic: {}, message: {}",
                        thread::current().name().unwrap(),
                        pattern,
                        msg.topic,
                        msg.message
                    );
                }
            }
            Ok(Err(e)) => {
                error!("|{:>20}| error: {}", thread::current().name().unwrap(), e);
            }
            Err(e) => {
                error!(
                    "|{:>20}| timeout : {} {} ",
                    thread::current().name().unwrap(),
                    pattern,
                    e
                );
            }
        }
    }
}
