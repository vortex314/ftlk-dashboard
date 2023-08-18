#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use regex::Regex;

#[macro_use]
extern crate log;
use log::{debug, error, info, trace, warn};
use serde_yaml::mapping::Entry;
use serde_yaml::Value;
use simplelog::SimpleLogger;
//==================================================================================================
use fltk::button::Button;
use fltk::frame::Frame;
use fltk::app::{awake, redraw, App};
use fltk::enums::Color;
use fltk::enums::Event;
use fltk::group::{Group, Tabs};
use fltk::window::DoubleWindow;
use fltk::{prelude::*, *};
use fltk_grid::Grid;
use fltk_table::{SmartTable, TableOpts};
use fltk::app::AppScheme;
//==================================================================================================
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;
use std::fmt::Error;
use std::thread::{self, sleep, Thread};
use std::env;
use std::sync::{Arc, Mutex};
//==================================================================================================
use tokio::task::block_in_place;
use tokio::sync::broadcast;
use tokio::time::{self, Duration};
use tokio::sync::mpsc::{Sender,Receiver,channel};
use tokio::task;
use tokio_stream::StreamExt;

mod logger;
mod pubsub;
mod config;
mod store;
mod decl;
use logger::init_logger;
use store::sub_table::EntryList;
use pubsub::{PubSubEvent, PubSubCmd,mqtt_bridge,redis_bridge};

const PATH : &str = "src/config.yaml";
const H_PIXEL : i32 = 800;
const V_PIXEL : i32 = 600;

use decl::DeclarativeApp;
use decl::Widget;

fn load_fn(path: &'static str) -> Option<Widget> {
    let s = std::fs::read_to_string(path).ok()?;
    // We want to see the serde error on the command line while we're developing
    serde_yaml::from_str(&s).map_err(|e| eprintln!("{e}")).ok()
}

#[tokio::main(flavor = "multi_thread", worker_threads = 1)]
async fn main() {
    env::set_var("RUST_LOG", "info");
    init_logger();
    info!("Starting up. Reading config file {}.", PATH);

    let config = Box::new(config::load_yaml_file(PATH));

    let (mut tx_publish, mut rx_publish) = broadcast::channel::<PubSubEvent>(16);
    let (mut tx_redis_cmd, mut rx_redis_cmd) = channel::<PubSubCmd>(16);

    let redis_config = config["redis"].clone();
    let mqtt_config = config["mqtt"].clone();
    let bc = tx_publish.clone();

    tokio::spawn(async move {
        mqtt_bridge::mqtt(mqtt_config, tx_publish).await;
    });
    tokio::spawn(async move {
        redis_bridge::redis(redis_config, bc).await;
    });
    info!("Starting up fltk");
    DeclarativeApp::new(1024, 768, "MyApp", "src/config.yaml", load_fn)
    .run(|_| {})
    .unwrap();
/*     let mut _app = App::default().with_scheme(AppScheme::Gtk);
    let config = config.clone();
    let mut win = window::Window::default()
        .with_size(H_PIXEL, V_PIXEL)
        .center_screen()
        .with_label("Hello from rust");
 
    let mut entry_list = EntryList::new();
    let tab = Tabs::new(0, 0, H_PIXEL, V_PIXEL, "");
    
        let grp = group::Group::new(20, 20, H_PIXEL - 20, V_PIXEL - 20, "Table");
        let mut table = SmartTable::default()
            .with_size(H_PIXEL - 20, V_PIXEL - 20)
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
        grp.end();
    
    {
        let grp1 = group::Group::new(20, 20, H_PIXEL - 40, V_PIXEL - 40, "Dashboard");
        let mut grid = Grid::default_fill();
        grid.set_layout(16, 10);
        grid.debug(true);
   //     let mut widgets = window_fill(&mut grid, *config, tx_redis_cmd.clone());
        grid.end();
        grp1.end();
    }

    tab.end();

    win.end();
    win.show();

    let sub = rx_publish.resubscribe();
    while _app.wait() {
        let mut received = false;
        while let Ok(x) = rx_publish.try_recv() {
            match x {
                PubSubEvent::Publish { topic, message } => {
                    entry_list.add(topic, message);
                    received = true;
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
                if  row == entry_count.try_into().unwrap() {
                    break;
                }
            }
            table.redraw();
        }
    }*/
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
            Ok(Err(e)) => {
                error!(" error: {}", e);
            }
            Err(e) => {
                error!("timeout : {} {} ", pattern, e);
            }
        }
    }
}
