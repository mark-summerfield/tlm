// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use super::CONFIG;
use crate::application::Application;
use crate::fixed::{Action, APPNAME, TICK_TIMEOUT, TINY_TIMEOUT};
use crate::model::TrackID;
use crate::util;
use fltk::{app, dialog, prelude::*, tree::TreeItem};

impl Application {
    pub(crate) fn on_startup(&mut self) {
        let filename = {
            let config = CONFIG.get().read().unwrap();
            config.last_file.clone()
        };
        if filename.exists() {
            self.load_tlm(&filename);
            self.populate_history_menu_button();
        }
    }

    pub(crate) fn on_time_update(&mut self) {
        self.seek(self.time_slider.value());
        app::redraw(); // redraws the world
    }

    pub(crate) fn on_tick(&mut self) {
        if self.playing {
            let pos = self.player.stream_position(self.handle);
            let length = self.wav.length();
            if self.player.voice_count() == 0 {
                // Reached the end
                self.on_track_next();
                return;
            }
            self.time_slider.set_value(pos);
            self.time_label.set_label(&format!(
                "{}/{}",
                util::humanized_time(pos),
                util::humanized_time(length)
            ));
            app::redraw(); // redraws the world
            #[allow(clippy::clone_on_copy)]
            let sender = self.sender.clone();
            app::add_timeout3(TICK_TIMEOUT, move |_| {
                sender.send(Action::Tick);
            });
        }
    }

    pub(crate) fn on_tree_item_double_clicked(&mut self) {
        if let Some(item) = self.tlm.track_tree.first_selected_item() {
            self.maybe_play_or_replay(item);
        }
    }

    pub(crate) fn maybe_play_or_replay(&mut self, item: TreeItem) {
        match unsafe { item.user_data::<TrackID>() } {
            Some(tid) => {
                if tid == self.current.tid {
                    self.on_track_replay();
                    return;
                }
            }
            None => return,
        };
        let mut treepath = item.label().unwrap_or_default();
        let mut opt_parent = item.parent();
        while let Some(parent) = opt_parent {
            opt_parent = parent.parent();
            if parent.depth() > 0 {
                let label = parent.label().unwrap_or_default();
                treepath.insert(0, '/');
                treepath.insert_str(0, &label);
            }
        }
        self.select_track_in_tree(treepath, item);
        self.on_track_replay();
    }

    pub(crate) fn select_track_in_tree(
        &mut self,
        treepath: String,
        item: TreeItem,
    ) {
        if let Some(tid) = unsafe { item.user_data::<TrackID>() } {
            if let Some(track_item) = self.tlm.track_for_tid.get(&tid) {
                self.current.tid = tid;
                if let Some(treepath) = treepath.strip_prefix("ROOT/") {
                    self.current.treepath = treepath.to_string();
                } else {
                    self.current.treepath = treepath.clone();
                }
                self.current.track = track_item.filename.clone();
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

    pub(crate) fn ok_to_clear(&mut self) -> bool {
        if self.tlm.is_dirty() {
            dialog::message_title(&format!("Unsaved Changes — {APPNAME}"));
            match dialog::choice2_default(
                "Save changes?",
                "&Don't Save",
                "&Save",
                "&Cancel",
            ) {
                Some(index) => match index {
                    0 => true, // don't save and continue
                    1 => {
                        self.on_file_save();
                        true // save and continue
                    }
                    _ => false, // don't save and don't continue
                },
                None => false, // don't save and don't continue
            }
        } else {
            true // no unsaved changes, so continue
        }
    }
}
