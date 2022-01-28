// Copyright © 2021-22 Mark Summerfield. All rights reserved.
// License: GPLv3

use crate::application::Application;

impl Application {
    pub(crate) fn on_load_bookmarked_track(&mut self) {
        dbg!("on_load_bookmarked_track");
        // TODO
        /*
        let index = self.bookmarks_menu_button.value();
        if index > -1 {
            if let Some(track) = self.bookmarks_menu_button.text(index) {
                self.load_remembered_track(&track);
            }
        }
        */
    }

    pub(crate) fn on_bookmarks_add(&mut self) {
        dbg!("on_bookmarks_add");
        // TODO
        /*
        let track = {
            let config = CONFIG.get().read().unwrap();
            if config.bookmarks.contains(&config.track) {
                None
            } else {
                Some(config.track.clone())
            }
        };
        let mut changed = false;
        if let Some(track) = track {
            let mut config = CONFIG.get().write().unwrap();
            if config.bookmarks.len() >= BOOKMARKS_SIZE {
                config.bookmarks.truncate(BOOKMARKS_SIZE - 1); // make room
            }
            config.bookmarks.push(track);
            config.bookmarks.sort();
            changed = true;
        }
        if changed {
            self.populate_bookmarks_menu_button();
        }
        */
    }

    pub(crate) fn on_bookmarks_remove(&mut self) {
        dbg!("on_bookmarks_remove");
        // TODO
        /*
        let track = {
            let config = CONFIG.get().read().unwrap();
            if !config.bookmarks.contains(&config.track) {
                None
            } else {
                Some(config.track.clone())
            }
        };
        let mut changed = false;
        if let Some(track) = track {
            let mut config = CONFIG.get().write().unwrap();
            if let Ok(index) = config.bookmarks.binary_search(&track) {
                config.bookmarks.remove(index);
                changed = true;
            }
        }
        if changed {
            self.populate_bookmarks_menu_button();
        }
        */
    }
}
