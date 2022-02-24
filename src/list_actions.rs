// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use crate::application::Application;
use crate::new_list_form;
use crate::util;
use std::path::Path;

impl Application {
    pub(crate) fn on_list_new(&mut self) {
        let mut top_levels = vec![];
        if let Some(root) = self.tlm.track_tree.first() {
            let mut opt_item = root.next(); // first top-level child
            while let Some(item) = opt_item {
                opt_item = item.next_sibling();
                if let Some(name) = item.label() {
                    top_levels.push(name);
                }
            }
        }
        let form =
            new_list_form::Form::new(&self.current.track, &top_levels);
        if *form.ok.borrow() {
            let name = &*form.name.borrow();
            let parent_list = &*form.parent_list.borrow();
            let folder_or_playlist = &*form.folder_or_playlist.borrow();
            let include_subdirs = *form.include_subdirs.borrow();
            if !folder_or_playlist.exists() {
                self.new_empty_list(parent_list, name);
            } else if folder_or_playlist.is_file() {
                self.new_list_from_playlist(
                    parent_list,
                    name,
                    folder_or_playlist,
                );
            } else if folder_or_playlist.is_dir() {
                self.new_list_from_folder(
                    parent_list,
                    name,
                    folder_or_playlist,
                    include_subdirs,
                );
            }
        }
    }

    fn new_empty_list(&mut self, parent_list: &str, name: &str) {
        if let Some((treepath, item)) =
            self.tlm.add_empty_list(parent_list, name)
        {
            self.select_track_in_tree(treepath, item)
        }
    }

    fn new_list_from_playlist(
        &mut self,
        parent_list: &str,
        name: &str,
        playlist: &Path,
    ) {
        let name = if name.is_empty() {
            util::canonicalize(playlist)
        } else {
            name.to_string()
        };
        dbg!("new_list_from_playlist", parent_list, name, playlist); // TODO
    }

    fn new_list_from_folder(
        &mut self,
        parent_list: &str,
        name: &str,
        folder: &Path,
        include_subdirs: bool,
    ) {
        let name = if name.is_empty() {
            util::canonicalize(folder)
        } else {
            name.to_string()
        };
        dbg!(
            "new_list_from_folder",
            parent_list,
            name,
            folder,
            include_subdirs
        ); // TODO
    }

    pub(crate) fn on_list_rename(&mut self) {
        println!("on_list_rename"); // TODO
    }

    pub(crate) fn on_list_promote(&mut self) {
        println!("on_list_promote"); // TODO
    }

    pub(crate) fn on_list_move_up(&mut self) {
        println!("on_list_move_up"); // TODO
    }

    pub(crate) fn on_list_move_down(&mut self) {
        println!("on_list_move_down"); // TODO
    }

    pub(crate) fn on_list_demote(&mut self) {
        println!("on_list_demote"); // TODO
    }

    pub(crate) fn on_list_export(&mut self) {
        println!("on_list_export"); // TODO
    }

    pub(crate) fn on_list_import(&mut self) {
        println!("on_list_import"); // TODO
    }

    pub(crate) fn on_list_delete(&mut self) {
        println!("on_list_delete"); // TODO
    }
}
