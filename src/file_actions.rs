// Copyright © 2021-22 Mark Summerfield. All rights reserved.
// License: GPLv3

use super::CONFIG;
use crate::application::Application;
use crate::fixed::APPNAME;
use crate::options_form;
use fltk::{
    dialog::{FileDialog, FileDialogType},
    prelude::*,
};

impl Application {
    pub(crate) fn on_file_new(&mut self) {
        println!("FileNew"); // TODO
    }

    pub(crate) fn on_file_open(&mut self) {
        let mut form = FileDialog::new(FileDialogType::BrowseFile);
        form.set_title(&format!("Open TLM File — {APPNAME}"));
        // TODO get_data_dir
        // let _ = form.set_directory(&util::get_track_dir()); // Ignore error
        form.set_filter("TLM Files\t*.tlm");
        form.show();
        let filename = form.filename();
        if filename.exists() {
            dbg!("on_file_open", filename);
        }
    }

    pub(crate) fn on_file_save(&mut self) {
        println!("FileSave"); // TODO
    }

    pub(crate) fn on_file_save_as(&mut self) {
        println!("FileSaveAs"); // TODO
    }

    pub(crate) fn on_file_configure(&mut self) {
        dbg!("on_file_configure");
        // TODO
        /*
        let old_size = {
            let config = CONFIG.get().read().unwrap();
            config.history_size
        };
        */
        let form = options_form::Form::default();
        /*
        let ok = *form.ok.borrow();
        if ok {
            let new_size = {
                let config = CONFIG.get().read().unwrap();
                config.history_size
            };
            if old_size != new_size {
                self.populate_history_menu_button();
            }
        }
        */
    }

    pub(crate) fn on_file_quit(&mut self) {
        let mut config = CONFIG.get().write().unwrap();
        config.window_x = self.main_window.x();
        config.window_y = self.main_window.y();
        config.window_width = self.main_window.width();
        config.window_height = self.main_window.height();
        config.volume = self.volume_slider.value();
        config.save();
        self.app.quit();
    }
}
