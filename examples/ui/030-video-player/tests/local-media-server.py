# -*- coding: utf-8 -*-
"""PLAN-617 T-06 (dev harness): serve the real media root over HTTP.

WHY THIS EXISTS
---------------
The plan's T-05 puts scan + range streaming in the generated Rust backend. That
backend cannot be *built* from the plan-617 worktree yet: `crates/auto-lang`
declares an optional cross-repo path dep
(`autodown-core = path = "../../../auto-down/..."`) which resolves from the main
checkout but not from a worktree, and a full `cargo build -p auto` is a long
haul. This harness supplies the same two endpoints from the same media root so
the FRONTEND integration (T-06) can be built and verified now; T-05 replaces the
server side while keeping the JSON shape and the range semantics identical.

ENDPOINTS
---------
  GET /scan          -> JSON array of {id,title,file_name,rel_dir,size_str,
                        bytes,ext,video_url} (recursive, whitelist-filtered)
  GET /stream/<id>   -> 206 partial content, Accept-Ranges, Content-Range
  GET /health        -> plain OK

Static defaults only exist for a byte-range implementation; there is no
dependency on anything outside the standard library.
"""
import json
import os
import re
import sys
import urllib.parse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

ROOT = os.environ.get("AUTO_MEDIA_ROOT", r"E:\Video")
PORT = int(os.environ.get("AUTO_MEDIA_PORT", "8099"))
EXTS = {"mp4", "m4v", "webm", "mkv", "mov", "avi"}


def human(n):
    for unit in ("B", "KB", "MB", "GB", "TB"):
        if n < 1024 or unit == "TB":
            return ("%d %s" % (n, unit)) if unit == "B" else ("%.1f %s" % (n, unit))
        n /= 1024.0


def index_root():
    """Recursive scan. Deliberately does NOT follow symlinks/junctions."""
    out = []
    for dirpath, dirnames, filenames in os.walk(ROOT, followlinks=False):
        for fn in filenames:
            ext = fn.rsplit(".", 1)[-1].lower() if "." in fn else ""
            if ext not in EXTS:
                continue
            full = os.path.join(dirpath, fn)
            try:
                st = os.lstat(full)
            except OSError:
                continue
            if os.path.islink(full):
                continue
            rel = os.path.relpath(full, ROOT).replace("\\", "/")
            rel_dir = os.path.dirname(rel).replace("\\", "/")
            out.append({
                "rel_path": rel,
                "rel_dir": rel_dir,
                "file_name": fn,
                "title": re.sub(r"\.[^.]+$", "", fn),
                "bytes": st.st_size,
                "size_str": human(st.st_size),
                "ext": ext,
            })
    # natural sort: S01E02 before S01E10
    def key(e):
        return [int(p) if p.isdigit() else p.lower()
                for p in re.split(r"(\d+)", e["rel_path"])]
    out.sort(key=key)
    for i, e in enumerate(out, 1):
        e["id"] = i
        e["video_url"] = "http://127.0.0.1:%d/stream/%d" % (PORT, i)
    return out


ENTRIES = index_root()
BY_ID = {e["id"]: e for e in ENTRIES}


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def _cors(self):
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Headers", "Range, Content-Type")
        self.send_header("Access-Control-Expose-Headers", "Content-Range, Accept-Ranges")

    def log_message(self, *a):
        pass

    def do_OPTIONS(self):
        self.send_response(204)
        self._cors()
        self.send_header("Content-Length", "0")
        self.end_headers()

    def do_GET(self):
        path = urllib.parse.urlparse(self.path).path
        if path == "/health":
            body = b"OK"
            self.send_response(200)
            self.send_header("Content-Type", "text/plain")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if path == "/scan":
            body = json.dumps(ENTRIES).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self._cors()
            self.end_headers()
            self.wfile.write(body)
            return
        if path.startswith("/stream/"):
            self._stream(path.rsplit("/", 1)[-1])
            return
        self.send_response(404)
        self.send_header("Content-Length", "0")
        self.end_headers()

    def _stream(self, sid):
        try:
            entry = BY_ID[int(sid)]
        except (ValueError, KeyError):
            self.send_response(404)
            self.send_header("Content-Length", "0")
            self.end_headers()
            return
        full = os.path.join(ROOT, entry["rel_path"].replace("/", os.sep))
        if not os.path.isfile(full):
            self.send_response(404)
            self.send_header("Content-Length", "0")
            self.end_headers()
            return
        size = os.path.getsize(full)
        ctype = {"mp4": "video/mp4", "m4v": "video/mp4", "webm": "video/webm",
                 "mkv": "video/x-matroska", "mov": "video/quicktime",
                 "avi": "video/x-msvideo"}.get(entry["ext"], "application/octet-stream")
        rng = self.headers.get("Range")
        start, end = 0, size - 1
        status = 200
        if rng:
            m = re.match(r"bytes=(\d*)-(\d*)", rng.strip())
            if m:
                g1, g2 = m.group(1), m.group(2)
                if g1:
                    start = int(g1)
                    end = int(g2) if g2 else size - 1
                elif g2:                      # suffix form: bytes=-N
                    start = max(0, size - int(g2))
                if start >= size or start > end:
                    self.send_response(416)
                    self.send_header("Content-Range", "bytes */%d" % size)
                    self.send_header("Content-Length", "0")
                    self._cors()
                    self.end_headers()
                    return
                end = min(end, size - 1)
                status = 206
        length = end - start + 1
        self.send_response(status)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(length))
        self.send_header("Accept-Ranges", "bytes")
        if status == 206:
            self.send_header("Content-Range", "bytes %d-%d/%d" % (start, end, size))
        self._cors()
        self.end_headers()
        remaining = length
        with open(full, "rb") as f:
            f.seek(start)
            while remaining > 0:
                chunk = f.read(min(262144, remaining))
                if not chunk:
                    break
                try:
                    self.wfile.write(chunk)
                except (BrokenPipeError, ConnectionResetError):
                    return
                remaining -= len(chunk)


if __name__ == "__main__":
    if not os.path.isdir(ROOT):
        print("media root missing: %s" % ROOT, file=sys.stderr)
        sys.exit(2)
    srv = ThreadingHTTPServer(("127.0.0.1", PORT), Handler)
    print("media root : %s" % ROOT)
    print("indexed    : %d file(s)" % len(ENTRIES))
    print("listening  : http://127.0.0.1:%d/ (scan, stream/<id>, health)" % PORT)
    sys.stdout.flush()
    srv.serve_forever()
