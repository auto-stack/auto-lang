#!/usr/bin/env python3
# plan897_make_fixtures.py — 生成 %TEMP%/plan897/zipfx.zip（确定性内容）。
# 条目：ascii.txt(deflate,19B) cjk.txt(deflate,39B) bin/logo.png(stored,72B)。
import os, zipfile

base = os.path.join(os.environ.get("TEMP", os.environ.get("TMP", "/tmp")), "plan897")
os.makedirs(base, exist_ok=True)
zp = os.path.join(base, "zipfx.zip")
ascii_data = b"The quick brown fox"
cjk_data = "山月不知心底事，水调歌头。".encode("utf-8")
png_data = bytes([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] + [(i % 251) for i in range(64)])
with zipfile.ZipFile(zp, "w", zipfile.ZIP_DEFLATED) as z:
    z.writestr("ascii.txt", ascii_data)
    z.writestr("cjk.txt", cjk_data)
with zipfile.ZipFile(zp, "a", zipfile.ZIP_STORED) as z:
    z.writestr("bin/logo.png", png_data)
# 非 UTF-8 条目（latin-1 字节）供 Z-04
with zipfile.ZipFile(zp, "a", zipfile.ZIP_DEFLATED) as z:
    z.writestr("latin.txt", b"caf\xe9")
print(zp)
print("sizes:", len(ascii_data), len(cjk_data), len(png_data))
print("png sha256:", hashlib.sha256(png_data).hexdigest())
