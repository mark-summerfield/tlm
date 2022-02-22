// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use crate::fixed::{APPNAME, BUTTON_HEIGHT, BUTTON_WIDTH, ICON, PAD};
use crate::util;
use fltk::{
    app,
    button::{Button, CheckButton, RadioRoundButton},
    dialog::{FileDialog, FileDialogType},
    enums::FrameType,
    frame::Frame,
    group::Flex,
    image::SvgImage,
    input::Input,
    menu::Choice,
    prelude::*,
    window::Window,
};
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};

pub struct Form {
    form: Window,
    pub ok: Rc<RefCell<bool>>,
    pub name: Rc<RefCell<String>>,
    pub parent_list: Rc<RefCell<String>>,
    pub folder_or_playlist: Rc<RefCell<PathBuf>>,
    pub include_subdirs: Rc<RefCell<bool>>,
}

impl Form {
    pub fn new(current_track: &Path, top_levels: &[String]) -> Self {
        let ok = Rc::from(RefCell::from(false));
        let name = Rc::from(RefCell::from(String::new()));
        let parent_list = Rc::from(RefCell::from(String::new()));
        let folder_or_playlist = Rc::from(RefCell::from(PathBuf::new()));
        let include_subdirs = Rc::from(RefCell::from(false));
        let mut form = make_form();
        let mut vbox = Flex::default().size_of_parent().column();
        vbox.set_margin(PAD);
        vbox.set_pad(PAD);
        let mut widgets = make_widgets(top_levels);
        let (button_row, mut buttons) = make_buttons();
        vbox.set_size(&button_row, BUTTON_HEIGHT);
        vbox.end();
        form.end();
        form.make_modal(true);
        add_event_handlers(
            &mut form,
            &mut widgets,
            &mut buttons,
            current_track.to_path_buf(),
            Rc::clone(&ok),
            Rc::clone(&name),
            Rc::clone(&parent_list),
            Rc::clone(&folder_or_playlist),
            Rc::clone(&include_subdirs),
        );
        widgets.parent_list_combo.take_focus().unwrap();
        form.show();
        while form.shown() {
            app::wait();
        }
        Self {
            form,
            ok,
            name,
            parent_list,
            folder_or_playlist,
            include_subdirs,
        }
    }
}

impl Drop for Form {
    fn drop(&mut self) {
        app::delete_widget(self.form.clone());
    }
}

struct Widgets {
    pub parent_list_button: Button,
    pub parent_list_combo: Choice,
    pub folder_radio: RadioRoundButton,
    pub playlist_radio: RadioRoundButton,
    pub choose_button: Button,
    pub folder_or_playlist_label: Frame,
    pub include_subdirs_checkbox: CheckButton,
    pub name_button: Button,
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

fn make_widgets(top_levels: &[String]) -> Widgets {
    let width = (WIDTH / 5).max(BUTTON_WIDTH * 2);
    let mut column = Flex::default().column();
    column.set_pad(PAD);
    let mut row1 = Flex::default().row();
    row1.set_pad(PAD);
    let mut parent_list_button =
        Button::default().with_label("&Parent List");
    parent_list_button.set_frame(FrameType::NoBox);
    parent_list_button.visible_focus(false);
    let mut parent_list_combo = Choice::default();
    parent_list_combo.add_choice("<Top-Level>");
    for top_level in top_levels {
        parent_list_combo.add_choice(&top_level);
    }
    parent_list_combo.set_value(0);
    row1.set_size(&parent_list_button, width);
    row1.end();
    let mut row2 = Flex::default().row();
    row2.set_pad(PAD);
    let mut folder_radio =
        RadioRoundButton::default().with_label("&Add tracks from folder");
    folder_radio.toggle(true);
    let playlist_radio = RadioRoundButton::default()
        .with_label("&Import tracks from playist");
    RadioRoundButton::default().with_label("Create &Empty list");
    row2.end();
    let mut row3 = Flex::default().row();
    row3.set_pad(PAD);
    let choose_button = Button::default().with_label("C&hoose…");
    let mut folder_or_playlist_label = Frame::default();
    folder_or_playlist_label.set_frame(FrameType::DownFrame);
    row3.set_size(&choose_button, width);
    row3.end();
    let mut row4 = Flex::default().row();
    row4.set_pad(PAD);
    let include_subdirs_checkbox =
        CheckButton::default().with_label("Include &Subfolders");
    include_subdirs_checkbox.set_checked(true);
    row4.set_size(&include_subdirs_checkbox, width);
    row4.end();
    let mut row5 = Flex::default().row();
    row5.set_pad(PAD);
    let mut name_button = Button::default().with_label("&Name");
    name_button.set_frame(FrameType::NoBox);
    name_button.visible_focus(false);
    let name_input = Input::default();
    row5.set_size(&name_button, width);
    row5.end();
    column.end();
    column.set_size(&row1, BUTTON_HEIGHT);
    column.set_size(&row2, BUTTON_HEIGHT);
    column.set_size(&row3, BUTTON_HEIGHT);
    column.set_size(&row4, BUTTON_HEIGHT);
    column.set_size(&row5, BUTTON_HEIGHT);
    Widgets {
        parent_list_button,
        parent_list_combo,
        folder_radio,
        playlist_radio,
        choose_button,
        folder_or_playlist_label,
        include_subdirs_checkbox,
        name_button,
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
    widgets: &mut Widgets,
    buttons: &mut Buttons,
    current_track: PathBuf,
    ok: Rc<RefCell<bool>>,
    name: Rc<RefCell<String>>,
    parent_list: Rc<RefCell<String>>,
    folder_or_playlist: Rc<RefCell<PathBuf>>,
    include_subdirs: Rc<RefCell<bool>>,
) {
    widgets.parent_list_button.set_callback({
        let mut parent_list_combo = widgets.parent_list_combo.clone();
        move |_| {
            parent_list_combo.take_focus().unwrap();
        }
    });
    widgets.name_button.set_callback({
        let mut name_input = widgets.name_input.clone();
        move |_| {
            name_input.take_focus().unwrap();
        }
    });
    widgets.choose_button.set_callback({
        let folder_radio = widgets.folder_radio.clone();
        let playlist_radio = widgets.playlist_radio.clone();
        let mut folder_or_playlist_label =
            widgets.folder_or_playlist_label.clone();
        move |_| {
            if folder_radio.is_toggled() {
                folder_or_playlist_label
                    .set_label(&get_tracks_folder(current_track.clone()));
            } else if playlist_radio.is_toggled() {
                folder_or_playlist_label.set_label(&get_playlist_filename(
                    current_track.clone(),
                ));
            }
            app::redraw(); // redraws the world
        }
    });
    buttons.ok_button.set_callback({
        let mut form = form.clone();
        let name = name.clone();
        let parent_list = parent_list.clone();
        let folder_or_playlist = folder_or_playlist.clone();
        let include_subdirs = include_subdirs.clone();
        let name_input = widgets.name_input.clone();
        let parent_list_combo = widgets.parent_list_combo.clone();
        let folder_or_playlist_label =
            widgets.folder_or_playlist_label.clone();
        let include_subdirs_checkbox =
            widgets.include_subdirs_checkbox.clone();
        move |_| {
            *ok.borrow_mut() = true;
            *name.borrow_mut() = name_input.value();
            *parent_list.borrow_mut() =
                parent_list_combo.choice().unwrap_or_default();
            *folder_or_playlist.borrow_mut() =
                PathBuf::from(folder_or_playlist_label.label());
            *include_subdirs.borrow_mut() =
                include_subdirs_checkbox.is_checked();
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

fn get_tracks_folder(current_track: PathBuf) -> String {
    let mut form = FileDialog::new(FileDialogType::BrowseDir);
    form.set_title(&format!("Read Folder — {APPNAME}"));
    let _ = form.set_directory(&util::get_track_dir(&current_track));
    form.show();
    let folder = form.filename();
    if folder.exists() {
        folder.display().to_string()
    } else {
        String::new()
    }
}

fn get_playlist_filename(current_track: PathBuf) -> String {
    let mut form = FileDialog::new(FileDialogType::BrowseFile);
    form.set_title(&format!("Read Playlist — {APPNAME}"));
    let _ = form.set_directory(&util::get_track_dir(&current_track));
    form.set_filter("Playlists\t*.{m3u,pls,xspf}");
    form.show();
    let filename = form.filename();
    if filename.exists() {
        filename.display().to_string()
    } else {
        String::new()
    }
}

const WIDTH: i32 = 640;
const HEIGHT: i32 = PAD + ((BUTTON_HEIGHT + PAD) * 6);
