use fltk::button::Button;
use fltk::enums::Color;
use fltk::widget::Widget;
use fltk::{enums::*, prelude::*, *};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use std::time::SystemTime;
use serde_yaml::Value;

use crate::decl::DeclWidget;
use crate::pubsub::PubSubEvent;
use crate::widget::dnd_callback;
use crate::widget::GridRectangle;
use crate::widget::PubSubWidget;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct Status {
    status_frame: frame::Frame,
    src_topic: String,
    last_update: SystemTime ,
    src_timeout: u128,
    grid_rectangle: GridRectangle,
}

impl Status {
    pub fn new() -> Status {
        info!("Status::new()");
        let mut status_frame = frame::Frame::default().with_label("Status");
        status_frame.set_frame(FrameType::BorderBox);
        status_frame.set_color(Color::from_u32(0x00ff00));
        status_frame.handle(move |w, ev| {
            dnd_callback(&mut w.as_base_widget(), ev)
        });
        Status {
            status_frame,
            src_topic: "".to_string(),
            last_update: std::time::UNIX_EPOCH,
            src_timeout: 1000,
            grid_rectangle: GridRectangle::new(1,1,1,1),
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

impl PubSubWidget for Status {
     fn config(&mut self,props: Value)  {
        info!("Status::config()");
        let w = props["size"][0].as_i64().unwrap() * 32;
        let h = props["size"][1].as_i64().unwrap() * 32;
        let x = props["pos"][0].as_i64().unwrap() * 32;
        let y = props["pos"][1].as_i64().unwrap() * 32;
        self.src_topic = props["src_topic"].as_str().unwrap().to_string();
        props["src_timeout"].as_i64().map(|i| self.src_timeout = i as u128);
        self.status_frame.resize(x as i32,y as i32,w as i32,h as i32);
        props["label"].as_str().map(|s| self.status_frame.set_label(s));
        info!("Status size : {},{} pos : {},{} ", x,y,w,h);
    }

    fn on(&mut self, event: PubSubEvent) {
        match event {
            PubSubEvent::Publish { topic, message } => {
                if topic != self.src_topic {
                    return;
                }
                info!("Status::on() topic: {} vs src_topic : {}", topic, self.src_topic);
                self.last_update=std::time::SystemTime::now();
                self.status_frame.set_color(Color::from_hex(0x00ff00));
                self.status_frame.parent().unwrap().redraw();
            }
            PubSubEvent::Timer1sec => {
                let delta = std::time::SystemTime::now()
                    .duration_since(self.last_update)
                    .unwrap()
                    .as_millis();
                if delta > self.src_timeout {
                    info!("Status::on() {} Expired",self.src_topic);
                    self.status_frame.set_color(Color::from_hex(0xff0000));
                    self.status_frame.parent().unwrap().redraw();
                }
            }
        }
    }
    fn set_publish_channel(&mut self, channel: tokio::sync::mpsc::Sender<PubSubEvent>) {
        info!("Status::set_publish_channel()");
    }
}
