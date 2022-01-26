// Copyright © 2021-22 Mark Summerfield. All rights reserved.
// License: GPLv3

use crate::fixed::{
    APPNAME, SCALE_MAX, SCALE_MIN, WINDOW_HEIGHT_MIN, WINDOW_WIDTH_MIN,
};
use crate::util;
use fltk::{app, dialog};
use ini::Ini;
use std::{collections::VecDeque, env, path::PathBuf};

type RecentFiles = VecDeque<PathBuf>;

#[derive(Clone, Debug)]
pub struct Config {
    pub window_x: i32,
    pub window_y: i32,
    pub window_height: i32,
    pub window_width: i32,
    pub window_scale: f32,
    pub volume: f64,
    pub filename: PathBuf,
    pub last_file: PathBuf,
    pub recent_files: RecentFiles,
}

impl Config {
    pub fn new() -> Self {
        let mut config = Config {
            filename: get_config_filename(),
            ..Default::default()
        };
        if let Ok(ini) = Ini::load_from_file(&config.filename) {
            if let Some(properties) = ini.section(Some(WINDOW_SECTION)) {
                read_window_properties(properties, &mut config);
            }
            if let Some(properties) = ini.section(Some(GENERAL_SECTION)) {
                read_general_properties(properties, &mut config);
            }
        }
        config
    }

    pub fn save(&self) {
        if self.filename.to_string_lossy() == "" {
            self.warning("failed to save configuration: no filename");
        } else {
            let mut ini = Ini::new();
            ini.with_section(Some(WINDOW_SECTION))
                .set(X_KEY, self.window_x.to_string())
                .set(Y_KEY, self.window_y.to_string())
                .set(WIDTH_KEY, self.window_width.to_string())
                .set(HEIGHT_KEY, self.window_height.to_string())
                .set(SCALE_KEY, app::screen_scale(0).to_string());
            ini.with_section(Some(GENERAL_SECTION))
                .set(VOLUME_KEY, self.volume.to_string())
                .set(LAST_FILE_KEY, self.last_file.to_string_lossy());
            self.save_recent_files(&mut ini);
            match ini.write_to_file(&self.filename) {
                Ok(_) => {}
                Err(err) => self.warning(&format!(
                    "failed to save configuration: {err}"
                )),
            }
        }
    }

    fn save_recent_files(&self, ini: &mut Ini) {
        for (i, filename) in self.recent_files.iter().enumerate() {
            let key = format!("{RECENT_FILE_KEY}{}", i + 1);
            ini.with_section(Some(GENERAL_SECTION))
                .set(key, filename.to_string_lossy());
            if i + 1 == 9 {
                break;
            }
        }
    }

    fn warning(&self, message: &str) {
        dialog::message_title(&format!("Warning — {APPNAME}",));
        dialog::message(util::x() - 200, util::y() - 100, message);
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            window_x: -1,
            window_y: -1,
            window_height: 480,
            window_width: 640,
            window_scale: 1.0,
            volume: 0.5,
            filename: PathBuf::new(),
            last_file: PathBuf::new(),
            recent_files: RecentFiles::new(),
        }
    }
}

fn get_config_filename() -> PathBuf {
    let mut dir = dirs::config_dir();
    let mut dot = "";
    if dir.is_none() {
        if env::consts::FAMILY == "unix" {
            dot = ".";
        }
        dir = dirs::home_dir();
    }
    if let Some(dir) = dir {
        dir.join(format!("{dot}{}.ini", APPNAME.to_lowercase()))
    } else {
        PathBuf::new()
    }
}

fn read_window_properties(
    properties: &ini::Properties,
    config: &mut Config,
) {
    let max_x = (app::screen_size().0 - 100.0) as i32;
    let max_y = (app::screen_size().1 - 100.0) as i32;
    if let Some(value) = properties.get(X_KEY) {
        config.window_x = util::get_num(value, 0, max_x, config.window_x)
    }
    if let Some(value) = properties.get(Y_KEY) {
        config.window_y = util::get_num(value, 0, max_y, config.window_y)
    }
    if let Some(value) = properties.get(WIDTH_KEY) {
        config.window_width = util::get_num(
            value,
            WINDOW_WIDTH_MIN,
            max_x,
            config.window_width,
        )
    }
    if let Some(value) = properties.get(HEIGHT_KEY) {
        config.window_height = util::get_num(
            value,
            WINDOW_HEIGHT_MIN,
            max_y,
            config.window_height,
        )
    }
    if let Some(value) = properties.get(SCALE_KEY) {
        config.window_scale =
            util::get_num(value, SCALE_MIN, SCALE_MAX, config.window_scale);
        if !util::isone32(config.window_scale) {
            app::set_screen_scale(0, config.window_scale);
        }
    }
}

fn read_general_properties(
    properties: &ini::Properties,
    config: &mut Config,
) {
    if let Some(value) = properties.get(VOLUME_KEY) {
        config.volume = util::get_num(value, 0.0, 1.0, config.volume)
    }
    if let Some(value) = properties.get(LAST_FILE_KEY) {
        config.last_file = PathBuf::from(value);
    }
    config.recent_files.clear();
    for i in 1..=9 {
        let key = format!("{RECENT_FILE_KEY}{i}");
        if let Some(value) = properties.get(&key) {
            let value = PathBuf::from(value);
            if !config.recent_files.contains(&value) && value.exists() {
                config.recent_files.push_back(value);
            }
        }
    }
}

static WINDOW_SECTION: &str = "Window";
static X_KEY: &str = "x";
static Y_KEY: &str = "y";
static WIDTH_KEY: &str = "width";
static HEIGHT_KEY: &str = "height";
static SCALE_KEY: &str = "scale";
static GENERAL_SECTION: &str = "General";
static VOLUME_KEY: &str = "volume";
static LAST_FILE_KEY: &str = "lastfile";
static RECENT_FILE_KEY: &str = "recentfile";
