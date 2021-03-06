// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use crate::model::Track;
use anyhow::{bail, Result};
use std::{
    fs::File,
    io::{self, BufRead},
    path::{Path, PathBuf},
    str::FromStr,
};

/*
BNF:
    M3U      ::= '#EXTM3U' ENTRY+
    ENTRY    ::= INFO FILENAME
    INFO     ::= '#EXTINF:' SECONDS ',' TITLE
    SECONDS  ::= -?\\d+
    TITLE    ::= .+
    FILENAME ::= .+

Example:
    #EXTM3U

    #EXTINF:-1,You and I
    /home/mark/music/Queen/05-You_and_I.mp3
*/

pub fn read_m3u(filename: &Path) -> Result<Vec<Track>> {
    static EXTM3U: &str = "#EXTM3U";
    static EXTINF: &str = "#EXTINF:";
    enum Expect {
        Header,
        Info,
        Filename,
    }
    let mut state = Expect::Header;
    let mut secs = 0.0;
    let mut tracks = vec![];
    let file = File::open(filename)?;
    let reader = io::BufReader::new(file);
    for (lino, line) in reader.lines().enumerate() {
        if let Ok(line) = line {
            let line = line.trim();
            if line.is_empty() {
                continue; // ignore blank lines
            }
            match state {
                Expect::Header => {
                    if line != EXTM3U {
                        bail!(
                            "{}:expected {EXTM3U} header: {line}",
                            lino + 1
                        )
                    }
                    state = Expect::Info;
                }
                Expect::Info => {
                    if !line.starts_with(EXTINF) {
                        bail!("{}:expected {EXTINF} line: {line}", lino + 1)
                    }
                    let line = line
                        .trim_start_matches(EXTINF)
                        .trim_start_matches(':');
                    if let Some((left, _title)) = line.split_once(',') {
                        secs = f64::from_str(left).unwrap_or(0.0);
                        state = Expect::Filename;
                    } else {
                        bail!("{}:invalid {EXTINF} line: {line}", lino + 1)
                    }
                }
                Expect::Filename => {
                    if line.starts_with(EXTINF) {
                        bail!(
                            "{}:unexpected {EXTINF} line: {line}",
                            lino + 1
                        )
                    }
                    let filename = PathBuf::from(line);
                    if filename.exists() {
                        tracks.push(Track::new(filename, secs));
                    } else {
                        println!("skipping missing track: {}", line);
                    }
                    secs = 0.0;
                    state = Expect::Info;
                }
            };
        }
    }
    Ok(tracks)
}
