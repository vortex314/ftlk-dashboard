use fltk::{enums::*, prelude::*, *};
use std::cell::RefCell;
use std::rc::Rc;

use crate::decl::DeclWidget;
use crate::widget::PubSubWidget;
use crate::pubsub::PubSubEvent;
use tokio::sync::mpsc;

use fltk::widget::Widget;

#[derive(Debug, Clone)]
pub struct Gauge {
    main_wid: group::Group,
    value: Rc<RefCell<f64>>,
    value_frame: frame::Frame,
    src_topic: String,
    publish_channel: Option<mpsc::Sender<PubSubEvent>>,
}

impl PubSubWidget for Gauge {
    fn config(props: DeclWidget) -> Self {
        info!("Gauge::new()");
        let size = props.size.unwrap_or(vec![10, 10]);
        let pos = props.pos.unwrap_or(vec![0, 0]);
        info!(" size : {:?} pos : {:?} ", size, pos);
        let value = Rc::from(RefCell::from(0.));
        let title = props.label.unwrap_or("No Title".to_string());
        let mut main_wid =
            group::Group::new(pos[0] * 32, pos[1] * 32, size[0] * 32, size[1] * 32, None)
                .with_label(title.as_str())
                .with_align(Align::Top);
        let mut value_frame =
            frame::Frame::new(main_wid.x(), main_wid.y() + 80, main_wid.w(), 40, "0");
        value_frame.set_label_size(26);
        main_wid.end();
        let value_c = value.clone();
        main_wid.draw(move |w| {
            draw::set_draw_rgb_color(230, 230, 230);
            draw::draw_pie(w.x(), w.y(), w.w(), w.h(), 0., 180.);
            draw::set_draw_hex_color(0xb0bf1a);
            draw::draw_pie(
                w.x(),
                w.y(),
                w.w(),
                w.h(),
                (100. - *value_c.borrow()) as f64 * 1.8,
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
            w.draw_children();
        });
        Self {
            main_wid,
            value,
            value_frame,
            src_topic: props.src_topic.unwrap_or("".to_string()),
            publish_channel: None,
        }
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
                    *self.value.borrow_mut() = v;
                });
                self.main_wid.redraw();
            }
        }
    }
    fn set_publish_channel(&mut self,channel : tokio::sync::mpsc::Sender<PubSubEvent>) {
        self.publish_channel = Some(channel);
    }
}
impl Gauge {
    
    pub fn value(&self) -> f64 {
        *self.value.borrow()
    }
    pub fn set_value(&mut self, val: f64) {
        *self.value.borrow_mut() = val;
        self.value_frame.set_label(&val.to_string());
        self.main_wid.redraw();
    }

    pub fn configure(&mut self, widget: &DeclWidget) {
        widget
            .label
            .as_ref()
            .map(|label| self.main_wid.set_label(label));
        widget.labelcolor.as_ref().map(|labelcolor| {
            if let Ok(col) = enums::Color::from_hex_str(labelcolor) {
                self.main_wid.set_label_color(col);
            }
        });
    }
}

widget_extends!(Gauge, group::Group, main_wid);

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
