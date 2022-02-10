// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use crate::fixed::{APPNAME, BUTTON_HEIGHT, BUTTON_WIDTH, ICON, PAD};
use fltk::{
    app, browser::HoldBrowser, button::Button, frame::Frame, group::Flex,
    image::SvgImage, prelude::*, window::Window,
};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Reply {
    Action(usize), // index
    Clear,
    Cancel,
}

pub struct Form {
    form: Window,
    pub reply: Rc<RefCell<Reply>>,
}

impl Form {
    pub fn new(title: &str, action: &str, list: &[String]) -> Self {
        let reply = Rc::from(RefCell::from(Reply::Cancel));
        let mut form = make_form(title);
        let mut vbox = Flex::default().size_of_parent().column();
        vbox.set_margin(PAD);
        vbox.set_pad(PAD);
        let (button_row, mut widgets) = make_widgets(action, list);
        vbox.set_size(&button_row, BUTTON_HEIGHT);
        vbox.end();
        form.end();
        form.make_modal(true);
        add_event_handlers(&mut form, &mut widgets, Rc::clone(&reply));
        widgets.browser.take_focus().unwrap();
        form.show();
        while form.shown() {
            app::wait();
        }
        Self { form, reply }
    }
}

impl Drop for Form {
    fn drop(&mut self) {
        app::delete_widget(self.form.clone());
    }
}

struct Widgets {
    pub browser: HoldBrowser,
    pub action_button: Button,
    pub clear_button: Button,
    pub cancel_button: Button,
}

fn make_form(title: &str) -> Window {
    let image = SvgImage::from_data(ICON).unwrap();
    let mut form = Window::default()
        .with_size(WIDTH, HEIGHT)
        .with_label(&format!("{title} — {APPNAME}"));
    if let Some(window) = app::first_window() {
        form.set_pos(window.x() + 50, window.y() + 100);
    }
    form.set_icon(Some(image));
    form
}

fn make_widgets(action: &str, list: &[String]) -> (Flex, Widgets) {
    let mut row = Flex::default().row();
    row.set_pad(PAD);
    let mut browser = HoldBrowser::default();
    for item in list {
        browser.add(item);
    }
    if list.len() > 1 {
        browser.select(2);
    } else if !list.is_empty() {
        browser.select(1);
    }
    browser.top_line(1);
    row.end();
    let mut row = Flex::default().size_of_parent().row();
    row.set_pad(PAD);
    Frame::default(); // pad left of buttons
    let action_button = Button::default().with_label(action);
    let clear_button = Button::default().with_label("Clear &List");
    let cancel_button = Button::default().with_label("&Cancel");
    Frame::default(); // pad right of buttons
    let width = BUTTON_WIDTH + PAD;
    row.set_size(&action_button, width);
    row.set_size(&clear_button, width);
    row.set_size(&cancel_button, width);
    row.end();
    (row, Widgets { browser, action_button, clear_button, cancel_button })
}

fn add_event_handlers(
    form: &mut Window,
    widgets: &mut Widgets,
    reply: Rc<RefCell<Reply>>,
) {
    widgets.action_button.set_callback({
        let reply_a = Rc::clone(&reply);
        let mut form = form.clone();
        let browser = widgets.browser.clone();
        move |_| {
            // Browser uses 1-based indexing
            let index = (browser.value() as usize) - 1;
            *reply_a.borrow_mut() = Reply::Action(index);
            form.hide();
        }
    });
    widgets.clear_button.set_callback({
        let reply_b = Rc::clone(&reply);
        let mut form = form.clone();
        move |_| {
            *reply_b.borrow_mut() = Reply::Clear;
            form.hide();
        }
    });
    widgets.cancel_button.set_callback({
        let reply_c = Rc::clone(&reply);
        let mut form = form.clone();
        move |_| {
            *reply_c.borrow_mut() = Reply::Cancel;
            form.hide();
        }
    });
}

const WIDTH: i32 = 480;
const HEIGHT: i32 = 360;
