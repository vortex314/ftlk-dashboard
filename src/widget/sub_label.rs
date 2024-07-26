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
use std::time::{Duration, Instant};
use std::time::SystemTime;

use crate::config::file_xml::WidgetParams;
use crate::pubsub::{payload_as_f64, payload_decode, PubSubEvent};
use crate::widget::hms;
use crate::widget::Context;
use crate::WidgetMsg;
use tokio::sync::mpsc;

use evalexpr::Value as V;
use evalexpr::*;

use super::PubSubWidget;

#[derive(Debug, Clone)]
pub struct SubLabel {
    value: f64,
    last_update: SystemTime,
    timeout : Duration,
    eval_expr: Option<Node>,
    cfg: WidgetParams,
    ctx: Context,
    frame: Option<fltk::frame::Frame>,
    
}

impl SubLabel {
    pub fn new(cfg: &WidgetParams) -> Self {
        Self {
            value: 0.0,
            last_update: std::time::UNIX_EPOCH,
            timeout : Duration::from_millis(cfg.timeout.unwrap_or(3) as u64),
            eval_expr: None,
            cfg: cfg.clone(),
            ctx: Context::new(),
            frame: None,
        }
    }
}

impl PubSubWidget for SubLabel {
    fn draw(&mut self) {
        let mut frame = fltk::frame::Frame::new(
            self.cfg.rect.x,
            self.cfg.rect.y,
            self.cfg.rect.w,
            self.cfg.rect.h,
            None,
        );
        frame.set_frame(FrameType::ThinUpBox);
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
                        info!("Dragging");
                        let (x1, y1) = app::event_coords();
                        (new_x, new_y) = (origins.0 + x1 - x, origins.1 + y1 - y);
                        w.set_pos(new_x, new_y);
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


                let topic_cfg = self.cfg.src_topic.as_ref().clone();
                if *topic == *topic_cfg.unwrap() {
                    self.last_update = SystemTime::now();
                    self.frame.as_mut().map(|f: &mut frame::Frame| f.set_color(Color::from_u32(0xFFFFFF)));
                    let _ = payload_as_f64(&payload).and_then(|v| {
                        self.value = v;
                        let binding = "".to_string();
                        let suffix = self.cfg.suffix.as_ref().unwrap_or(&binding);
                        let line = format!("{:.2}{}", self.value, suffix);
                        self.frame.as_mut().map(|f| f.set_label(&line));
                        Ok(())
                    });
                }
            }
            WidgetMsg::Tick => {
                let now = SystemTime::now();
                let elapsed = now.duration_since(self.last_update).unwrap();
                if elapsed > self.timeout {
                    self.frame.as_mut().map(|f| f.set_color(Color::from_u32(0x808080)));
                }
            }
        }
    }
}