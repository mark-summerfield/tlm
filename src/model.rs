// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use crate::util;
use anyhow::{bail, Result};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use fltk::tree::Tree;
use std::collections::{HashMap, HashSet, VecDeque};
use std::{
    fs::File,
    io::prelude::*,
    io::BufReader,
    path::{Path, PathBuf},
    str::FromStr,
};

type TreePath = String;
type TrackID = i32;
type TrackForTID = HashMap<TrackID, Track>;

#[derive(Debug)]
pub struct Track {
    filename: PathBuf,
    secs: f64,
}

pub struct Model {
    pub filename: PathBuf,
    pub dirty: bool,
    pub track_tree: Tree,
    pub track_for_tid: TrackForTID,
    pub history: VecDeque<TreePath>,
}

impl Model {
    pub fn new(track_tree: Tree) -> Self {
        Self {
            filename: PathBuf::new(),
            dirty: false,
            track_tree,
            track_for_tid: TrackForTID::default(),
            history: VecDeque::default(),
        }
    }

    pub fn has_current_treepath(&self) -> bool {
        if let Some(treepath) = self.history.front() {
            !treepath.is_empty()
        } else {
            false
        }
    }

    pub fn current_treepath(&self) -> TreePath {
        if let Some(treepath) = self.history.front() {
            treepath.to_string()
        } else {
            TreePath::new()
        }
    }

    pub fn add_to_history(&mut self, treepath: TreePath) -> bool {
        let changed =
            util::maybe_add_to_deque(&mut self.history, treepath, 35);
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

    fn clear(&mut self) {
        self.dirty = false;
        self.track_for_tid.clear();
        self.history.clear();
        self.track_tree.clear();
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
            let item = self.track_tree.add(&treepath);
            item.unwrap().set_user_data(tid);
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

    pub fn save(&mut self, filename: &Path) -> Result<()> {
        dbg!("save", filename);
        /*
        if !filename.to_string_lossy().is_empty() {
            self.filename = filename.to_path_buf();
        }
        let mut done = HashSet::<String>::new();
        let file = File::create(&self.filename)?;
        let mut gz = GzEncoder::new(file, Compression::best());
        gz.write_all(b"\x0CTLM\t100\n\x0CTRACKS\n")?;
        for treeitem in &self.tree {
            let treepath: Vec<&str> =
                treeitem.treepath.split('/').collect();
            for (i, subpath) in treepath.iter().enumerate() {
                let indent = INDENT.to_string().repeat(i + 1);
                let subpath = format!("{indent}{subpath}\n");
                if !done.contains(&subpath) {
                    gz.write_all(subpath.as_bytes())?;
                    done.insert(subpath);
                }
            }
            if let Some(track) = self.track_for_tid.get(&treeitem.tid) {
                gz.write_all(
                    format!(
                        "{}\t{:.3}\n",
                        track.filename.display(),
                        track.secs
                    )
                    .as_bytes(),
                )?;
            }
        }
        gz.write_all("\x0CHISTORY\n".as_bytes())?;
        gz.finish()?;
        */
        self.dirty = false;
        Ok(())
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
