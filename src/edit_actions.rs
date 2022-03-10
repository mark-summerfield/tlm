// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use crate::application::Application;
use crate::model::TrackID;
use fltk::prelude::*;

impl Application {
    pub(crate) fn on_edit_move_up(&mut self) {
        if let Some(mut item) = self.tlm.track_tree.first_selected_item() {
            if let Some(prev) = item.prev_sibling() {
                if item.move_above(&prev).is_ok() {
                    self.tlm.set_dirty();
                    self.tlm.track_tree.redraw();
                }
            }
        }
    }

    pub(crate) fn on_edit_move_down(&mut self) {
        if let Some(mut item) = self.tlm.track_tree.first_selected_item() {
            if let Some(next) = item.next_sibling() {
                if item.move_below(&next).is_ok() {
                    self.tlm.set_dirty();
                    self.tlm.track_tree.redraw();
                }
            }
        }
    }

    /*
    Move the selected item to be the last child of its grandparent if it has
    one. (Top-level items have a parent of root and no grandparent, so can't
    be promoted.)
    */
    pub(crate) fn on_edit_promote(&mut self) {
        if let Some(mut item) = self.tlm.track_tree.first_selected_item() {
            if let Some(parent) = item.parent() {
                if parent.is_root() {
                    return; // can't promote beyond the root
                }
                if let Some(grand_parent) = parent.parent() {
                    let index = grand_parent.children();
                    if item.move_into(&grand_parent, index).is_ok() {
                        self.tlm.set_dirty();
                        self.tlm.track_tree.redraw();
                    }
                }
            }
        }
    }

    /*
    Move the selected item to be the last child of the previous
    sibling if there is one and it is a _list_ (i.e., that has no TID).
    */
    pub(crate) fn on_edit_demote(&mut self) {
        if let Some(mut item) = self.tlm.track_tree.first_selected_item() {
            if let Some(prev) = item.prev_sibling() {
                let tid = unsafe { prev.user_data::<TrackID>() };
                if tid.is_none() {
                    // List not Track
                    let index = prev.children();
                    if item.move_into(&prev, index).is_ok() {
                        self.tlm.set_dirty();
                        self.tlm.track_tree.redraw();
                    }
                }
            }
        }
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
