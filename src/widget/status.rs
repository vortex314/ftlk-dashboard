use fltk::button::Button;
use fltk::enums::Color;
use fltk::widget::Widget;
use fltk::{enums::*, prelude::*, *};
use std::cell::RefCell;
use std::rc::Rc;
use serde_yaml::Value;

use crate::decl::DeclWidget;
use crate::pubsub::PubSubEvent;
use crate::widget::dnd_callback;
use crate::widget::GridRectangle;
use crate::widget::PubSubWidget;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct Status {
    status_frame: frame::Frame,
    src_topic: String,
    last_update: Rc<RefCell<u128>>,
    src_timeout: u128,
    grid_rectangle: GridRectangle,
}

impl Status {
    pub fn new(x: i32, y: i32, w: i32, h: i32,label :&str) -> Rc<RefCell<Box<Self>>> {
        info!("Status::new()");
        let mut status_frame = frame::Frame::new(x, y, w, h,"");
        status_frame.set_label(label);
        status_frame.set_frame(FrameType::BorderBox);
        status_frame.set_color(Color::from_u32(0x00ff00));

        let mut me = Status {
            status_frame,
            src_topic: "".to_string(),
            last_update: Rc::new(RefCell::new(0)),
            src_timeout: 1000,
            grid_rectangle: GridRectangle::new(x, y, w, h),
        };
        let rc = Rc::new(RefCell::new(Box::new(me)));

        let x = rc.clone();

        rc.borrow_mut().status_frame.handle(move |w, ev| {
            let v1 = dnd_callback(&mut w.as_base_widget(), ev);
            let v2 = x.borrow_mut().config_dialog(&mut w.as_base_widget(), ev);
            v1 || v2
        });

        rc
    }



    fn config_dialog(&mut self, w: &mut Widget, ev: Event) -> bool {
        match ev {
            enums::Event::Push => {
                if app::event_button() == 3 {
                    let mut win = window::Window::new(
                        app::event_x_root(),
                        app::event_y_root(),
                        400,
                        300,
                        "Dialog",
                    );
                    let mut main = group::Flex::default_fill().column();

                    let mut urow = group::Flex::default().row();
                    {
                        urow.set_pad(20);
                        frame::Frame::default()
                            .with_label("Source topic:")
                            .with_align(enums::Align::Inside | enums::Align::Right);
                        let username = input::Input::default();
                        urow.fixed(&username, 180);
                        urow.end();
                    }
                    main.fixed(&urow, 30);
                    win.end();
                    win.show();
                }
                true // Important! to make Drag work
            }
            _ => false,
        }
    }
}

impl PubSubWidget for Status {
     fn config(props: Value) -> Rc<RefCell<Box<dyn PubSubWidget>>> {
        info!("Status::config()");
        let w = props["size"][0].as_i64().unwrap() * 32;
        let h = props["size"][1].as_i64().unwrap() * 32;
        let x = props["pos"][0].as_i64().unwrap() * 32;
        let y = props["pos"][1].as_i64().unwrap() * 32;
        let label = props["label"].as_str().unwrap();
        info!("Status size : {},{} pos : {},{} ", x,y,w,h);
        Self::new(x as i32 ,y as i32 ,w as i32,h as i32,label)
    }
    fn on(&mut self, event: PubSubEvent) {
        match event {
            PubSubEvent::Publish { topic, message } => {
                if topic != self.src_topic {
                    return;
                }
                self.status_frame.set_color(Color::from_hex(0x00ff00));
            }
            PubSubEvent::Timer1sec => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                if now - *self.last_update.borrow() > self.src_timeout {
                    self.status_frame.set_color(Color::from_hex(0xff0000));
                }
            }
        }
    }
    fn set_publish_channel(&mut self, channel: tokio::sync::mpsc::Sender<PubSubEvent>) {
        info!("Status::set_publish_channel()");
    }
}
impl Status {}

//widget_extends!(Status, frame::Frame, status_frame);
