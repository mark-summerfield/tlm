#!/usr/bin/env python3
# Copyright © 2021 Mark Summerfield. All rights reserved.
# License: GPLv3

import collections
import enum
import gzip
import os
import pathlib
import sys

import mutagen

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


    @property
    def filename(self):
        return self._filename


    @filename.setter
    def filename(self, filename):
        self._filename = filename
        if os.path.exists(filename):
            self.load()


    def clear(self):
        self.tree = Group('')
        self.bookmarks = []
        self.history = collections.deque()
        self.current_track = None


    def load(self, filename=None):
        if filename is not None:
            self._filename = filename
        with open(self._filename, 'rb') as file:
            opener = (open if file.read(4) == MAGIC.encode('ascii') else
                      gzip.open)
        self.clear()
        stack = [self.tree]
        prev_indent = 0
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
                        prev_indent = self._read_group(stack, prev_indent,
                                                       lino, line)
                    elif not line.startswith('\f'):
                        self._read_track(stack[-1], lino, line)
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


    def _read_group(self, stack, prev_indent, lino, line):
        name = line.lstrip(INDENT)
        indent = len(line) - len(name)
        group = Group(name)
        if indent == 1:
            self.tree.append(group)
            stack[:] = [self.tree, group]
        elif indent > prev_indent: # child
            stack[-1].append(group)
            stack.append(group)
        elif indent <= prev_indent: # same level or higher
            for _ in range(prev_indent - indent + 1):
                stack.pop() # move back up to same or higher parent
            stack[-1].append(group)
            stack.append(group)
        return indent


    def _read_track(self, group, lino, line):
        try:
            filename, secs = line.split('\t', maxsplit=1)
            group.append(Track(filename, float(secs)))
        except ValueError as err:
            raise Error(f'error:{lino}: failed to read track: {err}')


    def save(self, *, filename=None, compress=True):
        if filename is not None:
            self._filename = filename
        opener = gzip.open if compress else open
        with opener(self._filename, 'wt', encoding='utf-8') as file:
            file.write('\fMB\t100\n\fTRACKS\n')
            self._write_tree(file, self.tree)
            file.write('\fBOOKMARKS\n\fHISTORY\n\fCURRENT\n')


    def _write_tree(self, file, tree, depth=1):
        pad = depth * INDENT
        for kid in tree.kids:
            if isinstance(kid, Group):
                file.write(f'{pad}{kid.name}\n')
                self._write_tree(file, kid, depth + 1)
            else:
                file.write(f'{kid.filename}\t{kid.secs:.03f}\n')


    def paths(self):
        for path in self._paths(self.tree, ''):
            yield path


    def _paths(self, tree, prefix):
        prefix = f'{prefix}/{tree.name}' if prefix else tree.name
        if prefix:
            yield prefix
        for kid in tree.kids:
            if isinstance(kid, Group):
                for path in self._paths(kid, prefix):
                    yield path
            elif kid.treename:
                yield f'{prefix}/{kid.treename}'


    def secs_for(self, tree=None):
        if tree is None:
            tree = self.tree
        return self._secs_for(tree)


    def _secs_for(self, tree):
        secs = 0.0
        for kid in tree.kids:
            if isinstance(kid, Group):
                secs += self._secs_for(kid)
            else:
                secs += kid.secs
        return secs


@enum.unique
class _State(enum.Enum):
    WANT_MAGIC = enum.auto()
    WANT_TRACK_HEADER = enum.auto()
    IN_TRACKS = enum.auto()
    IN_BOOKMARKS = enum.auto()
    IN_HISTORY = enum.auto()
    IN_CURRENT = enum.auto()
    END = enum.auto()


class Group:

    def __init__(self, name):
        self.name = name
        self.kids = []


    def __repr__(self):
        return f'Group({self.name!r})'


    def subgroup(self, group_name):
        for kid in self.kids:
            if kid.name == group_name:
                return kid


    def append(self, group_or_track):
        self.kids.append(group_or_track)


class Track:

    def __init__(self, filename, secs):
        self._filename = filename
        self._title = None
        self._secs = secs
        self._album = None
        self._artist = None
        self._number = 0


    def __repr__(self):
        return f'Track({self.filename!r}, {self.secs:0.3f})'


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
    def treename(self):
        return (pathlib.Path(self.filename).stem.replace('-', ' ')
                .replace('_', ' ').lstrip('0123456789 '))


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


def print_tree(tree, indent=0):
    pad = ' ' * indent
    print(f'{pad}{tree!r}')
    for kid in tree.kids:
        if isinstance(kid, Group):
            print_tree(kid, indent + 1)
        else:
            print(f'{pad}{pad}{kid!r}')


if __name__ == '__main__':
    if len(sys.argv) == 1 or sys.argv[1] in {'-h', '--help', 'help'}:
        raise SystemExit(
            'usage: mb.py <-t|--tree> infile.mb | infile.mb outfile.mb')
    filename = sys.argv[1]
    if filename in {'-t', '--tree'}:
        mb = Mb(sys.argv[2])
        for path in mb.paths():
            print(path)
    else:
        infile = pathlib.Path(sys.argv[1]).resolve()
        outfile = pathlib.Path(sys.argv[2]).resolve()
        if infile == outfile:
            raise SystemExit('infile and outfile must be different')
        mb = Mb(infile)
        mb.save(filename=outfile)
        print('saved', outfile)
