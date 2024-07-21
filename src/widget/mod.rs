use crate::pubsub::PubSubEvent;
use fltk::button::Button;
use fltk::enums;
use fltk::input::Input;
use fltk::prelude::*;
use fltk::widget::Widget;
use fltk::*;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_yaml::Value;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;

pub mod gauge;
pub mod sub_gauge ;
pub mod sub_plot;
pub mod sub_status;
pub mod sub_text;

pub use sub_gauge::SubGauge as SubGauge;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WidgetParams {
    widget: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pos: Option<(i32, i32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<(i32, i32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    src_topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    src_topics: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    src_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    src_suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    src_eval: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    src_timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    src_range: Option<(f64, f64)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    src_ranges: Option<Vec<(f64, f64)>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    src_timespan: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    src_samples: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    dst_topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dst_on: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dst_off: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dst_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    samples_timespan_sec: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    samples_max_count: Option<u64>,
}
// get WidgetParams from yaml value

use serde_yaml::Error;
impl WidgetParams {
    pub fn new() -> WidgetParams {
        WidgetParams {
            widget: "".to_string(),
            label: None,
            pos: Some((0, 0)),
            size: Some((0, 0)),
            image: None,
            src_topic: None,
            src_topics: None,
            src_prefix: None,
            src_suffix: None,
            src_eval: None,
            src_timeout: None,
            src_range: None,
            src_ranges: None,
            src_timespan: None,
            src_samples: None,
            dst_topic: None,
            dst_on: None,
            dst_off: None,
            dst_format: None,
            samples_timespan_sec: None,
            samples_max_count: None,
        }
    }
    pub fn from_value(v: Value) -> Option<WidgetParams> {
        let x: Result<WidgetParams, Error> = serde_yaml::from_value(v);
        if let Ok(w) = x {
            return Some(w);
        }
        None
    }

    pub fn save_params(&self) -> Value {
        serde_yaml::to_value(&self).unwrap()
    }
}

pub fn hms(msec: u64) -> String {
    let hours = msec / 3_600_000;
    let minutes = (msec % 3_600_000) / 60_000;
    let seconds = (msec % 60_000) / 1000;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

pub trait PubSubWidget {
    fn on(&mut self, event: PubSubEvent);
    fn set_publish_channel(&mut self, channel: mpsc::Sender<PubSubEvent>);
    fn set_config(&mut self, props: WidgetParams);
    fn get_config(&self) -> Option<WidgetParams>;
    fn set_context(&mut self, context: Context);
}

#[derive(Debug, Clone)]
pub struct Context {
    pub grid_width: i32,
    pub grid_height: i32,
    pub screen_width: i32,
    pub screen_height: i32,
    pub background_color: enums::Color,
    pub font_color: enums::Color,
    pub valuator_color: enums::Color,

    pub theme: Option<String>,
    pub publish_channel: Option<mpsc::Sender<PubSubEvent>>,
}

impl Context {
    pub const fn new() ->   Context {
        Context {
            grid_width: 32,
            grid_height: 32,
            screen_width: 1024,
            screen_height: 768,
            background_color: enums::Color::White,
            font_color: enums::Color::Black,
            valuator_color: enums::Color::Green,
            theme: None,
            publish_channel: None,
        }
    }
}

static mut CONTEXT: Context =  Context::new() ;

#[derive(Debug, Clone)]
pub struct GridRectangle {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl GridRectangle {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }
    pub fn overlaps(&self, other: &GridRectangle) -> bool {
        self.x < other.x + other.w
            && self.x + self.w > other.x
            && self.y < other.y + other.h
            && self.y + self.h > other.y
    }
}

pub fn grid_pos_change(w: &mut Widget, new_x: i32, new_y: i32, inc_x: i32, inc_y: i32) -> bool {
    let x1 = w.x();
    let y1 = w.y();
    if x1 % inc_x != new_x % inc_x || y1 % inc_y != new_y % inc_y {
        w.set_pos((new_x / inc_x) * inc_x, (new_y / inc_y) * inc_y);
        return true;
    }
    false
}

pub fn dnd_callback(w: &mut Widget, ev: enums::Event) -> bool {
    match ev {
        enums::Event::Push => {
            true // Important! to make Drag work
        }
        enums::Event::Drag => {
            if grid_pos_change(w, app::event_x(), app::event_y(), 32, 32) {
                w.parent().unwrap().parent().unwrap().redraw();
            }
            true
        }
        _ => {
            // info!(" event {:?}",ev);
            false
        }
    }
}
