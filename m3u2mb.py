#!/usr/bin/env python3
# Copyright © 2021 Mark Summerfield. All rights reserved.
# License: GPLv3

import enum
import gzip
import os
import pprint
import sys

import mutagen


class Group:

    def __init__(self, name):
        self.name = name
        self.kids = []


    def __repr__(self):
        return f'Group({self.name})#{len(self.kids)}'


    def subgroup(self, group_name):
        for kid in self.kids:
            if kid.name == group_name:
                return kid


    def append(self, group_or_track):
        self.kids.append(group_or_track)


class Track:

    def __init__(self, filename, secs):
        self.filename = filename
        self.secs = secs


    def __repr__(self):
        return f'Track({self.filename}, {self.secs:0.3f})'


def main():
    if len(sys.argv) == 1 or sys.argv[1] in {'-h', '--help', 'help'}:
        raise SystemExit('usage: m3u2mb.py <dirname>')
    folder = sys.argv[1]
    tree = read_folder(folder)
    write_mb(tree, os.path.basename(folder) + '.mb')


def read_folder(folder):
    group = tree = Group('') # top-level
    subgroup = None
    for root, _, files in os.walk(folder):
        groupname = os.path.basename(root)
        if groupname == 'playlists':
            group = tree
            subgroup = None
        else:
            subgroup = group.subgroup(groupname)
            if subgroup is None:
                subgroup = Group(groupname)
                group.append(subgroup)
        for filename in files:
            if not filename.upper().endswith('.M3U'):
                continue
            subsubgroupname = filename[:-4]
            if subgroup is None:
                subgroup = group
            subsubgroup = subgroup.subgroup(subsubgroupname)
            if subsubgroup is None:
                subsubgroup = Group(subsubgroupname)
                subgroup.append(subsubgroup)
            fullname = os.path.join(root, filename)
            for track in tracks(fullname):
                try:
                    meta = mutagen.File(track)
                    secs = meta.info.length
                except (mutagen.MutagenError, FileNotFoundError):
                    continue
                subsubgroup.append(Track(track, secs))
    return tree


def write_mb(tree, mb):
    with gzip.open(mb, 'wt', encoding='utf-8') as file:
        file.write('\fMB\t100\n\fTRACKS\n')
        write_tree(file, tree)
        file.write('\fBOOKMARKS\n\fHISTORY\n\fCURRENT\n')
    print('wrote', mb)


def write_tree(file, tree, depth=1):
    pad = depth * '>'
    for item in tree.kids:
        if isinstance(item, Group):
            file.write(f'{pad}{item.name}\n')
            write_tree(file, item, depth + 1)
        else:
            file.write(f'{item.filename}\t{item.secs:.03f}\n')


class Error(Exception):
    pass


def tracks(m3u): # Copied & modified from PLE
    '''
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
    '''
    M3U_EXTM3U = '#EXTM3U'
    M3U_EXTINF = '#EXTINF:'

    class Want(enum.Enum):
        M3U = enum.auto()
        INFO = enum.auto()
        FILENAME = enum.auto()

    state = Want.M3U
    with open(m3u, 'rt', encoding='utf-8') as file:
        for lino, line in enumerate(file, 1):
            line = line.strip()
            if not line:
                continue # ignore blank lines
            if state is Want.M3U:
                if line != M3U_EXTM3U:
                    raise Error(f'{lino}:invalid M3U header: {line!r}')
                state = Want.INFO
            elif state is Want.INFO:
                if not line.startswith(M3U_EXTINF):
                    raise Error(
                        f'{lino}:invalid {M3U_EXTINF} line: {line!r}')
                state = Want.FILENAME
            elif state is Want.FILENAME:
                if line.startswith(M3U_EXTINF):
                    raise Error(f'{lino}:unexpected {M3U_EXTINF} '
                                f'line: {line!r}')
                if line:
                    yield line
                state = Want.INFO


if __name__ == '__main__':
    main()
