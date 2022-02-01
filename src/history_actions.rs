// Copyright © 2021-22 Mark Summerfield. All rights reserved.
// License: GPLv3

use crate::application::Application;

impl Application {
    pub(crate) fn on_history_clear(&mut self) {
        println!("on_history_clear"); // TODO
    }

    pub(crate) fn on_add_to_history(&mut self) {
        dbg!("on_add_to_history");
        // See model.rs add_to_history
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
