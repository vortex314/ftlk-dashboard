/*
based on plotters demo : https://github.com/fltk-rs/demos/tree/master/plotters
canvas gauges code to copy : https://github.com/Mikhus/canvas-gauges
https://canvas-gauges.com/documentation/examples/ : looks good !
*/
use fltk::button::Button;
use fltk::enums::Color;
use fltk::widget::Widget;
use fltk::{enums::*, prelude::*, *};
use fltk::{image::IcoImage, prelude::*, *};
use plotters::chart::ChartState;
use plotters::coord::types::RangedCoordf32;
use plotters::{prelude::*, style::Color as PlColor};
use plotters_bitmap::{bitmap_pixel::RGBPixel, BitMapBackend};

use std::{collections::VecDeque, error::Error, time::SystemTime};

const W: usize = 737;
const H: usize = 432;

const SAMPLE_RATE: f64 = 10_000.0;
const FREAME_RATE: f64 = 30.0;

use serde::de::value;
use serde_yaml::Value;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use crate::decl::DeclWidget;
use crate::pubsub::PubSubEvent;
use crate::widget::dnd_callback;
use crate::widget::hms;
use crate::widget::{Context, GridRectangle, PubSubWidget, WidgetParams,CONTEXT};
use tokio::sync::mpsc;

use evalexpr::Value as V;
use evalexpr::*;
use rand::prelude::*;

#[derive(Clone)]
pub struct SubPlot {
    frame: frame::Frame,
    buf: Vec<u8>,
    //    cs: ChartState<Cartesian2d<RangedCoordf32, RangedCoordf32>>,
    data: VecDeque<(f64, f64)>,
    last_update: SystemTime,
    eval_expr: Option<Node>,
    widget_params: WidgetParams,
    ctx: Context,
    start_ts: SystemTime,
}

impl SubPlot {
    pub fn new() -> Self {
        let mut buf = vec![0u8; W * H * 3];
        let mut frame = frame::Frame::default().with_label("SubPlot");
        frame.handle(|f, ev| dnd_callback(&mut f.as_base_widget(), ev));
        let mut last_flushed = 0.0;
        SubPlot {
            frame,
            buf,
            data: VecDeque::new(),
            last_update: std::time::UNIX_EPOCH,
            eval_expr: None,
            widget_params: WidgetParams::new(),
            ctx: Context::new(),
            start_ts: SystemTime::now(),
        }
    }

    fn reconfigure(&mut self) {
        info!("SubPlot::topic {}", self.widget_params.src_topic.clone().unwrap_or("".into()));
        if let Some(size) = self.widget_params.size {
            if let Some(pos) = self.widget_params.pos {
                self.frame.resize(
                    pos.0 * self.ctx.grid_width,
                    pos.1 * self.ctx.grid_height,
                    size.0 * self.ctx.grid_width,
                    size.1 * self.ctx.grid_height,
                );
            }
        }
        self.widget_params
            .src_eval
            .as_ref()
            .map(|expr| build_operator_tree(expr.as_str()).map(|x| self.eval_expr = Some(x)));
        self.draw();
        
        // Do proper error handling here
    }

    fn draw(&mut self) {
        let w = self.frame.w();
        let h = self.frame.h();
        let x_range = -(self.widget_params.src_timespan.unwrap_or(60) as f64) ;
        let y_range = self.widget_params.y_range.clone();
        let y_range = y_range.unwrap_or(vec!(0.0,1.0));
        let mut buf = vec![0u8; (w * h * 3) as usize];
        {
            let drawing_area =
                BitMapBackend::<RGBPixel>::with_buffer_and_format(&mut buf, (w as u32, h as u32))
                    .unwrap()
                    .into_drawing_area();
            drawing_area.fill(&WHITE).unwrap();
            let mut chart_context = ChartBuilder::on(&drawing_area)
                .margin(10)
                .set_all_label_area_size(30)
                .build_cartesian_2d(x_range .. 0.0, y_range[0]..y_range[1])
                .unwrap();

            chart_context
                .configure_mesh()
                .bold_line_style(&GREEN.mix(0.5))
                .light_line_style(&TRANSPARENT)
                .draw()
                .unwrap();
            let now = now() ;
            chart_context
                .draw_series(LineSeries::new(
                    self.data.iter().map(|&(x, y)| (x - now , y)),
                    &RED,
                ))
                .unwrap();
        };
        draw::draw_rgb(&mut self.frame, &buf).unwrap();
    }


}

fn now() -> f64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}

impl PubSubWidget for SubPlot {
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
        let max_samples: usize = self.widget_params.src_samples.unwrap_or(100) as usize;
        match event {
            PubSubEvent::Publish { topic, message } => {
                if topic == src_topic {
                    let mut value = message.clone();
                    let _ = message
                        .parse::<f64>()
                        .map(|f| {
                            let now = now();
                            info!("SubPlot:{} = {}", now,f);
                            self.data.push_back((now, f));
                            if self.data.len() > max_samples {
                                self.data.pop_front();
                            }
                            self.draw();
                        })
                        .unwrap();
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
                self.draw();
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
