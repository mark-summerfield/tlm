#!/usr/bin/env python3
# Copyright © 2021 Mark Summerfield. All rights reserved.
# License: GPLv3

import collections
import enum
import gzip
import os
import pathlib
import sys

MAGIC = '\fMB\t'
VERSION = '100'
INDENT = '\v'


class Error(Exception):
    pass


class Mb:

    def __init__(self, filename=None):
        self.clear()
        self._filename = filename
        if self._filename is not None and os.path.exists(self._filename):
            self.load()


    def clear(self):
        self.tree = [] # list of TreeData: fltk::tree::Tree
        self.track_for_tid = {} # key=tid value=Track: HashMap<usize, Track>
        self.tid = 1
        self.bookmarks = []
        self.history = collections.deque()
        self.current_track = None


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
        self.clear()
        treepath = []
        state = _State.WANT_MAGIC
        with opener(self._filename, 'rt', encoding='utf-8') as file:
            for lino, line in enumerate(file, 1):
                line = line.rstrip()
                if not line:
                    continue # ignore blank lines
                if state is _State.IN_TRACKS and line == '\fBOOKMARKS':
                    state = _State.IN_BOOKMARKS
                elif state is _State.IN_BOOKMARKS and line == '\fHISTORY':
                    state = _State.IN_HISTORY
                elif state is _State.IN_HISTORY and line == '\fCURRENT':
                    state = _State.IN_CURRENT
                elif state is _State.WANT_MAGIC:
                    if not line.startswith(MAGIC):
                        raise Error(f'error:{lino}: not a .mb file')
                    # NOTE We ignore the version for now
                    state = _State.WANT_TRACK_HEADER
                elif state is _State.WANT_TRACK_HEADER:
                    if line != '\fTRACKS':
                        raise Error(f'error:{lino}: missing TRACKS')
                    state = _State.IN_TRACKS
                elif state is _State.IN_TRACKS:
                    if line.startswith(INDENT):
                        self._read_group(treepath, lino, line)
                    elif not line.startswith('\f'):
                        self._read_track(treepath, lino, line)
                elif state is _State.IN_BOOKMARKS:
                    self.bookmarks.append(line)
                elif state is _State.IN_HISTORY:
                    self.history.append(line)
                elif state is _State.IN_CURRENT:
                    self.current_track = line
                    state = _State.END
                elif state is _State.END:
                    raise Error(f'error:{lino}: spurious data at the end')
                else:
                    raise Error(f'error:{lino}: invalid .mb file')


    def _read_group(self, treepath, lino, line):
        name = line.lstrip(INDENT)
        indent = len(line) - len(name)
        prev_indent = len(treepath)
        if indent == 1:
            treepath[:] = [name]
        elif indent > prev_indent: # child
            treepath.append(name)
        elif indent <= prev_indent: # same level or higher
            for _ in range(prev_indent - indent + 1):
                treepath.pop() # move back up to same or higher parent
            treepath.append(name)


    def _read_track(self, treepath, lino, line):
        try:
            filename, secs = line.split('\t', maxsplit=1)
            self.track_for_tid[self.tid] = Track(filename, float(secs))
            self.tree.append(TreeData('/'.join(treepath), self.tid))
            self.tid += 1
        except ValueError as err:
            raise Error(f'error:{lino}: failed to read track: {err}')


    def save(self, *, filename=None, compress=True):
        if filename is not None:
            self._filename = filename
        done = set()
        opener = gzip.open if compress else open
        with opener(self._filename, 'wt', encoding='utf-8') as file:
            file.write('\fMB\t100\n\fTRACKS\n')
            for treedata in self.tree:
                for i, path in enumerate(treedata.treepath.split('/'), 1):
                    path = f'{i * INDENT}{path}\n'
                    if path not in done:
                        file.write(path)
                        done.add(path)
                track = self.track_for_tid[treedata.tid]
                file.write(f'{track.filename}\t{track.secs:.03f}\n')
            file.write('\fBOOKMARKS\n\fHISTORY\n\fCURRENT\n')


@enum.unique
class _State(enum.Enum):
    WANT_MAGIC = enum.auto()
    WANT_TRACK_HEADER = enum.auto()
    IN_TRACKS = enum.auto()
    IN_BOOKMARKS = enum.auto()
    IN_HISTORY = enum.auto()
    IN_CURRENT = enum.auto()
    END = enum.auto()


Track = collections.namedtuple('Track', 'filename secs')
TreeData = collections.namedtuple('TreeData', 'treepath tid')


if __name__ == '__main__':
    if len(sys.argv) == 1 or sys.argv[1] in {'-h', '--help', 'help'}:
        raise SystemExit(
            'usage: mb.py <-t|--tree> infile.mb | infile.mb outfile.mb')
    filename = sys.argv[1]
    if filename in {'-t', '--tree'}:
        mb = Mb(sys.argv[2])
        for treedata in mb.tree:
            print(treedata)
        for (tid, track) in mb.track_for_tid.items():
            print(tid, track)
    else:
        infile = pathlib.Path(sys.argv[1]).resolve()
        outfile = pathlib.Path(sys.argv[2]).resolve()
        if infile == outfile:
            raise SystemExit('infile and outfile must be different')
        mb = Mb(infile)
        mb.save(filename=outfile)
        print('saved', outfile)
