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
use crate::widget::hms;
use crate::widget::{GridRectangle, PubSubWidget, WidgetParams};
use tokio::sync::mpsc;

use evalexpr::Value as V;
use evalexpr::*;

#[derive(Debug, Clone)]
pub struct SubText {
    frame: frame::Frame,
    last_update: SystemTime,
    eval_expr: Option<Node>,
    widget_params: WidgetParams,
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
            last_update: std::time::UNIX_EPOCH,
            eval_expr: None,
            widget_params: WidgetParams::new(),
        }
    }

    fn reconfigure(&mut self) {
        info!("SubText::config() {:?}", self.widget_params);
        if let Some(size) = self.widget_params.size {
            if let Some(pos) = self.widget_params.pos {
                self.frame
                    .resize(pos.0 * 32, pos.1 * 32, size.0 * 32, size.1 * 32);
            }
        }
        self.widget_params
            .label
            .as_ref()
            .map(|s| self.frame.set_label(s.as_str()));
        self.widget_params
            .src_eval
            .as_ref()
            .map(|expr| build_operator_tree(expr.as_str()).map(|x| self.eval_expr = Some(x)));
        // Do proper error handling here
    }
}

impl PubSubWidget for SubText {
    fn set_config(&mut self, props: WidgetParams) {
        debug!("Status::config() {:?}", props);
        self.widget_params = props;
        self.reconfigure();
    }

    fn get_config(&self) -> Option<WidgetParams> {
        Some(self.widget_params.clone())
    }
    fn on(&mut self, event: PubSubEvent) {
        let src_topic = self.widget_params.src_topic.clone().unwrap_or("".into());
        let src_prefix = self.widget_params.src_prefix.clone().unwrap_or("".into());
        let src_suffix = self.widget_params.src_suffix.clone().unwrap_or("".into());

        match event {
            PubSubEvent::Publish { topic, message } => {
                if topic == src_topic {
                    let mut value = message.clone();
                    debug!(
                        "SubText::on() topic: {} vs src_topic : {}",
                        topic, src_topic
                    );
                    self.eval_expr.as_ref().map(|n| {
                        let mut context = HashMapContext::new();
                        context
                            .set_value("msg_str".into(), message.clone().into())
                            .unwrap();
                        let _ = message.parse::<i64>().map(|i| {
                            context.set_value("msg_i64".into(), i.into()).unwrap();
                        });
                        context
                            .set_function(
                                "hms".into(),
                                Function::new(|argument| {
                                    if let Ok(int) = argument.as_int() {
                                        Ok(V::String(hms(int as u64)))
                                    } else {
                                        Err(EvalexprError::expected_number(argument.clone()))
                                    }
                                }),
                            )
                            .unwrap();
                        n.eval_with_context(&context)
                            .map(|x| {
                                debug!("SubText::on() eval_expr : {}", x.as_string().unwrap());
                                value = x.as_string().unwrap();
                            })
                            .unwrap();
                    });
                    self.last_update = std::time::SystemTime::now();
                    let text = format!("{}{}{}", src_prefix, value, src_suffix);
                    self.frame.set_label(&text);
                    self.frame.set_color(Color::from_hex(0x00ff00));
                    self.frame.parent().unwrap().redraw();
                }
            }
            PubSubEvent::Timer1sec => {
                let delta = std::time::SystemTime::now()
                    .duration_since(self.last_update)
                    .unwrap()
                    .as_millis();
                if delta > self.widget_params.src_timeout.unwrap() as u128 {
                    debug!("SubText::on() {} Expired", src_topic);
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
