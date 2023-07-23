use crate::pubsub_widget::{get_pos, get_size, value_string_default, PubSubWidget, PubSubEvent, PubSubCmd};
use fltk::{app::*, button::*, frame::*, group::*, prelude::*, window::*};
use fltk_grid::Grid;
use serde_yaml::Value;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task;
use tokio::task::block_in_place;
use tokio::time::{self, Duration};
use tokio_stream::StreamExt;

pub struct SubLabel {
    frame: Frame,
    src_topic: String,
    prefix: String,
    suffix: String,
}

impl PubSubWidget for SubLabel {
    fn new(grid: &mut Grid, config: &Value, tx_redis_cmd: Sender<PubSubCmd>) -> SubLabel {
        info!("creating SubLabel");
        let mut frame = Frame::default();
        let src_topic = value_string_default(config, "src_topic", "");
        let prefix = value_string_default(config, "prefix", "");
        let suffix = value_string_default(config, "suffix", "");
        frame.set_label(&src_topic);
        let (x, y) = get_pos(config).unwrap();
        let (wy, wx) = get_size(config).unwrap();
        grid.insert(&mut frame, y, x);
        frame.set_size(wx, wy);

        SubLabel {
            frame,
            src_topic,
            prefix,
            suffix,
        }
    }
    fn on_publish(&mut self, topic: &String, message: &String) {
        if self.src_topic == *topic {
            let l = format!("{}{}{}", self.prefix, message, self.suffix);
            self.frame.set_label(l.as_str());
        }
    }
}
