// Copyright © 2021-22 Mark Summerfield. All rights reserved.
// License: GPLv3

use super::CONFIG;
use crate::application::Application;
use crate::fixed::{Action, TICK_TIMEOUT};
use crate::util;
use fltk::{app, prelude::*};

impl Application {
    pub(crate) fn on_startup(&mut self) {
        let filename = {
            let config = CONFIG.get().read().unwrap();
            config.last_file.clone()
        };
        if filename.exists() {
            self.load_tlm(&filename);
        } else {
            // self.update_recent_files_menu(); // TODO WHEN POSSIBLE (RECENT)
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

    fn load_remembered_track(&mut self, track: &str) {
        dbg!("load_remembered_track");
        //let track = track.replace(PATH_SEP, "/");
        //let (_, track) = track.split_at(3);
        //self.auto_play_track(PathBuf::from(track));
    }
}
