#!/usr/bin/env python3
# Copyright © 2021 Mark Summerfield. All rights reserved.
# License: GPLv3

import enum
import gzip
import os
import sys

import mutagen


def main():
    if len(sys.argv) == 1 or sys.argv[1] in {'-h', '--help', 'help'}:
        raise SystemExit('usage: m3u2mb.py <dirname>')
    folder = sys.argv[1]
    create_mb(folder, os.path.basename(folder) + '.mb')


def create_mb(folder, mb):
    pairs = []
    groups = set()
    for root, _, files in os.walk(folder):
        for filename in files:
            if not filename.upper().endswith('.M3U'):
                continue
            group = os.path.basename(root)
            if group.upper() == 'PLAYLISTS':
                group = 'Ungrouped'
            groups.add(group)
            fullname = os.path.join(root, filename)
            pairs.append((group, fullname))
    pairs.sort(key=lambda p: (p[0].upper(), p[1].upper()))
    with gzip.open(mb, 'wt', encoding='utf-8') as file:
        file.write('*MB>100\n*G\n')
        for group in sorted(groups, key=str.upper):
            file.write(f'{group}\n')
        file.write('*C\n*H\n') # no current; empty history
        for group, m3u in pairs:
            name = os.path.splitext(os.path.basename(m3u))[0]
            track_list = []
            for track in tracks(m3u):
                try:
                    meta = mutagen.File(track)
                    secs = meta.info.length
                except (mutagen.MutagenError, FileNotFoundError):
                    continue
                track_list.append((track, secs))
            if track_list:
                file.write(f'*P>{group}>{name}\n')
                for track, secs in track_list:
                    file.write(f'{track}>{secs:.01f}\n')
    print('wrote', mb)


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
