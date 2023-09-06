use fltk::button::Button;
use fltk::enums::Color;
use fltk::widget::Widget;
use fltk::{enums::*, prelude::*, *};
use serde::de::value;
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
use crate::widget::{get_params, hms};
use tokio::sync::mpsc;

use evalexpr::*;
use evalexpr::Value as V;

#[derive(Debug, Clone)]
pub struct SubText {
    frame: frame::Frame,
    src_topic: String,
    src_prefix: String,
    src_suffix: String,
    src_timeout: u128,
    eval_expr : Option<Node>,
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
            eval_expr : None,
            grid_rectangle: GridRectangle::new(1, 1, 1, 1),
        }
    }
}

impl PubSubWidget for SubText {
    fn config(&mut self, props: Value) {
        if let Some(pr)  = get_params(props.clone()) {
            info!("SubText::config() {:?}",pr);
            if let Some(size) = pr.size {   
                if let Some(pos) = pr.pos {
                self.frame.resize(pos.0*32,pos.1*32,size.0*32,size.1*32);
                }
            }
            pr.src_topic.map(|s| self.src_topic = s);
            pr.src_prefix.map(|s| self.src_prefix = s);
            pr.src_suffix.map(|s| self.src_suffix = s);
            pr.src_timeout.map(|i| self.src_timeout = i as u128);
            pr.label.map(|s| self.frame.set_label(s.as_str()));
            pr.src_eval.map(|expr|  build_operator_tree(expr.as_str()).map(|x| self.eval_expr = Some(x) )); // Do proper error handling here
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
            serde_yaml::Value::Number(serde_yaml::Number::from(self.frame.x())),
        );
        pos.insert(
            serde_yaml::Value::String("y".to_string()),
            serde_yaml::Value::Number(serde_yaml::Number::from(self.frame.y())),
        );
        let mut size = serde_yaml::Mapping::new();
        size.insert(
            serde_yaml::Value::String("w".to_string()),
            serde_yaml::Value::Number(serde_yaml::Number::from(self.frame.width())),
        );
        size.insert(
            serde_yaml::Value::String("h".to_string()),
            serde_yaml::Value::Number(serde_yaml::Number::from(self.frame.height())),
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
                let mut value= message.clone();
                debug!(
                    "SubText::on() topic: {} vs src_topic : {}",
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
                        debug!("SubText::on() eval_expr : {}", x.as_string().unwrap());
                        value = x.as_string().unwrap();
                    }).unwrap();
                });
                self.last_update = std::time::SystemTime::now();
                let text = format!("{}{}{}", self.src_prefix, value, self.src_suffix);
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
                    debug!("SubText::on() {} Expired", self.src_topic);
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
