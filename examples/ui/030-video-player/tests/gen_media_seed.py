# -*- coding: utf-8 -*-
"""PLAN-617 T-06 (dev harness): replace the mock playlist with the real index.

Reads the media server's /scan output and rewrites three places in app.at:

  1. the `var playlist = [...]` literal  -> the real 14 entries
  2. the `.SelectVideo` handler          -> a generic lookup instead of the
                                            per-id hardcoded blocks
  3. the `<video>` element               -> gains native `controls` so the clip
                                            is actually playable *interim*
                                            (T-07/T-08 replace this with the
                                            controlled contract + custom OSD)

Everything not listed above is left byte-identical.
"""
import io
import json
import re
import sys
import urllib.request

APP = (r"D:\autostack\.wt\lang-617\auto-lang\examples\ui"
       r"\030-video-player\src\front\app.at")
SCAN = "http://127.0.0.1:8099/scan"

entries = json.load(urllib.request.urlopen(SCAN, timeout=10))
print("scan entries:", len(entries))

# ---- 1. playlist literal ---------------------------------------------------
def dq(s):
    """`.at` has no \\u escapes and reads single quotes as CHAR literals, so every
    string must be double-quoted with only backslash/quote escaped. (Python's !r
    emits single quotes - that is what broke the first bake.)"""
    return u'"' + s.replace(u"\\", u"\\\\").replace(u'"', u'\\"') + u'"'


items = []
for e in entries:
    items.append(
        u"            {\n"
        u"                id: %d,\n"
        u"                title: %s,\n"
        u"                file_name: %s,\n"
        u"                rel_dir: %s,\n"
        u"                size_str: %s,\n"
        u"                video_url: %s\n"
        u"            }" % (
            e["id"],
            dq(e["title"]),
            dq(e["file_name"]),
            dq(e["rel_dir"]),
            dq(e["size_str"]),
            dq(e["video_url"]),
        )
    )
new_literal = (u"        // ---- 播放列表（由 E:\\Video 递归扫描真实得到；T-06）----\n"
               u"        // 生成物：tests/gen_media_seed.py（T-05 落地后改为运行时扫描）\n"
               u"        var playlist = [\n"
               + u",\n".join(items) + u"\n        ]\n")

src = io.open(APP, encoding="utf-8").read()
lines = src.split(u"\n")

start = next(i for i, l in enumerate(lines) if l.strip() == u"var playlist = [")
end = next(i for i in range(start + 1, len(lines)) if lines[i] == u"        ]")
print("playlist literal: lines %d..%d -> %d new lines"
      % (start + 1, end + 1, len(new_literal.split(u"\n")) - 1))
lines = lines[:start] + new_literal.rstrip(u"\n").split(u"\n") + lines[end + 1:]
src = u"\n".join(lines)

# ---- 2. generic SelectVideo ------------------------------------------------
sv_start = next(i for i, l in enumerate(lines)
                if l.strip() == u".SelectVideo(id int) -> {")
sv_end = next(i for i in range(sv_start + 1, len(lines)) if lines[i] == u"        }")
new_sv = u"""        .SelectVideo(id int) -> {
            .current_id = id
            // 从真实队列按 id 取项（不再按 id 硬编码内容）
            for item in .playlist {
                if item.id == id {
                    .current_title = item.title
                    .current_file = item.file_name
                    .current_video_url = item.video_url
                    .current_size = item.size_str
                }
            }
            // 时长/分辨率/编码在未解复用前无从得知：一律留空标注，交由
            // <video> 的 loadedmetadata 回灌（T-08），不再编造。
            .current_res = "—"
            .current_codec = "—"
            .current_time_sec = 0
            .current_time_str = "00:00"
            .total_time_sec = 0
            .total_time_str = "00:00"
            .time_display_fmt = "00:00 / 00:00"
            .progress_pct = 0
            .is_playing = true
            .toast_msg = "开始播放: " + .current_file
            .show_toast = true
        }"""
print("SelectVideo: lines %d..%d -> 1 block" % (sv_start + 1, sv_end + 1))
lines = lines[:sv_start] + new_sv.split(u"\n") + lines[sv_end + 1:]

# ---- 3. interim native controls on the video element -----------------------
for i, l in enumerate(lines):
    if l.strip() == u"src: .current_video_url":
        if i + 1 < len(lines) and lines[i + 1].strip() == u"controls: true":
            print("video: controls already present (idempotent skip)")
        else:
            lines.insert(i + 1, u"                        controls: true")
            print("video: added `controls: true` at line", i + 2)
        break
else:
    print("ERROR: video src line not found", file=sys.stderr)
    sys.exit(1)

# ---- 4. honest default src (first real entry) -----------------------------
first_url = entries[0]["video_url"]
for i, l in enumerate(lines):
    if l.strip().startswith(u'var current_video_url str ='):
        lines[i] = u'        var current_video_url str = "%s"' % first_url
        print("default src ->", first_url)
        break

io.open(APP, "w", encoding="utf-8", newline="\n").write(u"\n".join(lines))
print("app.at rewritten; lines =", len(lines))
