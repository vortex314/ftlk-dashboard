#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use config::file_xml::{get_widget_params, load_dashboard, load_xml_file};
use fltk::valuator::Dial;
use limero::{ActorTrait, SinkRef, SinkTrait, SourceTrait};
use minidom::Element;
use regex::Regex;

#[macro_use]
extern crate log;
use log::{debug, error, info, trace, warn};
use serde_yaml::mapping::Entry;
use serde_yaml::Value;
use simplelog::SimpleLogger;
//==================================================================================================
use fltk::app::AppScheme;
use fltk::app::{awake, redraw, App};
use fltk::button::Button;
use fltk::draw::Rect;
use fltk::enums::Color;
use fltk::enums::Event;
use fltk::frame::Frame;
use fltk::group::{Group, HGrid, Tabs};
use fltk::misc::Progress;
use fltk::widget::Widget;
use fltk::window::DoubleWindow;
use fltk::{prelude::*, *};
use fltk_table::{SmartTable, TableOpts};
use fltk_theme::{
    color_themes, widget_themes, ColorTheme, SchemeType, ThemeType, WidgetScheme, WidgetTheme,
};

//==================================================================================================
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::env;
use std::fmt::Error;
use std::fs::File;
use std::io::prelude::*;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep, Thread};
//==================================================================================================
use tokio::sync::broadcast;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::RwLock;

use tokio::task;
use tokio::task::block_in_place;
use tokio::time::{self, Duration};
use tokio_stream::StreamExt;

mod config;
mod logger;
mod pubsub;
use pubsub::zenoh_pubsub::*;
mod store;
mod widget;
use logger::init_logger;
use pubsub::{mqtt_bridge, redis_bridge, PubSubCmd, PubSubEvent};
use store::sub_table::EntryList;
use widget::sub_gauge::SubGauge;
use widget::sub_label::SubLabel;
use widget::PubSubWidget;

use widget::*;
mod limero;

use rand::random;

pub fn default_str(opt: Option<String>, default: &str) -> String {
    opt.unwrap_or(default.to_string())
}

#[derive(Debug)]
enum MyError<'a> {
    Io(std::io::Error),
    Xml(minidom::Error),
    Yaml(serde_yaml::Error),
    Str(&'a str),
    String(String),
}

fn start_pubsub(
    cfg: &Element,
    event_sink: SinkRef<PubSubEvent>,
) -> Result<SinkRef<PubSubCmd>, String> {
    let zenoh = cfg
        .get_child("Zenoh", "")
        .ok_or("Zenoh section not found")?;
    let mut zenoh_actor = PubSubActor::new();
    let pubsub_cmd = zenoh_actor.sink_ref();
    zenoh_actor.add_listener(event_sink);
    tokio::spawn(async move {
        zenoh_actor.run().await;
    });
    pubsub_cmd.push(PubSubCmd::Connect);
    pubsub_cmd.push(PubSubCmd::Subscribe {
        topic: "**".to_string(),
    });
    Ok(pubsub_cmd)
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), MyError<'static>> {
    env::set_var("RUST_LOG", "info");
    init_logger();
    info!("Starting up. Reading config file .");

    let mut event_sink = limero::Sink::new(100);

    let root_config = load_xml_file("./config.xml").map_err(MyError::Xml)?;

    let pubsub_config = root_config
        .get_child("PubSub", "")
        .ok_or(MyError::Str("PubSub section not found"))?;
    let pubsub_cmd =
        start_pubsub(&pubsub_config, event_sink.sink_ref()).map_err(MyError::String)?;

    let dashboard_config = root_config
        .get_child("Dashboard", "")
        .ok_or(MyError::Str("Dashboard section not found"))?;
    let widgets_params = load_dashboard(&dashboard_config).map_err(MyError::String)?;
    let mut context = Context::new();
    let window_params =
        get_widget_params(Rect::new(0, 0, 0, 0), &dashboard_config).map_err(MyError::String)?;

    context.screen_width = window_params.width.unwrap_or(1024);
    context.screen_height = window_params.height.unwrap_or(768);
    let window_rect = Rect::new(0, 0, context.screen_width, context.screen_height);
    info!("Starting up fltk");

    let mut _app = App::default().with_scheme(AppScheme::Oxy);
    let mut win = window::Window::default()
        .with_size(context.screen_width, context.screen_height)
        .with_label(&default_str(window_params.label, "FLTK dashboard").as_str());
    win.make_resizable(true);

    let mut widgets = Vec::<Box<dyn PubSubWidget>>::new();

    for widget_params in widgets_params {
        let widget_type = widget_params.name.as_str();
        info!("Loading widget {}", widget_type);
        match widget_type {
            "Gauge" => {
                let mut widget = SubLabel::new(&widget_params);
                widget.draw();

                widgets.push(Box::new(widget));
            }
            "Label" => {
                let mut widget = SubLabel::new(&widget_params);
                widget.draw();

                widgets.push(Box::new(widget));
            }
            "Table" => {
                let mut widget = SubLabel::new(&widget_params);
                widget.draw();
            }
            "Progress" => {
                let mut widget = SubLabel::new(&widget_params);
                widget.draw();
            }
            "Plot" => {
                let mut widget = SubLabel::new(&widget_params);
                widget.draw();
            }
            _ => {
                warn!("Unknown widget type {}", widget_type);
            }
        };
    }

    win.end();
    win.show();

    while _app.wait() {
        let m = event_sink.next().await.unwrap();
        for widget in widgets.iter_mut() {
            widget.update(&m);
        }
        win.redraw();
        thread::sleep(std::time::Duration::from_millis(100));
    }
    Ok(())
}
