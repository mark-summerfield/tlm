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
    Select(usize), // index
    Delete(usize), // index
    DeleteAll,
    Cancel,
}

#[derive(Clone)]
struct Widgets {
    pub browser: HoldBrowser,
    pub select_button: Button,
    pub delete_button: Button,
    pub delete_all_button: Button,
    pub cancel_button: Button,
}

pub struct Form {
    form: Window,
    widgets: Widgets,
    pub reply: Rc<RefCell<Reply>>,
}

impl Form {
    pub fn new(
        title: &str,
        select: &str,
        delete: &str,
        list: &[String],
    ) -> Self {
        let reply = Rc::from(RefCell::from(Reply::Cancel));
        let mut form = make_form(title);
        let mut vbox = Flex::default().size_of_parent().column();
        vbox.set_margin(PAD);
        vbox.set_pad(PAD);
        let (button_row, mut widgets) = make_widgets(select, delete, list);
        vbox.set_size(&button_row, BUTTON_HEIGHT);
        vbox.end();
        form.end();
        form.make_modal(true);
        widgets.browser.take_focus().unwrap();
        form.show();
        let mut app =
            Self { form: form.clone(), widgets: widgets.clone(), reply };
        app.add_event_handlers(&mut widgets.cancel_button);
        update_ui(&mut widgets);
        while form.shown() {
            app::wait();
        }
        app
    }

    fn add_event_handlers(&mut self, cancel_button: &mut Button) {
        let reply = Rc::clone(&self.reply);
        self.widgets.browser.handle({
            let mut widgets = self.widgets.clone();
            move |browser, _| {
                if browser.has_focus()
                    && !app::event_inside_widget(&browser.scrollbar())
                {
                    update_ui(&mut widgets);
                }
                false
            }
        });
        self.widgets.select_button.set_callback({
            let reply_a = Rc::clone(&reply);
            let mut form = self.form.clone();
            let browser = self.widgets.browser.clone();
            move |_| {
                // Browser uses 1-based indexing
                let index = (browser.value() as usize) - 1;
                *reply_a.borrow_mut() = Reply::Select(index);
                form.hide();
            }
        });
        self.widgets.delete_button.set_callback({
            let reply_b = Rc::clone(&reply);
            let mut form = self.form.clone();
            let browser = self.widgets.browser.clone();
            move |_| {
                // Browser uses 1-based indexing
                let index = (browser.value() as usize) - 1;
                *reply_b.borrow_mut() = Reply::Delete(index);
                form.hide();
            }
        });
        self.widgets.delete_all_button.set_callback({
            let reply_c = Rc::clone(&reply);
            let mut form = self.form.clone();
            move |_| {
                *reply_c.borrow_mut() = Reply::DeleteAll;
                form.hide();
            }
        });
        cancel_button.set_callback({
            let reply_d = Rc::clone(&reply);
            let mut form = self.form.clone();
            move |_| {
                *reply_d.borrow_mut() = Reply::Cancel;
                form.hide();
            }
        });
    }
}

impl Drop for Form {
    fn drop(&mut self) {
        app::delete_widget(self.form.clone());
    }
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

fn make_widgets(
    select: &str,
    delete: &str,
    list: &[String],
) -> (Flex, Widgets) {
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
    let select_button = Button::default().with_label(select);
    let delete_button = Button::default().with_label(delete);
    let delete_all_button = Button::default().with_label("Delete &All");
    let cancel_button = Button::default().with_label("&Cancel");
    Frame::default(); // pad right of buttons
    let width = BUTTON_WIDTH + PAD;
    row.set_size(&select_button, width);
    row.set_size(&delete_button, width);
    row.set_size(&delete_all_button, width);
    row.set_size(&cancel_button, width);
    row.end();
    (
        row,
        Widgets {
            browser,
            select_button,
            delete_button,
            delete_all_button,
            cancel_button,
        },
    )
}

fn update_ui(widgets: &mut Widgets) {
    if widgets.browser.size() == 0 {
        widgets.select_button.deactivate();
    } else {
        widgets.select_button.activate();
    }
    // Can't delete or clear 1 or 0 items or if first selected
    let disable = widgets.browser.size() < 2 || widgets.browser.selected(1);
    if disable {
        widgets.delete_button.deactivate();
        widgets.delete_all_button.deactivate();
    } else {
        widgets.delete_button.activate();
        widgets.delete_all_button.activate();
    }
    app::redraw(); // redraws the world
}

const WIDTH: i32 = 480;
const HEIGHT: i32 = 360;
