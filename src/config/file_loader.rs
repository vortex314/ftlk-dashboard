#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use fltk::draw::Rect;
use log::{error, info, warn};
use minidom::Element;
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::fmt::format;
use std::fs::File;
use std::io::BufRead;
use std::sync::*;
use std::thread;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crate::config::file_xml::Tag;

use crate::limero::SinkRef;
use crate::widget::SubGauge;
use crate::widget::{Context, PubSubWidget, WidgetParams};

use super::file_xml::load_xml_file;

pub struct FileLoader {
    file_name: String,
    config: Tag,
    widgets: Vec<Box<dyn PubSubWidget + Send>>,
}

impl FileLoader {
    pub fn new(file_name: &str) -> Self {
        let config = Tag::new("Dashboard".to_string());
        FileLoader {
            file_name: file_name.to_string(),
            config,
            widgets: Vec::new(),
        }
    }

    pub fn loader(&mut self) -> Result<(), String> {
        self.config = load_xml_file(&self.file_name)?;
        self.load(&self.config)?;
        Ok(())
    }

    fn load(&mut self, cfg: &Tag) -> Result<(), String> {
        if cfg.name != "Dashboard" {
            return Err("Invalid config file. Missing Dashboard tag.".to_string());
        }
        let mut rect = Rect::new(0, 0, cfg.width.unwrap_or(1025), cfg.height.unwrap_or(769));
        self.widgets.clear(); // clear existing widgets for reload
        cfg.children.iter().for_each(|child| {
            info!("Loading widget {}", child.name);
            let mut sub_widgets = load_widgets(rect, child);
            self.widgets.append(&mut sub_widgets);
            if child.width.is_some() {
                rect.x += child.width.unwrap();
            }
            if child.height.is_some() {
                rect.y += child.height.unwrap();
            }
        });
        Ok(())
    }
}

fn load_widgets(rect: Rect, cfg: &Tag) -> Vec<Box<dyn PubSubWidget + Send>> {
    let mut widgets: Vec<Box<dyn PubSubWidget + Send>> = Vec::new();
    let mut rect = rect;

    rect.y = rect.y + cfg.height.unwrap_or(0);
    rect.x = rect.x + cfg.width.unwrap_or(0);
    info!(
        "{} : {} {:?}",
        cfg.name,
        cfg.label.as_ref().get_or_insert(&String::from("NO_LABEL")),
        rect
    );

    match cfg.name.as_str() {
        "Row" => {
            cfg.children.iter().for_each(|child| {
                let mut sub_widgets = load_widgets(rect, child);
                widgets.append(&mut sub_widgets);
                rect.x += child.width.unwrap_or(0);
            });
        }
        "Col" => {
            cfg.children.iter().for_each(|child| {
                let mut sub_widgets = load_widgets(rect, child);
                widgets.append(&mut sub_widgets);
                rect.y += child.height.unwrap_or(0);
            });
        }

        "Gauge" => {
            let widget = SubGauge::new(rect, cfg);
            widgets.push(Box::new(widget));
        }
        "Label" => {
            let widget = Label::new(rect, cfg);
            widgets.push(Box::new(widget));
        }
        "Progress" => {
            let widget = Progress::new(rect, cfg);
            widgets.push(Box::new(widget));
        }
        "Button" => {
            let widget = Button::new(rect, cfg);
            widgets.push(Box::new(widget));
        }
        "Slider" => {
            let widget = Slider::new(rect, cfg);
            widgets.push(Box::new(widget));
        }
        "Table" => {
            let widget = Table::new(rect, cfg);
            widgets.push(Box::new(widget));
        }
        "Plot" => {
            let widget = Plot::new(rect, cfg);
            widgets.push(Box::new(widget));
        }
        _ => {
            warn!("Unknown widget: {}", cfg.name);
        }
    }
    widgets
}
