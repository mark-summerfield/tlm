// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use crate::fixed::{Action, TINY_TIMEOUT};
use crate::html_form;
use crate::main_window;
use crate::model::{Model, TrackID, TreePath};
use fltk::{
    app,
    app::{channel, App, Receiver, Scheme, Sender},
    button::Button,
    frame::Frame,
    menu::SysMenuBar,
    misc::HelpView,
    prelude::*,
    valuator::HorFillSlider,
    window::Window,
};
use soloud::{audio::Wav, prelude::*, Soloud};
use std::path::PathBuf;

pub struct Application {
    pub(crate) app: App,
    pub(crate) main_window: Window,
    pub(crate) menubar: SysMenuBar,
    pub(crate) prev_button: Button,
    pub(crate) replay_button: Button,
    pub(crate) play_pause_button: Button,
    pub(crate) next_button: Button,
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
    pub(crate) current_treepath: TreePath,
    pub(crate) current_track: PathBuf,
    pub(crate) current_tid: TrackID,
    pub(crate) tlm: Model,
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
            current_treepath: TreePath::new(),
            current_track: PathBuf::new(),
            current_tid: 0,
            tlm: Model::new(widgets.track_tree),
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
            app::add_timeout3(TINY_TIMEOUT, move |_| {
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
                    Action::ClearInfo => self.info_view.set_value(""),
                    Action::FileNew => self.on_file_new(),
                    Action::FileOpen => self.on_file_open(),
                    Action::FileOpenRecent => self.on_file_open_recent(),
                    Action::FileSave => self.on_file_save(),
                    Action::FileSaveAs => self.on_file_save_as(),
                    Action::FileConfigure => self.on_file_configure(),
                    Action::FileQuit => self.on_file_quit(),
                    Action::ListNew => self.on_list_new(),
                    Action::ListRename => self.on_list_rename(),
                    Action::ListMoveUp => self.on_list_move_up(),
                    Action::ListMoveDown => self.on_list_move_down(),
                    Action::ListMoveTo => self.on_list_move_to(),
                    Action::ListMerge => self.on_list_merge(),
                    Action::ListCopy => self.on_list_copy(),
                    Action::ListExport => self.on_list_export(),
                    Action::ListImport => self.on_list_import(),
                    Action::ListDelete => self.on_list_delete(),
                    Action::ListUndelete => self.on_list_undelete(),
                    Action::HelpAbout => self.on_help_about(),
                    Action::HelpHelp => self.on_help_help(),
                    Action::OnStartup => self.on_startup(),
                    Action::Tick => self.on_tick(),
                    Action::TimeUpdate => self.on_time_update(),
                    Action::TrackNew => self.on_track_new(),
                    Action::TrackPrevious => self.on_track_previous(),
                    Action::TrackPlayOrPause => {
                        self.on_track_play_or_pause()
                    }
                    Action::TrackReplay => self.on_track_replay(),
                    Action::TrackNext => self.on_track_next(),
                    Action::TrackPlayAgain => self.on_track_play_again(),
                    Action::TrackLouder => self.on_volume_up(),
                    Action::TrackQuieter => self.on_volume_down(),
                    Action::TrackMoveUp => self.on_track_move_up(),
                    Action::TrackMoveDown => self.on_track_move_down(),
                    Action::TrackMoveToList => self.on_track_move_to_list(),
                    Action::TrackCopyToList => self.on_track_copy_to_list(),
                    Action::TrackFind => self.on_track_find(),
                    Action::TrackDelete => self.on_track_delete(),
                    Action::TrackUndelete => self.on_track_undelete(),
                    Action::TreeItemDoubleClicked => {
                        self.on_tree_item_double_clicked()
                    }
                    Action::VolumeUpdate => self.on_volume_update(),
                }
            }
        }
    }

    pub fn clear_info_after(&mut self, secs: f64) {
        #[allow(clippy::clone_on_copy)]
        let sender = self.sender.clone();
        app::add_timeout3(secs, move |_| {
            sender.send(Action::ClearInfo);
        });
    }

    pub fn update_ui(&mut self) {
        dbg!("update_ui");
        // TODO
        /*
        if has_track {
            self.prev_button.activate();
            self.replay_button.activate();
            self.play_pause_button.activate();
            self.next_button.activate();
            self.again_button.activate();
            self.time_slider.activate();
        } else {
            self.prev_button.deactivate();
            self.replay_button.deactivate();
            self.play_pause_button.deactivate();
            self.next_button.deactivate();
            self.again_button.deactivate();
            self.time_slider.deactivate();
        }
        if has_history {
            self.track_history_button.activate();
        } else {
            self.track_history_button.deactivate();
        }
        */
    }
}
