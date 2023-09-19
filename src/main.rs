#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use fltk::valuator::Dial;
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
mod decl;
mod logger;
mod pubsub;
mod store;
mod widget;
use logger::init_logger;
use pubsub::{mqtt_bridge, redis_bridge, PubSubCmd, PubSubEvent};
use store::sub_table::EntryList;
use widget::sub_gauge::SubGauge;
use widget::sub_status::SubStatus;
use widget::sub_text::SubText;
use widget::sub_plot::SubPlot;
use widget::*;
use widget::{PubSubWidget, WidgetParams};

const PATH: &str = "src/config.yaml";

use decl::DeclWidget;
use decl::DeclarativeApp;

fn load_fn(path: &'static str) -> Option<DeclWidget> {
    let s = std::fs::read_to_string(path).ok()?;
    // We want to see the serde error on the command line while we're developing
    serde_yaml::from_str(&s).map_err(|e| eprintln!("{e}")).ok()
}

fn grid_pos_change(w: &mut Widget, new_x: i32, new_y: i32, inc_x: i32, inc_y: i32) -> bool {
    let x1 = w.x();
    let y1 = w.y();
    if x1 % inc_x != new_x % inc_x || y1 % inc_y != new_y % inc_y {
        w.set_pos((new_x / inc_x) * inc_x, (new_y / inc_y) * inc_y);
        return true;
    }
    false
}

#[tokio::main(flavor = "multi_thread", worker_threads = 1)]
async fn main() {
    env::set_var("RUST_LOG", "info");
    init_logger();
    info!("Starting up. Reading config file {}.", PATH);

    let config = Box::new(config::load_yaml_file(PATH));

    let (mut tx_publish, mut rx_publish) = broadcast::channel::<PubSubEvent>(16);
    let (mut tx_redis_cmd, mut rx_redis_cmd) = channel::<PubSubCmd>(16);
    let mut context = Context::new();
    let screen_resolution: Vec<i64> = config["screen"]["resolution"]
        .as_sequence()
        .unwrap()
        .iter()
        .map(|x| x.as_i64().unwrap_or(400))
        .collect();
    let grid: Vec<i64> = config["screen"]["grid"]
    .as_sequence()
    .unwrap()
    .iter()
    .map(|x| x.as_i64().unwrap_or(32))
    .collect();

    context.screen_width = screen_resolution[0] as i32;
    context.screen_height = screen_resolution[1] as i32;
    context.grid_width = grid[0] as i32;
    context.grid_height = grid[1] as i32;


    let redis_config = config["redis"].clone();
    let mqtt_config = config["mqtt"].clone();
    let widgets_config = config["widgets"].clone();
    let bc = tx_publish.clone();

    let mut widgets: Vec<Rc<RefCell<dyn PubSubWidget>>> = Vec::new();
    let wrc = Rc::new(RefCell::new(widgets));

    tokio::spawn(async move {
        mqtt_bridge::mqtt(mqtt_config, tx_publish).await;
    });
    tokio::spawn(async move {
        let _ = redis_bridge::redis(redis_config, bc).await;
    });
    info!("Starting up fltk");
    /*     DeclarativeApp::new(1024, 768, "MyApp", "src/config.yaml", load_fn)
    .run(|_| {})
    .unwrap();*/
    let mut _app = App::default().with_scheme(AppScheme::Oxy);
    let config = config.clone();
    let mut win = window::Window::default()
        .with_size(context.screen_width, context.screen_height)
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
        let wrc2 = wrc.clone();
        button.handle(move |b, ev| {
            if ev == Event::Push {
                info!("Save config");
                let mut file = File::create("test.yaml").unwrap();
                let mut s = String::new();
                let mut widgets_value = Value::Sequence(Vec::new());
                let mut map_all = BTreeMap::new();

                for widget in wrc2.borrow().iter() {
                    let param = widget.borrow().get_config().unwrap();
                    let param_value = serde_yaml::to_value(param).unwrap();
                    widgets_value.as_sequence_mut().unwrap().push(param_value);
                }
                map_all.insert("widgets".to_string(), widgets_value);
                let cfg = serde_yaml::to_string(&map_all).unwrap().to_string();
                s.push_str(&cfg);
                file.write_all(s.as_bytes()).unwrap();
                true
            } else {
                false
            }
        });
        let wrc1 = wrc.clone();
        widgets_config
            .as_sequence()
            .unwrap()
            .iter()
            .for_each(move |m| {
                let widget_type = m["widget"].as_str().unwrap();
                let wiget_par = serde_yaml::to_string(&m).unwrap().to_string();
                match widget_type {
                    "SubPlot" => {
                        let mut widget = SubPlot::new();
                        WidgetParams::from_value(m.clone()).map(|p| widget.set_config(p));
                        widget.set_context(context.clone());
                        wrc1.borrow_mut().push(Rc::new(RefCell::new(widget)));
                    }
                    "SubText" => {
                        let mut widget = SubText::new();
                        WidgetParams::from_value(m.clone()).map(|p| widget.set_config(p));
                        widget.set_context(context.clone());
                        wrc1.borrow_mut().push(Rc::new(RefCell::new(widget)));
                    }
                    "SubStatus" => {
                        let mut widget = SubStatus::new();
                        WidgetParams::from_value(m.clone()).map(|p| widget.set_config(p));
                        widget.set_context(context.clone());
                        wrc1.borrow_mut().push(Rc::new(RefCell::new(widget)));
                    }
                    "SubGauge" => {
                        let mut widget = SubGauge::new();
                        WidgetParams::from_value(m.clone()).map(|p| widget.set_config(p));
                        widget.set_context(context.clone());
                        wrc1.borrow_mut().push(Rc::new(RefCell::new(widget)));
                    }
                    _ => {
                        warn!("Unknown widget type {}", widget_type);
                    }
                };
            });
    }
    //     let mut widgets = window_fill(&mut grid, *config, tx_redis_cmd.clone());
    grp_dashboard.end();
    tab.end();
    win.end();
    win.show();
    let sub = rx_publish.resubscribe();
    let mut widgets_rc = wrc.clone();
    app::add_timeout3(1.0, move |_x| {
        debug!("add_timeout3");
        widgets_rc.borrow().iter().for_each(|w| {
            w.borrow_mut().on(PubSubEvent::Timer1sec);
        });
        app::repeat_timeout3(1.0, _x);
    });
    while _app.wait() {
        let mut received = false;
        let widgets_rc2 = wrc.clone();
        while let Ok(x) = rx_publish.try_recv() {
            match x {
                PubSubEvent::Publish { topic, message } => {
                    entry_list.add(topic.clone(), message.clone());
                    received = true;
                    widgets_rc2.borrow().iter().for_each(|w| {
                        w.borrow_mut().on(PubSubEvent::Publish {
                            topic: topic.clone(),
                            message: message.clone(),
                        });
                    });
                }
                PubSubEvent::Timer1sec => {
                    info!("Timer1sec");
                    widgets_rc2.borrow().iter().for_each(|w| {
                        w.borrow_mut().on(PubSubEvent::Timer1sec);
                    });
                }
            }
        }
        if received {
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
