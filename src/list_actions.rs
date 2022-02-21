// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use crate::application::Application;
use crate::new_list_form::{self, NewListResult};

impl Application {
    pub(crate) fn on_list_new(&mut self) {
        let form = new_list_form::Form::default();
        let reply = &*form.result.borrow();
        println!("on_list_new: {:?}", reply); // TODO
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
