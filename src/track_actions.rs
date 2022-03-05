// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use crate::application::Application;
use crate::fixed::{
    Action, PATH_SEP, PAUSE_ICON, PLAY_ICON, TINY_TIMEOUT, TOOLBUTTON_SIZE,
};
use crate::list_form::{self, Reply};
use crate::util;
use fltk::{app, image::SvgImage, prelude::*};
use soloud::prelude::*;
use std::{thread, time::Duration};

impl Application {
    pub(crate) fn on_track_add(&mut self) {
        // util::sanitize(name, "New Track")
        println!("on_track_add"); // TODO
    }

    pub(crate) fn on_track_previous(&mut self) {
        if let Some(mut item) = self.tlm.track_tree.first_selected_item() {
            if let Some(prev) = item.prev_sibling() {
                item.deselect();
                self.maybe_play_or_replay(prev);
            }
        }
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
        if self.current.has_track() {
            if self.playing {
                self.on_track_play_or_pause(); // PAUSE
            }
            self.seek(0.0);
            self.on_track_play_or_pause(); // PLAY
        }
    }

    pub(crate) fn on_track_next(&mut self) {
        if let Some(mut item) = self.tlm.track_tree.first_selected_item() {
            if let Some(prev) = item.next_sibling() {
                item.deselect();
                self.maybe_play_or_replay(prev);
            }
        }
    }

    pub(crate) fn on_play_history_track(&mut self) {
        let index = self.history_menu_button.value();
        self.play_history_track(index);
    }

    fn play_history_track(&mut self, index: i32) {
        if index > -1 {
            if let Some(treepath) = self.history_menu_button.text(index) {
                let treepath = treepath[3..].replace(PATH_SEP, "/");
                if let Some(mut item) =
                    self.tlm.track_tree.first_selected_item()
                {
                    item.deselect();
                }
                if let Some(item) = self.tlm.track_tree.find_item(&treepath)
                {
                    self.maybe_play_or_replay(item);
                }
            }
        }
    }

    pub(crate) fn on_track_history(&mut self) {
        let list = {
            let mut list = vec![];
            for treepath in self.tlm.history_iter() {
                list.push(treepath.clone());
            }
            list
        };
        let form =
            list_form::Form::new("History", "&Play", "&Delete", &list[..]);
        let reply = *form.reply.borrow();
        match reply {
            Reply::Select(index) => self.play_history_track(index as i32),
            Reply::Delete(index) => {
                if index > 0 {
                    self.tlm.history_delete_item(index);
                    self.populate_history_menu_button();
                }
            }
            Reply::DeleteAll => {
                self.tlm.history_shrink();
                self.populate_history_menu_button();
            }
            Reply::Cancel => (),
        }
    }

    pub(crate) fn on_track_find(&mut self) {
        println!("on_track_find"); // TODO
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

    pub(crate) fn on_track_delete(&mut self) {
        println!("on_track_delete"); // TODO
    }

    pub(crate) fn load_track(&mut self) {
        if self.playing {
            self.on_track_play_or_pause(); // PAUSE
            self.player.stop_all();
        }
        let message = match self.wav.load(&self.current.track) {
            Ok(_) => {
                self.handle = self.player.play(&self.wav);
                self.player.set_pause(self.handle, true);
                self.player.set_volume(
                    self.handle,
                    self.volume_slider.value() as f32,
                );
                let secs = self.wav.length();
                self.time_slider.set_range(0.0, secs);
                self.time_slider.set_step(secs, 20);
                self.time_slider.set_value(0.0);
                self.time_label.set_label(&format!(
                    "{}/{}",
                    util::humanized_time(0.0),
                    util::humanized_time(secs)
                ));
                self.tlm.history_add_to(
                    self.current.treepath.clone(),
                    self.current.tid,
                    secs,
                );
                self.populate_history_menu_button();
                util::get_track_data_html(&self.current.track)
            }
            Err(_) => format!("Failed to open {:?}", &self.current.track),
        };
        self.info_view.set_value(&message);
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
