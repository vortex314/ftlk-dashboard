use app::fonts;
use draw::Rect;
use fltk::button::Button;
use fltk::draw::LineStyle;
use fltk::enums::Color;
use fltk::widget::Widget;
use fltk::{enums::*, prelude::*, *};
use serde_yaml::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;
use std::time::SystemTime;

use crate::config::file_xml::WidgetParams;
use crate::limero::{SinkRef, SinkTrait};
use crate::pubsub::{payload_as_f64, payload_decode, payload_encode, PubSubCmd, PubSubEvent};
use crate::widget::hms;
use crate::widget::Context;
use tokio::sync::mpsc;

use evalexpr::Value as V;
use evalexpr::*;

use super::{PubSubWidget, WidgetMsg};

#[derive( Clone)]
pub struct PubButton {
    value: f64,
    last_update: SystemTime,
    eval_expr: Option<Node>,
    cfg: WidgetParams,
    ctx: Context,
    frame : Option<fltk::frame::Frame>,
    pubsub_cmd : SinkRef<PubSubCmd>,
}

impl PubButton
 {
    pub fn new(cfg: &WidgetParams, pubsub_cmd:SinkRef<PubSubCmd>) -> Self {
        Self {
            value: 0.0,
            last_update: std::time::UNIX_EPOCH,
            eval_expr: None,
            cfg: cfg.clone(),
            ctx: Context::new(),
            frame: None,
            pubsub_cmd,
        }
    }
}

impl PubSubWidget for PubButton {
    fn draw(&mut self) {
        let mut frame = fltk::frame::Frame::new(
            self.cfg.rect.x,
            self.cfg.rect.y,
            self.cfg.rect.w,
            self.cfg.rect.h,
            None,
        );
//        frame.set_frame(FrameType::BorderBox);
        frame.set_align(Align::Center);
        let label = self.cfg.label.as_ref().unwrap().clone();
    //    self.cfg.label.as_ref().map(|s| frame.set_label(s.as_str()));
        let mut button = Button::default().with_size(self.cfg.rect.w-10, self.cfg.rect.h-5).center_of(&frame);
        button.set_color(Color::Blue);
        button.set_label_color(Color::White);
        self.cfg.label.as_ref().map(|s| button.set_label(s.as_str()));
        button.handle( {
            let pubsub_cmd = self.pubsub_cmd.clone();
            let dst_topic = self.cfg.dst_topic.as_ref().unwrap().clone();
            let on_value = self.cfg.on.clone();
            let off_value = self.cfg.off.clone();
            move |w, ev| match ev {
                enums::Event::Push => {
                    if app::event_mouse_button() == app::MouseButton::Left && on_value.is_some() {
                        pubsub_cmd.push(PubSubCmd::Publish {
                            topic: dst_topic.clone(),
                            payload: to_cbor(on_value.as_ref().unwrap()),
                        });
                        true
                    } else {
                        false
                    }
                }
                enums::Event::Released => {
                    if app::event_mouse_button() == app::MouseButton::Left && off_value.is_some() {
                        pubsub_cmd.push(PubSubCmd::Publish {
                            topic: dst_topic.clone(),
                            payload: to_cbor(off_value.as_ref().unwrap()),
                        });
                        true
                    } else {
                        false
                    }
                }
                _ev => false,
            }
        });


        let pubsub_cmd = self.pubsub_cmd.clone();
        let dst_topic = self.cfg.dst_topic.as_ref().unwrap().clone();
        let on_value = self.cfg.on.clone();
        let off_value = self.cfg.off.clone();

        frame.handle({
            move |w, ev| match ev {
                enums::Event::Push => {
                    if app::event_mouse_button() == app::MouseButton::Left && on_value.is_some() {
                        pubsub_cmd.push(PubSubCmd::Publish {
                            topic: dst_topic.clone(),
                            payload: to_cbor(on_value.as_ref().unwrap()),
                        });
                        true
                    } else {
                        false
                    }
                }
                enums::Event::Released => {
                    if app::event_mouse_button() == app::MouseButton::Left && off_value.is_some() {
                        pubsub_cmd.push(PubSubCmd::Publish {
                            topic: dst_topic.clone(),
                            payload: to_cbor(off_value.as_ref().unwrap()),
                        });
                        true
                    } else {
                        false
                    }
                }
                _ev => false,
            }
        });
        self.frame = Some(frame);

    }

    fn update(&mut self, event: & WidgetMsg) {
        match event {
            WidgetMsg::Pub { topic, payload } => {
                let topic_cfg = self.cfg.src_topic.as_ref().clone();
                if *topic == *topic_cfg.unwrap() {
                    info!("SubLabel: {:?}", payload);
                    let _ = payload_as_f64(&payload).and_then  (|v| {
                        self.value = v;
                        let binding = " ?? ".to_string();
                        let suffix = self.cfg.suffix.as_ref().unwrap_or(&binding);
                        let line = format!("{:.2} {}", self.value,suffix);
                        self.frame.as_mut().map(|f| f.set_label(&line));
                        Ok(())
                    });
                }
            }
            _ => {}
        }
    }
}


fn to_cbor(value: &str) -> Vec<u8> {
    if is_integer(value) {
        let v = value.parse::<i64>().unwrap();
        payload_encode(v)
    } else 
    if is_float(value) {
        let v = value.parse::<f64>().unwrap();
        payload_encode(v)
    } else if is_bool(value) {
        let v = value.parse::<bool>().unwrap();
        payload_encode(v)
    } else {
        payload_encode(value)
    }
}

fn is_bool(value: &str) -> bool {
    value == "true" || value == "false"
}

fn is_float(value: &str) -> bool {
    value.parse::<f64>().is_ok()
}

fn is_integer(value: &str) -> bool {
    value.parse::<i64>().is_ok()
}
