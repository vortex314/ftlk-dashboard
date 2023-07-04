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

pub enum RedisEvent {
    Connected(),
    Disconnected(),
    Publish( String,  String),
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
    rx_event: Receiver<RedisEvent>,
    tx_event: Sender<RedisEvent>,
    rx: Receiver<RedisCommand>,
    thread: SafeThread,
    subscriptions: Vec<String>,
    client: Option<redis::Client>,
    connection: Option<redis::Connection>,
}

impl<'a> Redis {
    pub fn new(thread: SafeThread, receiver: Receiver<RedisCommand>) -> Redis {
        let (tx_event, rx_event) = bounded(100);
        Redis {
            host: String::from("localhost"),
            port: 6379,
            connect_timer: None,
            connected: false,
            rx_event,
            tx_event,
            rx: receiver,
            thread,
            subscriptions: Vec::new(),
            client: None,
            connection: None,
        }
    }

    pub fn config(&mut self, config: Value) {
        self.host = config["host"].as_str().unwrap_or("localhost").to_string();
        self.port = u16::try_from(config["port"].as_i64().unwrap_or(6379)).unwrap();
    }
    pub fn connect(&mut self) {
        info!("Connecting to redis");
        let url = format!("redis://{}:{}?resp3=true", self.host, self.port);
        info!("Connecting to redis at {}", url);
        self.client = Some(redis::Client::open(url).unwrap());
        self.connection = Some(self.client.as_ref().unwrap().get_connection().unwrap());
        info!("Connected to redis");
        self.connected = true;
        self.tx_event.send(RedisEvent::Connected()).unwrap();
        let _r1 = self.connection.as_ref().unwrap().set_read_timeout(Some(Duration::from_millis(100)));
        let _r2 = self.connection.as_ref().unwrap().set_write_timeout(Some(Duration::from_millis(100)));
        let v = redis::cmd("hello").query::<redis::Value>(self.connection.as_mut().unwrap());
  //      info!("Redis hello {:?}", v.unwrap());
        let r = redis::cmd("hello").arg("3").query::<redis::Value>(self.connection.as_mut().unwrap());
        info!("Redis hello {:?}", r.unwrap());
    }

    fn run(&mut self) {
        self.tx_event.send(RedisEvent::Disconnected()).unwrap();
        loop {
            match self.rx_event.recv() {
                Ok(RedisEvent::Connected()) => {}
                Ok(RedisEvent::Disconnected()) => {
                    self.connect();
                }
                Ok(RedisEvent::Publish(topic,payload)) => {}
                _ => {}
            }
        }
    }
}
