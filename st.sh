#!/bin/bash
unrecognized.py -q
python3 -m flake8 --ignore=E261,E303,W504 *.py
python3 -m vulture *.py \
    | grep -v '60% confidenc'
tokei -f -slines -c80 -tPython,Rust -etarget
grep --color=auto --exclude-dir=target --include=*.rs -r format......,
git st
