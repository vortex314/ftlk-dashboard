use serde_yaml::Value;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use log::{debug, error, info, trace, warn};

mod file_change;
pub(crate) mod file_xml;

use file_change::FileChange;
use file_xml::load_xml_file;
use file_xml::WidgetParams;



