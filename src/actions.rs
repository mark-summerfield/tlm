// Copyright © 2021-22 Mark Summerfield. All rights reserved.
// License: GPLv3

use super::CONFIG;
use crate::application::Application;
use crate::fixed::{
    about_html, Action, APPNAME, HELP_HTML, PATH_SEP, PAUSE_ICON,
    PLAY_ICON, TICK_TIMEOUT, TINY_TIMEOUT, TOOLBUTTON_SIZE,
};
use crate::html_form;
use crate::options_form;
use crate::util;
use fltk::{
    app,
    dialog::{FileDialog, FileDialogType},
    image::SvgImage,
    prelude::*,
};
use soloud::prelude::*;
use std::{path::PathBuf, thread, time::Duration};

impl Application {
    pub(crate) fn on_startup(&mut self) {
        self.load_track();
    }

    pub(crate) fn on_open(&mut self) {
        let mut form = FileDialog::new(FileDialogType::BrowseFile);
        form.set_title(&format!("Choose Track — {APPNAME}"));
        let _ = form.set_directory(&util::get_track_dir()); // Ignore error
        form.set_filter("Audio Files\t*.{flac,mogg,mp3,oga,ogg,wav}");
        form.show();
        let filename = form.filename();
        if filename.exists() {
            self.auto_play_track(filename);
        }
    }

    pub(crate) fn on_previous(&mut self) {
        dbg!("on_previous");
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

    pub(crate) fn on_replay(&mut self) {
        dbg!("on_replay");
        // TODO
        /*
        {
            let config = CONFIG.get().read().unwrap();
            if !config.track.exists() {
                return;
            }
        }
        if self.playing {
            self.on_play_or_pause(); // PAUSE
        }
        {
            let mut config = CONFIG.get().write().unwrap();
            config.pos = 0.0;
        }
        self.seek(0.0);
        self.on_play_or_pause(); // PLAY
        */
    }

    pub(crate) fn on_play_or_pause(&mut self) {
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

    pub(crate) fn on_next(&mut self) {
        dbg!("on_next");
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

    pub(crate) fn on_time_update(&mut self) {
        self.seek(self.time_slider.value());
        app::redraw(); // redraws the world
    }

    pub(crate) fn on_options(&mut self) {
        dbg!("on_options");
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

    pub(crate) fn on_about(&mut self) {
        html_form::Form::new(
            "About",
            &about_html(&self.player),
            true,
            480,
            300,
            false,
        );
    }

    pub(crate) fn on_help(&mut self) {
        if let Some(helpform) = &mut self.helpform {
            helpform.show();
        } else {
            self.helpform = Some(html_form::Form::new(
                "Help", HELP_HTML, false, 400, 440, true,
            ));
        }
    }

    pub(crate) fn on_quit(&mut self) {
        let mut config = CONFIG.get().write().unwrap();
        config.window_x = self.main_window.x();
        config.window_y = self.main_window.y();
        config.window_width = self.main_window.width();
        config.window_height = self.main_window.height();
        config.volume = self.volume_slider.value();
        config.save();
        self.app.quit();
    }

    pub(crate) fn on_tick(&mut self) {
        if self.playing {
            let pos = self.player.stream_position(self.handle);
            let length = self.wav.length();
            if self.player.voice_count() == 0 {
                // Reached the end
                self.on_next();
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

    pub(crate) fn load_track(&mut self) {
        if self.playing {
            self.on_play_or_pause(); // PAUSE
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
            self.on_play_or_pause(); // PAUSE
        }
        {
            let mut config = CONFIG.get().write().unwrap();
            config.track = track;
            config.pos = 0.0;
        }
        self.load_track();
        self.on_play_or_pause(); // PLAY
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

    fn load_remembered_track(&mut self, track: &str) {
        let track = track.replace(PATH_SEP, "/");
        let (_, track) = track.split_at(3);
        self.auto_play_track(PathBuf::from(track));
    }

    pub(crate) fn on_add_bookmark(&mut self) {
        dbg!("on_add_bookmark");
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

    pub(crate) fn on_delete_bookmark(&mut self) {
        dbg!("on_delete_bookmark");
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
