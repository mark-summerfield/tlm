// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use crate::application::Application;
use crate::model::Track;
use crate::new_list_form;
use crate::playlists;
use crate::util;
use anyhow::anyhow;
use std::path::Path;

impl Application {
    pub(crate) fn on_list_add(&mut self) {
        let mut top_levels = vec![];
        if let Some(root) = self.tlm.track_tree.first() {
            let mut opt_item = root.next(); // first top-level child
            while let Some(item) = opt_item {
                opt_item = item.next_sibling();
                if let Some(name) = item.label() {
                    top_levels.push(name);
                }
            }
        }
        let form =
            new_list_form::Form::new(&self.current.track, &top_levels);
        if *form.ok.borrow() {
            let name = &*form.name.borrow();
            let parent_list = &*form.parent_list.borrow();
            let folder_or_playlist = &*form.folder_or_playlist.borrow();
            let include_subdirs = *form.include_subdirs.borrow();
            if !folder_or_playlist.exists() {
                self.new_empty_list(parent_list, name);
            } else if folder_or_playlist.is_file() {
                self.new_list_from_playlist(
                    parent_list,
                    name,
                    folder_or_playlist,
                );
            } else if folder_or_playlist.is_dir() {
                self.new_list_from_folder(
                    parent_list,
                    name,
                    folder_or_playlist,
                    include_subdirs,
                );
            }
        }
    }

    fn new_empty_list(&mut self, parent_list: &str, name: &str) {
        let name = util::sanitize(name, "New List");
        if let Some((treepath, item)) =
            self.tlm.add_empty_list(parent_list, &name)
        {
            self.select_track_in_tree(treepath, item);
            self.update_ui();
        }
    }

    fn new_list_from_playlist(
        &mut self,
        parent_list: &str,
        name: &str,
        playlist: &Path,
    ) {
        let name = if name.is_empty() {
            util::canonicalize(playlist)
        } else {
            util::sanitize(name, "New List")
        };
        if let Some((treepath, item)) =
            self.tlm.add_empty_list(parent_list, &name)
        {
            static MESSAGE: &str = "can only read .m3u playlists";
            let reply = match playlist.extension() {
                Some(suffix) => {
                    if let Some(suffix) = suffix.to_str() {
                        match suffix {
                            "m3u" | "M3U" => playlists::read_m3u(playlist),
                            _ => Err(anyhow!(MESSAGE)),
                        }
                    } else {
                        Err(anyhow!(MESSAGE))
                    }
                }
                None => Err(anyhow!(MESSAGE)),
            };
            let mut first = None;
            match reply {
                Ok(tracks) => {
                    for track in tracks {
                        if let Some((path, item)) =
                            self.tlm.add_track(&treepath, track)
                        {
                            if first.is_none() {
                                first = Some((path, item));
                            }
                        }
                    }
                    self.tlm.clear_selection();
                    if let Some((treepath, item)) = first {
                        self.select_track_in_tree(treepath, item);
                    } else {
                        self.select_track_in_tree(treepath, item);
                    }
                    self.update_ui();
                }
                Err(err) => util::popup_error_message(&err.to_string()),
            };
        }
    }

    fn new_list_from_folder(
        &mut self,
        parent_list: &str,
        name: &str,
        folder: &Path,
        include_subdirs: bool,
    ) {
        let name = if name.is_empty() {
            util::canonicalize(folder)
        } else {
            util::sanitize(name, "New List")
        };
        if let Some((treepath, item)) =
            self.tlm.add_empty_list(parent_list, &name)
        {
            let walker = if include_subdirs {
                walkdir::WalkDir::new(folder).sort_by_file_name()
            } else {
                walkdir::WalkDir::new(folder)
                    .sort_by_file_name()
                    .max_depth(1)
            };
            let mut first = None;
            for entry in walker.into_iter().filter_map(|e| e.ok()) {
                if !entry.path().is_file() {
                    continue;
                }
                if let Some((path, item)) = self.tlm.add_track(
                    &treepath,
                    Track::new(entry.into_path(), 0.0),
                ) {
                    if first.is_none() {
                        first = Some((path, item));
                    }
                }
            }
            self.tlm.clear_selection();
            if let Some((treepath, item)) = first {
                self.select_track_in_tree(treepath, item);
            } else {
                self.select_track_in_tree(treepath, item);
            }
            self.update_ui();
        }
    }

    pub(crate) fn on_list_rename(&mut self) {
        // util::sanitize(new_name, old_name)
        println!("on_list_rename"); // TODO
        /*
        self.tlm.set_dirty(); // unless already done
        self.tlm.track_tree.redraw();
        self.update_ui();
        */
    }

    pub(crate) fn on_list_export(&mut self) {
        println!("on_list_export"); // TODO
    }

    pub(crate) fn on_list_import(&mut self) {
        println!("on_list_import"); // TODO
        self.update_ui();
    }
}
