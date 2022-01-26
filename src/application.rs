// Copyright © 2021-22 Mark Summerfield. All rights reserved.
// License: GPLv3

use crate::fixed::Action;
use crate::html_form;
use crate::main_window;
use fltk::{
    app,
    app::{channel, App, Receiver, Scheme, Sender},
    button::Button,
    frame::Frame,
    menu::SysMenuBar,
    misc::HelpView,
    prelude::*,
    tree::Tree,
    valuator::HorFillSlider,
    window::Window,
};
use soloud::{audio::Wav, prelude::*, Soloud};

pub struct Application {
    pub(crate) app: App,
    pub(crate) main_window: Window,
    pub(crate) menubar: SysMenuBar,
    pub(crate) prev_button: Button,
    pub(crate) replay_button: Button,
    pub(crate) play_pause_button: Button,
    pub(crate) next_button: Button,
    pub(crate) track_tree: Tree,
    pub(crate) info_view: HelpView,
    pub(crate) volume_slider: HorFillSlider,
    pub(crate) volume_label: Frame,
    pub(crate) time_slider: HorFillSlider,
    pub(crate) time_label: Frame,
    pub(crate) helpform: Option<html_form::Form>,
    pub(crate) player: Soloud,
    pub(crate) wav: Wav,
    pub(crate) handle: soloud::Handle,
    pub(crate) playing: bool,
    pub(crate) first_to_play: bool,
    pub(crate) sender: Sender<Action>,
    pub(crate) receiver: Receiver<Action>,
}

impl Application {
    pub fn new() -> Self {
        let app = App::default().with_scheme(Scheme::Gleam);
        let (sender, receiver) = channel::<Action>();
        let mut widgets = main_window::make(sender);
        main_window::add_event_handlers(&mut widgets.main_window, sender);
        widgets.main_window.show();
        let mut player =
            Soloud::default().expect("Cannot access audio backend");
        player.set_pause_all(true);
        let load = main_window::update_widgets_from_config(&mut widgets);
        let mut volume_slider = widgets.volume_slider.clone();
        let mut time_slider = widgets.time_slider.clone();
        let mut app = Self {
            app,
            main_window: widgets.main_window,
            menubar: widgets.menubar,
            prev_button: widgets.prev_button,
            replay_button: widgets.replay_button,
            play_pause_button: widgets.play_pause_button,
            next_button: widgets.next_button,
            track_tree: widgets.track_tree,
            info_view: widgets.info_view,
            volume_slider: widgets.volume_slider,
            volume_label: widgets.volume_label,
            time_slider: widgets.time_slider,
            time_label: widgets.time_label,
            helpform: None,
            player,
            wav: Wav::default(),
            handle: unsafe { soloud::Handle::from_raw(0) },
            playing: false,
            first_to_play: true,
            sender,
            receiver,
        };
        #[allow(clippy::clone_on_copy)]
        let sender = sender.clone();
        volume_slider.set_callback(move |_| {
            sender.send(Action::VolumeUpdate);
        });
        #[allow(clippy::clone_on_copy)]
        let sender = sender.clone();
        time_slider.set_callback(move |_| {
            sender.send(Action::TimeUpdate);
        });
        if load {
            #[allow(clippy::clone_on_copy)]
            let sender = sender.clone();
            app::add_timeout3(0.01, move |_| {
                sender.send(Action::OnStartup);
            });
        } else {
            app.update_ui();
        }
        app
    }

    pub fn run(&mut self) {
        while self.app.wait() {
            if let Some(action) = self.receiver.recv() {
                match action {
                    Action::OnStartup => self.on_startup(),
                    Action::OpenMusicBox => self.on_open(),
                    Action::Previous => self.on_previous(),
                    Action::Replay => self.on_replay(),
                    Action::PlayOrPause => self.on_play_or_pause(),
                    Action::Tick => self.on_tick(),
                    Action::Next => self.on_next(),
                    Action::VolumeDown => self.on_volume_down(),
                    Action::VolumeUp => self.on_volume_up(),
                    Action::VolumeUpdate => self.on_volume_update(),
                    Action::TimeUpdate => self.on_time_update(),
                    Action::Options => self.on_options(),
                    Action::About => self.on_about(),
                    Action::Help => self.on_help(),
                    Action::Quit => self.on_quit(),
                }
            }
        }
    }

    pub fn update_ui(&mut self) {
        dbg!("update_ui");
        // TODO
        /*
        let (has_track, has_history, has_bookmarks) = {
            let config = CONFIG.get().read().unwrap();
            (
                config.track.exists(),
                !config.history.is_empty(),
                !config.bookmarks.is_empty(),
            )
        };
        if has_track {
            self.prev_button.activate();
            self.replay_button.activate();
            self.play_pause_button.activate();
            self.next_button.activate();
            self.time_slider.activate();
            self.add_bookmark_button.activate();
        } else {
            self.prev_button.deactivate();
            self.replay_button.deactivate();
            self.play_pause_button.deactivate();
            self.next_button.deactivate();
            self.time_slider.deactivate();
            self.add_bookmark_button.deactivate();
        }
        if has_history {
            self.history_menu_button.activate();
        } else {
            self.history_menu_button.deactivate();
        }
        if has_bookmarks {
            self.bookmarks_menu_button.activate();
            self.delete_bookmark_button.activate();
        } else {
            self.bookmarks_menu_button.deactivate();
            self.delete_bookmark_button.deactivate();
        }
        */
    }
}
