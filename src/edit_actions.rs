// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use crate::application::Application;
use crate::fixed::{APPNAME, DELETED_NAME, TOP_LEVEL_NAME};
use crate::model::TrackID;
use fltk::{dialog, prelude::*, tree::TreeItem};

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

    pub(crate) fn on_edit_delete(&mut self) {
        if let Some(mut item) = self.tlm.track_tree.first_selected_item() {
            let mut opt_parent = item.parent();
            while let Some(parent) = opt_parent {
                opt_parent = parent.parent();
                if parent.depth() == 1 {
                    if let Some(name) = parent.label() {
                        if name == DELETED_NAME {
                            self.maybe_delete(&mut item);
                        } else {
                            self.move_to_deleted(&mut item);
                        }
                    }
                }
            }
        }
    }

    fn move_to_deleted(&mut self, item: &mut TreeItem) {
        if let Some(root) = self.tlm.track_tree.root() {
            let deleted_root = if let Some(child) =
                root.find_child_item(DELETED_NAME)
            {
                Some(child)
            } else {
                if let Some((_, child)) =
                    self.tlm.add_empty_list(TOP_LEVEL_NAME, DELETED_NAME)
                {
                    Some(child)
                } else {
                    None
                }
            };
            if let Some(deleted_root) = deleted_root {
                if item
                    .move_into(&deleted_root, deleted_root.children())
                    .is_ok()
                {
                    self.tlm.set_dirty();
                    self.tlm.track_tree.redraw();
                    self.update_ui();
                }
            }
        }
    }

    fn maybe_delete(&mut self, item: &mut TreeItem) {
        let tid = unsafe { item.user_data::<TrackID>() };
        let name = if let Some(name) = item.label() {
            name
        } else {
            if tid.is_none() {
                "List".to_string()
            } else {
                "Track".to_string()
            }
        };
        let message = format!(
            "Permanently delete the “{name}” {}?",
            if tid.is_none() {
                "list and any tracks and lists it contains"
            } else {
                "track"
            }
        );
        dialog::message_title(&format!("Delete — {APPNAME}"));
        if let Some(i) =
            dialog::choice2_default(&message, "&Cancel", "D&elete", "")
        {
            if i == 1 {
                if self.tlm.track_tree.remove(&item).is_ok() {
                    self.tlm.set_dirty();
                    self.tlm.track_tree.redraw();
                    self.update_ui();
                }
            }
        }
    }
}
