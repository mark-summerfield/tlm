// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod actions;
mod application;
mod config;
mod file_actions;
mod fixed;
mod help_actions;
mod html_form;
mod list_actions;
mod list_form;
mod main_window;
mod model;
mod new_list_form;
mod options_form;
mod playlists;
mod track_actions;
mod util;

use crate::application::Application;
use crate::fixed::initialize_time_icons;
use config::Config;
use state::Storage;
use std::{panic, sync};

pub static CONFIG: Storage<sync::RwLock<Config>> = Storage::new();

fn main() {
    panic::set_hook(Box::new(|info| {
        let err = dbg!(&info);
        util::popup_error_message(&err.to_string());
    }));
    CONFIG.set(sync::RwLock::new(Config::new()));
    initialize_time_icons();
    let mut app = Application::new();
    app.run();
}
