// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use crate::util::capitalize_first;
use chrono::prelude::*;
use fltk::{app, image::SvgImage, prelude::ImageExt};
use soloud::Soloud;
use std::env;
use std::sync;

pub static APPNAME: &str = "TLM";
pub static VERSION: &str = "1.0.0";
pub static TOP_LEVEL_NAME: &str = "<Top-Level>";
pub static FOUND_NAME: &str = "<Found>";
pub static DELETED_NAME: &str = "<Deleted>";
pub static HELP_HTML: &str = include_str!("../data/help.html");
pub const ICON: &str = include_str!("../images/tlm.svg");
pub const FILE_NEW_ICON: &str = include_str!("../images/document-new.svg");
pub const FILE_OPEN_ICON: &str =
    include_str!("../images/document-open.svg");
pub const FILE_SAVE_ICON: &str =
    include_str!("../images/document-save.svg");
pub const LIST_ICON: &str = include_str!("../images/list.svg");
pub const LIST_ADD_ICON: &str = include_str!("../images/list-new.svg");
pub const PROMOTE_ICON: &str = include_str!("../images/promote.svg");
pub const DEMOTE_ICON: &str = include_str!("../images/demote.svg");
pub const MOVE_UP_ICON: &str = include_str!("../images/move-up.svg");
pub const MOVE_DOWN_ICON: &str = include_str!("../images/move-down.svg");
pub const LIST_IMPORT_ICON: &str = include_str!("../images/import.svg");
pub const TRACK_ADD_ICON: &str = include_str!("../images/track-new.svg");
pub const TRACK_FIND_ICON: &str = include_str!("../images/track-find.svg");
pub const HISTORY_ICON: &str = include_str!("../images/history.svg");
pub const PREV_ICON: &str =
    include_str!("../images/media-seek-backward.svg");
pub const REPLAY_ICON: &str = include_str!("../images/replay.svg");
pub const PLAY_ICON: &str =
    include_str!("../images/media-playback-start.svg");
pub const PAUSE_ICON: &str =
    include_str!("../images/media-playback-pause.svg");
pub const NEXT_ICON: &str =
    include_str!("../images/media-seek-forward.svg");
pub const VOLUME_ICON: &str =
    include_str!("../images/audio-volume-high.svg");
pub const TIME_ICON: &str = include_str!("../images/time.svg");
pub const PATH_SEP: &str = "→";
pub const MAX_RECENT_FILES: usize = 9;
pub const MIN_HISTORY_SIZE: usize = 2;
pub const MAX_HISTORY_SIZE: usize = 35;
pub const PAD: i32 = 6;
pub const WINDOW_WIDTH_MIN: i32 = 590;
pub const WINDOW_HEIGHT_MIN: i32 = 380;
pub const TOOLBUTTON_SIZE: i32 = 22;
pub const TOOLBAR_HEIGHT: i32 = ((TOOLBUTTON_SIZE * 3) / 2) + PAD;
pub const BUTTON_HEIGHT: i32 = 30;
pub const BUTTON_WIDTH: i32 = 70;
pub const TREE_ICON_SIZE: i32 = 16;
pub const SCALE_MIN: f32 = 0.5;
pub const SCALE_MAX: f32 = 3.5;
pub const TINY_TIMEOUT: f64 = 0.1;
pub const TICK_TIMEOUT: f64 = 0.1;
pub const INFO_TIMEOUT: f64 = 10.0;
pub static MENU_CHARS: [char; 35] = [
    '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E',
    'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

pub static TIME_ICONS: state::Storage<sync::RwLock<Vec<SvgImage>>> =
    state::Storage::new();

pub fn initialize_time_icons() {
    const C1_ICON: &str = include_str!("../images/c1.svg");
    const C2_ICON: &str = include_str!("../images/c2.svg");
    const C3_ICON: &str = include_str!("../images/c3.svg");
    const C4_ICON: &str = include_str!("../images/c4.svg");
    const C5_ICON: &str = include_str!("../images/c5.svg");
    const C6_ICON: &str = include_str!("../images/c6.svg");
    const C7_ICON: &str = include_str!("../images/c7.svg");
    const C8_ICON: &str = include_str!("../images/c8.svg");
    const C9_ICON: &str = include_str!("../images/c9.svg");
    const C10_ICON: &str = include_str!("../images/c10.svg");
    const C11_ICON: &str = include_str!("../images/c11.svg");
    let mut icons = vec![];
    for name in [
        C1_ICON, C2_ICON, C3_ICON, C4_ICON, C5_ICON, C6_ICON, C7_ICON,
        C8_ICON, C9_ICON, C10_ICON, C11_ICON,
    ]
    .iter()
    {
        let mut icon = SvgImage::from_data(name).unwrap();
        icon.scale(TREE_ICON_SIZE, TREE_ICON_SIZE, true, true);
        icons.push(icon);
    }
    TIME_ICONS.set(sync::RwLock::new(icons));
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Action {
    ClearInfo,
    EditCopyToList,
    EditDelete,
    EditDemote,
    EditMoveDown,
    EditMoveToList,
    EditMoveUp,
    EditPromote,
    FileConfigure,
    FileNew,
    FileOpen,
    FileOpenRecent,
    FileQuit,
    FileSave,
    FileSaveAs,
    HelpAbout,
    HelpHelp,
    ListAdd,
    ListExport,
    ListImport,
    ListRename,
    OnStartup,
    PlayHistoryTrack,
    Tick,
    TimeUpdate,
    TrackAdd,
    TrackFind,
    TrackHistory,
    TrackLouder,
    TrackNext,
    TrackPlayOrPause,
    TrackPrevious,
    TrackQuieter,
    TrackReplay,
    TreeItemDoubleClicked,
    VolumeUpdate,
}

pub fn about_html(player: &Soloud) -> String {
    let year = Local::today().year();
    let year = if year == 2022 {
        year.to_string()
    } else {
        format!("2022-{}", year - 2000)
    };
    format!(
        "<p><center><font size=6 color=navy><b>{APPNAME}</b> v{VERSION}
</font></center></p>
<p><center><font color=navy size=5>Track List Manager manages playlists and plays tracks.</font></center></p>
<p><center><font size=4>
<a href=\"https://github.com/mark-summerfield/tlm\">https://github.com/mark-summerfield/tlm</a>
</font></center></p>
<p><center>
<font size=4 color=green>
Copyright © {year} Mark Summerfield.<br>
All rights reserved.<br>
License: GPLv3.</font>
</center></p>
<p><center><font size=4 color=#555>
Rust {} • fltk-rs {} • FLTK {}<br>Soloud {}/{} • {}/{}
</font></center></p>",
        rustc_version_runtime::version(),
        app::crate_version(),
        app::version_str(),
        player.version(),
        player.backend_string(),
        capitalize_first(env::consts::OS),
        env::consts::ARCH
    )
}
