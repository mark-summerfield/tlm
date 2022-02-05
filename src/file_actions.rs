// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use super::CONFIG;
use crate::application::Application;
use crate::fixed::{APPNAME, INFO_TIMEOUT, MAX_RECENT_FILES, TINY_TIMEOUT};
use crate::model::TrackID;
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
                self.clear_info_after(INFO_TIMEOUT);
            }
        };
        app::redraw(); // redraws the world
    }

    fn select_recent_track(&mut self) {
        if let Some(treepath) = self.tlm.history.front() {
            let treepath = treepath.clone();
            if let Some(item) = self.tlm.track_tree.find_item(&treepath) {
                self.select_track_in_tree(treepath, item);
            }
        } else {
            let mut treepath = String::new();
            let mut opt_item = self.tlm.track_tree.first();
            while let Some(item) = opt_item {
                treepath.push_str(&item.label().unwrap_or_default());
                if item.has_children() {
                    treepath.push('/');
                    opt_item = item.next();
                } else {
                    self.select_track_in_tree(treepath, item);
                    break;
                }
            }
        }
    }

    pub(crate) fn select_track_in_tree(
        &mut self,
        treepath: String,
        item: TreeItem,
    ) {
        if let Some(tid) = unsafe { item.user_data::<TrackID>() } {
            if let Some(track_item) = self.tlm.track_for_tid.get(&tid) {
                self.current_tid = tid;
                if let Some(treepath) = treepath.strip_prefix("ROOT/") {
                    self.current_treepath = treepath.to_string();
                } else {
                    self.current_treepath = treepath.clone();
                }
                self.current_track = track_item.filename.clone();
                self.load_track();
            }
        }
        let mut opt_parent = item.parent();
        while let Some(mut parent) = opt_parent {
            parent.open();
            opt_parent = parent.parent();
        }
        let _ = self.tlm.track_tree.select(&treepath, false);
        let mut tree = self.tlm.track_tree.clone();
        app::add_timeout3(TINY_TIMEOUT, move |_| {
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
        if let Err(err) = self.tlm.save() {
            util::popup_error_message(&format!("Failed to save: {err}"));
        }
    }

    pub(crate) fn on_file_save_as(&mut self) {
        println!("FileSaveAs"); // TODO pop up native file dialog
                                //self.tlm.save_as();
    }

    pub(crate) fn on_file_configure(&mut self) {
        dbg!("on_file_configure");
        let old_size = {
            let config = CONFIG.get().read().unwrap();
            config.history_size
        };
        let form = options_form::Form::default();
        let ok = *form.ok.borrow();
        if ok {
            let new_size = {
                let config = CONFIG.get().read().unwrap();
                config.history_size
            };
            if old_size != new_size {
                // self.populate_history_menu_button(); // TODO
            }
        }
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
