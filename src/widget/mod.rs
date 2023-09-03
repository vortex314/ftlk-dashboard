use crate::pubsub::PubSubEvent;
use tokio::sync::mpsc;
use fltk::widget::Widget;
use fltk::button::Button;
use fltk::enums;
use fltk::input::Input;
use fltk::*;
use fltk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use serde_yaml::Value;

pub mod status;
pub mod gauge;
pub mod sub_text;

struct PubSubParams {
    src_topic: String,
    src_timeout: u128,
}

pub trait PubSubWidget {
    fn on(&mut self,event : PubSubEvent );
    fn set_publish_channel(&mut self,channel : mpsc::Sender<PubSubEvent>);
    fn config(&mut self, props:Value) ;
}


#[derive(Debug, Clone)]
pub struct GridRectangle {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl GridRectangle {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }
    pub fn overlaps(&self, other: &GridRectangle) -> bool {
        self.x < other.x + other.w
            && self.x + self.w > other.x
            && self.y < other.y + other.h
            && self.y + self.h > other.y
    }
}

pub fn grid_pos_change(w: &mut Widget, new_x: i32, new_y: i32, inc_x: i32, inc_y: i32) -> bool {
    let x1 = w.x();
    let y1 = w.y();
    if x1 % inc_x != new_x % inc_x || y1 % inc_y != new_y % inc_y {
        w.set_pos((new_x / inc_x) * inc_x, (new_y / inc_y) * inc_y);
        return true;
    }
    false
}


pub fn dnd_callback(w: &mut Widget, ev: enums::Event) -> bool {
    match ev {
        enums::Event::Push => {
            /*if app::event_button() == 3 {
                let mut win =
                    window::Window::new(app::event_x_root(), app::event_y_root(), 400, 300, "Dialog");
                let mut input = input::Input::new(100, 100, 160, 25, "Input");
                input.set_value("Hello World!");
                let mut button = Button::new(100, 150, 160, 25, "Ok");
                button.set_callback(|w| {
                    w.parent().unwrap().hide();
                });
                win.end();
                win.show();
            }*/
            true // Important! to make Drag work
        }
        enums::Event::Drag => {
            info!(
                "Drag {} {} {} ",
                app::event_x(),
                app::event_y(),
                app::event_button()
            );
            if grid_pos_change( w, app::event_x(), app::event_y(), 32, 32) {
                w.parent().unwrap().parent().unwrap().redraw();
            }
            true
        }
        /*enums::Event::Move => {
            info!(
                "Move {} {} {} ",
                app::event_x(),
                app::event_y(),
                app::event_button()
            );
            let dist_right_border = (w.x() + w.w() - app::event_x()).abs();
            let dist_bottom_border = (w.y() + w.h() - app::event_y()).abs();
            let is_on_right_bottom_corner = (dist_right_border < 10 && dist_bottom_border < 10);
            info!(
                "dist_right_border {} dist_bottom_border {} is_on_right_bottom_corner {}",
                dist_right_border, dist_bottom_border, is_on_right_bottom_corner
            );
            let mut win = w.window().unwrap();
            if is_on_right_bottom_corner {
                info!("is_on_right_bottom_corner");
                win.set_cursor(enums::Cursor::SE);
                w.set_size(app::event_x() - w.x(), app::event_y() - w.y());
            } else {
                win.set_cursor(enums::Cursor::Default);
            }
            true
        }*/
        _ => false,
    }
}