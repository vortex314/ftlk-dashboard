use app::fonts;
use draw::Rect;
use fltk::button::Button;
use fltk::draw::LineStyle;
use fltk::enums::Color;
use fltk::widget::Widget;
use fltk::{enums::*, prelude::*, *};
use rand::random;
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
use crate::WidgetMsg;
use tokio::sync::mpsc;

use evalexpr::Value as V;
use evalexpr::*;

use super::PubSubWidget;

#[derive(Clone)]
pub struct BrokerAlive {
    value: f64,
    last_update: SystemTime,
    eval_expr: Option<Node>,
    cfg: WidgetParams,
    ctx: Context,
    frame: Option<fltk::frame::Frame>,
    topic: String,
    sinkref_cmd : SinkRef<PubSubCmd>,
}

impl BrokerAlive {
    pub fn new(cfg: &WidgetParams,sinkref_cmd : SinkRef<PubSubCmd>) -> Self {
        // get random topic
        let topic = format!("dst/broker/alive/{}", random::<u32>());
        Self {
            value: 0.0,
            last_update: std::time::UNIX_EPOCH,
            eval_expr: None,
            cfg: cfg.clone(),
            ctx: Context::new(),
            frame: None,
            topic,
            sinkref_cmd,
        }
    }
}

impl PubSubWidget for BrokerAlive {
    fn draw(&mut self) {
        let mut frame = fltk::frame::Frame::new(
            self.cfg.rect.x,
            self.cfg.rect.y,
            self.cfg.rect.w,
            self.cfg.rect.h,
            None,
        );
        frame.set_frame(FrameType::BorderBox);
        frame.set_color(Color::from_u32(0xFFFFFF));
        frame.set_label_font(Font::HelveticaBold);
        frame.set_align(Align::Center);
        let label = self.cfg.label.as_ref().unwrap().clone();
        self.cfg.label.as_ref().map(|s| frame.set_label(s.as_str()));

        let origins = (self.cfg.rect.x, self.cfg.rect.y);
        frame.handle({
            let mut x = 0;
            let mut y = 0;
            let mut new_x = 0;
            let mut new_y = 0;
            let mut origins = origins;
            move |w, ev| match ev {
                enums::Event::Push => {
                    if app::event_mouse_button() == app::MouseButton::Right {
                        (x, y) = app::event_coords();
                        true
                    } else {
                        false
                    }
                }
                enums::Event::Drag => {
                    if app::event_mouse_button() == app::MouseButton::Right {
                        let (x1, y1) = app::event_coords();
                        (new_x, new_y) = (origins.0 + x1 - x, origins.1 + y1 - y);
                        true
                    } else {
                        false
                    }
                }
                enums::Event::Released => {
                    if app::event_mouse_button() == app::MouseButton::Right {
                        origins = (new_x, new_y);
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

    fn update(&mut self, event: &WidgetMsg) {
        match event {
            WidgetMsg::Pub { topic, payload } => {
                if *topic == self.topic {
                    self.frame.as_mut().map( |mut f| f.set_color(Color::from_u32(0x00FF00)));
                    self.last_update = SystemTime::now();
                }
            }
            WidgetMsg::Tick => {
                self.sinkref_cmd.push(PubSubCmd::Publish {
                    topic: self.topic.clone(),
                    payload: payload_encode("OK"),
                });
                if self.last_update.elapsed().unwrap().as_millis() > 1100 {
                    self.frame.as_mut().map( |mut f| f.set_color(Color::from_u32(0xFF0000)));
                } 
            }
        }
    }
}
