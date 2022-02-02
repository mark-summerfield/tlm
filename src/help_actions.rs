// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use crate::application::Application;
use crate::fixed::{about_html, HELP_HTML};
use crate::html_form;

impl Application {
    pub(crate) fn on_help_about(&mut self) {
        html_form::Form::new(
            "About",
            &about_html(&self.player),
            true,
            540,
            300,
            false,
        );
    }

    pub(crate) fn on_help_help(&mut self) {
        if let Some(helpform) = &mut self.helpform {
            helpform.show();
        } else {
            self.helpform = Some(html_form::Form::new(
                "Help", HELP_HTML, false, 400, 440, true,
            ));
        }
    }
}
