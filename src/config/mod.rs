use serde_yaml::Value;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use log::{debug, error, info, trace, warn};

mod file_loader;
mod file_change;
pub(crate) mod file_xml;

use file_change::FileChange;
use file_loader::FileLoader;
use file_xml::load_xml_file;
use file_xml::WidgetParams;



pub fn load_yaml_file(path: &str) -> BTreeMap<String, Value> {
    let mut file = File::open(path).expect(std::format!("Unable to open file {} ", path).as_str());
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file ");
    let v: BTreeMap<String, Value> = serde_yaml::from_str(&contents).expect("Unable to parse YAML");
    v
}
