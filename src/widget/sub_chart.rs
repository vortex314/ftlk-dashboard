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
use crate::widget::{Context, GridRectangle, PubSubWidget, WidgetParams};
use tokio::sync::mpsc;

use evalexpr::Value as V;
use evalexpr::*;

#[derive(Debug, Clone)]
pub struct SubChart {
    chart: misc::Chart,
    last_update: SystemTime,
    eval_expr: Option<Node>,
    widget_params: WidgetParams,
    ctx: Context,
}

impl SubChart {
    pub fn new() -> Self {
        info!("SubChart::new()");
        let mut chart = misc::Chart::default();
        chart.handle(move |w, ev| dnd_callback(& mut w.as_base_widget(), ev));
        SubChart {
            chart,
            last_update: std::time::UNIX_EPOCH,
            eval_expr: None,
            widget_params: WidgetParams::new(),
            ctx: Context::new(),
        }
    }

    fn reconfigure(&mut self) {
        info!("SubChart::config() {:?}", self.widget_params);
        if let Some(size) = self.widget_params.size {
            if let Some(pos) = self.widget_params.pos {
                self.chart.resize(
                    pos.0 * self.ctx.grid_width,
                    pos.1 * self.ctx.grid_height,
                    size.0 * self.ctx.grid_width,
                    size.1 * self.ctx.grid_height,
                );
            }
        }
        self.chart.set_type(misc::ChartType::Line);
        self.chart.set_bounds(0.0, 100.0);
        self.chart.set_text_size(18);
        self.chart
            .add(88.4, "Rust", enums::Color::from_u32(0xcc9c59));
        self.chart.add(8.4, "C++", enums::Color::Red);
        self.chart.add(3.2, "C", enums::Color::Black);
        self.chart.set_color(enums::Color::White);
        self.widget_params
            .src_eval
            .as_ref()
            .map(|expr| build_operator_tree(expr.as_str()).map(|x| self.eval_expr = Some(x)));
        // Do proper error handling here
    }
}

impl PubSubWidget for SubChart {
    fn set_config(&mut self, props: WidgetParams) {
        debug!("Status::config() {:?}", props);
        self.widget_params = props;
        self.reconfigure();
    }

    fn set_context(&mut self, context: Context) {
        self.ctx = context;
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
                        "SubChart::on() topic: {} vs src_topic : {}",
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
                                debug!("SubChart::on() eval_expr : {}", x.as_string().unwrap());
                                value = x.as_string().unwrap();
                            })
                            .unwrap();
                    });
                    self.last_update = std::time::SystemTime::now();
                }
            }
            PubSubEvent::Timer1sec => {
                /*let delta = std::time::SystemTime::now()
                    .duration_since(self.last_update)
                    .unwrap()
                    .as_millis();
                if delta > self.widget_params.src_timeout.unwrap() as u128 {
                    debug!("SubChart::on() {} Expired", src_topic);
                    self.frame.set_color(Color::from_hex(0xff0000));
                    self.frame.parent().unwrap().redraw();
                }*/
            }
        }
    }
    fn set_publish_channel(&mut self, channel: tokio::sync::mpsc::Sender<PubSubEvent>) {
        info!("Status::set_publish_channel()");
    }
}