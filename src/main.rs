#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
use fltk::button::Button;
use pretty_env_logger;
use regex::Regex;

#[macro_use]
extern crate log;
use fltk::app::{awake, App};
use fltk::enums::Color;
use fltk::enums::Event;
use fltk::window::DoubleWindow;
use fltk::{prelude::*, *};
use fltk_grid::Grid;

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

fn split_underscore(str: &String) -> (Option<&str>, Option<&str>) {
    let mut it = str.split("_");
    (it.next(), it.next())
}

fn get_array_of_2(object: &Value, key: &str) -> Option<(i64, i64)> {
    info!(
        " get array of 2 for '{}' in {:?}",
        key,
        object[key].as_sequence()
    );
    let field1 = object[key]
        .as_sequence()
        .and_then(|seq| Some(seq[0].clone()))
        .and_then(|v| v.as_i64());

    let field2 = object[key]
        .as_sequence()
        .and_then(|seq| Some(seq[1].clone()))
        .and_then(|v| v.as_i64());

    field1
        .and(field2)
        .and(Some((field1.unwrap(), field2.unwrap())))
}

fn get_pos(object: &Value) -> Option<(usize, usize)> {
    let pos = get_array_of_2(object, "pos");
    pos.and_then(|(v1, v2)| {
        let f1 = usize::try_from(v1);
        let f2 = usize::try_from(v2);
        if f1.is_ok() && f2.is_ok() {
            Some((f1.unwrap(), f2.unwrap()))
        } else {
            None
        }
    })
}

fn window_fill(grid: &mut Grid, config: BTreeMap<String, Value>) {
    for (config_key, config_value) in config.iter() {
        let mut position = (0, 0);
        let (widget, id) = split_underscore(config_key);
        let params = config_value;
        match widget.unwrap() {
            "frame" => {
                let (x, y) = get_pos(params).unwrap();
                grid.insert(&mut Button::default(), y, x)
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
struct PublishMessage {
    topic: String,
    message: String,
}

#[derive(Debug, Clone)]
enum RedisEvent {
    Publish { topic: String, message: String },
    Stop,
}

struct ButtonPub {
    button: Button,
    dst_topic: String,
    dst_value: String,
}

impl ButtonPub {
    fn new(_title: &str) -> ButtonPub {
        ButtonPub {
            button: Button::new(0, 0, 200, 30, "title"),
            dst_topic: String::new(),
            dst_value: String::new(),
        }
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 1)]
async fn main() {
    //    pretty_env_logger::init();

    let _ = SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());
    info!("Starting up. Reading config file {}.", PATH);

    let config = Box::new(load_yaml_file(PATH));

    let (mut tx_publish, mut rx_publish) = broadcast::channel::<RedisEvent>(16);
    let (tx_redis_cmd, rx_redis_cmd) = mpsc::channel::<RedisEvent>(16);
    // let rx1 = tx.subscribe();
    // let rx2 = tx.subscribe();

    let redis_config = config["redis"].clone();

    tokio::spawn(async move {
        redis(redis_config, tx_publish).await;
    });
    info!("Starting up fltk");

    let mut _app = app::App::default().with_scheme(app::Scheme::Gleam);
    let config = config.clone();
    let mut win = window::Window::default()
        .with_size(1024, 768)
        .center_screen()
        .with_label("Hello from rust");
    let mut grid = Grid::default_fill();
    grid.set_layout(16, 10);
    grid.debug(true);
    window_fill(&mut grid, *config);
    let mut _but = button::Button::default()
        .with_size(160, 30)
        .center_of(&win)
        .with_label("Click me!");

    _but.handle(|button, event| {
        info!("Button event {:?}", event);
        match event {
            Event::Push => button.set_color(Color::from_rgb(255, 0, 0)),
            _ => {}
        }
        true
    });
    win.end();
    win.show();
    // sleep
    let sub = rx_publish.resubscribe();
    while _app.wait() {
        while let Ok(x) = rx_publish.try_recv() {
            match x {
                RedisEvent::Publish { topic, message } => _but.set_label(topic.as_str()),
                _ => {}
            }
        }
    }
    //        _app.run().expect("app failed to run");
}

async fn redis(config: Value, tx_broadcast: broadcast::Sender<RedisEvent>) {
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
            awake();
            match tx_broadcast.send(RedisEvent::Publish {
                topic: msg.get_channel_name().to_string(),
                message: msg.get_payload().unwrap(),
            }) {
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
async fn receiver(mut rx: broadcast::Receiver<RedisEvent>, pattern: &str) {
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
            Ok(Ok(RedisEvent::Publish { topic, message })) => {
                if topic.starts_with(pattern) {
                    _time_last = std::time::Instant::now();
                    info!(
                        "|{:>20}| Widget pattern : {} topic: {}, message: {}",
                        thread::current().name().unwrap(),
                        pattern,
                        topic,
                        message
                    );
                }
            }
            Ok(Err(e)) => {
                error!("|{:>20}| error: {}", thread::current().name().unwrap(), e);
            }
            Ok(Ok(RedisEvent::Stop)) => {}
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
