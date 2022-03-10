// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use crate::application::Application;
use fltk::app;

impl Application {
    pub(crate) fn on_edit_move_up(&mut self) {
        if let Some(mut item) = self.tlm.track_tree.first_selected_item() {
            if let Some(prev) = item.prev_sibling() {
                if item.move_above(&prev).is_ok() {
                    app::redraw(); // redraws the world
                }
            }
        }
    }

    pub(crate) fn on_edit_move_down(&mut self) {
        if let Some(mut item) = self.tlm.track_tree.first_selected_item() {
            if let Some(next) = item.next_sibling() {
                if item.move_below(&next).is_ok() {
                    app::redraw(); // redraws the world
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
        if let Some(item) = self.tlm.track_tree.first_selected_item() {
            if let Some(parent) = item.parent() {
                if parent.is_root() {
                    return; // can't promote beyond the root
                }
                if let Some(mut grand_parent) = parent.parent() {
                    let index = grand_parent.children() + 1;
                    let err = grand_parent.move_into(&item, index);
                    // let err = grand_parent.move_into(&item, index);
                    dbg!(item.label(), parent.label(), grand_parent.label(), index, err);
                }
            }
        }
    }

    /*
    Move the selected item to be the last child of the first _following_
    sibling that is a _list_ (i.e., that has no TID) if there is one.
    */
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
