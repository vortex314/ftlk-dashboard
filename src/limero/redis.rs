use redis::{Client, Commands, Connection, RedisResult};
use serde_yaml::Value;
use std::error::Error;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::{collections::BTreeMap, fmt::format};

use crate::limero::limero::{SafeThread, SafeTimer};
use crate::{Thread, Timer};
use crossbeam::channel::{bounded, unbounded, Receiver, Sender};
use log::{debug, error, info, trace, warn};
use std::time::Duration;

pub struct PublishMsg {
    topic: String,
    payload: String,
}

pub enum RedisCommand {
    Subscribe(String, Sender<PublishMsg>),
    Publish(String, String),
    Reconnect(),
}

pub struct Redis {
    host: String,
    port: u16,
    connect_timer: Option<SafeTimer>,
    connected: bool,
    rx: Receiver<RedisCommand>,
    thread: SafeThread,
}

impl<'a> Redis {
    pub fn new(thread: SafeThread,receiver : Receiver<RedisCommand>) -> Redis {
        Redis {
            host: String::from("localhost"),
            port: 6379,
            connect_timer: None,
            connected: false,
            rx:receiver,
            thread,
        }
    }

    pub fn config(&mut self, config: Value) {
        self.host = config["host"].as_str().unwrap_or("localhost").to_string();
        self.port = u16::try_from(config["port"].as_i64().unwrap_or(6379)).unwrap();
    }
    pub fn connect(&mut self) {
        info!("Connecting to redis");
        let t = self
            .thread
            .borrow_mut()
            .new_timer(true, Duration::from_millis(1000));
        t.borrow_mut()
            .on_expire(Arc::new(Mutex::new(Box::new(|| {
                info!("Timer expired");
                let url = format!("redis://{}:{}", self.host, self.port);
                info!("Connecting to redis at {}", url);
                let client = redis::Client::open(url).unwrap();
                let con = client.get_connection();
            }))));

        info!("Timer created");
        self.connect_timer = Some(t);
    }

}
