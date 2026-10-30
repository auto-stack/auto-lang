"""Bounded real-TCP replay of original file and SSE handler programs.

This exercises the owned reply seam, not the full original test inventories.
"""
from __future__ import annotations

import argparse
import hashlib
import http.client
import json
import os
from pathlib import Path
import re
import socket
import subprocess
import tempfile
import time
import uuid


def request(port, method, path, headers=None):
    connection = http.client.HTTPConnection("127.0.0.1", port, timeout=10)
    try:
        connection.request(method, path, headers=headers or {})
        response = connection.getresponse()
        return response.status, dict(response.getheaders()), response.read()
    finally:
        connection.close()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--exe", required=True, type=Path)
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[4]
    file_source = (repo / "crates/auto-lang/src/tests/plan729_http_server_file_tests.rs").read_bytes()
    sse_source = (repo / "crates/auto-lang-core/src/vm/ffi/http_server.rs").read_bytes()
    file_match = re.search(r'fn program\(root: &str\).*?format!\(\s*r#"(.*?)"#', file_source.decode(), re.S)
    sse_match = re.search(r'fn e2e_sse_generator_handler\(\).*?start_server\(\s*r#"(.*?)"#', sse_source.decode(), re.S)
    if not file_match or not sse_match:
        raise ValueError("original handler shape changed; re-audit rather than substitute")
    with socket.socket() as reservation:
        reservation.bind(("127.0.0.1", 0))
        port = reservation.getsockname()[1]
    token = "p890" + uuid.uuid4().hex
    with tempfile.TemporaryDirectory(prefix="p890-response-") as temporary:
        root = Path(temporary)
        public = root / "public"
        public.mkdir()
        payload = bytes(range(256)) * 1024
        (public / "raw.bin").write_bytes(payload)
        program = (file_match[1].replace("{root}", public.as_posix())
                   .replace("{{", "{").replace("}}", "}"))
        program += sse_match[1]
        program += f'\n#[api(method = "GET", path = "/{token}")]\nfn marker() str {{ "{token}" }}\n'
        source = root / "api.at"
        source.write_text(program, encoding="utf-8")
        env = dict(os.environ, AUTO_HTTP_PORT=str(port), AUTO_EXECUTION_ENGINE="autovm")
        with (root / "stdout").open("w+") as stdout, (root / "stderr").open("w+") as stderr:
            child = subprocess.Popen([str(args.exe.resolve()), str(source), "--install-adapter"],
                                     env=env, stdin=subprocess.PIPE, stdout=stdout, stderr=stderr, text=True)
            try:
                deadline = time.monotonic() + 30
                ready = False
                while time.monotonic() < deadline:
                    if child.poll() is not None:
                        raise AssertionError("SDK exited before response probe became ready")
                    try:
                        status, _, body = request(port, "GET", f"/{token}")
                        ready = status == 200 and json.loads(body) == token
                        if ready:
                            break
                    except (OSError, ValueError, http.client.HTTPException):
                        pass
                    time.sleep(0.05)
                assert ready, "exact service identity not ready"
                for path in ("/files/raw.bin", "/async/raw.bin"):
                    status, _, body = request(port, "GET", path)
                    assert status == 200 and body == payload, (path, status, len(body))
                status, headers, body = request(port, "HEAD", "/files/raw.bin")
                assert status == 200 and not body
                assert headers.get("content-length", headers.get("Content-Length")) == str(len(payload)), headers
                status, _, body = request(port, "GET", "/files/raw.bin", {"Range": "bytes=2-9"})
                assert status == 206 and body == payload[2:10], (status, body)
                status, _, body = request(port, "GET", "/etag/raw.bin", {"If-None-Match": "v1"})
                assert status == 304 and not body, (status, body)
                status, _, _ = request(port, "GET", "/files/absent.bin")
                assert status == 404, status
                status, _, _ = request(port, "GET", "/badopts/raw.bin")
                assert status == 500, status
                status, headers, body = request(port, "GET", "/api/counter")
                assert status == 200, status
                assert any(k.lower() == "content-type" and v.startswith("text/event-stream")
                           for k, v in headers.items()), headers
                assert [line[5:].strip() for line in body.splitlines() if line.startswith(b"data:")] == [b"1", b"2", b"3"], body
                assert child.stdin is not None
                child.stdin.write("shutdown\n")
                child.stdin.flush()
                assert child.wait(timeout=20) == 0, "graceful owner return required"
                stderr.seek(0)
                diagnostics = stderr.read()
                assert "probe shutdown delivered=true" in diagnostics
                assert "probe terminal scopes=0 files=0 uploads=(0, 0, 0) streams=0 live_ops=0" in diagnostics, diagnostics
                print(json.dumps({"scope": "owned-reply-prototype", "sync_async_file_exact": True,
                                  "head_range_etag_errors": True, "sse_ordered_eof": [1, 2, 3],
                                  "child_exit": child.returncode,
                                  "file_source_sha256": hashlib.sha256(file_source).hexdigest(),
                                  "sse_source_sha256": hashlib.sha256(sse_source).hexdigest()}))
            finally:
                if child.poll() is None:
                    child.kill()
                    child.wait(timeout=5)
                stdout.seek(0)
                stderr.seek(0)
                print(stdout.read(), end="")
                print(stderr.read(), end="")


if __name__ == "__main__":
    main()
