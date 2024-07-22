use fltk::draw::Rect;
use log::{debug, error, info, trace, warn};
use minidom::Element;
use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::io::Error;
use std::io::Read;
use std::{collections::BTreeMap, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_xml_rs::from_str;

#[derive(Debug,Clone)]
pub struct WidgetParams {
    pub name: String,
    pub rect: Rect,
    pub label: Option<String>,
    pub height: Option<i32>,
    pub width: Option<i32>,
    pub text_size: Option<i32>,
    pub msec: Option<i32>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub timeout: Option<i32>,
    pub src_topic: Option<String>,
    pub dst_topic: Option<String>,
    pub pressed: Option<String>,
    pub released: Option<String>,
    pub prefix: Option<String>,
    pub postfix: Option<String>,
    pub unit: Option<String>,
    pub ok: Option<String>,
    pub ko: Option<String>,
    pub url: Option<String>,
    pub image: Option<String>,
    pub on: Option<String>,
    pub off: Option<String>,
    pub children: Vec<WidgetParams>,
    pub max_samples: Option<usize>,
    pub max_timespan: Option<i32>,
}

pub fn get_widget_params(rect:Rect,element: &Element) -> Result<WidgetParams, String> {
    let mut widget_params = WidgetParams::new(String::from(element.name()),rect);
    for (attr_name,attr_value) in element.attrs(){
        match attr_name {
            "label" => {
                widget_params.label = Some(String::from(attr_value));
            }
            "src" => {
                widget_params.src_topic = Some(String::from(attr_value));
            }
            "dst" => {
                widget_params.dst_topic = Some(String::from(attr_value));
            }
            "pressed" => {
                widget_params.pressed = Some(String::from(attr_value));
            }
            "released" => {
                widget_params.released = Some(String::from(attr_value));
            }
            "prefix" => {
                widget_params.prefix = Some(String::from(attr_value));
            }
            "postfix" => {
                widget_params.postfix = Some(String::from(attr_value));
            }
            "unit" => {
                widget_params.unit = Some(String::from(attr_value));
            }
            "image" => {
                widget_params.image = Some(String::from(attr_value));
            }
            "url" => {
                widget_params.url = Some(String::from(attr_value));
            }
            "ok" => {
                widget_params.ok = Some(String::from(attr_value));
            }
            "nok" => {
                widget_params.ko = Some(String::from(attr_value));
            }
            "h" => {
                widget_params.rect.h = FromStr::from_str(attr_value).unwrap();
                widget_params.height = Some(FromStr::from_str(attr_value).unwrap());
            }
            "w" => {
                widget_params.rect.w = FromStr::from_str(attr_value).unwrap();
                widget_params.width = Some(FromStr::from_str(attr_value).unwrap());
            }
            "min" => {
                widget_params.min = Some(FromStr::from_str(attr_value).unwrap());
            }
            "max" => {
                widget_params.max = Some(FromStr::from_str(attr_value).unwrap());
            }
            "timeout" => {
                widget_params.timeout = Some(FromStr::from_str(attr_value).unwrap());
            }
            "msec" => {
                widget_params.msec = Some(FromStr::from_str(attr_value).unwrap());
            }
            "on" => {
                widget_params.on = Some(String::from(attr_value));
            }
            "off" => {
                widget_params.off = Some(String::from(attr_value));
            }
            "text_size" => {
                widget_params.text_size = Some(FromStr::from_str(attr_value).unwrap());
            }
            "samples" => {
                widget_params.max_samples = Some(FromStr::from_str(attr_value).unwrap());
            }
            "timespan" => {
                widget_params.max_timespan = Some(FromStr::from_str(attr_value).unwrap());
            }
            _ => {
                error!("Unknown attribute: {}", attr_value);
            }
        };
    }
    Ok(widget_params)
}

impl WidgetParams {
    pub fn new(name: String,rect : Rect) -> Self {
        Self {
            name,
            rect,
            label: None,
            height: None,
            width: None,
            text_size: None,
            msec: None,
            min: None,
            max: None,
            timeout: None,
            src_topic: None,
            dst_topic: None,
            pressed: None,
            released: None,
            prefix: None,
            postfix: None,
            unit: None,
            ok: None,
            ko: None,
            url: None,
            image: None,
            on: None,
            off: None,
            children: Vec::new(),
            max_samples: None,
            max_timespan: None,
        }
    }
}

pub fn load_xml_file(path: &str) -> Result<Element, minidom::Error> {
    let mut file = File::open(path).expect(std::format!("Unable to open file {} ", path).as_str());
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file ");
    contents.parse::<Element>()
}

pub fn load_dashboard(root:&Element ) -> Result<Vec<WidgetParams>, String> {
    let mut widgets: Vec<WidgetParams> = Vec::new();
    let mut cfg = get_widget_params(Rect::new(0,0,0,0),&root,)?;
    if cfg.name != "Dashboard" {
        return Err("Invalid config file. Missing Dashboard tag.".to_string());
    }
    let mut rect = Rect::new(0, 0, cfg.width.unwrap_or(1025), cfg.height.unwrap_or(769));
    cfg.rect = rect;
    for child_element in root.children() {
        let child = get_widget_params(rect,child_element)?;
        info!("Loading widget {}", child.name);
        let mut sub_widgets = load_widgets(rect, child_element)?;
        widgets.append(&mut sub_widgets);
        if child.width.is_some() {
            rect.x += child.width.unwrap();
        }
        if child.height.is_some() {
            rect.y += child.height.unwrap();
        }
    };
    Ok(widgets)
}

fn load_widgets(rect: Rect, element: &Element) -> Result<Vec<WidgetParams>, String> {
    let cfg = get_widget_params(rect,element)?;
    let mut widgets: Vec<WidgetParams> = Vec::new();
    let mut rect = cfg.rect;


    info!(
        "{} : {} {:?}",
        cfg.name,
        cfg.label.as_ref().get_or_insert(&String::from("NO_LABEL")),
        cfg.rect
    );

    match cfg.name.as_str() {
        "Row" => {
            rect.h = cfg.height.unwrap_or(rect.h);
            for child_element in element.children() {
                let child = get_widget_params(rect,child_element)?;
                let mut sub_widgets = load_widgets(rect, child_element)?;
                widgets.append(&mut sub_widgets);
                rect.x += child.width.unwrap_or(0);
            }
        }
        "Col" => {
            rect.w = cfg.width.unwrap_or(rect.w);

            for child_element in element.children() {
                let child = get_widget_params(rect,child_element)?;
                let mut sub_widgets = load_widgets(rect, child_element)?;
                widgets.append(&mut sub_widgets);
                rect.y += child.height.unwrap_or(0);
            }
        }
        _ => {
            widgets.push(cfg.clone());
            rect.y = rect.y + cfg.height.unwrap_or(0);
            rect.x = rect.x + cfg.width.unwrap_or(0);
        }

    }
    Ok(widgets)
}

pub fn split_underscore(str: &String) -> (Option<&str>, Option<&str>) {
    let mut it = str.split("_");
    (it.next(), it.next())
}
