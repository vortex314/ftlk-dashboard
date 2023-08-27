use fltk::{enums::*, prelude::*, *};
use std::cell::RefCell;
use std::rc::Rc;

use crate::decl::PubSubWidget;
use crate::decl::Widget;
use crate::pubsub::PubSubEvent;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct Status {
    status_frame: frame::Frame,
    src_topic: String,
    last_update: Rc<RefCell<u128>>,
    src_timeout: u128,
}

impl PubSubWidget for Status {
    fn new(props: Widget) -> Self {
        info!("Status::new()");
        let size = props.size.unwrap_or(vec![10, 10]);
        let pos = props.pos.unwrap_or(vec![0, 0]);
        info!("Status size : {:?} pos : {:?} ", size, pos);
        let value = Rc::from(RefCell::from(0.));
        let label = props.label.unwrap_or("No Title".to_string());

        let mut status_frame = frame::Frame::new(
            pos[0] * 32,
            pos[1] * 32,
            size[0] * 32,
            size[1] * 32,
            None,
        );
    //    status_frame.set_label(label);

        status_frame.set_label_size(26);
      /*   status_frame.draw(move |w| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            if now - *me.last_update.borrow() > me.src_timeout {
                status_frame.set_label_color(Color::Red);
            } else {
                status_frame.set_label_color(Color::White);
            }
        });*/
        let me = Status {
            status_frame,
            src_topic: props.src_topic.unwrap_or("".to_string()),
            last_update: Rc::new(RefCell::new(0)),
            src_timeout: props.src_timeout.unwrap_or(1000),
        };
        me
    }
    fn on(&mut self, event: PubSubEvent) {
        match event {
            PubSubEvent::Publish { topic, message } => {
                if topic != self.src_topic {
                    return;
                }
                self.status_frame.set_color(Color::from_hex(0x00ff00));
            }
            _ => {}
        }
    }
    fn set_publish_channel(&mut self, channel: tokio::sync::mpsc::Sender<PubSubEvent>) {
        info!("Status::set_publish_channel()");
    }
}
impl Status {}

widget_extends!(Status, frame::Frame, status_frame);
