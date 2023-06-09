extern crate log;
use log::{debug, error, info, trace, warn};
use serde_yaml::Value;

use fltk::app::{awake, App};
use crossbeam::channel::{bounded, unbounded, Receiver, Sender};
use std::fmt::Error;
use std::thread::{self, sleep, Thread};
use tokio::sync::broadcast;
use tokio::time::{self, Duration};
use tokio::{sync::mpsc, task};
use tokio_stream::StreamExt;

#[derive(Debug, Clone)]
pub struct PublishMessage {
    topic: String,
    message: String,
}

#[derive(Debug, Clone)]
pub enum RedisEvent {
    Publish { topic: String, message: String },
    Stop,
}

pub enum RedisCmd {
    Stop,
    Publish { topic: String, message: String },
    Subscribe { topic : String }
}

pub async fn redis(config: Value, tx_broadcast: broadcast::Sender<RedisEvent>) {
    loop {
        let url = format!(
            "redis://{}:{}/",
            config["host"].as_str().or(Some("localhost")).unwrap(),
            config["port"].as_str().or(Some("6379")).unwrap()
        );
        let client = redis::Client::open(url.clone()).unwrap();
        info!(
            "Redis connecting {} ...  ",
            url
        );
        let connection = client.get_async_connection().await.unwrap();
        let mut pubsub = connection.into_pubsub();
    //    let redis_cmd_channel = connection.into_monitor();
        pubsub.psubscribe("*").await.unwrap();

        let mut pubsub_stream = pubsub.into_on_message();
      /*   tokio::spawn(async move {
            while let Some(cmd) = rx_cmd.recv().await {
                match cmd {
                    RedisCmd::Stop => {
                        info!("RedisCmd::Stop");
                        return;
                    }
                    RedisCmd::Publish { topic, message } => {
                        info!("RedisCmd::Publish");
                        let _ : () = redis::cmd("PUBLISH").arg(topic).arg(message).query_async(&mut pubsub).await.unwrap();
                    }
                    RedisCmd::Subscribe { topic } => {
                        info!("RedisCmd::Subscribe");
                        let _ : () = redis::cmd("PSUBSCRIBE").arg(topic).query_async(&mut pubsub).await.unwrap();
                    }
                }
            }
        }
        );*/

        while let Some(msg) = pubsub_stream.next().await {
            info!(
                "Redis topic: {}",
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