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
use crate::widget::GridRectangle;
use crate::widget::{dnd_callback, hms};
use crate::widget::{Context, PubSubWidget, WidgetParams};
use crate::config::file_xml::WidgetParams; 
use tokio::sync::mpsc;

use evalexpr::Value as V;
use evalexpr::*;

#[derive(Debug, Clone)]
pub struct SubGauge {
    grp: group::Group,
    frame: frame::Frame,
    value: f64,
    last_update: SystemTime,
    eval_expr: Option<Node>,
    widget_params: WidgetParams,
    ctx: Context,
}

fn clap(x:f64,min:f64,max:f64) -> f64 {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}

impl SubGauge {
    pub fn new(rect:Rect,cfg:&WidgetParams) -> Self {
        let mut grp = group::Group::default().with_align(Align::Top);
        let mut frame = frame::Frame::default()
            .with_label("50%")
            .with_rect(rect.x,rect.y,rect.w,rect.h)
            .with_align(Align::Bottom);
        frame.set_frame(FrameType::BorderBox);
        frame.set_color(Color::White);
        grp.end();
        grp.handle(move |w, ev| dnd_callback(&mut w.as_base_widget(), ev));
        SubGauge {
            grp,
            frame,
            value: 0.0,
            last_update: std::time::UNIX_EPOCH,
            eval_expr: None,
            widget_params: WidgetParams::new(),
            ctx: Context::new(),
        }
    }

    fn draw(&mut self) {
        let (min,max) = self.widget_params.src_range.unwrap_or((0.,100.));
        let value = clap(self.value,min,max);
        let angle = (1. - (value-min)/(max-min)) * 270. - 45.;
        let w = &mut self.frame;
        w.draw(move|w| {
            info!("SubGauge::draw() w={},{},{},{}", w.x(),w.y(),w.w(),w.h());
            draw::set_draw_color(Color::Black);
            draw::draw_pie(w.x(), w.y(), w.w(), w.h(), -45., 225.); // total angle 270
            draw::set_draw_color(Color::Green);
            draw::draw_pie(w.x(), w.y(), w.w(), w.h(), angle, 225.);
            draw::set_draw_color(Color::White);
            draw::draw_pie(
                w.x() + w.w() / 10,
                w.y() + w.h() / 10,
                w.w() * 8 / 10,
                w.h() * 8 / 10,
                0.,
                360.,
            );
            let (center_x, center_y) = (w.x() + w.w() / 2, w.y() + w.h() / 2);
            let x2 = center_x as f64 + (w.w() / 2 - 10) as f64 * angle.to_radians().cos();
            let y2 = center_y as f64 - (w.h() / 2 - 10) as f64 * angle.to_radians().sin();
            draw::set_draw_color(Color::Black);
            draw::set_line_style(LineStyle::Solid, 3);
            draw::draw_line(center_x, center_y, x2 as i32, y2 as i32);
        });

        w.redraw();
        w.show();

    }

    fn get_major_minor_ticks(min:f64,max:f64) -> (Vec<f64>,Vec<f64>) {
        let mut major_ticks = Vec::new();
        let mut minor_ticks = Vec::new();
        let delta = (max - min).abs();
        let major = 10.0_f64.powf(delta.log10().floor());
        let minor = major/10.;

        let mut x = min;
        while x < max {
            major_ticks.push(x);
            x += major;
        }
        x = min;
        while x < max {
            minor_ticks.push(x);
            x += minor;
        }
        (major_ticks,minor_ticks)
    }

    fn reconfigure(&mut self) {
        info!("SubGauge::topic {:?}", self.widget_params.src_topic.clone().unwrap_or("".into()));
        if let Some(size) = self.widget_params.size {
            if let Some(pos) = self.widget_params.pos {
                self.grp.resize(
                    pos.0 * self.ctx.grid_width,
                    pos.1 * self.ctx.grid_height,
                    size.0 * self.ctx.grid_width,
                    size.1 * self.ctx.grid_height,
                );
                self.frame.resize(
                    pos.0 * self.ctx.grid_width,
                    pos.1 * self.ctx.grid_height,
                    size.0 * self.ctx.grid_width,
                    size.1 * self.ctx.grid_height,
                );
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
        self.draw();
    }
}

impl PubSubWidget for SubGauge {
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
                if topic != src_topic {
                    return;
                }
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
                            debug!("SubGauge::on() eval_expr : {}", x);
                            self.value = x.as_float().unwrap();
                        })
                        .unwrap();
                });
                let _ = message.parse::<f64>().map(|f| {
                    info!("SubGauge:{}={}", src_topic,f);
                    self.value = f;
                });
                self.last_update = std::time::SystemTime::now();
                let _ = message.parse::<f64>().map(|f| {
                    info!("SubGauge:{}={}", src_topic,f);
                    self.value = f;
                });
                let text = format!("{}{:.1}{}", src_prefix, self.value, src_suffix);
                self.frame.set_label(&text);
                self.draw();
            }
            PubSubEvent::Timer1sec => {
                let delta = std::time::SystemTime::now()
                    .duration_since(self.last_update)
                    .unwrap()
                    .as_millis();
                if delta > self.widget_params.src_timeout.unwrap() as u128 {
                    debug!("Status::on() {} Expired", src_topic);
                    self.frame.set_color(Color::from_hex(0x606060));
                    self.draw();
                }
            }
        }
    }
    fn set_publish_channel(&mut self, channel: tokio::sync::mpsc::Sender<PubSubEvent>) {
        info!("Status::set_publish_channel()");
    }
}
