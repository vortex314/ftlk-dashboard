use fltk::{app::*, button::*, frame::*, group::*, prelude::*, window::*};
use crate::pubsub_widget::{PubSubWidget, get_pos, get_size, value_string_default, PubSubEvent, PubSubCmd};
use serde_yaml::Value;
use fltk_grid::Grid;
use tokio::task::block_in_place;
use tokio::sync::broadcast;
use tokio::time::{self, Duration};
use tokio::sync::mpsc::{Sender,Receiver};
use tokio::task;
use tokio_stream::StreamExt;
pub struct PubButton {
    button: Button,
    dst_topic: String,
    label: String,
}

impl PubSubWidget for PubButton {
    fn new(grid: &mut Grid, config: &Value, tx_redis_cmd: Sender<PubSubCmd>) -> PubButton {
        info!("creating PubButton");
        let mut button = Button::default();
        let dst_topic = value_string_default(config, "dst_topic", "unknown dst_topic");
        let label = value_string_default(config, "label", &dst_topic);
        button.set_label(&label);
        let (x, y) = get_pos(config).unwrap();
        let (wy, wx) = get_size(config).unwrap();
        grid.insert(&mut button, y, x);
        button.set_size(wx, wy);
        let mut topic = dst_topic.clone();

        button.set_callback(move |_| {
            let message = String::from("1");
            info!("sending {} => {:?}", topic.clone(), message);
            let topic = topic.clone();
            let cmd = PubSubCmd::Publish { topic, message };
            let _ = tx_redis_cmd.send(cmd);
        });

        PubButton {
            button,
            dst_topic,
            label,
        }
    }
    fn on_publish(&mut self, topic: &String, message: &String) {}
}