// Copyright © 2021-22 Mark Summerfield. All rights reserved.
// License: GPLv3
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod actions;
mod application;
mod config;
mod file_actions;
mod fixed;
mod help_actions;
mod history_actions;
mod html_form;
mod list_actions;
mod main_window;
mod model;
mod options_form;
mod track_actions;
mod util;

use crate::application::Application;
use crate::fixed::APPNAME;
use config::Config;
use fltk::dialog;
use state::Storage;
use std::{panic, sync};

pub static CONFIG: Storage<sync::RwLock<Config>> = Storage::new();

fn main() {
    panic::set_hook(Box::new(|info| {
        let err = dbg!(&info);
        dialog::message_title(&format!("Error — {APPNAME}"));
        let x = util::x() - 200;
        let y = util::y() - 100;
        dialog::message(x, y, &err.to_string());
    }));
    CONFIG.set(sync::RwLock::new(Config::new()));
    let mut app = Application::new();
    app.run();
}
