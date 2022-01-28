// Copyright © 2021-22 Mark Summerfield. All rights reserved.
// License: GPLv3

use crate::application::Application;
use crate::fixed::{
    Action, PAUSE_ICON, PLAY_ICON, TINY_TIMEOUT, TOOLBUTTON_SIZE,
};
use crate::util;
use fltk::{app, image::SvgImage, prelude::*};
use soloud::prelude::*;
use std::{thread, time::Duration};

impl Application {
    pub(crate) fn on_track_new(&mut self) {
        println!("on_track_new"); // TODO
    }

    pub(crate) fn on_track_previous(&mut self) {
        dbg!("on_track_previous");
        // TODO
        /*
        let track = {
            let config = CONFIG.get().read().unwrap();
            config.track.clone()
        };
        if let Some(track) =
            util::get_prev_or_next_track(&track, WhichTrack::Previous)
        {
            self.auto_play_track(track);
        }
        */
    }

    pub(crate) fn on_track_play_or_pause(&mut self) {
        let icon = if self.playing {
            self.player.set_pause(self.handle, true);
            PLAY_ICON
        } else {
            self.player.set_pause(self.handle, false);
            #[allow(clippy::clone_on_copy)]
            let sender = self.sender.clone();
            app::add_timeout3(TINY_TIMEOUT, move |_| {
                sender.send(Action::Tick);
            });
            PAUSE_ICON
        };
        let mut icon = SvgImage::from_data(icon).unwrap();
        icon.scale(TOOLBUTTON_SIZE, TOOLBUTTON_SIZE, true, true);
        self.play_pause_button.set_image(Some(icon));
        self.playing = !self.playing;
    }

    pub(crate) fn on_track_replay(&mut self) {
        dbg!("on_track_replay");
        // TODO
        /*
        {
            let config = CONFIG.get().read().unwrap();
            if !config.track.exists() {
                return;
            }
        }
        if self.playing {
            self.on_track_play_or_pause(); // PAUSE
        }
        {
            let mut config = CONFIG.get().write().unwrap();
            config.pos = 0.0;
        }
        self.seek(0.0);
        self.on_track_play_or_pause(); // PLAY
        */
    }

    pub(crate) fn on_track_next(&mut self) {
        dbg!("on_track_next");
        // TODO
        /*
        let track = {
            let config = CONFIG.get().read().unwrap();
            config.track.clone()
        };
        if let Some(track) =
            util::get_prev_or_next_track(&track, WhichTrack::Next)
        {
            self.auto_play_track(track);
        }
        */
    }

    pub(crate) fn on_volume_down(&mut self) {
        self.change_volume(
            (self.volume_slider.value() as f32 - 0.05).max(0.0),
        );
    }

    pub(crate) fn on_volume_up(&mut self) {
        self.change_volume(
            (self.volume_slider.value() as f32 + 0.05).min(1.0),
        );
    }

    pub(crate) fn on_volume_update(&mut self) {
        let volume = self.volume_slider.value() as f32;
        self.player.set_volume(self.handle, volume);
        self.volume_label
            .set_label(&format!("{}%", (volume * 100.0).round()));
        app::redraw(); // redraws the world
    }

    pub(crate) fn on_track_move_up(&mut self) {
        println!("on_track_move_up"); // TODO
    }

    pub(crate) fn on_track_move_down(&mut self) {
        println!("on_track_move_down"); // TODO
    }

    pub(crate) fn on_track_move_to_list(&mut self) {
        println!("on_track_move_to_list"); // TODO
    }

    pub(crate) fn on_track_copy_to_list(&mut self) {
        println!("on_track_copy_to_list"); // TODO
    }

    pub(crate) fn on_track_find(&mut self) {
        println!("on_track_find"); // TODO
    }

    pub(crate) fn on_track_delete(&mut self) {
        println!("on_track_delete"); // TODO
    }

    pub(crate) fn on_track_undelete(&mut self) {
        println!("on_track_undelete"); // TODO
    }

    pub(crate) fn load_track(&mut self) {
        if self.playing {
            self.on_track_play_or_pause(); // PAUSE
            self.player.stop_all();
        }
        dbg!("load_track");
        // TODO
        /*
        let config = CONFIG.get().read().unwrap();
        let message = match self.wav.load(&config.track) {
            Ok(_) => {
                self.handle = self.player.play(&self.wav);
                self.player.set_pause(self.handle, true);
                self.player.set_volume(
                    self.handle,
                    self.volume_slider.value() as f32,
                );
                self.time_slider.set_range(0.0, self.wav.length());
                self.time_slider.set_step(self.wav.length(), 20);
                self.time_slider.set_value(0.0);
                self.time_label.set_label(&format!(
                    "{}/{}",
                    util::humanized_time(pos),
                    util::humanized_time(self.wav.length())
                ));
                #[allow(clippy::clone_on_copy)]
                let sender = self.sender.clone();
                app::add_timeout3(TINY_TIMEOUT, move |_| {
                    sender.send(Action::AddToHistory);
                });
                util::get_track_data_html(&config.track)
            }
            Err(_) => {
                LOAD_ERROR.replace("FILE", &config.track.to_string_lossy())
            }
        };
        self.info_view.set_value(&message);
        */
        self.update_ui();
        app::redraw(); // redraws the world
    }

    pub(crate) fn change_volume(&mut self, volume: f32) {
        self.player.set_volume(self.handle, volume);
        self.volume_slider.set_value(volume as f64);
        self.volume_label
            .set_label(&format!("{}%", (volume * 100.0).round()));
        app::redraw(); // redraws the world
    }

    pub(crate) fn auto_play_track(&mut self, track: std::path::PathBuf) {
        dbg!("auto_play_track");
        // TODO
        /*
        if self.playing {
            self.on_track_play_or_pause(); // PAUSE
        }
        {
            let mut config = CONFIG.get().write().unwrap();
            config.track = track;
            config.pos = 0.0;
        }
        self.load_track();
        self.on_track_play_or_pause(); // PLAY
        */
    }

    pub(crate) fn seek(&mut self, pos: f64) {
        if self.player.seek(self.handle, pos).is_ok() {
            while self.player.stream_position(self.handle) < pos {
                thread::sleep(Duration::from_millis(100));
            }
        }
        self.time_slider.set_value(pos);
        self.time_label.set_label(&format!(
            "{}/{}",
            util::humanized_time(pos),
            util::humanized_time(self.wav.length())
        ));
        app::redraw(); // redraws the world
    }
}
