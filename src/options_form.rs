// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use super::CONFIG;
use crate::fixed::{
    APPNAME, BUTTON_HEIGHT, BUTTON_WIDTH, DEF_HISTORY_SIZE, ICON,
    MAX_HISTORY_SIZE, MIN_HISTORY_SIZE, PAD, SCALE_MAX, SCALE_MIN,
};
use crate::util;
use fltk::{
    app,
    button::{Button, CheckButton},
    enums::{Align, FrameType},
    frame::Frame,
    group::Flex,
    image::SvgImage,
    misc::Spinner,
    prelude::*,
    window::Window,
};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Form {
    form: Window,
    pub ok: Rc<RefCell<bool>>,
}

impl Form {
    pub fn default() -> Self {
        let ok = Rc::from(RefCell::from(false));
        let mut form = make_form();
        let mut vbox = Flex::default().size_of_parent().column();
        vbox.set_margin(PAD);
        vbox.set_pad(PAD);
        make_config_row();
        let mut widgets = make_widgets();
        let (button_row, mut buttons) = make_buttons();
        vbox.set_size(&button_row, BUTTON_HEIGHT);
        vbox.end();
        form.end();
        form.make_modal(true);
        add_event_handlers(
            &mut form,
            &widgets,
            &mut buttons,
            Rc::clone(&ok),
        );
        widgets.history_size_spinner.take_focus().unwrap();
        form.show();
        while form.shown() {
            app::wait();
        }
        Self { form, ok }
    }
}

impl Drop for Form {
    fn drop(&mut self) {
        app::delete_widget(self.form.clone());
    }
}

struct Widgets {
    pub history_size_spinner: Spinner,
    pub auto_save_checkbox: CheckButton,
    pub scale_spinner: Spinner,
}

struct Buttons {
    pub ok_button: Button,
    pub cancel_button: Button,
}

fn make_form() -> Window {
    let image = SvgImage::from_data(ICON).unwrap();
    let mut form = Window::default()
        .with_size(WIDTH, HEIGHT)
        .with_label(&format!("Configure — {APPNAME}"));
    if let Some(window) = app::first_window() {
        form.set_pos(window.x() + 50, window.y() + 100);
    }
    form.set_icon(Some(image));
    form
}

fn make_config_row() {
    let mut row = Flex::default().row();
    let label = Frame::default()
        .with_label("Config")
        .with_align(Align::Inside | Align::Left);
    let config = CONFIG.get().read().unwrap();
    let mut filename_label = Frame::default()
        .with_label(&config.filename.to_string_lossy())
        .with_align(Align::Inside | Align::Left);
    filename_label.set_frame(FrameType::EngravedFrame);
    row.set_size(&label, WIDTH / 6);
    row.end();
}

fn make_widgets() -> Widgets {
    let config = CONFIG.get().read().unwrap();
    let history_size_spinner = make_row(
        "&History Size",
        config.history_size as f64,
        &format!("The maximum number of tracks to keep in the history menu (default {DEF_HISTORY_SIZE})", ),
        MIN_HISTORY_SIZE as f64,
        MAX_HISTORY_SIZE as f64,
        1.0,
    );
    let mut row = Flex::default().row();
    row.set_pad(PAD);
    Frame::default().with_size(PAD, PAD);
    let mut auto_save_checkbox = CheckButton::default()
        .with_label("&Auto Save")
        .with_align(Align::Inside | Align::Left);
    auto_save_checkbox.set_checked(config.auto_save);
    row.end();
    let scale_spinner = make_row(
        "&Scale",
        config.window_scale as f64,
        "User interface scale (default 1.0)",
        SCALE_MIN as f64,
        SCALE_MAX as f64,
        0.1,
    );
    Widgets { history_size_spinner, auto_save_checkbox, scale_spinner }
}

fn make_row(
    label: &str,
    value: f64,
    tooltip: &str,
    minimum: f64,
    maximum: f64,
    step: f64,
) -> Spinner {
    let mut row = Flex::default().row();
    row.set_pad(PAD);
    let mut label = Button::default()
        .with_label(label)
        .with_align(Align::Inside | Align::Left);
    label.set_frame(FrameType::NoBox);
    label.clear_visible_focus();
    let mut spinner = Spinner::default();
    spinner.set_value(value);
    spinner.set_step(step);
    spinner.set_range(minimum, maximum);
    spinner.set_tooltip(tooltip);
    spinner.set_wrap(false);
    row.end();
    label.set_callback({
        let mut spinner = spinner.clone();
        move |_| {
            spinner.take_focus().unwrap();
        }
    });
    spinner
}

fn make_buttons() -> (Flex, Buttons) {
    let mut row = Flex::default().size_of_parent().row();
    row.set_pad(PAD);
    Frame::default(); // pad left of buttons
    let ok_button = Button::default().with_label("&OK");
    let cancel_button = Button::default().with_label("&Cancel");
    Frame::default(); // pad right of buttons
    row.set_size(&ok_button, BUTTON_WIDTH);
    row.set_size(&cancel_button, BUTTON_WIDTH);
    row.end();
    (row, Buttons { ok_button, cancel_button })
}

fn add_event_handlers(
    form: &mut Window,
    widgets: &Widgets,
    buttons: &mut Buttons,
    ok: Rc<RefCell<bool>>,
) {
    buttons.ok_button.set_callback({
        let auto_save_checkbox = widgets.auto_save_checkbox.clone();
        let history_size_spinner = widgets.history_size_spinner.clone();
        let scale_spinner = widgets.scale_spinner.clone();
        let mut form = form.clone();
        move |_| {
            *ok.borrow_mut() = true;
            let mut config = CONFIG.get().write().unwrap();
            let scale = scale_spinner.value() as f32;
            if !util::isclose32(config.window_scale, scale) {
                config.window_scale = scale;
                app::set_screen_scale(0, scale);
            }
            config.history_size = history_size_spinner.value() as usize;
            config.auto_save = auto_save_checkbox.is_checked();
            form.hide();
        }
    });
    buttons.cancel_button.set_callback({
        let mut form = form.clone();
        move |_| {
            form.hide();
        }
    });
}

const WIDTH: i32 = 340;
const HEIGHT: i32 = 150;
