#!/usr/bin/env python3
"""Pack icon-<size>.png files into a PNG-compressed .ico (Windows Vista and later).
usage: make_ico.py <dir-with-pngs> <out.ico>"""
import struct
import sys

SIZES = [16, 32, 48, 256]
src, dst = sys.argv[1], sys.argv[2]
images = [(s, open(f"{src}/icon-{s}.png", "rb").read()) for s in SIZES]
out = bytearray(struct.pack("<HHH", 0, 1, len(images)))
offset = 6 + 16 * len(images)
for size, data in images:
    dim = 0 if size >= 256 else size  # 0 means 256
    out += struct.pack("<BBBBHHII", dim, dim, 0, 0, 1, 32, len(data), offset)
    offset += len(data)
for _, data in images:
    out += data
open(dst, "wb").write(out)
print(f"wrote {dst}: {len(out)} bytes, sizes {SIZES}")
