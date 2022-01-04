#!/usr/bin/env python3
# Copyright © 2021 Mark Summerfield. All rights reserved.
# License: GPLv3

import collections
import enum
import gzip
import os

import mutagen


MAGIC = '*MB>'
VERSION = '100'


class Error(Exception):
    pass


HistoryItem = collections.namedtuple('HistoryItem', 'group playlist track')


class Mb:

    def __init__(self, filename=None):
        self.groups = []
        self._filename = filename
        self.current_group = None
        self.history = collections.deque()
        if self._filename is not None and os.path.exists(self._filename):
            self.load()


    @property
    def filename(self):
        return self._filename


    @filename.setter
    def filename(self, filename):
        self._filename = filename
        if os.path.exists(filename):
            self.load()


    def load(self, filename=None):
        if filename is not None:
            self._filename = filename
        with open(self._filename, 'rb') as file:
            opener = (open if file.read(4) == MAGIC.encode('ascii') else
                      gzip.open)
        self.groups.clear()
        self.history.clear()
        current = _Current()
        playlist = None
        state = _State.MAGIC
        with opener(self._filename, 'rt', encoding='utf-8') as file:
            for lino, line in enumerate(file, 1):
                line = line.strip()
                if not line:
                    continue # ignore blank lines
                if state is _State.INGROUPS and line == '*C':
                    state = _State.CURRENT
                    continue
                if state in {_State.CURRENT,
                             _State.HISTORY} and line == '*H':
                    state = _State.INHISTORY
                    continue
                if state in {_State.INHISTORY,
                             _State.TRACK} and line.startswith('*P'):
                    state = _State.PLAYLIST # No continue: parse whole line
                if state is _State.MAGIC:
                    if not line.startswith(MAGIC):
                        raise Error(f'error:{lino}: not a .mb file')
                    # NOTE We ignore the version for now
                    state = _State.GROUPS
                elif state is _State.GROUPS:
                    if line != '*G':
                        raise Error(f'error:{lino}: missing groups list')
                    state = _State.INGROUPS
                elif state is _State.INGROUPS:
                    self.groups.append(Group(line))
                elif state is _State.CURRENT:
                    if line != '*H':
                        self._read_current(lino, line, current)
                    state = _State.HISTORY
                elif state is _State.INHISTORY:
                    self._read_history(lino, line)
                elif state is _State.PLAYLIST:
                    playlist = self._read_playlist(lino, line)
                    state = _State.TRACK
                elif state is _State.TRACK:
                    self._read_track(lino, line, playlist)
        self._update_currents(current)


    def _read_current(self, lino, line, current):
        parts = line.split('>')
        if len(parts) == 4:
            current.group_name = parts[0]
            current.playlist_name = parts[1]
            current.track_name = parts[2]
            try:
                current.pos = float(parts[3])
            except ValueError:
                pass
        else:
            raise Error(f'error:{lino}: invalid current')


    def _read_history(self, lino, line):
        parts = line.split('>')
        if len(parts) == 3: # group>playlist>track
            self.history.append(HistoryItem(*parts))
        else:
            raise Error(f'error:{lino}: invalid history item')


    def _read_playlist(self, lino, line):
        if not line.startswith('*P'):
            raise Error(f'error:{lino}: expected playlist')
        parts = line.split('>')
        if len(parts) == 3: # *P>group>playlist
            playlist = Playlist(parts[2])
            group_name = parts[1]
            for group in self.groups:
                if group.title == group_name:
                    break
            else:
                group = Group(group_name)
            group.playlists.append(playlist)
            return playlist
        raise Error(f'error:{lino}: invalid playlist')


    def _read_track(self, lino, line, playlist):
        if playlist is None:
            raise Error(f'error:{lino}: track with no playlist')
        parts = line.split('>')
        if len(parts) != 2: # track>secs
            raise Error(f'error:{lino}: invalid track')
        try:
            secs = float(parts[1])
        except ValueError:
            secs = -1
        playlist.tracks.append(Track(parts[0], secs))


    def _update_currents(self, current):
        if current.group_name is not None:
            for group in self.groups:
                if group.title == current.group_name:
                    self.current_group = group
                    if current.playlist_name is not None:
                        for playlist in group.playlists:
                            if playlist.title == current.playlist_name:
                                group.current_playlist = playlist
                                if current.track_name is not None:
                                    for track in playlist.tracks:
                                        if (track.title ==
                                                current.track_name):
                                            playlist.current_track = track
                                            playlist.current_pos = (
                                                current.pos)


    def save(self, *, filename=None, compress=True):
        if filename is not None:
            self._filename = filename
        opener = gzip.open if compress else open
        with opener(self._filename, 'wt', encoding='utf-8') as file:
            file.write(f'{MAGIC}{VERSION}\n*G\n')
            for group in self.groups:
                file.write(f'{group.title}\n')
            file.write('*C\n*H\n') # no current; empty history
            for group in self.groups:
                for playlist in group.playlists:
                    file.write(f'*P>{group.title}>{playlist.title}\n')
                    for track in playlist.tracks:
                        file.write(f'{track.filename}>{track.secs:0.1f}\n')


    @property
    def secs(self):
        total = 0.0
        for group in self.groups:
            if group.secs > 0:
                total += group.secs
        return total


class _State(enum.Enum):
    MAGIC = enum.auto()
    GROUPS = enum.auto()
    INGROUPS = enum.auto()
    CURRENT = enum.auto()
    HISTORY = enum.auto()
    INHISTORY = enum.auto()
    PLAYLIST = enum.auto()
    TRACK = enum.auto()


class _Current:

    def __init__(self):
        self.group_name = None
        self.playlist_name = None
        self.track_name = None
        self.pos = 0


class Group:

    def __init__(self, title):
        self.title = title
        self.playlists = []
        self.current_playlist = None


    @property
    def secs(self):
        total = 0.0
        for playlist in self.playlists:
            if playlist.secs > 0:
                total += playlist.secs
        return total


class Playlist:

    def __init__(self, title):
        self.title = title
        self.tracks = []
        self.current_track = None
        self.current_pos = 0


    @property
    def secs(self):
        total = 0.0
        for track in self.tracks:
            if track.secs > 0:
                total += track.secs
        return total


class Track:

    def __init__(self, filename, secs=-1):
        self._filename = filename
        self._title = None
        self._secs = secs
        self._album = None
        self._artist = None
        self._number = 0


    def _populate_metadata(self):
        def get_meta_item(meta, name):
            try:
                return meta[name][0]
            except (KeyError, IndexError):
                pass

        try:
            meta = mutagen.File(self._filename)
            if meta is not None:
                self._title = get_meta_item(meta, 'title')
                self._secs = meta.info.length
                self._album = get_meta_item(meta, 'album')
                self._artist = get_meta_item(meta, 'artist')
                try:
                    self._number = int(meta['tracknumber'][0])
                except (IndexError, ValueError):
                    self._number = 0
                return
        except (mutagen.MutagenError, FileNotFoundError):
            pass
        if self._title is None:
            self._title = (
                os.path.splitext(os.path.basename(self._filename))[0]
                .replace('-', ' ').replace('_', ' '))


    @property
    def filename(self):
        return self._filename


    @property
    def title(self):
        if self._title is None:
            self._populate_metadata()
        return self._title


    @property
    def secs(self):
        if self._secs <= 0:
            self._populate_metadata()
        return self._secs


    @property
    def album(self):
        if self._album is None:
            self._populate_metadata()
        return self._album


    @property
    def artist(self):
        if self._artist is None:
            self._populate_metadata()
        return self._artist


    @property
    def number(self):
        if self._number == 0:
            self._populate_metadata()
        return self._number


if __name__ == '__main__':
    import sys

    if len(sys.argv) == 1 or sys.argv[1] in {'-h', '--help', 'help'}:
        raise SystemExit('usage: mb.py <musicbox.mb>')
    filename = sys.argv[1]
    if filename.endswith('.mb'):
        mb = Mb(filename)
        print(f'read {mb.filename} containing {len(mb.groups):,} groups '
              f'totalling {mb.secs:.0f} secs')
        for group in mb.groups:
            print(f'  Group: {group.title} containing '
                  f'{len(group.playlists):,} playlists totalling '
                  f'{group.secs:.0f} secs')
            for playlist in group.playlists:
                print(f'    Playlist: {playlist.title} containing '
                      f'{len(playlist.tracks):,} tracks totalling '
                      f'{playlist.secs:.0f} secs')
