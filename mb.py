#!/usr/bin/env python3
# Copyright © 2021 Mark Summerfield. All rights reserved.
# License: GPLv3

import collections
import enum
import gzip
import os

import mutagen


MAGIC = '\fMB\t'
VERSION = '100'

# TODO To find a track or group just iterate the tree_list or look up
#      directly via the git or tid.
# NOTE gid = 0 (top-level; unnamed) or 1-9998
# NOTE gid = 9999 (pseudo-group; Search Results)
# NOTE tid ≥ 10_000


class Error(Exception):
    pass


class Mb:

    def __init__(self, filename=None):
        self._group_for_gid = {} # gid → Group
        self._track_for_tid = {} # tid → Tracks
        self._tree_list = [] # list of gids then tids in load order
        self._filename = filename
        self.current_tid = None
        self.history = collections.deque() # tids
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
        self._group_for_gid.clear()
        self._track_for_tid.clear()
        self._tree_list.clear()
        self.history.clear()
        self.current_tid = None
        state = _State.WANT_MAGIC
        with opener(self._filename, 'rt', encoding='utf-8') as file:
            for lino, line in enumerate(file, 1):
                line = line.rstrip()
                if not line:
                    continue # ignore blank lines
                # Change state
                if state is _State.IN_GROUPS and line == '\fTRACKS':
                    state = _State.WANT_TRACK_FIELDS
                    continue
                if state is _State.IN_TRACKS and line == '\fBOOKMARKS':
                    state = _State.WANT_BOOKMARKS_FIELDS
                    continue
                if state is _State.IN_BOOKMARKS and line == '\fHISTORY':
                    state = _State.WANT_HISTORY_FIELDS
                    continue
                if state is _State.IN_HISTORY and line == '\fCURRENT':
                    state = _State.WANT_CURRENT_FIELDS
                    continue
                # Read data
                if state is _State.WANT_MAGIC:
                    if not line.startswith(MAGIC):
                        raise Error(f'error:{lino}: not a .mb file')
                    # NOTE We ignore the version for now
                    state = _State.WANT_GROUP_HEADER
                elif state is _State.WANT_GROUP_HEADER:
                    if line != '\fGROUPS':
                        raise Error(f'error:{lino}: expected groups header')
                    state = _State.WANT_GROUP_FIELDS
                elif state is _State.WANT_GROUP_FIELDS:
                    if not line.startswith('\vGID'):
                        raise Error(f'error:{lino}: expected groups fields')
                    state = _State.IN_GROUPS
                elif state is _State.IN_GROUPS:
                    self._read_group(lino, line)
                elif state is _State.WANT_TRACK_FIELDS:
                    if not line.startswith('\vTID'):
                        raise Error(f'error:{lino}: expected tracks fields')
                    state = _State.IN_TRACKS
                elif state is _State.IN_TRACKS:
                    self._read_track(lino, line)
                elif state is _State.WANT_BOOKMARKS_FIELDS:
                    if not line.startswith('\vTID'):
                        raise Error(
                            f'error:{lino}: expected bookmark fields')
                    state = _State.IN_BOOKMARKS
                elif state is _State.IN_BOOKMARKS:
                    self._read_bookmark(lino, line)
                elif state is _State.WANT_HISTORY_FIELDS:
                    if not line.startswith('\vTID'):
                        raise Error(
                            f'error:{lino}: expected history fields')
                    state = _State.IN_HISTORY
                elif state is _State.IN_HISTORY:
                    self._read_history(lino, line)
                elif state is _State.WANT_CURRENT_FIELDS:
                    if not line.startswith('\vTID'):
                        raise Error(
                            f'error:{lino}: expected current fields')
                    state = _State.IN_CURRENT
                elif state is _State.IN_CURRENT:
                    self._read_current(lino, line)
                else:
                    raise Error(f'error:{lino}: invalid .mb file')


    def _read_group(self, lino, line):
        try:
            gid, name, pgid = line.split('\t', maxsplit=2)
            group = Group(gid, name, pgid)
            self._group_for_gid[gid] = group
            self._tree_list.append(gid)
        except ValueError as err:
            raise Error(f'error:{lino}: failed to read group: {err}')


    def _read_track(self, lino, line):
        try:
            tid, filename, secs, pgid = line.split('\t', maxsplit=3)
            track = Track(tid, filename, float(secs), pgid)
            self._track_for_tid[tid] = track
            self._tree_list.append(tid)
        except ValueError as err:
            raise Error(f'error:{lino}: failed to read track: {err}')


    def _read_bookmark(self, lino, line):
        pass # TODO


    def _read_history(self, lino, line):
        pass # TODO


    def _read_current(self, lino, line):
        pass # TODO


    def save(self, *, filename=None, compress=True):
        if filename is not None:
            self._filename = filename
        opener = gzip.open if compress else open
        with opener(self._filename, 'wt', encoding='utf-8') as file:
            file.write(f'{MAGIC}{VERSION}\n')
            # TODO


    @property
    def tree_path(self, gid_or_tid):
        return '' # TODO 'Group name/Group name/Track name'


    @property
    def secs(self):
        pass # TODO


@enum.unique
class _State(enum.Enum):
    WANT_MAGIC = enum.auto()
    WANT_GROUP_HEADER = enum.auto()
    WANT_GROUP_FIELDS = enum.auto()
    IN_GROUPS = enum.auto()
    WANT_TRACK_FIELDS = enum.auto()
    IN_TRACKS = enum.auto()
    WANT_BOOKMARKS_FIELDS = enum.auto()
    IN_BOOKMARKS = enum.auto()
    WANT_HISTORY_FIELDS = enum.auto()
    IN_HISTORY = enum.auto()
    WANT_CURRENT_FIELDS = enum.auto()
    IN_CURRENT = enum.auto()


class Group:

    def __init__(self, gid, name, pgid=0):
        self.gid = gid
        self.name = name
        self.pgid = pgid


    def __repr__(self):
        return f'Group({self.gid}, {self.name}, {self.pgid})'


    def __lt__(self, group):
        sname = self.name.upper()
        gname = group.name.upper()
        if sname != gname:
            return sname < gname
        return self.gid < group.gid



class Track:

    def __init__(self, tid, filename, secs, pgid=0):
        self.tid = tid
        self._filename = filename
        self.pgid = pgid
        self._title = None
        self._secs = secs
        self._album = None
        self._artist = None
        self._number = 0


    def __repr__(self):
        return (f'Track({self.tid}, {self.filename}, {self.secs:0.3f}, '
                f'{self.pgid})')


    def __lt__(self, track):
        sname = self.filename.upper()
        tname = track.filename.upper()
        if sname != tname:
            return sname < tname
        return self.tid < track.tid


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
        print(f'{len(mb._group_for_gid):,} groups; '
              f'{len(mb._track_for_tid):,} tracks')
        # TODO print summary (how many groups & tracks etc)
        # TODO print all
        # TODO save as
        # TODO print current & history
