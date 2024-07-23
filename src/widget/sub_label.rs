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

use crate::pubsub::PubSubEvent;
use crate::widget::hms;
use crate::widget::Context;
use crate::config::file_xml::WidgetParams; 
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

fn dnd_callback(widget: &mut dyn WidgetExt, ev: Event) -> bool {
    match ev {
        Event::DndDrag => {
            info!("DndDrag");
            true
        }
        Event::DndEnter => {
            widget.set_color(Color::Red);
            true
        }
        Event::DndLeave => {
            widget.set_color(Color::White);
            true
        }
        Event::DndRelease => {
            widget.set_color(Color::White);
            true
        }
        _ => false,
    }
}

impl SubLabel {
    pub fn new(cfg:&WidgetParams) -> Self {
        SubLabel {
            value: 0.0,
            last_update: std::time::UNIX_EPOCH,
            eval_expr: None,
            cfg: cfg.clone(),
            ctx: Context::new(),
        }
    }



    pub fn draw(&mut self) {
        let mut frame = fltk::frame::Frame::new(self.cfg.rect.x,self.cfg.rect.y,self.cfg.rect.w,self.cfg.rect.h,None);
        frame.set_frame(FrameType::BorderBox);
        frame.set_color(Color::from_u32(0xFFFFFF));
        frame.set_align(Align::Center);
        let label = self.cfg.label.as_ref().unwrap().clone();
        frame.handle({
            let mut x = 0;
            let mut y = 0;
            move |w, ev| match ev {
                enums::Event::Push => {
                    let coords = app::event_coords();
                    x = coords.0;
                    y = coords.1;
                    info!("Pushed {} at {}, {}",label ,x, y);
                    true
                }
                enums::Event::Drag => {
                    w.set_pos(app::event_x_root() - x, app::event_y_root() - y);
                    info!("Drag {} at {}, {}",label ,app::event_x_root() - x, app::event_y_root() - y);

                    true
                }
                _ => false,
            }
        });        
        self.cfg.label.as_ref().map(|s| frame.set_label(s.as_str()));
    }
}


