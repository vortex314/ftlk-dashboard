use fltk::button::Button;
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
use crate::widget::{dnd_callback,get_params};
use crate::widget::GridRectangle;
use crate::widget::PubSubWidget;
use tokio::sync::mpsc;

use evalexpr::*;
use evalexpr::Value as V;


#[derive(Debug, Clone)]
pub struct SubGauge {
    grp: group::Group,
    frame: frame::Frame,
    value: Rc<RefCell<f64>>,
    src_topic: String,
    src_prefix: String,
    src_suffix: String,
    src_range: (f64,f64 ),
    src_eval: String,
    src_timeout: u128,
    last_update: SystemTime,
    grid_rectangle: GridRectangle,
    eval_expr : Option<Node>
}

fn hms(msec: u64) -> String {
    let hours = msec / 3_600_000;
    let minutes = (msec % 3_600_000) / 60_000;
    let seconds = (msec % 60_000) / 1000;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

impl SubGauge {
    pub fn new() -> Self {
        info!("SubGauge::new()");
        let mut grp = group::Group::default()
            .with_align(Align::Top);
        let mut frame =
            frame::Frame::default().with_label("50%").with_align(Align::Bottom );
        frame.set_label_size(26);
        grp.end();
        grp.handle(move |w, ev| {
            dnd_callback(&mut w.as_base_widget(), ev)
        });
        let mut sub_gauge = SubGauge {
            grp,
            frame,
            value: Rc::from(RefCell::from(50.)),
            src_topic: "".to_string(),
            src_prefix: "".to_string(),
            src_suffix: "".to_string(),
            src_range: (0.,100.),
            src_eval: "".to_string(),
            last_update: std::time::UNIX_EPOCH,
            src_timeout: 1000,
            grid_rectangle: GridRectangle::new(1, 1, 1, 1),
            eval_expr : None
        };
        let value_c = sub_gauge.value.clone();
        sub_gauge.frame.draw(move |w| {
            let value = *value_c.borrow();
            draw::set_draw_rgb_color(230, 230, 230);
            draw::draw_pie(w.x(), w.y(), w.w(), w.h(), -45., 225.); // total angle 270
            draw::set_draw_hex_color(0xb0bf1a);
            draw::draw_pie(
                w.x(),
                w.y(),
                w.w(),
                w.h(),
                (100. - value) as f64 * 2.7 - 45.,
                225.,
            );
            draw::set_draw_color(Color::from_hex(0x2a2a2a));
            draw::draw_pie(
                w.x() - 50 + w.w() / 2,
                w.y() - 50 + w.h() / 2,
                100,
                100,
                0.,
                360.,
            );
 //           w.draw_children();
        });
        sub_gauge
    }
}

impl PubSubWidget for SubGauge {
    fn config(&mut self, props: Value) {
        info!("SubGauge::config()");
        if let Some(pr)  = get_params(props.clone()) {
            info!("Status::config() {:?}",pr);
            if let Some(size) = pr.size {   
                if let Some(pos) = pr.pos {
                self.grp.resize(pos.0*32,pos.1*32,size.0*32,size.1*32);
                self.frame.resize(pos.0*32,pos.1*32,size.0*32,size.1*32);
                }
            }
            pr.src_topic.map(|s| self.src_topic = s);
            pr.src_prefix.map(|s| self.src_prefix = s);
            pr.src_suffix.map(|s| self.src_suffix = s);
            pr.src_timeout.map(|i| self.src_timeout = i as u128);
            pr.label.map(|s| self.frame.set_label(s.as_str()));
            pr.src_range.map(| f| self.src_range = f);
            pr.src_eval.map(|expr|  build_operator_tree(expr.as_str()).map(|x| self.eval_expr = Some(x) )); // Do proper error handling here
        }
    }
    fn on(&mut self, event: PubSubEvent) {
        match event {
            PubSubEvent::Publish { topic, message } => {
                if topic != self.src_topic {
                    return;
                }
                info!(
                    "SubGauge::on() topic: {} vs src_topic : {}",
                    topic, self.src_topic
                );
                self.eval_expr.as_ref().map(|n| {
                    let mut context =  HashMapContext::new();
                    context.set_value("msg_str".into(), message.clone().into()).unwrap();
                    let _ = message.parse::<i64>().map(|i| {
                        context.set_value("msg_i64".into(), i.into()).unwrap();
                    });
                    context.set_function("hms".into(),    Function::new(|argument| {
                        if let Ok(int) = argument.as_int() {
                            Ok(V::String(hms(int as u64)))
                        } else {
                            Err(EvalexprError::expected_number(argument.clone()))
                        }
                    })).unwrap();
                    n.eval_with_context(&context).map(|x| {
                        info!("SubGauge::on() eval_expr : {}", x);
                        *self.value.borrow_mut() = x.as_float().unwrap();
                    }).unwrap();
                });
                let _ = message.parse::<f64>().map(|f| {
                    info!("SubGauge::on() f : {}", f);
                    *self.value.borrow_mut() = (f /1000.) % 100.;
                });
                self.last_update = std::time::SystemTime::now();
                let text = format!("{}{}{}", self.src_prefix, message, self.src_suffix);
                self.frame.set_label(&text);
                self.frame.set_color(Color::from_hex(0x00ff00));
                self.frame.redraw();
            }
            PubSubEvent::Timer1sec => {
                let delta = std::time::SystemTime::now()
                    .duration_since(self.last_update)
                    .unwrap()
                    .as_millis();
                if delta > self.src_timeout {
                    info!("Status::on() {} Expired", self.src_topic);
                    self.frame.set_color(Color::from_hex(0xff0000));
                    self.frame.redraw();
                }
            }
        }
    }
    fn set_publish_channel(&mut self, channel: tokio::sync::mpsc::Sender<PubSubEvent>) {
        info!("Status::set_publish_channel()");
    }
}


    