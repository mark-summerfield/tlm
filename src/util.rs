// Copyright © 2021-22 Mark Summerfield. All rights reserved.
// License: GPLv3

use super::CONFIG;
use fltk::app;
use lofty::{self, Accessor, ItemKey, ItemValue, Probe};
use std::{
    cmp,
    path::{Path, PathBuf},
    str,
};

pub fn x() -> i32 {
    (app::screen_size().0 / 2.0) as i32
}

pub fn y() -> i32 {
    (app::screen_size().1 / 2.0) as i32
}

pub fn capitalize_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

// Returns a number converted from the given str or the default
pub fn get_num<T>(s: &str, minimum: T, maximum: T, default: T) -> T
where
    T: num::Num + cmp::PartialOrd + Copy + str::FromStr,
{
    match s.parse() {
        Ok(n) if minimum <= n && n <= maximum => n,
        _ => default,
    }
}

pub fn isclose32(a: f32, b: f32) -> bool {
    (a..=(a + f32::EPSILON)).contains(&b)
}

pub fn isone32(n: f32) -> bool {
    (1.0..=(1.0 + f32::EPSILON)).contains(&n)
}

pub fn get_tlm_dir() -> PathBuf {
    let config = CONFIG.get().read().unwrap();
    if config.last_file.exists() {
        if let Some(path) = config.last_file.parent() {
            return path.to_path_buf();
        }
    }
    for file in &config.recent_files {
        if let Some(path) = file.parent() {
            return path.to_path_buf();
        }
    }
    if let Some(path) = dirs::document_dir() {
        return path;
    }
    if let Some(path) = dirs::home_dir() {
        return path;
    }
    PathBuf::from(".")
}

pub fn get_track_dir() -> PathBuf {
    dbg!("get_track_dir");
    // TODO GET FROM tlm Model
    /*
    let config = CONFIG.get().read().unwrap();
    if config.track.exists() {
        if let Some(path) = config.track.parent() {
            return path.to_path_buf();
        }
    }
    */
    if let Some(path) = dirs::audio_dir() {
        return path;
    }
    if let Some(path) = dirs::home_dir() {
        return path;
    }
    PathBuf::from(".")
}

pub fn humanized_time(secs: f64) -> String {
    const HR_SIGN: char = 'h';
    const MIN_SIGN: char = '′';
    const SEC_SIGN: char = '″';
    if secs <= 0.0 {
        return format!("0{SEC_SIGN}");
    }
    let hrs = (secs / 3600.0).floor();
    let mut secs = secs % 3600.0;
    let mut mins = (secs / 60.0).floor();
    secs %= 60.0;
    let mut hours = format!("{hrs:.0}");
    let mut minutes = format!("{mins:.0}");
    if minutes == "60" {
        hours = format!("{:.0}", hrs + 1.0);
        mins = 0.0;
    }
    let mut seconds = format!("{secs:.0}");
    if seconds == "60" {
        minutes = format!("{:.0}", mins + 1.0);
        seconds.clear();
    }
    if hours == "0" || hours.is_empty() {
        hours.clear();
    } else {
        hours.push(HR_SIGN);
    }
    if minutes == "0" || minutes.is_empty() {
        minutes.clear();
    } else {
        minutes.push(MIN_SIGN);
    }
    if seconds == "0" || seconds.is_empty() {
        seconds.clear();
    } else {
        seconds.push(SEC_SIGN);
    }
    if hours.is_empty() && minutes.is_empty() && seconds.is_empty() {
        format!("0{SEC_SIGN}")
    } else {
        format!("{hours}{minutes}{seconds}")
    }
}

pub fn get_track_data_html(track: &Path) -> String {
    let name = if let Some(name) = track.file_stem() {
        name.to_string_lossy()
    } else {
        track.to_string_lossy()
    };
    let name = name.replace(&['_', '-'][..], " ");
    match get_track_tag(track) {
        Ok(Some(data)) => {
            let mut text = String::from("<font color=navy><b>");
            if !data.title.is_empty() {
                text.push_str(&data.title);
            } else {
                text.push_str(&name);
            }
            text.push_str("</b></font><br>");
            if !data.album.is_empty() {
                text.push_str("<font color=green>");
                text.push_str(&data.album);
                text.push_str("</font>");
            }
            if data.number > 0 {
                text.push_str("<font color=green>");
                text.push_str(&format!(" (#{})", data.number));
                text.push_str("</font>");
            }
            if !data.album.is_empty() || data.number > 0 {
                text.push_str("<br>");
            }
            if !data.album.is_empty() {
                text.push_str("<font color=green>");
                text.push_str(&data.artist);
                text.push_str("</font><br>");
            }
            text.push_str("<font color=#008B8B>");
            text.push_str(&track.to_string_lossy());
            text.push_str("</font>");
            text
        }
        _ => {
            format!(
                "<font color=navy><b>{name}</b></font><br>
                <font color=#008B8B>{}</font>",
                track.to_string_lossy()
            )
        }
    }
}

pub struct TrackData {
    pub title: String,
    pub album: String,
    pub artist: String,
    pub number: i32,
}

fn get_track_tag(track: &Path) -> lofty::Result<Option<TrackData>> {
    let tags = Probe::open(track)?.guess_file_type()?.read(false)?;
    if let Some(tag) = tags.primary_tag() {
        Ok(Some(TrackData {
            title: if let Some(title) = tag.title() {
                title.to_owned()
            } else {
                String::new()
            },
            album: if let Some(album) = tag.album() {
                album.to_owned()
            } else {
                String::new()
            },
            artist: if let Some(artist) = tag.artist() {
                artist.to_owned()
            } else {
                String::new()
            },
            number: if let Some(num_item) =
                tag.get_item_ref(&ItemKey::TrackNumber)
            {
                match num_item.value() {
                    ItemValue::Text(text) => match text.parse::<i32>() {
                        Ok(n) => n,
                        _ => 0,
                    },
                    _ => 0,
                }
            } else {
                0
            },
        }))
    } else {
        Ok(None)
    }
}

pub enum WhichTrack {
    Previous,
    Next,
}

pub fn get_prev_or_next_track(
    track: &Path,
    which: WhichTrack,
) -> Option<PathBuf> {
    let tracks = get_sorted_tracks(track);
    if let Ok(index) = tracks.binary_search(&track.to_path_buf()) {
        match which {
            WhichTrack::Previous => {
                if index > 0 {
                    return Some(tracks[index - 1].clone());
                }
            }
            WhichTrack::Next => {
                if index + 1 < tracks.len() {
                    return Some(tracks[index + 1].clone());
                }
            }
        }
    }
    None
}

fn get_sorted_tracks(track: &Path) -> Vec<PathBuf> {
    let mut tracks = vec![];
    let suffixes = vec!["flac", "mogg", "mp3", "oga", "ogg", "wav"];
    if let Some(dir) = track.parent() {
        if let Ok(walker) = dir.read_dir() {
            #[allow(clippy::manual_flatten)]
            for entry in walker {
                if let Ok(entry) = entry {
                    if let Ok(kind) = entry.file_type() {
                        if !kind.is_file() {
                            continue;
                        }
                    }
                    let path = entry.path();
                    if let Some(extension) = path.extension() {
                        let extension = extension.to_string_lossy();
                        for suffix in &suffixes {
                            if *suffix == extension {
                                tracks.push(path);
                                break;
                            }
                        }
                    }
                }
            }
            tracks.sort();
        }
    }
    tracks
}

// Returns a name suitable as the last component of a tree path
pub fn canonicalize(track: &Path) -> String {
    let mut s = String::new();
    if let Some(stem) = track.file_stem() {
        s = stem.to_string_lossy().to_string();
    }
    s = s.trim_end_matches(&['-', '_', ' ']).to_string();
    let t = &s.trim_start_matches(&[
        '-', '_', ' ', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    ]);
    if !t.is_empty() {
        s = t.to_string();
    }
    s.replace(&['-', '_', '/', '\\'], " ")
}
