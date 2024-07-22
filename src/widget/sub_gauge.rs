use draw::Rect;
use fltk::button::Button;
use fltk::draw::LineStyle;
use fltk::enums::Color;
use fltk::widget::Widget;
use fltk::{enums::*, prelude::*, *};
use fltk_theme::colors::aqua::dark::windowFrameTextColor;
use serde_yaml::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;
use std::time::SystemTime;

use crate::pubsub::PubSubEvent;
use crate::widget:: hms;
use crate::widget::Context;
use crate::config::file_xml::WidgetParams; 
use tokio::sync::mpsc;

use evalexpr::Value as V;
use evalexpr::*;

#[derive(Debug, Clone)]
pub struct SubGauge {
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
    pub fn new(cfg:&WidgetParams) -> Self {

 //       grp.handle(move |w, ev| dnd_callback(&mut w.as_base_widget(), ev));
        SubGauge {
            value: 0.0,
            last_update: std::time::UNIX_EPOCH,
            eval_expr: None,
            widget_params: cfg.clone(),
            ctx: Context::new(),
        }
    }

    pub fn draw(&mut self) {
        let cfg = &self.widget_params;
        let mut grp = group::Group::default().with_align(Align::Top);
        let mut frame = frame::Frame::new(cfg.rect.x,cfg.rect.y,cfg.rect.w,cfg.rect.h,None);
        frame.set_frame(FrameType::BorderBox);
        frame.set_color(Color::White);

        let min = self.widget_params.min.unwrap_or(0.);
        let max = self.widget_params.max.unwrap_or(100.);
        let value = clap(self.value,min,max);
        let angle = (1. - (value-min)/(max-min)) * 270. - 45.;
        frame.draw(move|w| {
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

        frame.redraw();
        frame.show();
        grp.end();

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

    
}


