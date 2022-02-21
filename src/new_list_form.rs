// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use super::CONFIG;
use crate::fixed::{
    APPNAME, BUTTON_HEIGHT, BUTTON_WIDTH, ICON, MAX_HISTORY_SIZE,
    MIN_HISTORY_SIZE, PAD, SCALE_MAX, SCALE_MIN,
};
use crate::util;
use fltk::{
    app,
    button::{Button, CheckButton, RadioRoundButton},
    enums::{Align, FrameType},
    frame::Frame,
    group::Flex,
    image::SvgImage,
    input::Input,
    menu::Choice,
    misc::Spinner,
    prelude::*,
    window::Window,
};
use std::{cell::RefCell, path::PathBuf, rc::Rc};

// if folder_or_playlist is a folder then read tracks from the folder;
// if folder_or_playlist is a file then read tracks from the playlist file
#[derive(Clone, Debug)]
pub struct NewListResult {
    pub ok: bool, // false means user canceled; true means create new list
    pub name: Option<String>, // None means use stem of folder_or_playlist
    pub parent_list: Option<String>, // None means top-level
    pub folder_or_playlist: Option<PathBuf>, // None means empty list
    pub include_subdirs: bool, // only meaningful for folders
}

impl NewListResult {
    pub fn default() -> Self {
        NewListResult {
            ok: false,
            name: None,
            parent_list: None,
            folder_or_playlist: None,
            include_subdirs: true,
        }
    }
}

pub struct Form {
    form: Window,
    pub result: Rc<RefCell<NewListResult>>,
}

impl Default for Form {
    fn default() -> Self {
        let result = Rc::from(RefCell::from(NewListResult::default()));
        let mut form = make_form();
        let mut vbox = Flex::default().size_of_parent().column();
        vbox.set_margin(PAD);
        vbox.set_pad(PAD);
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
            Rc::clone(&result),
        );
        widgets.parent_list_combo.take_focus().unwrap();
        form.show();
        while form.shown() {
            app::wait();
        }
        Self { form, result }
    }
}

impl Drop for Form {
    fn drop(&mut self) {
        app::delete_widget(self.form.clone());
    }
}

struct Widgets {
    pub parent_list_combo: Choice,
    pub folder_or_playlist_label: Frame,
    pub include_subdirs_checkbox: CheckButton,
    pub name_input: Input,
}

struct Buttons {
    pub ok_button: Button,
    pub cancel_button: Button,
}

fn make_form() -> Window {
    let image = SvgImage::from_data(ICON).unwrap();
    let mut form = Window::default()
        .with_size(WIDTH, HEIGHT)
        .with_label(&format!("New List — {APPNAME}"));
    if let Some(window) = app::first_window() {
        form.set_pos(window.x() + 50, window.y() + 100);
    }
    form.set_icon(Some(image));
    form
}

fn make_widgets() -> Widgets {
    let left_width = (WIDTH / 5).max(BUTTON_WIDTH * 2);
    let right_width = (WIDTH / 4).max(BUTTON_WIDTH * 2);
    let mut column = Flex::default().column();
    column.set_pad(PAD);
    let mut row1 = Flex::default().row();
    row1.set_pad(PAD);
    let mut parent_list_button =
        Button::default().with_label("&Parent List");
    parent_list_button.set_frame(FrameType::NoBox);
    parent_list_button.visible_focus(false);
    // TODO add handler to make it give focus to parent_list_combo
    let mut parent_list_combo = Choice::default();
    parent_list_combo.add_choice("<Top-Level>");
    parent_list_combo.set_value(0);
    // TODO add other top-level lists
    row1.set_size(&parent_list_button, left_width);
    row1.end();
    let mut row2 = Flex::default().row();
    row2.set_pad(PAD);
    // TODO these aren't showing as radio buttons!
    let folder_radio =
        RadioRoundButton::default().with_label("&Add tracks from folder");
    let playlist_radio = RadioRoundButton::default()
        .with_label("&Import tracks from playist");
    let mut empty_radio =
        RadioRoundButton::default().with_label("Create &Empty list");
    empty_radio.toggle(true);
    row2.end();
    let mut row3 = Flex::default().row();
    row3.set_pad(PAD);
    let choose_button = Button::default().with_label("C&hoose…");
    let mut folder_or_playlist_label = Frame::default(); // TODO
    folder_or_playlist_label.set_frame(FrameType::DownFrame);
    let mut include_subdirs_checkbox =
        CheckButton::default().with_label("Include &Subfolders"); // TODO
    include_subdirs_checkbox.set_checked(true);
    row3.set_size(&choose_button, left_width);
    row3.set_size(&include_subdirs_checkbox, right_width);
    row3.end();
    let mut row4 = Flex::default().row();
    row4.set_pad(PAD);
    let mut name_button = Button::default().with_label("&Name");
    name_button.set_frame(FrameType::NoBox);
    name_button.visible_focus(false);
    // TODO add handler to make it give focus to name_input
    let name_input = Input::default();
    row4.set_size(&name_button, left_width);
    row4.end();
    column.end();
    column.set_size(&row1, BUTTON_HEIGHT);
    column.set_size(&row2, BUTTON_HEIGHT);
    column.set_size(&row3, BUTTON_HEIGHT);
    column.set_size(&row4, BUTTON_HEIGHT);
    Widgets {
        parent_list_combo,
        folder_or_playlist_label,
        include_subdirs_checkbox,
        name_input,
    }
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
    result: Rc<RefCell<NewListResult>>,
) {
    buttons.ok_button.set_callback({
        // TODO
        let mut form = form.clone();
        move |_| {
            let result = &*result.borrow_mut();
            // TODO
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

const WIDTH: i32 = 640;
const HEIGHT: i32 = PAD + ((BUTTON_HEIGHT + PAD) * 5);
