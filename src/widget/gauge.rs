use fltk::{enums::*, prelude::*, *};
use std::cell::RefCell;
use std::rc::Rc;
use serde_yaml::Value;

use crate::decl::DeclWidget;
use crate::widget::PubSubWidget;
use crate::pubsub::PubSubEvent;
use tokio::sync::mpsc;

use fltk::widget::Widget;

#[derive(Debug, Clone)]
pub struct Gauge {
    value: f64,
    frame: frame::Frame,
    src_topic: String,
}

impl Gauge {
    pub fn new() -> Self {
        frame::Frame::default().with_label("Gauge");
        Gauge {
            value: 0.,
            frame: frame::Frame::default().with_label("Gauge"),
            src_topic: "".to_string(),
        }
    }
}

impl PubSubWidget for Gauge {
    fn config(&mut self,props: Value){
 /*        info!("Gauge::new()");
        let w = props["size"][0].as_i64().unwrap() * 32;
        let h = props["size"][1].as_i64().unwrap() * 32;
        let x = props["pos"][0].as_i64().unwrap() * 32;
        let y = props["pos"][1].as_i64().unwrap() * 32;
        self.frame.resize(x as i32,y as i32,w as i32,h as i32);
        props["label"].as_str().map(|s| self.frame.set_label(s));
        info!("Status size : {},{} pos : {},{} ", x,y,w,h);
        let mut frame =
            frame::Frame::default_fill();
        frame.set_label_size(26);
        let value_c = self.value.clone();
        frame.draw(move |w| {
            draw::set_draw_rgb_color(230, 230, 230);
            draw::draw_pie(w.x(), w.y(), w.w(), w.h(), 0., 180.);
            draw::set_draw_hex_color(0xb0bf1a);
            draw::draw_pie(
                w.x(),
                w.y(),
                w.w(),
                w.h(),
                (100. - value_c) as f64 * 1.8,
                180.,
            );
            draw::set_draw_color(Color::White);
            draw::draw_pie(
                w.x() - 50 + w.w() / 2,
                w.y() - 50 + w.h() / 2,
                100,
                100,
                0.,
                360.,
            );
            self.frame.redraw();
        });*/
    }
    fn get_config(&self ) -> Value {
        let mut props = serde_yaml::Mapping::new();
        props.insert(
            serde_yaml::Value::String("type".to_string()),
            serde_yaml::Value::String("status".to_string()),
        );
        props.insert(
            serde_yaml::Value::String("src_topic".to_string()),
            serde_yaml::Value::String(self.src_topic.clone()),
        );

        let mut pos = serde_yaml::Mapping::new();
        pos.insert(
            serde_yaml::Value::String("x".to_string()),
            serde_yaml::Value::Number(serde_yaml::Number::from(self.frame.x())),
        );
        pos.insert(
            serde_yaml::Value::String("y".to_string()),
            serde_yaml::Value::Number(serde_yaml::Number::from(self.frame.y())),
        );
        let mut size = serde_yaml::Mapping::new();
        size.insert(
            serde_yaml::Value::String("w".to_string()),
            serde_yaml::Value::Number(serde_yaml::Number::from(self.frame.width())),
        );
        size.insert(
            serde_yaml::Value::String("h".to_string()),
            serde_yaml::Value::Number(serde_yaml::Number::from(self.frame.height())),
        );
        let mut pr = serde_yaml::Mapping::new();
        pr.insert(
            serde_yaml::Value::String("pos".to_string()),
            serde_yaml::Value::Mapping(pos),
        );
        pr.insert(
            serde_yaml::Value::String("size".to_string()),
            serde_yaml::Value::Mapping(size),
        );
        props.insert(
            serde_yaml::Value::String("params".to_string()),
            serde_yaml::Value::Mapping(pr),
        );
        serde_yaml::Value::Mapping(props)
    }
    fn on(&mut self, event: PubSubEvent) {
        match event {
            PubSubEvent::Publish { topic, message } => {
                if  topic != self.src_topic  {
                    return;
                }
                let val = message.parse::<f64>().map(|v| {
                    let mut v = v * 100.;
                    v= {if v > 100. {
                        100.
                    } else if v < 0. {
                        0.
                    } else {
                        v as f64
                    }};
                    self.value = v;
                });
                self.frame.redraw();
            }
            PubSubEvent::Timer1sec => {}
        }
    }
    fn set_publish_channel(&mut self,channel : tokio::sync::mpsc::Sender<PubSubEvent>) {
        
    }
}
impl Gauge {
    
    pub fn value(&self) -> f64 {
        self.value
    }
    pub fn set_value(&mut self, val: f64) {
        self.value = val;
        self.frame.set_label(&val.to_string());
        self.frame.redraw();
    }

    pub fn configure(&mut self, widget: &DeclWidget) {
        widget
            .label
            .as_ref()
            .map(|label| self.frame.set_label(label));
        widget.labelcolor.as_ref().map(|labelcolor| {
            if let Ok(col) = enums::Color::from_hex_str(labelcolor) {
                self.frame.set_label_color(col);
            }
        });
    }
}

// widget_extends!(Gauge, group::Group, main_wid);

/*
fn main() {
    let app = app::App::default();
    app::background(255, 255, 255);
    let mut win = window::Window::default().with_size(400, 300);
    let mut dial = MyDial::new(100, 100, 200, 200, "CPU Load %");
    dial.set_label_size(22);
    dial.set_label_color(Color::from_u32(0x797979));
    win.end();
    win.show();

    // get the cpu load value from somewhere, then call dial.set_value() in a callback or event loop
    dial.set_value(10);

    app.run().unwrap();
}
*/
