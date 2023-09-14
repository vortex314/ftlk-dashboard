use fltk::{enums::*, prelude::*, *};
use serde_yaml::Value;
use std::cell::RefCell;
use std::rc::Rc;

use crate::decl::DeclWidget;
use crate::pubsub::PubSubEvent;
use crate::widget::{PubSubWidget, WidgetParams,Context};
use tokio::sync::mpsc;

use fltk::widget::Widget;

#[derive(Debug, Clone)]
pub struct Gauge {
    value: f64,
    frame: frame::Frame,
    widget_params: WidgetParams,
    ctx: Context,
}

impl Gauge {
    pub fn new() -> Self {
        frame::Frame::default().with_label("Gauge");
        Gauge {
            value: 0.,
            frame: frame::Frame::default().with_label("Gauge"),
            widget_params: WidgetParams::new(),
            ctx: Context::new(),
        }
    }
    fn reconfigure(&mut self) {
        info!("Gauge::config() {:?}", self.widget_params);
        if let Some(size) = self.widget_params.size {
            if let Some(pos) = self.widget_params.pos {
                self.frame
                    .resize(pos.0 * self.ctx.grid_width,
                        pos.1 * self.ctx.grid_height,
                        size.0 * self.ctx.grid_width,
                        size.1 * self.ctx.grid_height,);
            }
        }
        self.widget_params
            .label.as_ref()
            .map(|s| self.frame.set_label(s.as_str()));
    }
}

impl PubSubWidget for Gauge {
    fn set_config(&mut self, props: WidgetParams) {
        self.widget_params = props;
        self.reconfigure();
    }

    fn set_context(&mut self, context: super::Context) {
        self.ctx = context;
    }

    fn get_config(&self) -> Option<WidgetParams> {
        Some(self.widget_params.clone())
    }
    fn on(&mut self, event: PubSubEvent) {
        let src_topic = self.widget_params.src_topic.clone().unwrap_or("".into());
        let src_prefix = self.widget_params.src_prefix.clone().unwrap_or("".into());
        let src_suffix = self.widget_params.src_suffix.clone().unwrap_or("".into());
        match event {
            PubSubEvent::Publish { topic, message } => {
                if topic != src_topic {
                    return;
                }
                let val = message.parse::<f64>().map(|v| {
                    let mut v = v * 100.;
                    v = {
                        if v > 100. {
                            100.
                        } else if v < 0. {
                            0.
                        } else {
                            v as f64
                        }
                    };
                    self.value = v;
                });
                self.frame.redraw();
            }
            PubSubEvent::Timer1sec => {}
        }
    }
    fn set_publish_channel(&mut self, channel: tokio::sync::mpsc::Sender<PubSubEvent>) {}
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
