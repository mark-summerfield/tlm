// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use crate::fixed::{MAX_HISTORY_SIZE, TIME_ICONS, TOP_LEVEL_NAME};
use crate::util;
use anyhow::{bail, Result};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use fltk::{
    enums::Color,
    image::SvgImage,
    tree::{Tree, TreeItem},
};
use std::collections::{HashMap, HashSet, VecDeque};
use std::{
    collections::vec_deque::Iter,
    fs::File,
    io::prelude::*,
    io::BufReader,
    path::{Path, PathBuf},
    str::FromStr,
};

pub type TreePath = String;
pub type TrackID = i32;
type TrackForTID = HashMap<TrackID, Track>;
const INVALID_TID: i32 = -1;

pub struct Current {
    pub treepath: TreePath,
    pub track: PathBuf,
    pub tid: TrackID,
}

impl Default for Current {
    fn default() -> Self {
        Self {
            treepath: TreePath::new(),
            track: PathBuf::new(),
            tid: INVALID_TID,
        }
    }
}

impl Current {
    pub fn has_track(&self) -> bool {
        self.tid != INVALID_TID
            && !self.treepath.is_empty()
            && self.track.exists()
    }
}

#[derive(Clone, Debug)]
pub struct Track {
    pub filename: PathBuf,
    pub secs: f64,
}

impl Track {
    pub fn new(filename: PathBuf, secs: f64) -> Self {
        Self { filename, secs }
    }
}

pub struct Model {
    pub filename: PathBuf,
    pub track_tree: Tree,
    pub track_for_tid: TrackForTID,
    pub next_tid: TrackID,
    history: VecDeque<TreePath>,
    dirty: bool,
}

impl Model {
    pub fn new(track_tree: Tree) -> Self {
        Self {
            filename: PathBuf::new(),
            track_tree,
            track_for_tid: TrackForTID::default(),
            next_tid: 1,
            history: VecDeque::default(),
            dirty: false,
        }
    }

    pub fn load(&mut self, filename: &Path) -> Result<()> {
        if filename.exists() {
            self.filename = filename.to_path_buf();
        }
        self.clear();
        let text = self.get_text()?;
        self.parse(text)
    }

    fn get_text(&self) -> Result<String> {
        let compressed = self.is_compressed()?;
        let mut text = String::new();
        let file = File::open(&self.filename)?;
        if compressed {
            let mut gz = GzDecoder::new(file);
            gz.read_to_string(&mut text)?;
        } else {
            let mut buffer = BufReader::new(file);
            buffer.read_to_string(&mut text)?;
        }
        Ok(text)
    }

    fn is_compressed(&self) -> Result<bool> {
        let mut file = File::open(&self.filename)?;
        let mut buffer = [0; 2]; // 0x1F 0x8B gzip magic
        file.read_exact(&mut buffer)?;
        Ok(buffer[0] == 0x1F && buffer[1] == 0x8B)
    }

    pub fn clear(&mut self) {
        self.dirty = false;
        self.track_for_tid.clear();
        self.history.clear();
        self.track_tree.clear();
    }

    pub(crate) fn clear_selection(&mut self) {
        if let Some(item) = self.track_tree.first() {
            let _ = self.track_tree.deselect_all(&item, false); // skip err
        }
    }

    pub fn history_add_to(
        &mut self,
        treepath: TreePath,
        tid: TrackID,
        secs: f64,
    ) -> bool {
        let mut changed = util::maybe_add_to_deque(
            &mut self.history,
            treepath,
            MAX_HISTORY_SIZE,
        );
        if let Some(track) = self.track_for_tid.get_mut(&tid) {
            if !util::isclose64(secs, track.secs) {
                track.secs = secs;
                changed = true;
            }
        }
        if changed {
            self.dirty = true;
        }
        changed
    }

    pub fn history_delete_item(&mut self, index: usize) {
        self.history.remove(index);
        self.dirty = true;
    }

    pub fn history_shrink(&mut self) {
        self.history.truncate(1);
        self.dirty = true;
    }

    pub fn history_iter(&self) -> Iter<TreePath> {
        self.history.iter()
    }

    pub fn history_front(&self) -> Option<&TreePath> {
        self.history.front()
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self) {
        self.dirty = true;
    }

    fn parse(&mut self, text: String) -> Result<()> {
        let mut treepath: Vec<TreePath> = vec![];
        let mut state = State::WantMagic;
        let mut lists_seen = HashSet::<TreePath>::default();
        let mut tracks_seen = HashSet::<TreePath>::default();
        for (i, line) in text.lines().enumerate() {
            let lino = i + 1;
            if line.is_empty() {
                continue;
            }
            if state == State::InTracks && line == "\x0CHISTORY" {
                state = State::InHistory;
            } else if state == State::WantMagic {
                if !line.starts_with("\x0CTLM\t") {
                    bail!("error:{lino}: not a .tlm file");
                }
                // We ignore the version for now
                state = State::WantTrackHeader;
            } else if state == State::WantTrackHeader {
                if !line.starts_with("\x0CTRACKS") {
                    bail!("error:{lino}: missing TRACKS");
                }
                state = State::InTracks;
            } else if state == State::InTracks {
                if line.starts_with(INDENT) {
                    self.read_list(&mut treepath, &mut lists_seen, line);
                } else {
                    self.read_track(
                        &treepath,
                        &mut tracks_seen,
                        lino,
                        line,
                    )?;
                }
            } else if state == State::InHistory {
                self.history.push_back(line.to_string());
            } else {
                bail!("error:{lino}: invalid .tlm file");
            }
        }
        Ok(())
    }

    fn read_list(
        &mut self,
        treepath: &mut Vec<TreePath>,
        lists_seen: &mut HashSet<TreePath>,
        line: &str,
    ) {
        let name = line.trim_start_matches(INDENT);
        let indent = line.len() - name.len();
        let prev_indent = treepath.len();
        // if indent > prev_indent ∴ child, so just append
        if indent == 1 {
            treepath.clear();
        } else if indent <= prev_indent {
            // ∴ same level or higher
            for _ in 0..(prev_indent - indent + 1) {
                treepath.pop();
            }
        }
        treepath.push(name.to_string());
        // Needed to ensure that empty lists are shown
        let full_treepath = treepath.join("/");
        if !lists_seen.contains(&full_treepath) {
            self.track_tree.add(&full_treepath);
            lists_seen.insert(full_treepath);
        }
    }

    fn read_track(
        &mut self,
        treepath: &[TreePath],
        tracks_seen: &mut HashSet<TreePath>,
        lino: usize,
        line: &str,
    ) -> Result<()> {
        if let Some((filename, secs)) = line.split_once(TAB) {
            let secs = f64::from_str(secs).unwrap_or(0.0);
            let filename = PathBuf::from(filename);
            self.track_for_tid
                .insert(self.next_tid, Track::new(filename.clone(), secs));
            let treepath = self.full_treepath(
                self.next_tid,
                &treepath.join("/"),
                &filename,
                tracks_seen,
            );
            if let Some(mut item) = self.track_tree.add(&treepath) {
                item.set_label_fgcolor(Color::from_hex(0x000075));
                item.set_user_data(self.next_tid);
                let icon = image_for_secs(secs);
                item.set_user_icon(Some(icon));
            }
            self.next_tid += 1;
            Ok(())
        } else {
            bail!("error:{lino}: failed to read track {line}");
        }
    }

    fn full_treepath(
        &mut self,
        tid: TrackID,
        treepath: &str,
        filename: &Path,
        tracks_seen: &mut HashSet<TreePath>,
    ) -> TreePath {
        let mut full_treepath =
            format!("{}/{}", treepath, util::canonicalize(filename));
        if tracks_seen.contains(&full_treepath) {
            full_treepath.push(' ');
            full_treepath.push('(');
            full_treepath.push_str(&tid.to_string());
            full_treepath.push(')');
        } else {
            tracks_seen.insert(full_treepath.clone());
        }
        full_treepath
    }

    pub fn add_empty_list(
        &mut self,
        parent: &str,
        name: &str,
    ) -> Option<(TreePath, TreeItem)> {
        let treepath = if parent == TOP_LEVEL_NAME {
            name.to_string()
        } else {
            format!("{parent}/{name}")
        };
        self.dirty = true;
        self.track_tree.add(&treepath).map(|item| (treepath, item))
    }

    pub fn add_track(
        &mut self,
        treepath: &TreePath,
        track: Track,
    ) -> Option<(TreePath, TreeItem)> {
        self.track_for_tid.insert(self.next_tid, track.clone());
        let treepath =
            format!("{}/{}", treepath, util::canonicalize(&track.filename));
        if let Some(mut item) = self.track_tree.add(&treepath) {
            item.set_label_fgcolor(Color::from_hex(0x000075));
            item.set_user_data(self.next_tid);
            let icon = image_for_secs(track.secs);
            item.set_user_icon(Some(icon));
            self.next_tid += 1;
            self.dirty = true;
            Some((treepath, item))
        } else {
            None
        }
    }

    pub fn save(&mut self) -> Result<()> {
        assert!(!self.filename.as_os_str().is_empty());
        let file = File::create(&self.filename)?;
        let mut gz = GzEncoder::new(file, Compression::best());
        gz.write_all(b"\x0CTLM\t100\n\x0CTRACKS\n")?;
        let mut opt_item = self.track_tree.first();
        while let Some(item) = opt_item {
            opt_item = item.next();
            if item.depth() == 0 {
                continue; // skip "ROOT"
            }
            let tid = unsafe { item.user_data::<TrackID>() };
            if let Some(tid) = tid {
                if let Some(track) = self.track_for_tid.get(&tid) {
                    gz.write_all(
                        format!(
                            "{}\t{:.3}\n",
                            track.filename.display(),
                            track.secs
                        )
                        .as_bytes(),
                    )?;
                }
            } else {
                let indent =
                    INDENT.to_string().repeat(item.depth() as usize);
                gz.write_all(
                    format!(
                        "{indent}{}\n",
                        item.label().unwrap_or_default()
                    )
                    .as_bytes(),
                )?;
            }
        }
        gz.write_all("\x0CHISTORY\n".as_bytes())?;
        for treepath in &self.history {
            gz.write_all(format!("{treepath}\n").as_bytes())?;
        }
        gz.finish()?;
        self.dirty = false;
        Ok(())
    }

    pub fn save_as(&mut self, filename: &Path) -> Result<()> {
        self.filename = filename.to_path_buf();
        self.save()
    }

    pub fn track_for_item(&self, item: &TreeItem) -> Option<Track> {
        let tid = unsafe { item.user_data::<TrackID>() };
        if let Some(tid) = tid {
            self.track_for_tid.get(&tid).cloned()
        } else {
            None
        }
    }
}

fn image_for_secs(secs: f64) -> SvgImage {
    let index = if secs < 150.0 {
        0
    } else {
        (secs / 300.0).round().min(10.0) as usize
    };
    let icons = TIME_ICONS.get().read().unwrap();
    icons[index].clone()
}

const INDENT: char = '\x0B';
const TAB: char = '\x09';

#[derive(PartialEq)]
enum State {
    WantMagic,
    WantTrackHeader,
    InTracks,
    InHistory,
}
