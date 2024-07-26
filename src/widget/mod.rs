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

// pub mod gauge;
pub mod sub_gauge ;
pub mod sub_label;
pub mod pub_button;
pub mod broker_alive;
// pub mod sub_plot;
// pub mod sub_status;
// pub mod sub_text;

pub use sub_gauge::SubGauge as SubGauge;
pub use sub_label::SubLabel as SubLabel;
pub use pub_button::PubButton as PubButton;
pub use broker_alive::BrokerAlive as BrokerAlive;

#[derive(Debug, Clone)]
pub enum WidgetMsg {
    Pub { topic : String, payload : Vec<u8>},
    Tick ,
}

pub trait PubSubWidget : Send {
    fn update(&mut self, event:& WidgetMsg);
    fn draw(&mut self);
}


pub fn hms(msec: u64) -> String {
    let hours = msec / 3_600_000;
    let minutes = (msec % 3_600_000) / 60_000;
    let seconds = (msec % 60_000) / 1000;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
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


