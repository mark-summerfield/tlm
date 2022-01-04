#!/bin/bash
unrecognized.py -q
python3 -m flake8 --ignore=E261,E303,W504 *.py
python3 -m vulture *.py \
    | grep -v '60% confidenc'
tokei -f -c80 -tPython,Rust -etarget
git st
