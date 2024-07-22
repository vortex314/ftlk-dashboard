#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use config::file_xml::{get_widget_params, load_dashboard, load_xml_file};
use fltk::valuator::Dial;
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
use fltk::enums::Color;
use fltk::enums::Event;
use fltk::frame::Frame;
use fltk::group::{Group, HGrid, Tabs};
use fltk::misc::Progress;
use fltk::widget::Widget;
use fltk::window::DoubleWindow;
use fltk::{prelude::*, *};
use fltk::draw::Rect;
use fltk_grid::Grid;
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
mod store;
mod widget;
use logger::init_logger;
use pubsub::{mqtt_bridge, redis_bridge, PubSubCmd, PubSubEvent};
use store::sub_table::EntryList;
use widget::sub_gauge::SubGauge;

use widget::*;
mod limero;

use rand::random;


#[derive(Debug)]
enum MyError<'a> {
    Io(std::io::Error),
    Xml(minidom::Error),
    Yaml(serde_yaml::Error),
    Str(&'a str),
    String(String),
}


#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(),MyError<'static> >{
    env::set_var("RUST_LOG", "info");
    init_logger();
    info!("Starting up. Reading config file .");

    let root_config = load_xml_file("./config.xml").map_err(MyError::Xml)?;
    let pubsub_config = root_config.get_child("PubSub","").ok_or(MyError::Str("PubSub section not found"))?;
    let dashboard_config = root_config.get_child("Dashboard","").ok_or(MyError::Str("Dashboard section not found"))?;
    let widgets = load_dashboard(&dashboard_config).map_err(MyError::String)?;
    let mut context = Context::new();
    let window_params = get_widget_params(Rect::new(0,0,0,0),&root_config).map_err(MyError::String)?;
    let window_rect = Rect::new(0, 0, window_params.width.unwrap_or(1024), window_params.height.unwrap_or(768));
    context.screen_width = window_params.width.unwrap_or(1024);
    context.screen_height = window_params.height.unwrap_or(768);

    info!("Starting up fltk");

    let mut _app = App::default().with_scheme(AppScheme::Oxy);
    let mut win = window::Window::default()
        .with_size(window_params.width.unwrap_or(1024), window_params.height.unwrap_or(768))
        .with_label("FLTK dashboard");
    win.make_resizable(true);

    let mut entry_list = EntryList::new();

    let mut tab = Tabs::new(0, 0, context.screen_width, context.screen_height, "");

    let grp_table = group::Group::new(20, 20, context.screen_width - 20, context.screen_height - 20, "Table");
    let mut table = SmartTable::default()
        .with_size(context.screen_width - 20, context.screen_height - 20)
        .center_of_parent()
        .with_opts(TableOpts {
            rows: 1000,
            cols: 4,
            editable: true,
            ..Default::default()
        });
    table.set_col_header(true);
    table.set_row_header(false);
    //       table.set_rows(50);
    //      table.set_cols(4);
    table.set_col_header_value(0, "Topic");
    table.set_col_header_value(1, "Message");
    table.set_col_header_value(2, "Time");
    table.set_col_header_value(3, "Count");
    table.set_row_height_all(30);
    let widths = vec![300, 300, 200, 180];
    for (i, w) in widths.iter().enumerate() {
        table.set_col_width(TryInto::<i32>::try_into(i).unwrap(), *w);
    }

    table.set_col_resize(true);
    table.set_row_resize(true);
    table.set_col_header_height(30);
    table.set_row_header_width(100);

    table.end();
    grp_table.end();

    let mut grp_dashboard =
        group::Group::new(20, 20, context.screen_width - 20, context.screen_height - 20, "Dashboard");

    {
        let mut button = Button::new(20, 20, 32, 32, "@filesave");
        button.handle(move |b, ev| {
            if ev == Event::Push {
                info!("Save config");
                true
            } else {
                false
            }
        });
        for widget_params in widgets {
            let widget_type = widget_params.name.as_str();
            match widget_type {

                "SubGauge" => {
                    let mut widget = SubGauge::new(&widget_params);
                }
                _ => {
                    warn!("Unknown widget type {}", widget_type);
                }
            };
        }
        
    }
    //     let mut widgets = window_fill(&mut grid, *config, tx_redis_cmd.clone());
    grp_dashboard.end();
    tab.end();
    win.end();
    win.show();

    app::add_timeout3(1.0, move |_x| {
        debug!("add_timeout3");
        app::repeat_timeout3(1.0, _x);
    });
    while _app.wait() {

        if true {
            let entry_count = entry_list.entries.len();
            let mut row = 0;
            // table.clear();
            for entry in entry_list.entries.iter() {
                //   info!("{} {} {} {}", row,entry.topic, entry.value, entry.time.time());
                table.set_cell_value(row, 0, entry.topic.as_str());
                table.set_cell_value(row, 1, entry.value.as_str());
                table.set_cell_value(
                    row,
                    2,
                    &format!("{}", entry.time.time().format("%H:%M:%S%.3f").to_string()),
                );
                table.set_cell_value(row, 3, &format!("{}", entry.count));
                row += 1;
                if row == entry_count.try_into().unwrap() {
                    break;
                }
            }
            table.redraw();
            awake();
        }
    }
    Ok(())
}

// async channel receiver
async fn receiver(mut rx: broadcast::Receiver<PubSubEvent>, pattern: &str) {
    let mut duration: Duration;
    const MAX_TIME: std::time::Duration = std::time::Duration::from_secs(10);
    let mut _time_last = std::time::Instant::now();
    let mut _alive: bool;

    loop {
        if _time_last.elapsed() > MAX_TIME {
            _alive = false;
            duration = Duration::from_millis(1000);
        } else {
            _alive = true;
            duration = MAX_TIME - _time_last.elapsed()
        }
        let event = time::timeout(duration, rx.recv()).await;
        match event {
            Ok(Ok(PubSubEvent::Publish { topic, message })) => {
                if topic.starts_with(pattern) {
                    _time_last = std::time::Instant::now();
                    info!(
                        "Widget pattern : {} topic: {}, message: {}",
                        pattern, topic, message
                    );
                }
            }
            Ok(Ok(PubSubEvent::Timer1sec)) => {
                _time_last = std::time::Instant::now();
                info!("Timer1sec");
            }
            Ok(Err(e)) => {
                error!(" error: {}", e);
            }
            Err(e) => {
                error!("timeout : {} {} ", pattern, e);
            }
        }
    }
}
