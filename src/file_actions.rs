// Copyright © 2021-22 Mark Summerfield. All rights reserved.
// License: GPLv3

use super::CONFIG;
use crate::application::Application;
use crate::fixed::{
    Action, APPNAME, FILE_RECENT_MENU, MAX_RECENT_FILES, MENU_CHARS,
};
use crate::options_form;
use crate::util;
use fltk::{
    app,
    dialog::{FileDialog, FileDialogType},
    enums::Shortcut,
    menu::MenuFlag,
    prelude::*,
};
use std::path::Path;

impl Application {
    pub(crate) fn on_file_new(&mut self) {
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

    pub(crate) fn on_file_open_recent(&mut self, index: usize) {
        let filename = {
            let config = CONFIG.get().read().unwrap();
            config.recent_files[index].clone()
        };
        if filename.exists() {
            self.load_tlm(&filename);
        }
    }

    pub(crate) fn load_tlm(&mut self, filename: &Path) {
        match self.tlm.load(filename) {
            Ok(_) => {
                self.update_title(filename);
                self.update_recent_files(filename);
                self.close_children();
                // TODO If self.tlm.has_current_treepath() then select
                // it ready to play
            }
            Err(err) => {
                self.clear_title();
                util::popup_error_message(&format!(
                    "Failed to open {filename:?}:\n{err}"
                ));
            }
        };
        app::redraw(); // redraws the world
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
        self.update_recent_files_menu();
    }

    pub(crate) fn update_recent_files_menu(&mut self) {
        if let Some(mut item) = self.menubar.find_item(FILE_RECENT_MENU) {
            item.clear();
            let base = FILE_RECENT_MENU.trim_end();
            let config = CONFIG.get().read().unwrap();
            for (i, filename) in config.recent_files.iter().enumerate() {
                let name = filename.to_string_lossy();
                // TODO FIXME
                self.menubar.add_emit(
                    &format!("{base}/&{} {name}", MENU_CHARS[i]),
                    Shortcut::None,
                    MenuFlag::Normal,
                    self.sender,
                    Action::FileOpenRecent(i),
                );
            }
        }
    }

    fn clear_title(&mut self) {
        self.main_window.set_label(APPNAME);
    }

    fn update_title(&mut self, filename: &Path) {
        let filename = if let Some(stem) = filename.file_stem() {
            stem.to_string_lossy()
        } else {
            filename.to_string_lossy()
        };
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
