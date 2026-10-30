"""PLAN-890 bounded upload seam replay using the existing PLAN-730 program.

This exercises the changed owned-body path; it does not claim that the whole
PLAN-730 suite, or the new explicit host adapter, has been delivered.
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


def request(port, method, path, body=None, headers=None):
    connection = http.client.HTTPConnection("127.0.0.1", port, timeout=20)
    try:
        connection.request(method, path, body=body, headers=headers or {})
        response = connection.getresponse()
        return response.status, response.read()
    finally:
        connection.close()


def multipart(boundary: str, note: str, data: bytes) -> bytes:
    return (
        f'--{boundary}\r\nContent-Disposition: form-data; name="note"\r\n\r\n{note}\r\n'
        f'--{boundary}\r\nContent-Disposition: form-data; name="file"; filename="blob.bin"\r\n\r\n'
    ).encode() + data + f"\r\n--{boundary}--\r\n".encode()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--exe", required=True, type=Path)
    args = parser.parse_args()
    repository = Path(__file__).resolve().parents[4]
    original = repository / "crates/auto-lang/src/tests/plan730_http_upload_tests.rs"
    original_bytes = original.read_bytes()
    match = re.search(r'fn upload_program\(.*?let mut src = format!\(\s*r#"(.*?)"#',
                      original_bytes.decode(), re.S)
    if match is None:
        raise ValueError("existing PLAN-730 fixture shape changed; re-audit instead of substituting an input")
    token = "p890" + uuid.uuid4().hex
    with socket.socket() as reservation:
        reservation.bind(("127.0.0.1", 0))
        port = reservation.getsockname()[1]
    with tempfile.TemporaryDirectory(prefix="p890-upload-") as temporary:
        root = Path(temporary)
        public, staging = root / "public", root / "private-staging"
        (public / "mp").mkdir(parents=True)
        (public / "raw").mkdir()
        staging.mkdir()
        program = (match[1].replace("{root_s}", public.as_posix())
                   .replace("{staging_s}", staging.as_posix()).replace("{{", "{").replace("}}", "}"))
        # This extra fixture-only marker proves listener identity before sending
        # upload data; the reused upload routes/handlers remain unchanged.
        program += f'\n#[api(method = "GET", path = "/{token}")]\nfn marker() str {{ "{token}" }}\n'
        source = root / "api.at"
        source.write_text(program, encoding="utf-8")
        env = dict(os.environ, AUTO_HTTP_PORT=str(port), AUTO_EXECUTION_ENGINE="autovm")
        with (root / "stdout.txt").open("w+") as stdout, (root / "stderr.txt").open("w+") as stderr:
            child = subprocess.Popen([str(args.exe.resolve()), str(source)], env=env,
                                     stdin=subprocess.PIPE, stdout=stdout, stderr=stderr, text=True)
            try:
                ready = False
                deadline = time.monotonic() + 30
                while time.monotonic() < deadline:
                    if child.poll() is not None:
                        raise AssertionError("SDK exited before startup")
                    try:
                        status, body = request(port, "GET", f"/{token}")
                        ready = status == 200 and json.loads(body) == token
                        if ready:
                            break
                    except (OSError, http.client.HTTPException, ValueError):
                        pass
                    time.sleep(0.05)
                assert ready, "exact service identity did not become ready"

                # Original accepted case keeps its 12 MiB input (> ordinary
                # 10 MiB buffered-body limit), rather than shrinking the oracle.
                payload = bytes(range(253)) * (12 * 1024 * 1024 // 253 + 1)
                payload = payload[:12 * 1024 * 1024]
                status, body = request(port, "POST", "/api/uploads/mp", multipart("P890BOUND", "hello", payload),
                                       {"Content-Type": "multipart/form-data; boundary=P890BOUND"})
                receipt = json.loads(body)
                assert status == 201 and receipt["ok"] is True, (status, receipt)
                assert int(receipt["size"]) == len(payload), receipt
                assert receipt["path"] == "mp/blob.bin", receipt
                assert (public / "mp/blob.bin").read_bytes() == payload
                status, downloaded = request(port, "GET", "/api/files/mp/blob.bin")
                assert status == 200 and downloaded == payload
                assert not list(staging.iterdir()), "staging retained after commit"

                status, body = request(port, "POST", "/api/uploads/mp", multipart("P890BOUND", "again", b"second"),
                                       {"Content-Type": "multipart/form-data; boundary=P890BOUND"})
                assert status == 409, (status, body)
                assert (public / "mp/blob.bin").read_bytes() == payload, "conflict changed published data"

                status, body = request(port, "POST", "/api/uploads/mp", multipart("P890BOUND", "deny", b"reject"),
                                       {"Content-Type": "multipart/form-data; boundary=P890BOUND"})
                assert status == 422, (status, body)
                assert not list(staging.iterdir()), "business rejection retained staging"

                # Send headers only: accepting 401 must not require/poll the
                # advertised body or issue a 100 Continue before authorization.
                with socket.create_connection(("127.0.0.1", port), timeout=5) as stream:
                    stream.settimeout(5)
                    stream.sendall(b"POST /api/early HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n"
                                   b"Content-Type: multipart/form-data; boundary=B\r\n"
                                   b"Expect: 100-continue\r\nContent-Length: 1000000\r\n\r\n")
                    first = b""
                    while b"\r\n" not in first:
                        chunk = stream.recv(4096)
                        assert chunk, "connection closed before authorization reply"
                        first += chunk
                        assert len(first) <= 65536, "authorization reply header exceeds its bound"
                    assert first.startswith(b"HTTP/1.1 401"), first[:200]
                assert not list(staging.iterdir()), "early authorization rejection created staging"

                assert child.stdin is not None
                child.stdin.write("shutdown\n")
                child.stdin.flush()
                assert child.wait(timeout=20) == 0
                stderr.seek(0)
                assert "probe shutdown delivered=true" in stderr.read()
                print(json.dumps({"scope": "owned-upload-seam", "multipart_bytes": len(payload),
                                  "download_exact": True, "conflict_preserved": True,
                                  "business_reject": 422, "header_only_reject": 401,
                                  "staging_empty": True, "shutdown_exit": 0,
                                  "original_fixture_sha256": hashlib.sha256(original_bytes).hexdigest()}))
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
