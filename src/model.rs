// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use crate::fixed::{MAX_HISTORY_SIZE, TRACK_ICON, TREE_ICON_SIZE};
use crate::util;
use anyhow::{bail, Result};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use fltk::{enums::Color, image::SvgImage, prelude::ImageExt, tree::Tree};
use std::collections::{HashMap, HashSet, VecDeque};
use std::{
    fs::File,
    io::prelude::*,
    io::BufReader,
    path::{Path, PathBuf},
    str::FromStr,
};

pub type TreePath = String;
pub type TrackID = i32;
type TrackForTID = HashMap<TrackID, Track>;

#[derive(Debug)]
pub struct Track {
    pub filename: PathBuf,
    pub secs: f64,
}

pub struct Model {
    pub filename: PathBuf,
    pub dirty: bool,
    pub track_tree: Tree,
    pub track_for_tid: TrackForTID,
    pub history: VecDeque<TreePath>,
    pub track_icon: SvgImage,
}

impl Model {
    pub fn new(track_tree: Tree) -> Self {
        let mut track_icon = SvgImage::from_data(TRACK_ICON).unwrap();
        track_icon.scale(TREE_ICON_SIZE, TREE_ICON_SIZE, true, true);
        Self {
            filename: PathBuf::new(),
            dirty: false,
            track_tree,
            track_for_tid: TrackForTID::default(),
            history: VecDeque::default(),
            track_icon,
        }
    }

    pub fn add_to_history(&mut self, treepath: TreePath) -> bool {
        let changed = util::maybe_add_to_deque(
            &mut self.history,
            treepath,
            MAX_HISTORY_SIZE,
        );
        if changed {
            self.dirty = true;
        }
        changed
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

    pub fn shrink_history(&mut self) {
        self.history.truncate(1);
        self.dirty = true;
    }

    fn parse(&mut self, text: String) -> Result<()> {
        let mut tid = 1;
        let mut treepath: Vec<TreePath> = vec![];
        let mut state = State::WantMagic;
        let mut seen = HashSet::<TreePath>::default();
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
                    self.read_list(&mut treepath, line);
                } else {
                    tid = self.read_track(
                        tid, &treepath, &mut seen, lino, line,
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

    fn read_list(&mut self, treepath: &mut Vec<TreePath>, line: &str) {
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
    }

    fn read_track(
        &mut self,
        tid: TrackID,
        treepath: &[TreePath],
        seen: &mut HashSet<TreePath>,
        lino: usize,
        line: &str,
    ) -> Result<TrackID> {
        if let Some((filename, secs)) = line.split_once(TAB) {
            let secs = f64::from_str(secs).unwrap_or(0.0);
            let filename = PathBuf::from(filename);
            self.track_for_tid
                .insert(tid, Track { filename: filename.clone(), secs });
            let treepath = self.full_treepath(
                tid,
                &treepath.join("/"),
                &filename,
                seen,
            );
            if let Some(mut item) = self.track_tree.add(&treepath) {
                item.set_label_fgcolor(color_for_secs(secs));
                item.set_user_data(tid);
                item.set_user_icon(Some(self.track_icon.clone()));
            }
            Ok(tid + 1)
        } else {
            bail!("error:{lino}: failed to read track {line}");
        }
    }

    fn full_treepath(
        &mut self,
        tid: TrackID,
        treepath: &str,
        filename: &Path,
        seen: &mut HashSet<TreePath>,
    ) -> TreePath {
        let mut full_treepath =
            format!("{}/{}", treepath, util::canonicalize(filename));
        if seen.contains(&full_treepath) {
            full_treepath.push(' ');
            full_treepath.push('(');
            full_treepath.push_str(&tid.to_string());
            full_treepath.push(')');
        } else {
            seen.insert(full_treepath.clone());
        }
        full_treepath
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

fn color_for_secs(secs: f64) -> Color {
    if secs < 300.0 {
        Color::from_hex(0x008080)
    } else if secs < 600.0 {
        Color::from_hex(0x008000)
    } else {
        Color::from_hex(0x000080)
    }
}
