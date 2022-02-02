// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use crate::util::capitalize_first;
use chrono::prelude::*;
use fltk::app;
use soloud::Soloud;
use std::env;

pub static APPNAME: &str = "TLM";
pub static VERSION: &str = "1.0.0";
pub static HELP_HTML: &str = include_str!("../data/help.html");
pub const ICON: &str = include_str!("../images/tlm.svg");
pub const FILE_NEW_ICON: &str = include_str!("../images/document-new.svg");
pub const FILE_OPEN_ICON: &str =
    include_str!("../images/document-open.svg");
pub const FILE_SAVE_ICON: &str =
    include_str!("../images/document-save.svg");
pub const LIST_NEW_ICON: &str = include_str!("../images/list-new.svg");
pub const LIST_MOVE_UP_ICON: &str =
    include_str!("../images/list-move-up.svg");
pub const LIST_MOVE_DOWN_ICON: &str =
    include_str!("../images/list-move-down.svg");
pub const LIST_IMPORT_ICON: &str = include_str!("../images/import.svg");
pub const TRACK_NEW_ICON: &str = include_str!("../images/track-new.svg");
pub const TRACK_MOVE_UP_ICON: &str =
    include_str!("../images/track-move-up.svg");
pub const TRACK_MOVE_DOWN_ICON: &str =
    include_str!("../images/track-move-down.svg");
pub const TRACK_FIND_ICON: &str = include_str!("../images/track-find.svg");
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
pub const MAX_RECENT_FILES: usize = 9;
pub const DEF_HISTORY_SIZE: usize = 26;
pub const MIN_HISTORY_SIZE: usize = 2;
pub const MAX_HISTORY_SIZE: usize = 35;
pub const PAD: i32 = 6;
pub const WINDOW_WIDTH_MIN: i32 = 512;
pub const WINDOW_HEIGHT_MIN: i32 = 380;
pub const TOOLBUTTON_SIZE: i32 = 22;
pub const TOOLBAR_HEIGHT: i32 = ((TOOLBUTTON_SIZE * 3) / 2) + PAD;
pub const BUTTON_HEIGHT: i32 = 30;
pub const BUTTON_WIDTH: i32 = 70;
pub const SCALE_MIN: f32 = 0.5;
pub const SCALE_MAX: f32 = 3.5;
pub const MINI_TIMEOUT: f64 = 0.01;
pub const TINY_TIMEOUT: f64 = 0.075;
pub const TICK_TIMEOUT: f64 = 0.1;
pub const INFO_TIMEOUT: f64 = 10.0;
pub static MENU_CHARS: [char; 35] = [
    '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E',
    'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Action {
    ClearInfo,
    FileConfigure,
    FileNew,
    FileOpen,
    FileOpenRecent,
    FileQuit,
    FileSave,
    FileSaveAs,
    HelpAbout,
    HelpHelp,
    ListNew,
    ListRename,
    ListMoveUp,
    ListMoveDown,
    ListMoveTo,
    ListMerge,
    ListCopy,
    ListExport,
    ListImport,
    ListDelete,
    ListUndelete,
    OnStartup,
    Tick,
    TimeUpdate,
    TrackNew,
    TrackPrevious,
    TrackPlayOrPause,
    TrackReplay,
    TrackNext,
    TrackPlayAgain,
    TrackLouder,
    TrackQuieter,
    TrackMoveUp,
    TrackMoveDown,
    TrackMoveToList,
    TrackCopyToList,
    TrackFind,
    TrackDelete,
    TrackUndelete,
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
<p><center><font color=navy size=5>An application to manage playlists and play tracks.</font></center></p>
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
