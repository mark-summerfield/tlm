// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use crate::application::Application;
use crate::new_list_form;

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
            new_list_form::Form::new(&self.current_track, &top_levels);
        if *form.ok.borrow() {
            let name = &*form.name.borrow();
            let parent_list = &*form.parent_list.borrow();
            let folder_or_playlist = &*form.folder_or_playlist.borrow();
            let include_subdirs = *form.include_subdirs.borrow();
            dbg!(
                "on_list_new: OK",
                name,
                parent_list,
                folder_or_playlist,
                include_subdirs
            ); // TODO
        } else {
            dbg!("on_list_new: Cancel"); // TODO DELETE
        }
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

    pub(crate) fn on_list_undelete(&mut self) {
        println!("on_list_undelete"); // TODO
    }
}
