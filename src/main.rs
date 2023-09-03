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
//==================================================================================================
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::env;
use std::fmt::Error;
use std::fs::File;
use std::io::prelude::*;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep, Thread};
use std::rc::Rc;
//==================================================================================================
use tokio::sync::broadcast;
use tokio::sync::mpsc::{channel, Receiver, Sender};
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
use widget::status::Status;
use widget::sub_text::SubText;
use widget::PubSubWidget;

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

    let pixels: Vec<i64> = config["screen"]["resolution"]
        .as_sequence()
        .unwrap()
        .iter()
        .map(|x| x.as_i64().unwrap_or(400))
        .collect();
    let window_width = pixels[0] as i32;
    let window_height = pixels[1] as i32;
    info!("Screen resolution {} x {}", window_width, window_height);

    let grid: Vec<i64> = config["screen"]["grid"]
        .as_sequence()
        .unwrap()
        .iter()
        .map(|x| x.as_i64().unwrap_or(32))
        .collect();
    let grid_cols = grid[0] as i32;
    let grid_rows = grid[1] as i32;

    let redis_config = config["redis"].clone();
    let mqtt_config = config["mqtt"].clone();
    let widgets_config = config["widgets"].clone();
    let bc = tx_publish.clone();
   
    let mut widgets:Vec<Rc<RefCell<dyn PubSubWidget>>> = Vec::new();

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
    let mut _app = App::default().with_scheme(AppScheme::Gtk);
    let config = config.clone();
    let mut win = window::Window::default()
        .with_size(window_width, window_height)
        .with_label("FLTK dashboard");
    win.make_resizable(true);

    let mut entry_list = EntryList::new();

    let mut tab = Tabs::new(0, 0, window_width, window_height, "");

    let grp_table = group::Group::new(20, 20, window_width - 20, window_height - 20, "Table");
    let mut table = SmartTable::default()
        .with_size(window_width - 20, window_height - 20)
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
        group::Group::new(20, 20, window_width - 40, window_height - 40, "Dashboard");
    //       grid.debug(true);

    {
        // iterate through widgets
        widgets_config.as_sequence().unwrap().iter().for_each(|m| {
            let widget_type = m["widget"].as_str().unwrap();
            match widget_type {
                "Status" => {
                    let mut widget = Status::new() ;
                    widget.config(m.clone());
                    widgets.push(Rc::new(RefCell::new(widget)));
                }
                "SubText" => {
                    let mut widget = SubText::new();
                    widget.config(m.clone());
                    widgets.push(Rc::new(RefCell::new(widget)));
                }
                _ => {
                    info!("Unknown widget type {}", widget_type);
                }
            };
        });
        let mut status_frame = Status::new();
        let mut button = Button::new(32, 32, 3 * 32, 32, "Button");
        button.handle({
            move |w, ev| match ev {
                enums::Event::Push => {
                    if app::event_button() == 3 {
                        let mut win =
                            window::Window::new(app::event_x(), app::event_y(), 400, 300, "Dialog");
                        let mut input = input::Input::new(100, 100, 160, 25, "Input");
                        input.set_value("Hello World!");
                        let mut button = Button::new(100, 150, 160, 25, "Ok");
                        button.set_callback(|w| {
                            w.parent().unwrap().hide();
                        });
                        win.end();
                        win.show();
                    }
                    true
                }
                enums::Event::Drag => {
                    info!(
                        "Drag {} {} {} ",
                        app::event_x(),
                        app::event_y(),
                        app::event_button()
                    );
                    if grid_pos_change(
                        &mut (w.as_base_widget()),
                        app::event_x(),
                        app::event_y(),
                        32,
                        32,
                    ) {
                        w.parent().unwrap().parent().unwrap().redraw();
                    }
                    true
                }
                /*enums::Event::Move => {
                    info!(
                        "Move {} {} {} ",
                        app::event_x(),
                        app::event_y(),
                        app::event_button()
                    );
                    let dist_right_border = (w.x() + w.w() - app::event_x()).abs();
                    let dist_bottom_border = (w.y() + w.h() - app::event_y()).abs();
                    let is_on_right_bottom_corner = (dist_right_border < 10 && dist_bottom_border < 10);
                    info!(
                        "dist_right_border {} dist_bottom_border {} is_on_right_bottom_corner {}",
                        dist_right_border, dist_bottom_border, is_on_right_bottom_corner
                    );
                    let mut win = w.window().unwrap();
                    if is_on_right_bottom_corner {
                        info!("is_on_right_bottom_corner");
                        win.set_cursor(enums::Cursor::SE);
                        w.set_size(app::event_x() - w.x(), app::event_y() - w.y());
                    } else {
                        win.set_cursor(enums::Cursor::Default);
                    }
                    true
                }*/
                _ => false,
            }
        });
    }
    {
        let mut progress_bar = Progress::new(32, 2 * 32, 3 * 32, 32, "");
        progress_bar.set_maximum(100.);
        progress_bar.set_value(50.);
        progress_bar.set_selection_color(Color::from_rgb(0, 255, 0));
        progress_bar.set_label("15 V");
        progress_bar.handle({
            move |w, ev| match ev {
                Event::Push => true,
                enums::Event::Drag => {
                    info!("key {:?}", app::event_original_key());
                    info!(
                        "Drag {} {} {} ",
                        app::event_x(),
                        app::event_y(),
                        app::event_button()
                    );
                    if grid_pos_change(
                        &mut (w.as_base_widget()),
                        app::event_x(),
                        app::event_y(),
                        32,
                        32,
                    ) {
                        w.parent().unwrap().parent().unwrap().redraw();
                    }

                    true
                }
                _ => false,
            }
        });
    }
    let mut slider = valuator::Slider::new(32, 3 * 32, 4 * 32, 32, "");
    slider.set_type(valuator::SliderType::HorizontalNice);
    slider.set_bounds(0., 1000.);
    slider.set_value(500.);
    slider.handle({
        move |w, ev| match ev {
            enums::Event::Drag => {
                info!(
                    "Drag {} {} {} ",
                    app::event_x(),
                    app::event_y(),
                    app::event_button()
                );
                if grid_pos_change(
                    &mut (w.as_base_widget()),
                    app::event_x(),
                    app::event_y(),
                    32,
                    32,
                ) {
                    w.parent().unwrap().parent().unwrap().redraw();
                }
                true
            }
            _ => false,
        }
    });

    let mut widget = Widget::new(32, 4 * 32, 3 * 32, 32, "Widget");
    widget.set_color(Color::Red);
    widget.set_label_color(Color::White);
    widget.draw(|w| {
        draw::draw_box(w.frame(), w.x(), w.y(), w.w(), w.h(), Color::Red);
        draw::set_draw_color(enums::Color::Red); // for the text
        draw::set_font(enums::Font::Helvetica, app::font_size());
        draw::draw_text2(&w.label(), w.x(), w.y(), w.w(), w.h(), w.align());
    });

    widget.handle({
        move |w, ev| match ev {
            Event::Push => true,
            enums::Event::Drag => {
                info!(
                    "Drag {} {} {} ",
                    app::event_x(),
                    app::event_y(),
                    app::event_button()
                );
                if grid_pos_change(
                    &mut (w.as_base_widget()),
                    app::event_x(),
                    app::event_y(),
                    32,
                    32,
                ) {
                    w.parent().unwrap().parent().unwrap().redraw();
                }
                true
            }
            _ => false,
        }
    });
    widget.show();

    //     let mut widgets = window_fill(&mut grid, *config, tx_redis_cmd.clone());
    grp_dashboard.end();
    tab.set_value(&(grp_dashboard.as_group().unwrap()));

    {
        let grp2 = group::Group::new(20, 20, window_width - 40, window_height - 40, "Test");
        /* let mut hgrid = HGrid::new(20, 20, window_width - 100, window_height - 100, "Test");
           button::Button::default();
        button::Button::default();
        button::Button::default();
        button::Button::default();
        button::Button::default();
        button::Button::default();
        button::Button::default();
        button::Button::default();
        hgrid.end();*/
        grp2.end();
    }

    tab.end();

    win.end();
    win.show();
    let widgets_rc = Rc::new(RefCell::new(widgets));
    let widgets_rc1 = widgets_rc.clone();
    let sub = rx_publish.resubscribe();
    app::add_timeout3(1.0,move |_x| {
        debug!("add_timeout3");
        widgets_rc1.borrow().iter().for_each(|w| {
            w.borrow_mut().on(PubSubEvent::Timer1sec);
        });
        app::repeat_timeout3(1.0, _x);       
    } );
    while _app.wait() {
        let mut received = false;
        let widgets_rc2 = widgets_rc.clone();
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
                PubSubEvent::Timer1sec  => {
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
            info!("Received {} messages", entry_count);
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
