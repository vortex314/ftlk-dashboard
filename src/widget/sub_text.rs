use fltk::button::Button;
use fltk::enums::Color;
use fltk::widget::Widget;
use fltk::{enums::*, prelude::*, *};
use serde_yaml::Value;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use std::time::SystemTime;

use crate::decl::DeclWidget;
use crate::pubsub::PubSubEvent;
use crate::widget::dnd_callback;
use crate::widget::GridRectangle;
use crate::widget::PubSubWidget;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct SubText {
    frame: frame::Frame,
    src_topic: String,
    src_prefix: String,
    src_suffix: String,
    src_timeout: u128,
    last_update: SystemTime,
    grid_rectangle: GridRectangle,
}

impl SubText {
    pub fn new() -> Self {
        info!("SubText::new()");
        let mut frame = frame::Frame::default().with_label("SubText");
        frame.set_frame(FrameType::BorderBox);
        frame.set_color(Color::from_u32(0x555555));
        frame.handle(move |w, ev| dnd_callback(&mut w.as_base_widget(), ev));
        SubText {
            frame,
            src_topic: "".to_string(),
            src_prefix: "".to_string(),
            src_suffix: "".to_string(),
            last_update: std::time::UNIX_EPOCH,
            src_timeout: 1000,
            grid_rectangle: GridRectangle::new(1, 1, 1, 1),
        }
    }
}

impl PubSubWidget for SubText {
    fn config(&mut self, props: Value) {
        info!("Status::config()");
        let w = props["size"][0].as_i64().unwrap() * 32;
        let h = props["size"][1].as_i64().unwrap() * 32;
        let x = props["pos"][0].as_i64().unwrap() * 32;
        let y = props["pos"][1].as_i64().unwrap() * 32;
        self.src_topic = props["src_topic"].as_str().unwrap_or("").to_string();
        self.src_prefix = props["src_prefix"].as_str().unwrap_or("").to_string();
        self.src_suffix = props["src_suffix"].as_str().unwrap_or("").to_string();
        self.src_timeout = props["src_timeout"].as_i64().unwrap_or(1000) as u128;
        self.frame.resize(x as i32, y as i32, w as i32, h as i32);
        props["label"].as_str().map(|s| self.frame.set_label(s));
        info!("Status size : {},{} pos : {},{} ", x, y, w, h);
    }
    fn on(&mut self, event: PubSubEvent) {
        match event {
            PubSubEvent::Publish { topic, message } => {
                if topic != self.src_topic {
                    return;
                }
                info!(
                    "Status::on() topic: {} vs src_topic : {}",
                    topic, self.src_topic
                );
                self.last_update = std::time::SystemTime::now();
                let text = format!("{}{}{}", self.src_prefix, message, self.src_suffix);
                self.frame.set_label(&text);
                self.frame.set_color(Color::from_hex(0x00ff00));
                self.frame.parent().unwrap().redraw();
            }
            PubSubEvent::Timer1sec => {
                let delta = std::time::SystemTime::now()
                    .duration_since(self.last_update)
                    .unwrap()
                    .as_millis();
                if delta > self.src_timeout {
                    info!("Status::on() {} Expired", self.src_topic);
                    self.frame.set_color(Color::from_hex(0xff0000));
                    self.frame.parent().unwrap().redraw();
                }
            }
        }
    }
    fn set_publish_channel(&mut self, channel: tokio::sync::mpsc::Sender<PubSubEvent>) {
        info!("Status::set_publish_channel()");
    }
}
