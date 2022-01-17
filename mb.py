#!/usr/bin/env python3
# Copyright © 2021 Mark Summerfield. All rights reserved.
# License: GPLv3

import collections
import enum
import gzip
import os
import pathlib

import mutagen

MAGIC = '\fMB\t'
VERSION = '100'
SEARCH_RESULTS_GID = 9999
MIN_GID = 0
MAX_GID = SEARCH_RESULTS_GID - 1
MIN_TID = MAX_GID + 1
MAX_TID = 2_000_000_000


def valid_gid(n, *, allow_search_results=True):
    if allow_search_results and n == SEARCH_RESULTS_GID:
        return True
    return MIN_GID <= n <= MAX_GID


def valid_tid(n):
    return MIN_TID <= n <= MAX_TID


def treename(filename):
    return (pathlib.Path(filename).stem.replace('-', ' ').replace('_', ' ')
            .lstrip('0123456789 '))


class Error(Exception):
    pass


class Mb:

    def __init__(self, filename=None):
        self.group_for_gid = {} # gid → Group
        self.track_for_tid = {} # tid → Tracks
        self.tree_list = [] # list of gids then tids in load order
        self.bookmarks = [] # tids
        self.history = collections.deque() # tids
        self.current_tid = None
        self._filename = filename
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


    def clear(self):
        self.group_for_gid.clear()
        self.group_for_gid[0] = Group(0, '', 0) # top-level (own parent)
        self.track_for_tid.clear()
        self.tree_list.clear()
        self.tree_list.append(0) # top-level
        self.bookmarks.clear()
        self.history.clear()
        self.current_tid = None


    def load(self, filename=None):
        if filename is not None:
            self._filename = filename
        with open(self._filename, 'rb') as file:
            opener = (open if file.read(4) == MAGIC.encode('ascii') else
                      gzip.open)
        self.clear()
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
            gid = int(gid)
            if not valid_gid(gid, allow_search_results=False):
                raise Error(f'error:{lino}: invalid group ID')
            pgid = int(pgid)
            if not valid_gid(pgid, allow_search_results=False):
                raise Error(f'error:{lino}: invalid parent group ID')
            if gid == pgid:
                raise Error(f'error:{lino}: only top-level group can be '
                            'its own parent')
            group = Group(gid, name, pgid)
            self.group_for_gid[gid] = group
            self.tree_list.append(gid)
        except ValueError as err:
            raise Error(f'error:{lino}: failed to read group: {err}')


    def _read_track(self, lino, line):
        try:
            tid, filename, secs, pgid = line.split('\t', maxsplit=3)
            tid = int(tid)
            if not valid_tid(tid):
                raise Error(f'error:{lino}: invalid track ID')
            pgid = int(pgid)
            if not valid_gid(pgid, allow_search_results=False):
                raise Error(f'error:{lino}: invalid parent group ID')
            track = Track(tid, filename, float(secs), pgid)
            self.track_for_tid[tid] = track
            self.tree_list.append(tid)
        except ValueError as err:
            raise Error(f'error:{lino}: failed to read track: {err}')


    def _read_bookmark(self, lino, line):
        try:
            tid = int(line)
            if not valid_tid(tid):
                raise Error(f'error:{lino}: invalid track ID')
            self.bookmarks.append(tid)
        except ValueError as err:
            raise Error(f'error:{lino}: invalid bookmark: {err}')


    def _read_history(self, lino, line):
        try:
            tid = int(line)
            if not valid_tid(tid):
                raise Error(f'error:{lino}: invalid track ID')
            self.history.append(tid)
        except ValueError as err:
            raise Error(f'error:{lino}: invalid history: {err}')


    def _read_current(self, lino, line):
        try:
            tid = int(line)
            if not valid_tid(tid):
                raise Error(f'error:{lino}: invalid track ID')
            self.current_tid = tid
        except ValueError as err:
            raise Error(f'error:{lino}: invalid history: {err}')


    def save(self, *, filename=None, compress=True):
        if filename is not None:
            self._filename = filename
        opener = gzip.open if compress else open
        with opener(self._filename, 'wt', encoding='utf-8') as file:
            file.write(f'{MAGIC}{VERSION}\n')
            file.write('\fGROUPS\n\vGID\tNAME\tPGID\n')
            in_tracks = False
            for n in self.tree_list:
                if n == 0:
                    continue
                if n <= MAX_GID:
                    group = self.group_for_gid[n]
                    file.write(f'{n}\t{group.name}\t{group.pgid}\n')
                else:
                    if not in_tracks:
                        file.write(
                            '\fTRACKS\n\vTID\tFILENAME\tSECS\tPGID\n')
                        in_tracks = True
                    track = self.track_for_tid[n]
                    file.write(f'{n}\t{track.filename}\t{track.secs:.03f}'
                               f'\t{track.pgid}\n')
            file.write('\fBOOKMARKS\n\vTID\n')
            for tid in self.bookmarks:
                file.write(str(tid))
            file.write('\fHISTORY\n\vTID\n')
            for tid in self.history:
                file.write(str(tid))
            file.write('\fCURRENT\n\vTID\n')
            if self.current_tid:
                file.write(f'{self.current_tid}\n')


    def path_for(self, gid_or_tid):
        if gid_or_tid <= MAX_GID:
            return self._path_for_group(gid_or_tid)
        return self._path_for_track(gid_or_tid)


    def _path_for_group(self, gid):
        path = collections.deque()
        while gid:
            group = self.group_for_gid[gid]
            path.appendleft(group.name)
            gid = group.pgid
        return '/'.join(path)


    def _path_for_track(self, tid):
        track = self.track_for_tid[tid]
        return (self._path_for_group(track.pgid) + '/' +
                treename(track.filename))


    def secs_for(self, gid_or_tid=None):
        if valid_tid(gid_or_tid): # One track
            return self.track_for_tid[gid_or_tid].secs
        total = 0.0 # Entire music box or one group (excl. child groups)
        for track in self.track_for_tid.values():
            if not gid_or_tid or track.pgid == gid_or_tid:
                total += track.secs
        return total


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
    import tempfile

    if len(sys.argv) == 1 or sys.argv[1] in {'-h', '--help', 'help'}:
        raise SystemExit('usage: mb.py <musicbox.mb>')
    infile = sys.argv[1]
    if infile.endswith('.mb'):
        mb = Mb(infile)
        print(f'{len(mb.group_for_gid):,} groups; '
              f'{len(mb.track_for_tid):,} tracks')
        zoutfile = tempfile.gettempdir() + '/mbz.mb'
        outfile = tempfile.gettempdir() + '/mb.mb'
        mb.save(filename=zoutfile)
        mb.save(filename=outfile, compress=False)
        print(
            f'saved compressed to {zoutfile} and uncompressed to {outfile}')
