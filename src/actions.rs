// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use super::CONFIG;
use crate::application::Application;
use crate::fixed::{Action, APPNAME, TICK_TIMEOUT};
use crate::util;
use fltk::{app, dialog, prelude::*};

impl Application {
    pub(crate) fn on_startup(&mut self) {
        let filename = {
            let config = CONFIG.get().read().unwrap();
            config.last_file.clone()
        };
        if filename.exists() {
            self.load_tlm(&filename);
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

    pub(crate) fn ok_to_clear(&mut self) -> bool {
        if self.tlm.dirty {
            dialog::message_title(&format!("Unsaved Changes — {APPNAME}"));
            match dialog::choice2_default(
                "Save changes?",
                "&Save",
                "&Don't Save",
                "&Cancel",
            ) {
                Some(index) => match index {
                    0 => {
                        self.on_file_save();
                        true // save and continue
                    }
                    1 => true,  // don't save and continue
                    _ => false, // don't save and don't continue
                },
                None => false, // don't save and don't continue
            }
        } else {
            true // no unsaved changes, so continue
        }
    }
}
