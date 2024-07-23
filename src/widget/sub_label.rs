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
use crate::pubsub::PubSubEvent;
use crate::widget::hms;
use crate::widget::Context;
use tokio::sync::mpsc;

use evalexpr::Value as V;
use evalexpr::*;

#[derive(Debug, Clone)]
pub struct SubLabel {
    value: f64,
    last_update: SystemTime,
    eval_expr: Option<Node>,
    cfg: WidgetParams,
    ctx: Context,
}

impl SubLabel {
    pub fn new(cfg: &WidgetParams) -> Self {
        SubLabel {
            value: 0.0,
            last_update: std::time::UNIX_EPOCH,
            eval_expr: None,
            cfg: cfg.clone(),
            ctx: Context::new(),
        }
    }

    pub fn draw(&mut self) {
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
                    if app::event_mouse_button() == app::MouseButton::Left {
                        (x, y) = app::event_coords();
                        true
                    } else {
                        false
                    }
                }
                enums::Event::Drag => {
                    let (x1, y1) = app::event_coords();
                    (new_x, new_y) = (origins.0 + x1 - x, origins.1 + y1 - y);
                    w.set_pos(new_x, new_y);
                    true
                }
                enums::Event::Released => {
                    if app::event_mouse_button() == app::MouseButton::Left {
                        origins = (new_x, new_y);
                        true
                    } else {
                        false
                    }
                }
                _ev => false,
            }
        });
    }
}
