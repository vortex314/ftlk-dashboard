#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
use fltk::button::Button;
use fltk::frame::Frame;
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
use tokio::task::block_in_place;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;
//use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

use crossbeam::channel::{bounded, unbounded, Receiver, Sender};
use std::fmt::Error;
use std::thread::{self, sleep, Thread};
use tokio::sync::broadcast;
use tokio::time::{self, Duration};
use tokio::{sync::mpsc, task};
use tokio_stream::StreamExt;
mod redis_bridge;
use redis_bridge::{redis, PublishMessage, RedisCmd, RedisEvent};
mod my_logger;
use std::env;

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

fn get_size(object: &Value) -> Option<(i32, i32)> {
    let pos = get_array_of_2(object, "size");
    pos.and_then(|(v1, v2)| {
        let f1 = i32::try_from(v1);
        let f2 = i32::try_from(v2);
        if f1.is_ok() && f2.is_ok() {
            Some((f1.unwrap(), f2.unwrap()))
        } else {
            None
        }
    })
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

fn value_string_default(object: &Value, key: &str, default: &str) -> String {
    object[key]
        .as_str()
        .map(String::from)
        .unwrap_or(String::from(default))
}

trait PubSubWidget {
    fn new(
        grid: &mut Grid,
        config: &Value,
        tx_redis_cmd: Sender<RedisCmd>,
    ) -> Self where Self:Sized;
    fn on_publish(&mut self, topic: &String, message: &String);
}



struct PubButton {
    button: Button,
    dst_topic: String,
    label: String,
}

impl PubSubWidget for PubButton {
    fn new(
        grid: &mut Grid,
        config: &Value,
        tx_redis_cmd: Sender<RedisCmd>,
    ) -> PubButton {
        info!("creating PubButton");
        let mut button = Button::default();
        let dst_topic = value_string_default(config, "dst_topic", "unknown dst_topic");
        let label = value_string_default(config, "label", &dst_topic);
        button.set_label(&label);
        let (x, y) = get_pos(config).unwrap();
        let (wy, wx) = get_size(config).unwrap();
        grid.insert(&mut button, y, x);
        button.set_size(wx, wy);
        let mut topic = dst_topic.clone();

        button.set_callback(   move |_| {
            let message= String::from("1");
            info!("sending {} => {:?}", topic.clone(),message);
            let topic = topic.clone();
            let cmd = RedisCmd::Publish { topic, message };
            let _ = tx_redis_cmd.send(cmd);
        });

        PubButton { button,
            dst_topic,
            label,
        }
    }
    fn on_publish(&mut self, topic: &String, message: &String) {
    }
}



struct SubLabel {
    frame: Frame,
    src_topic: String,
    prefix: String,
    suffix: String,
}

impl PubSubWidget for SubLabel {
    fn new(
        grid: &mut Grid,
        config: &Value,
        tx_redis_cmd: Sender<RedisCmd>,
    ) -> SubLabel {
        info!("creating SubLabel");
        let mut frame = Frame::default();
        frame.set_label("_+_");
        let (x, y) = get_pos(config).unwrap();
        let (wy, wx) = get_size(config).unwrap();
        grid.insert(&mut frame, y, x);
        frame.set_size(wx, wy);

        SubLabel {
            frame,
            src_topic: value_string_default(config, "src_topic", ""),
            prefix: value_string_default(config, "prefix", ""),
            suffix: value_string_default(config, "suffix", ""),
        }
    }
    fn on_publish(&mut self, topic: &String, message: &String) {
        if self.src_topic == *topic {
            let l = format!("{}{}{}",self.prefix,message,self.suffix );
            self.frame.set_label(l.as_str());
        }
    }
}

fn window_fill(
    grid: &mut Grid,
    config: BTreeMap<String, Value>,
    tx_redis_cmd: Sender<RedisCmd>,
) -> Vec<Box<dyn PubSubWidget>> {
    let mut v:Vec<Box<dyn PubSubWidget>> = Vec::new();
    for (config_key, config_value) in config.iter() {
        let mut position = (0, 0);
        let (widget, id) = split_underscore(config_key);
        let params = config_value;
        info!(
            " creating {}___{}",
            widget.unwrap_or("unknown"),
            id.unwrap_or("unknown")
        );
        match widget.unwrap() {
            "label" => {
                v.push(Box::new(SubLabel::new(grid, params, tx_redis_cmd.clone())));
            }
            "button" => {
                v.push(Box::new(PubButton::new(grid, params, tx_redis_cmd.clone())));
            }
            _ => {}
        }
    }
    v
}

#[tokio::main(flavor = "multi_thread", worker_threads = 1)]
async fn main() {
    env::set_var("RUST_LOG", "info");
    my_logger::init();
    //   let _ = SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());
    info!("Starting up. Reading config file {}.", PATH);

    let config = Box::new(load_yaml_file(PATH));

    let (mut tx_publish, mut rx_publish) = broadcast::channel::<RedisEvent>(16);
    let (mut tx_redis_cmd, mut rx_redis_cmd) = bounded(16);

    let redis_config = config["redis"].clone();

    tokio::spawn(async move {
        redis(redis_config, tx_publish).await;
    });
    info!("Starting up fltk");

    let mut _app = app::App::default();
    let config = config.clone();
    let mut win = window::Window::default()
        .with_size(1024, 768)
        .center_screen()
        .with_label("Hello from rust");
    let mut grid = Grid::default_fill();
    grid.set_layout(16, 10);
    grid.debug(true);
    let mut widgets = window_fill(&mut grid, *config, tx_redis_cmd.clone());
    grid.end();
    win.end();
    win.show();

    let sub = rx_publish.resubscribe();
    while _app.wait() {
        while let Ok(x) = rx_publish.try_recv() {
            match x {
                RedisEvent::Publish { topic, message } => {
                    for w in & mut widgets {
                        w.on_publish(&topic,&message); 
                    }
                }
                _ => {}
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
                        "Widget pattern : {} topic: {}, message: {}",
                        pattern, topic, message
                    );
                }
            }
            Ok(Err(e)) => {
                error!(" error: {}", e);
            }
            Ok(Ok(RedisEvent::Stop)) => {}
            Err(e) => {
                error!("timeout : {} {} ", pattern, e);
            }
        }
    }
}
