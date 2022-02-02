// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use super::CONFIG;
use crate::application::Application;
use crate::fixed::{APPNAME, INFO_TIMEOUT, MAX_RECENT_FILES, MINI_TIMEOUT};
use crate::options_form;
use crate::util;
use fltk::{
    app,
    dialog::{FileDialog, FileDialogType},
    prelude::*,
    tree::TreeItem,
};
use std::path::Path;

impl Application {
    pub(crate) fn on_file_new(&mut self) {
        if !self.ok_to_clear() {
            return;
        }
        println!("FileNew"); // TODO
    }

    pub(crate) fn on_file_open(&mut self) {
        let mut form = FileDialog::new(FileDialogType::BrowseFile);
        form.set_title(&format!("Open TLM File — {APPNAME}"));
        let _ = form.set_directory(&util::get_tlm_dir()); // Ignore error
        form.set_filter("TLM Files\t*.tlm");
        form.show();
        let filename = form.filename();
        if filename.exists() {
            self.load_tlm(&filename);
        }
    }

    pub(crate) fn on_file_open_recent(&mut self) {
        dbg!("on_file_open_recent");
        // TODO pop up dialog of recent files for use to
        // [&Open] [Clear &List] [&Cancel]
        /*
        let filename = {
            let config = CONFIG.get().read().unwrap();
            config.recent_files[index].clone()
        };
        if filename.exists() {
            self.load_tlm(&filename);
        }
        */
    }

    pub(crate) fn load_tlm(&mut self, filename: &Path) {
        if !self.ok_to_clear() {
            return;
        }
        match self.tlm.load(filename) {
            Ok(_) => {
                self.update_title(filename);
                self.update_recent_files(filename);
                self.close_children();
                self.info_view.set_value(&format!(
                    "Opened <font color=navy>{filename:?}</font>"
                ));
                self.select_recent_track();
            }
            Err(err) => {
                self.clear_title();
                self.info_view.set_value(&format!(
                    "Failed to open <font color=navy>{filename:?}</font>
                    <br><font color=red>{err}</font>"
                ));
            }
        };
        app::redraw(); // redraws the world
        self.clear_info_after(INFO_TIMEOUT);
    }

    fn select_recent_track(&mut self) {
        if let Some(treepath) = self.tlm.history.front() {
            if let Some(item) = self.tlm.track_tree.find_item(treepath) {
                self.select_tree_item(treepath.clone(), item);
            }
        }
        else {
            let mut treepath = String::new();
            let mut opt_item = self.tlm.track_tree.first();
            while let Some(mut item) = opt_item {
                treepath.push_str(&item.label().unwrap_or_default());
                if item.has_children() {
                    treepath.push('/');
                    opt_item = item.next();
                } else {
                    self.select_tree_item(treepath, item);
                    break;
                }
            }
        }
    }

    fn select_tree_item(&mut self, treepath: String, item: TreeItem) {
        let mut opt_parent = item.parent();
        while let Some(mut parent) = opt_parent {
            parent.open();
            opt_parent = parent.parent();
        }
        let _ = self.tlm.track_tree.select(&treepath, false);
        // TODO update self.info_view with track's details
        let mut tree = self.tlm.track_tree.clone();
        app::add_timeout3(MINI_TIMEOUT, move |_| {
            tree.show_item_middle(&item);
        });
    }

    fn update_recent_files(&mut self, filename: &Path) {
        let filename = filename.to_path_buf();
        {
            let mut config = CONFIG.get().write().unwrap();
            config.last_file = filename.clone();
            util::maybe_add_to_deque(
                &mut config.recent_files,
                filename,
                MAX_RECENT_FILES,
            );
        }
    }

    fn clear_title(&mut self) {
        self.main_window.set_label(APPNAME);
    }

    fn update_title(&mut self, filename: &Path) {
        let filename = util::file_stem(filename);
        self.main_window.set_label(&format!("{filename} — {APPNAME}"));
    }

    fn close_children(&mut self) {
        let mut opt_item = self.tlm.track_tree.first();
        while let Some(mut item) = opt_item {
            if item.depth() == 1 || item.depth() == 2 {
                item.close();
            }
            opt_item = self.tlm.track_tree.next(&item);
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
        if !self.ok_to_clear() {
            return;
        }
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
