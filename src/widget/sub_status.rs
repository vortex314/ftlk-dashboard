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
use crate::widget::GridRectangle;
use crate::widget::PubSubWidget;
use crate::widget::{dnd_callback, get_params};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct SubStatus {
    status_frame: frame::Frame,
    src_topic: String,
    last_update: SystemTime,
    src_timeout: u128,
    grid_rectangle: GridRectangle,
    alive: bool,
}

impl SubStatus {
    pub fn new() -> SubStatus {
        info!("Status::new()");
        let mut status_frame = frame::Frame::default().with_label("Status");
        status_frame.set_frame(FrameType::BorderBox);
        status_frame.set_color(Color::from_u32(0xff0000));
        status_frame.handle(move |w, ev| dnd_callback(&mut w.as_base_widget(), ev));
        SubStatus {
            status_frame,
            src_topic: "".to_string(),
            last_update: std::time::UNIX_EPOCH,
            src_timeout: 1000,
            grid_rectangle: GridRectangle::new(1, 1, 1, 1),
            alive: false,
        }
    }

    /*

    fn config_dialog(&mut self, w: &mut Widget, ev: Event) -> bool {
        match ev {
            enums::Event::Push => {
                if app::event_button() == 3 {
                    let mut win = window::Window::new(
                        app::event_x_root(),
                        app::event_y_root(),
                        400,
                        300,
                        "Dialog",
                    );
                    let mut main = group::Flex::default_fill().column();

                    let mut urow = group::Flex::default().row();
                    {
                        urow.set_pad(20);
                        frame::Frame::default()
                            .with_label("Source topic:")
                            .with_align(enums::Align::Inside | enums::Align::Right);
                        let username = input::Input::default();
                        urow.fixed(&username, 180);
                        urow.end();
                    }
                    main.fixed(&urow, 30);
                    win.end();
                    win.show();
                }
                true // Important! to make Drag work
            }
            _ => false,
        }
    }*/
}

impl PubSubWidget for SubStatus {
    fn config(&mut self, props: Value) {
        if let Some(pr) = get_params(props.clone()) {
            debug!("Status::config() {:?}", pr);
            if let Some(size) = pr.size {
                if let Some(pos) = pr.pos {
                    self.status_frame
                        .resize(pos.0 * 32, pos.1 * 32, size.0 * 32, size.1 * 32);
                }
            }
            pr.src_topic.map(|s| self.src_topic = s);
            pr.src_timeout.map(|i| self.src_timeout = i as u128);
            pr.label.map(|s| self.status_frame.set_label(s.as_str()));
        }
    }
    fn get_config(&self ) -> Value {
        let mut props = serde_yaml::Mapping::new();
        props.insert(
            serde_yaml::Value::String("type".to_string()),
            serde_yaml::Value::String("status".to_string()),
        );
        props.insert(
            serde_yaml::Value::String("src_topic".to_string()),
            serde_yaml::Value::String(self.src_topic.clone()),
        );
        let mut pos = serde_yaml::Mapping::new();
        pos.insert(
            serde_yaml::Value::String("x".to_string()),
            serde_yaml::Value::Number(serde_yaml::Number::from(self.status_frame.x())),
        );
        pos.insert(
            serde_yaml::Value::String("y".to_string()),
            serde_yaml::Value::Number(serde_yaml::Number::from(self.status_frame.y())),
        );
        let mut size = serde_yaml::Mapping::new();
        size.insert(
            serde_yaml::Value::String("w".to_string()),
            serde_yaml::Value::Number(serde_yaml::Number::from(self.status_frame.width())),
        );
        size.insert(
            serde_yaml::Value::String("h".to_string()),
            serde_yaml::Value::Number(serde_yaml::Number::from(self.status_frame.height())),
        );
        let mut pr = serde_yaml::Mapping::new();
        pr.insert(
            serde_yaml::Value::String("pos".to_string()),
            serde_yaml::Value::Mapping(pos),
        );
        pr.insert(
            serde_yaml::Value::String("size".to_string()),
            serde_yaml::Value::Mapping(size),
        );
        props.insert(
            serde_yaml::Value::String("params".to_string()),
            serde_yaml::Value::Mapping(pr),
        );
        serde_yaml::Value::Mapping(props)
    }

    fn on(&mut self, event: PubSubEvent) {
        match event {
            PubSubEvent::Publish { topic, message } => {
                if topic != self.src_topic {
                    return;
                }
                self.last_update = std::time::SystemTime::now();
                if !self.alive {
                    debug!("Status::on() {} Alive", self.src_topic);
                    self.alive = true;
                    self.status_frame.set_color(Color::from_hex(0x00ff00));
                    self.status_frame.parent().unwrap().redraw();
                }
            }
            PubSubEvent::Timer1sec => {
                let delta = std::time::SystemTime::now()
                    .duration_since(self.last_update)
                    .unwrap()
                    .as_millis();
                if delta > self.src_timeout {
                    if self.alive {
                        debug!("Status::on() {} Expired", self.src_topic);
                        self.alive = false;
                        self.status_frame.set_color(Color::from_hex(0xff0000));
                        self.status_frame.parent().unwrap().redraw();
                    }
                }
            }
        }
    }
    fn set_publish_channel(&mut self, channel: tokio::sync::mpsc::Sender<PubSubEvent>) {
        info!("Status::set_publish_channel()");
    }
}
