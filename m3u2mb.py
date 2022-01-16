#!/usr/bin/env python3
# Copyright © 2021 Mark Summerfield. All rights reserved.
# License: GPLv3

import enum
import gzip
import os
import sys

import mutagen


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
        self.filename = filename
        self.secs = secs
        self.pgid = pgid


    def __repr__(self):
        return (f'Track({self.tid}, {self.filename}, {self.secs:0.3f}, '
                f'{self.pgid})')


    def __lt__(self, track):
        sname = self.filename.upper()
        tname = track.filename.upper()
        if sname != tname:
            return sname < tname
        return self.tid < track.tid


def main():
    if len(sys.argv) == 1 or sys.argv[1] in {'-h', '--help', 'help'}:
        raise SystemExit('usage: m3u2mb.py <dirname>')
    folder = sys.argv[1]
    tracks, groups = read_folder(folder)
    write_mb(tracks, groups, os.path.basename(folder) + '.mb')


def read_folder(folder):
    track_list = []
    groups = dict(playlists=Group(0, '<top-level>', 1))
    gid = 0
    tid = 9999
    for root, _, files in os.walk(folder):
        groupname = os.path.basename(root)
        if groupname not in groups:
            gid += 1
            groups[groupname] = Group(gid, groupname)
        for filename in files:
            if not filename.upper().endswith('.M3U'):
                continue
            subgroupname = filename[:-4]
            if subgroupname not in groups:
                gid += 1
                groups[subgroupname] = Group(gid, subgroupname,
                                             groups[groupname].gid)
            fullname = os.path.join(root, filename)
            for track in tracks(fullname):
                try:
                    meta = mutagen.File(track)
                    secs = meta.info.length
                except (mutagen.MutagenError, FileNotFoundError):
                    continue
                tid += 1
                track_list.append(Track(tid, track, secs, gid))
    return track_list, groups.values()


def write_mb(tracks, groups, mb):
    with gzip.open(mb, 'wt', encoding='utf-8') as file:
        file.write('\fMB\t100\n')
        file.write('\fGROUPS\n\vGID\tNAME\tPGID\n')
        for group in groups:
            file.write(f'{group.gid}\t{group.name}\t{group.pgid}\n')
        file.write('\fTRACKS\n\vTID\tFILENAME\tSECS\tPGID\n')
        for track in tracks:
            file.write(f'{track.tid}\t{track.filename}\t{track.secs:.03f}'
                       f'\t{track.pgid}\n')
        file.write(
            '\fBBOOKMARKS\n\vTID\n\fHISTORY\n\vTID\n\fCURRENT\n\vTID\n')
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
