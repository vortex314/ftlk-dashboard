
use fltk::{prelude::*, *};
use notify::{
    event::{AccessKind, AccessMode, DataChange, EventKind, ModifyKind},
    Event, RecursiveMode, Watcher,
};
use serde_derive::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

pub const FRAMES: &[&str] = &[
    "NoBox",
    "FlatBox",
    "UpBox",
    "DownBox",
    "UpFrame",
    "DownFrame",
    "ThinUpBox",
    "ThinDownBox",
    "ThinUpFrame",
    "ThinDownFrame",
    "EngravedBox",
    "EmbossedBox",
    "EngravedFrame",
    "EmbossedFrame",
    "BorderBox",
    "ShadowBox",
    "BorderFrame",
    "ShadowFrame",
    "RoundedBox",
    "RShadowBox",
    "RoundedFrame",
    "RFlatBox",
    "RoundUpBox",
    "RoundDownBox",
    "DiamondUpBox",
    "DiamondDownBox",
    "OvalBox",
    "OShadowBox",
    "OvalFrame",
    "OFlatFrame",
    "PlasticUpBox",
    "PlasticDownBox",
    "PlasticUpFrame",
    "PlasticDownFrame",
    "PlasticThinUpBox",
    "PlasticThinDownBox",
    "PlasticRoundUpBox",
    "PlasticRoundDownBox",
    "GtkUpBox",
    "GtkDownBox",
    "GtkUpFrame",
    "GtkDownFrame",
    "GtkThinUpBox",
    "GtkThinDownBox",
    "GtkThinUpFrame",
    "GtkThinDownFrame",
    "GtkRoundUpFrame",
    "GtkRoundDownFrame",
    "GleamUpBox",
    "GleamDownBox",
    "GleamUpFrame",
    "GleamDownFrame",
    "GleamThinUpBox",
    "GleamThinDownBox",
    "GleamRoundUpBox",
    "GleamRoundDownBox",
];

pub(crate) fn get_frame(s: &str) -> Option<usize> {
    FRAMES.iter().position(|&x| x == s)
}


use fltk::{prelude::*, *};

macro_rules! handle_text {
    ($w: ident, $widget: ident) => {
        if let Some(col) = &$w.textcolor {
            if let Ok(col) = enums::Color::from_hex_str(col) {
                $widget.set_text_color(col);
            }
        }
        if let Some(f) = $w.textfont {
            if f < 14 {
                $widget.set_text_font(unsafe { std::mem::transmute(f) });
            }
        }
        if let Some(sz) = $w.textsize {
            $widget.set_text_size(sz);
        }
    };
}

pub(crate) fn handle_w<T>(w: &Widget, widget: &mut T)
where
    T: Clone + Send + Sync + WidgetExt + 'static,
{
    if let Some(id) = &w.id {
        widget.set_id(id);
    }
    if let Some(label) = &w.label {
        widget.set_label(label);
    }
    if w.x.is_some() || w.y.is_some() || w.w.is_some() || w.h.is_some() {
        widget.resize(
            w.x.unwrap_or(widget.x()),
            w.y.unwrap_or(widget.y()),
            w.w.unwrap_or(widget.w()),
            w.h.unwrap_or(widget.h()),
        );
    }
    if let Some(fixed) = w.fixed {
        if let Some(parent) = widget.parent() {
            if let Some(mut flex) = group::Flex::from_dyn_widget(&parent) {
                flex.fixed(widget, fixed);
            }
        }
    }
    if let Some(margin) = w.margin {
        if let Some(mut flex) = group::Flex::from_dyn_widget(widget) {
            flex.set_margin(margin);
        }
    }
    if w.left.is_some() || w.top.is_some() || w.right.is_some() || w.bottom.is_some() {
        if let Some(mut flex) = group::Flex::from_dyn_widget(widget) {
            let old = flex.margins();
            flex.set_margins(
                w.left.unwrap_or(old.0),
                w.top.unwrap_or(old.1),
                w.right.unwrap_or(old.2),
                w.bottom.unwrap_or(old.3),
            );
        }
    }
    if let Some(col) = &w.color {
        if let Ok(col) = enums::Color::from_hex_str(col) {
            widget.set_color(col);
        }
    }
    if let Some(col) = &w.selectioncolor {
        if let Ok(col) = enums::Color::from_hex_str(col) {
            widget.set_selection_color(col);
        }
    }
    if let Some(col) = &w.labelcolor {
        if let Ok(col) = enums::Color::from_hex_str(col) {
            widget.set_label_color(col);
        }
    }
    if let Some(children) = &w.children {
        for c in children {
            transform(c);
        }
    }
    if let Some(v) = w.hide {
        if v {
            widget.hide();
        }
    }
    if let Some(v) = w.deactivate {
        if v {
            widget.deactivate();
        }
    }
    if let Some(v) = w.visible {
        if v {
            widget.deactivate();
        }
    }
    if let Some(v) = w.resizable {
        if v {
            if let Some(mut grp) = widget.as_group() {
                grp.make_resizable(true);
            } else if let Some(parent) = widget.parent() {
                parent.resizable(widget);
            }
        }
    }
    if let Some(tip) = &w.tooltip {
        widget.set_tooltip(tip);
    }
    if let Some(path) = &w.image {
        widget.set_image(image::SharedImage::load(path).ok());
    }
    if let Some(path) = &w.deimage {
        widget.set_deimage(image::SharedImage::load(path).ok());
    }
    if let Some(sz) = w.labelsize {
        widget.set_label_size(sz);
    }
    if let Some(a) = w.align {
        widget.set_align(unsafe { std::mem::transmute(a) });
    }
    if let Some(a) = w.when {
        widget.set_trigger(unsafe { std::mem::transmute(a) });
    }
    if let Some(f) = w.labelfont {
        if f < 14 {
            widget.set_label_font(unsafe { std::mem::transmute(f) });
        }
    }
    if let Some(f) = &w.frame {
        if let Some(f) = get_frame(f) {
            widget.set_frame(unsafe { std::mem::transmute(f as i32) });
        }
    }
    if let Some(mut b) = button::Button::from_dyn_widget(widget) {
        if let Some(f) = &w.downframe {
            if let Some(f) = get_frame(f) {
                b.set_down_frame(unsafe { std::mem::transmute(f as i32) });
            }
        }
        if let Some(f) = &w.shortcut {
            if let Ok(sh) = f.parse::<i32>() {
                b.set_shortcut(unsafe { std::mem::transmute(sh) });
            }
        }
    }
    if let Some(mut b) = valuator::Slider::from_dyn_widget(widget) {
        if let Some(sz) = w.minimum {
            b.set_minimum(sz);
        }
        if let Some(sz) = w.maximum {
            b.set_maximum(sz);
        }
        if let Some(sz) = w.slidersize {
            b.set_slider_size(sz as _);
        }
        if let Some(sz) = w.step {
            b.set_step(sz, 1);
        }
    }
    if let Some(gap) = w.pad {
        if let Some(mut b) = group::Flex::from_dyn_widget(widget) {
            b.set_pad(gap);
        }
    }
    if let Some(grp) = group::Group::from_dyn_widget(widget) {
        grp.end();
    }
}

mod widget;
use widget::*;

pub(crate) fn transform(w: &Widget) {
    match w.widget.as_str() {
        "Gauge" => {
            let mut g = widget::gauge::Gauge::new(400,400,150,150,"gauge");
            g.set_value(50);
        //    handle_w(w, &mut g);
        }
        "Column" => {
            let mut c = group::Flex::default_fill().column();
            handle_w(w, &mut c);
        }
        "Row" => {
            let mut c = group::Flex::default_fill().row();
            handle_w(w, &mut c);
        }
        "Button" => {
            let mut b = button::Button::default_fill();
            handle_w(w, &mut b);
        }
        "CheckButton" => {
            let mut b = button::CheckButton::default_fill();
            handle_w(w, &mut b);
        }
        "RadioButton" => {
            let mut b = button::RadioButton::default_fill();
            handle_w(w, &mut b);
        }
        "ToggleButton" => {
            let mut b = button::ToggleButton::default_fill();
            handle_w(w, &mut b);
        }
        "RadioRoundButton" => {
            let mut b = button::RadioRoundButton::default_fill();
            handle_w(w, &mut b);
        }
        "ReturnButton" => {
            let mut b = button::ReturnButton::default_fill();
            handle_w(w, &mut b);
        }
        "Frame" => {
            let mut f = frame::Frame::default_fill();
            handle_w(w, &mut f);
        }
        "Group" => {
            let mut f = group::Group::default_fill();
            handle_w(w, &mut f);
        }
        "Pack" => {
            let mut f = group::Pack::default_fill();
            handle_w(w, &mut f);
        }
        "Tile" => {
            let mut f = group::Tile::default_fill();
            handle_w(w, &mut f);
        }
        "Tabs" => {
            let mut f = group::Tabs::default_fill();
            handle_w(w, &mut f);
        }
        "Scroll" => {
            let mut f = group::Scroll::default_fill();
            handle_w(w, &mut f);
        }
        "ColorChooser" => {
            let mut f = group::ColorChooser::default_fill();
            handle_w(w, &mut f);
        }
        "TextDisplay" => {
            let mut f = text::TextDisplay::default_fill();
            handle_text!(w, f);
            let buf = text::TextBuffer::default();
            f.set_buffer(buf);
            handle_w(w, &mut f);
        }
        "TextEditor" => {
            let mut f = text::TextEditor::default_fill();
            handle_text!(w, f);
            let buf = text::TextBuffer::default();
            f.set_buffer(buf);
            handle_w(w, &mut f);
        }
        "Input" => {
            let mut f = input::Input::default_fill();
            handle_text!(w, f);
            handle_w(w, &mut f);
        }
        "IntInput" => {
            let mut f = input::IntInput::default_fill();
            handle_text!(w, f);
            handle_w(w, &mut f);
        }
        "FloatInput" => {
            let mut f = input::FloatInput::default_fill();
            handle_text!(w, f);
            handle_w(w, &mut f);
        }
        "SecretInput" => {
            let mut f = input::SecretInput::default_fill();
            handle_text!(w, f);
            handle_w(w, &mut f);
        }
        "FileInput" => {
            let mut f = input::FileInput::default_fill();
            handle_text!(w, f);
            handle_w(w, &mut f);
        }
        "MultilineInput" => {
            let mut f = input::MultilineInput::default_fill();
            handle_text!(w, f);
            handle_w(w, &mut f);
        }
        "Output" => {
            let mut f = output::Output::default_fill();
            handle_text!(w, f);
            handle_w(w, &mut f);
        }
        "MultilineOutput" => {
            let mut f = output::Output::default_fill();
            handle_text!(w, f);
            handle_w(w, &mut f);
        }
        "MenuBar" => {
            let mut f = menu::MenuBar::default_fill();
            handle_text!(w, f);
            handle_w(w, &mut f);
        }
        "SysMenuBar" => {
            let mut f = menu::SysMenuBar::default_fill();
            handle_text!(w, f);
            handle_w(w, &mut f);
        }
        "Choice" => {
            let mut f = menu::Choice::default_fill();
            handle_text!(w, f);
            handle_w(w, &mut f);
        }
        "Slider" => {
            let mut f = valuator::Slider::default_fill();
            handle_w(w, &mut f);
        }
        "NiceSlider" => {
            let mut f = valuator::NiceSlider::default_fill();
            handle_w(w, &mut f);
        }
        "FillSlider" => {
            let mut f = valuator::FillSlider::default_fill();
            handle_w(w, &mut f);
        }
        "ValueSlider" => {
            let mut f = valuator::ValueSlider::default_fill();
            handle_text!(w, f);
            handle_w(w, &mut f);
        }
        "Dial" => {
            let mut f = valuator::Dial::default_fill();
            handle_w(w, &mut f);
        }
        "LineDial" => {
            let mut f = valuator::LineDial::default_fill();
            handle_w(w, &mut f);
        }
        "FillDial" => {
            let mut f = valuator::FillDial::default_fill();
            handle_w(w, &mut f);
        }
        "Counter" => {
            let mut f = valuator::Counter::default_fill();
            handle_w(w, &mut f);
        }
        "Scrollbar" => {
            let mut f = valuator::Scrollbar::default_fill();
            handle_w(w, &mut f);
        }
        "Roller" => {
            let mut f = valuator::Roller::default_fill();
            handle_w(w, &mut f);
        }
        "Adjuster" => {
            let mut f = valuator::Adjuster::default_fill();
            handle_w(w, &mut f);
        }
        "ValueInput" => {
            let mut f = valuator::ValueInput::default_fill();
            handle_text!(w, f);
            handle_w(w, &mut f);
        }
        "ValueOutput" => {
            let mut f = valuator::ValueOutput::default_fill();
            handle_text!(w, f);
            handle_w(w, &mut f);
        }
        "HorSlider" => {
            let mut f = valuator::HorSlider::default_fill();
            handle_w(w, &mut f);
        }
        "HorNiceSlider" => {
            let mut f = valuator::HorNiceSlider::default_fill();
            handle_w(w, &mut f);
        }
        "HorFillSlider" => {
            let mut f = valuator::HorFillSlider::default_fill();
            handle_w(w, &mut f);
        }
        "HorValueSlider" => {
            let mut f = valuator::HorValueSlider::default_fill();
            handle_text!(w, f);
            handle_w(w, &mut f);
        }
        "Browser" => {
            let mut f = browser::Browser::default_fill();
            handle_w(w, &mut f);
        }
        "SelectBrowser" => {
            let mut f = browser::SelectBrowser::default_fill();
            handle_w(w, &mut f);
        }
        "HoldBrowser" => {
            let mut f = browser::HoldBrowser::default_fill();
            handle_w(w, &mut f);
        }
        "FileBrowser" => {
            let mut f = browser::FileBrowser::default_fill();
            handle_w(w, &mut f);
        }
        "CheckBrowser" => {
            let mut f = browser::CheckBrowser::default_fill();
            handle_text!(w, f);
            handle_w(w, &mut f);
        }
        "MultiBrowser" => {
            let mut f = browser::MultiBrowser::default_fill();
            handle_w(w, &mut f);
        }
        "Table" => {
            let mut f = table::Table::default_fill();
            handle_w(w, &mut f);
        }
        "TableRow" => {
            let mut f = table::TableRow::default_fill();
            handle_w(w, &mut f);
        }
        "Tree" => {
            let mut f = tree::Tree::default_fill();
            handle_w(w, &mut f);
        }
        "Spinner" => {
            let mut f = misc::Spinner::default_fill();
            handle_text!(w, f);
            handle_w(w, &mut f);
        }
        "Chart" => {
            let mut f = misc::Chart::default_fill();
            handle_text!(w, f);
            handle_w(w, &mut f);
        }
        "Progress" => {
            let mut f = misc::Progress::default_fill();
            handle_w(w, &mut f);
        }
        "InputChoice" => {
            let mut f = misc::InputChoice::default_fill();
            handle_text!(w, f);
            handle_w(w, &mut f);
        }
        "HelpView" => {
            let mut f = misc::HelpView::default_fill();
            handle_text!(w, f);
            handle_w(w, &mut f);
        }
        _ => (),
    };
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Widget {
    widget: String,
    label: Option<String>,
    id: Option<String>,
    fixed: Option<i32>,
    color: Option<String>,
    labelcolor: Option<String>,
    children: Option<Vec<Widget>>,
    hide: Option<bool>,
    deactivate: Option<bool>,
    visible: Option<bool>,
    resizable: Option<bool>,
    selectioncolor: Option<String>,
    tooltip: Option<String>,
    image: Option<String>,
    deimage: Option<String>,
    labelfont: Option<u32>,
    labelsize: Option<i32>,
    align: Option<i32>,
    when: Option<i32>,
    frame: Option<String>,
    downframe: Option<String>,
    shortcut: Option<String>,
    pad: Option<i32>,
    minimum: Option<f64>,
    maximum: Option<f64>,
    step: Option<f64>,
    slidersize: Option<f64>,
    textfont: Option<i32>,
    textsize: Option<i32>,
    textcolor: Option<String>,
    x: Option<i32>,
    y: Option<i32>,
    w: Option<i32>,
    h: Option<i32>,
    margin: Option<i32>,
    left: Option<i32>,
    top: Option<i32>,
    right: Option<i32>,
    bottom: Option<i32>,
}

/// Entry point for your declarative app
#[derive(Debug, Clone)]
pub struct DeclarativeApp {
    a: app::App,
    w: i32,
    h: i32,
    label: String,
    #[allow(dead_code)]
    path: Option<&'static str>,
    widget: Option<Widget>,
    load_fn: fn(&'static str) -> Option<Widget>,
}

impl DeclarativeApp {
    /// Instantiate a new declarative app
    pub fn new(
        w: i32,
        h: i32,
        label: &str,
        path: &'static str,
        load_fn: fn(&'static str) -> Option<Widget>,
    ) -> Self {
        let widget = load_fn(path);
        let a = app::App::default().with_scheme(app::Scheme::Gtk);
        Self {
            a,
            w,
            h,
            label: label.to_string(),
            path: Some(path),
            widget,
            load_fn,
        }
    }

    #[cfg(feature = "json")]
    pub fn new_json(w: i32, h: i32, label: &str, path: &'static str) -> Self {
        fn load_fn(path: &'static str) -> Option<Widget> {
            let s = std::fs::read_to_string(path).ok()?;
            serde_json::from_str(&s).map_err(|e| eprintln!("{e}")).ok()
        }
        Self::new(w, h, label, path, load_fn)
    }

    #[cfg(feature = "json5")]
    pub fn new_json5(w: i32, h: i32, label: &str, path: &'static str) -> Self {
        fn load_fn(path: &'static str) -> Option<Widget> {
            let s = std::fs::read_to_string(path).ok()?;
            serde_json5::from_str(&s).map_err(|e| eprintln!("{e}")).ok()
        }
        Self::new(w, h, label, path, load_fn)
    }

    #[cfg(feature = "xml")]
    pub fn new_xml(w: i32, h: i32, label: &str, path: &'static str) -> Self {
        fn load_fn(path: &'static str) -> Option<Widget> {
            let s = std::fs::read_to_string(path).ok()?;
            serde_xml_rs::from_str(&s)
                .map_err(|e| eprintln!("{e}"))
                .ok()
        }
        Self::new(w, h, label, path, load_fn)
    }

    #[cfg(feature = "yaml")]
    pub fn new_yaml(w: i32, h: i32, label: &str, path: &'static str) -> Self {
        fn load_fn(path: &'static str) -> Option<Widget> {
            let s = std::fs::read_to_string(path).ok()?;
            serde_yaml::from_str(&s).map_err(|e| eprintln!("{e}")).ok()
        }
        Self::new(w, h, label, path, load_fn)
    }

    /// Instantiate a new declarative app
    pub fn new_inline(w: i32, h: i32, label: &str, widget: Option<Widget>) -> Self {
        let a = app::App::default().with_scheme(app::Scheme::Gtk);
        Self {
            a,
            w,
            h,
            label: label.to_string(),
            path: None,
            widget,
            load_fn: |_| None,
        }
    }

    /// Run your declarative app.
    /// The callback exposes the app's main window
    pub fn run<F: FnMut(&mut window::Window) + 'static>(
        &self,
        mut run_cb: F,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = &self.path {
            let mut win = window::Window::default()
                .with_size(self.w, self.h)
                .with_label(&self.label);
            if let Some(widget) = &self.widget {
                transform(widget);
            }
            win.end();
            win.show();

            if let Some(mut frst) = win.child(0) {
                frst.resize(0, 0, win.w(), win.h());
                win.resizable(&frst);
            }

            run_cb(&mut win);

            let flag = Arc::new(AtomicBool::new(false));
            app::add_timeout3(0.1, {
                let flag = flag.clone();
                let mut win = win.clone();
                move |_t| {
                    if flag.load(Ordering::Relaxed) {
                        run_cb(&mut win);
                        flag.store(false, Ordering::Relaxed);
                    }
                    app::repeat_timeout3(0.1, _t);
                }
            });

            let load_fn = self.load_fn;
            let mut watcher = notify::recommended_watcher({
                let path = <&str>::clone(path);
                move |res: Result<Event, notify::Error>| match res {
                    Ok(event) => {
                        let mut needs_update = false;
                        match event.kind {
                            EventKind::Access(AccessKind::Close(mode)) => {
                                if mode == AccessMode::Write {
                                    needs_update = true;
                                }
                            }
                            EventKind::Modify(ModifyKind::Data(DataChange::Content)) => {
                                needs_update = true;
                            }
                            _ => (),
                        }
                        if needs_update {
                            if let Some(wid) = (load_fn)(path) {
                                win.clear();
                                win.begin();
                                transform(&wid);
                                win.end();
                                if let Some(mut frst) = win.child(0) {
                                    frst.resize(0, 0, win.w(), win.h());
                                    win.resizable(&frst);
                                }
                                app::redraw();
                                flag.store(true, Ordering::Relaxed);
                            }
                        }
                    }
                    Err(e) => eprintln!("{}", e),
                }
            })?;
            watcher.watch(&PathBuf::from(path), RecursiveMode::NonRecursive)?;

            self.a.run()?;
        } else {
            self.run_once(run_cb)?;
        }
        Ok(())
    }

    /// Run the app without hot-reloading!
    pub fn run_once<F: FnMut(&mut window::Window) + 'static>(
        &self,
        mut run_cb: F,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut win = window::Window::default()
            .with_size(self.w, self.h)
            .with_label(&self.label);
        if let Some(widget) = &self.widget {
            transform(widget);
        }
        win.end();
        win.show();

        if let Some(mut frst) = win.child(0) {
            frst.resize(0, 0, win.w(), win.h());
            win.resizable(&frst);
        }

        run_cb(&mut win);

        self.a.run()?;
        Ok(())
    }

    /// Just load the image of the window
    pub fn dump_image(&self) {
        let mut win = window::Window::default()
            .with_size(self.w, self.h)
            .with_label(&self.label);
        if let Some(widget) = &self.widget {
            transform(widget);
        }
        win.end();
        win.show();

        if let Some(mut frst) = win.child(0) {
            frst.resize(0, 0, win.w(), win.h());
            win.resizable(&frst);
        }
        let sur = surface::SvgFileSurface::new(win.w(), win.h(), "temp.svg");
        surface::SvgFileSurface::push_current(&sur);
        draw::set_draw_color(enums::Color::White);
        draw::draw_rectf(0, 0, win.w(), win.h());
        sur.draw(&win, 0, 0);
        surface::SvgFileSurface::pop_current();
    }
}
