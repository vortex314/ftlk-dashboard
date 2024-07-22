
use log::{debug, error, info, trace, warn};
use serde_derive::{Deserialize, Serialize};
use std::{collections::BTreeMap, str::FromStr};
use std::fs::File;
use std::io::Error;
use std::io::Read;
use minidom::Element;

use serde::{Deserialize, Serialize};
use serde_xml_rs::from_str;

#[derive(Debug, Serialize, Deserialize)]
pub struct WidgetParams {
    pub name: String,
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
    pub max_samples:Option<usize>,
    pub max_timespan:Option<i32>,
}

fn get_tag(element: &Element) -> Result<WidgetParams,String> {
    let mut tag = WidgetParams::new(String::from(element.name()));
    element.attrs().for_each(|(attr_name,attr_value)| {
        match attr_name {
            "label" => {
                tag.label = Some(String::from(attr_value));
            },
            "src" => {
                tag.src_topic = Some(String::from(attr_value));
            },
            "dst" => {
                tag.dst_topic = Some(String::from(attr_value));
            },
            "pressed" => {
                tag.pressed = Some(String::from(attr_value));
            },
            "released" => {
                tag.released = Some(String::from(attr_value));
            },
            "prefix" => {
                tag.prefix = Some(String::from(attr_value));
            },
            "postfix" => {
                tag.postfix = Some(String::from(attr_value));
            },
            "unit" => {
                tag.unit = Some(String::from(attr_value));
            },
            "image" => {
                tag.image = Some(String::from(attr_value));
            },
            "url" => {
                tag.url = Some(String::from(attr_value));
            },            "ok" => {
                tag.ok = Some(String::from(attr_value));
            },
            "nok" => {
                tag.ko = Some(String::from(attr_value));
            },
            "h" => {
                tag.height = Some(FromStr::from_str(attr_value).unwrap());
            },
            "w" => {
                tag.width = Some(FromStr::from_str(attr_value).unwrap());
            },
            "min" => {
                tag.min = Some(FromStr::from_str(attr_value).unwrap());
            },
            "max" => {
                tag.max = Some(FromStr::from_str(attr_value.unwrap());
            },
            "timeout" => {
                tag.timeout = Some(FromStr::from_str(attr_value).unwrap());
            },
            "msec" => {
                tag.msec = Some(FromStr::from_str(attr_value).unwrap());
            },
            "on" => {
                tag.on = Some(String::from(attr_value));
            },
            "off" => {
                tag.off = Some(String::from(attr_value));
            },
            "text_size" => {
                tag.text_size = Some(FromStr::from_str(attr_value).unwrap());
            },
            "samples" => {
                tag.max_samples = Some(FromStr::from_str(attr_value).unwrap());
            },
            "timespan" => {
                tag.max_timespan = Some(FromStr::from_str(attr_value).unwrap());
            },
            _ => { return Err(format!("Unknown attribute: {}", attr_value)); },
        }
    });
    element.children().for_each(|child| {
        if let Some(mut child_tag) = get_tag(&child) {
            tag.children.push(child_tag);
        }
    });
    Some(tag)
}

impl WidgetParams {
    pub fn new(name:String) -> Self {
        Self {
            name,
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

pub fn load_xml_file(path: &str) -> Option<WidgetParams>{
    let mut file = File::open(path).expect(std::format!("Unable to open file {} ", path).as_str());
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file ");
    let res = contents.parse::<Element>();
    match res {
        Ok(root) => get_tag(&root),
        Err(e) => {
            error!("Error parsing xml file: {}", e);
            None
        }
    
    }
}

pub fn split_underscore(str: &String) -> (Option<&str>, Option<&str>) {
    let mut it = str.split("_");
    (it.next(), it.next())
}
