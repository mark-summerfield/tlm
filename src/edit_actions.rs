// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use crate::application::Application;

impl Application {
    pub(crate) fn on_edit_move_up(&mut self) {
        println!("on_edit_move_up"); // TODO
    }

    pub(crate) fn on_edit_move_down(&mut self) {
        println!("on_edit_move_down"); // TODO
    }

    pub(crate) fn on_edit_promote(&mut self) {
        println!("on_edit_promote"); // TODO
    }

    pub(crate) fn on_edit_demote(&mut self) {
        println!("on_edit_demote"); // TODO
    }

    pub(crate) fn on_edit_move_to_list(&mut self) {
        println!("on_edit_move_to_list"); // TODO
    }

    pub(crate) fn on_edit_copy_to_list(&mut self) {
        println!("on_edit_copy_to_list"); // TODO
    }

    // TODO If the list or track is inside the <Deleted> list, then up a
    // dialog offering [&Delete] [&Cancel] Otherwise simply move the list or
    // track to the top-level <Deleted> list (creating this list first if it
    // doesn't already exist).
    pub(crate) fn on_edit_delete(&mut self) {
        println!("on_edit_delete"); // TODO
    }
}
