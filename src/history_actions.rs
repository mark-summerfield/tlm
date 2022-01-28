// Copyright © 2021-22 Mark Summerfield. All rights reserved.
// License: GPLv3

use crate::application::Application;

impl Application {
    pub(crate) fn on_history_clear(&mut self) {
        println!("on_history_clear"); // TODO
    }

    pub(crate) fn on_add_to_history(&mut self) {
        dbg!("on_add_to_history");
        // TODO
        /*
        let mut changed = false;
        {
            // If current is already in history, move it to front of history
            let mut config = CONFIG.get().write().unwrap();
            if !config.history.is_empty()
                && config.history[0] == config.track
            {
                return; // current is already first in history
            }
            let mut i = 1;
            while i < config.history.len() {
                if config.history[i] == config.track {
                    changed = true;
                    break;
                }
                i += 1;
            }
            if changed {
                config.history.swap(0, i);
            }
        }
        if !changed {
            // If current not in history add it to the front
            let mut config = CONFIG.get().write().unwrap();
            let track = config.track.clone();
            let size = config.history_size;
            config.history.push_front(track);
            config.history.truncate(size);
        }
        main_window::populate_history_menu_button(
            &mut self.history_menu_button,
            self.sender,
        );
        */
    }

    pub(crate) fn on_load_history_track(&mut self) {
        dbg!("on_load_history_track");
        // TODO
        /*
        let index = self.history_menu_button.value();
        if index > -1 {
            if let Some(track) = self.history_menu_button.text(index) {
                self.load_remembered_track(&track);
            }
        }
        */
    }
}
