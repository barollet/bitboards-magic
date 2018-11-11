#!/usr/bin/env python3

import heapq

from os import listdir
from os.path import isfile, join

bishop_files = [f for f in listdir(".") if isfile(join(".", f)) and "b_" in f]
rook_files = [f for f in listdir(".") if isfile(join(".", f)) and "r_" in f]

for f in bishop_files + rook_files:
    with open(f) as content:
        lines = heapq.nsmallest(20, content.readlines(), key=lambda e: int(e.split()[2]))

    with open(f, "w") as content:
        for line in lines:
            content.write(line)

