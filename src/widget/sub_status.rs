use fltk::button::Button;
use fltk::enums::Color;
use fltk::widget::Widget;
use fltk::{enums::*, prelude::*, *};
use serde_yaml::Value;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use std::time::SystemTime;

use crate::decl::DeclWidget;
use crate::pubsub::PubSubEvent;
use crate::widget::dnd_callback;
use crate::widget::Context;
use crate::widget::GridRectangle;
use crate::widget::PubSubWidget;
use tokio::sync::mpsc;

use super::WidgetParams;

#[derive(Debug, Clone)]
pub struct SubStatus {
    status_frame: frame::Frame,
    last_update: SystemTime,
    widget_params: WidgetParams,
    alive: bool,
    ctx: Context,
}

impl SubStatus {
    pub fn new() -> SubStatus {
        let mut status_frame = frame::Frame::default().with_label("Status");
        status_frame.set_frame(FrameType::BorderBox);
        status_frame.set_color(Color::from_u32(0xff0000));
        status_frame.handle(move |w, ev| dnd_callback(&mut w.as_base_widget(), ev));
        SubStatus {
            status_frame,
            last_update: std::time::UNIX_EPOCH,
            alive: false,
            widget_params: WidgetParams::new(),
            ctx: Context::new(),
        }
    }

    fn reconfigure(&mut self) {
        info!("Status::topic {:?}", self.widget_params.src_topic.clone().unwrap_or("".into()));
        if let Some(size) = self.widget_params.size {
            if let Some(pos) = self.widget_params.pos {
                self.status_frame.resize(
                    pos.0 * self.ctx.grid_width,
                    pos.1 * self.ctx.grid_height,
                    size.0 * self.ctx.grid_width,
                    size.1 * self.ctx.grid_height,
                );
            }
        }
        self.widget_params
            .label
            .as_ref()
            .map(|s| self.status_frame.set_label(s.as_str()));
    }
}

/*

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
}*/

impl PubSubWidget for SubStatus {
    fn set_config(&mut self, props: WidgetParams) {
        debug!("Status::config() {:?}", props);
        self.widget_params = props;
        self.reconfigure();
    }

    fn set_context(&mut self, context: Context) {
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
                if topic == src_topic {
                    self.last_update = std::time::SystemTime::now();
                    if !self.alive {
                        debug!("Status::on() {} Alive", src_topic);
                        self.alive = true;
                        self.status_frame.set_color(Color::from_hex(0x00ff00));
                        self.status_frame.parent().unwrap().redraw();
                    }
                }
            }
            PubSubEvent::Timer1sec => {
                let delta = std::time::SystemTime::now()
                    .duration_since(self.last_update)
                    .unwrap()
                    .as_millis();
                if delta > self.widget_params.src_timeout.unwrap() as u128 {
                    if self.alive {
                        debug!("Status::on() {} Expired", src_topic);
                        self.alive = false;
                        self.status_frame.set_color(Color::from_hex(0xff0000));
                        self.status_frame.parent().unwrap().redraw();
                    }
                }
            }
        }
    }
    fn set_publish_channel(&mut self, channel: tokio::sync::mpsc::Sender<PubSubEvent>) {
        info!("Status::set_publish_channel()");
    }
}
