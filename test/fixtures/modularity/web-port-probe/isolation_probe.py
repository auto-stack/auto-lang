"""Real two-VM metadata oracle. Failure still drains both owned services."""
from __future__ import annotations

import argparse
import http.client
import json
import os
from pathlib import Path
import socket
import struct
import subprocess
import tempfile
import time


def get(port, path):
    connection = http.client.HTTPConnection("127.0.0.1", port, timeout=2)
    try:
        connection.request("GET", path)
        response = connection.getresponse()
        return response.status, json.loads(response.read())
    finally:
        connection.close()


def owner_resources(port, payload):
    for path, expected in (("/file", payload), ("/events", b"data: 7\n\ndata: 8\n\n")):
        connection = http.client.HTTPConnection("127.0.0.1", port, timeout=3)
        try:
            connection.request("GET", path)
            response = connection.getresponse()
            body = response.read()
            assert response.status == 200 and body == expected, (path, response.status, body)
        finally:
            connection.close()
    # Different second-program signature must not change the network's upload
    # classification. Headers-only Expect must still receive the first handler's
    # rejection without demanding the announced body.
    with socket.create_connection(("127.0.0.1", port), timeout=3) as stream:
        stream.settimeout(3)
        stream.sendall(b"POST /early HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n"
                       b"Expect: 100-continue\r\nContent-Length: 1000000\r\n\r\n")
        first = b""
        while b"\r\n" not in first:
            chunk = stream.recv(4096)
            assert chunk, "upload owner closed before rejection"
            first += chunk
            assert len(first) < 65536
        assert first.startswith(b"HTTP/1.1 418"), first[:200]


def ready(child, port, path, expected):
    deadline = time.monotonic() + 20
    while time.monotonic() < deadline:
        assert child.poll() is None, "owner process exited early"
        try:
            observed = get(port, path)
            if observed == (200, expected):
                return
        except (OSError, ValueError, http.client.HTTPException):
            pass
        time.sleep(0.03)
    raise AssertionError("exact owner identity did not become ready")


def admission_isolation(child, ports):
    connection = http.client.HTTPConnection("127.0.0.1", ports[0], timeout=5)
    try:
        connection.connect()
        connection.sock.setsockopt(socket.SOL_SOCKET, socket.SO_RCVBUF, 4096)
        connection.sock.setsockopt(socket.SOL_SOCKET, socket.SO_LINGER,
                                   struct.pack("HH" if os.name == "nt" else "ii", 1, 0))
        connection.request("GET", "/large")
        response = connection.getresponse()
        assert response.status == 200 and response.read(1) == b"x"
        first_busy = get(ports[0], "/one/7")
        second_free = get(ports[1], "/two/independent")
        print(json.dumps({"scope": "two-owner-permit-pressure", "first_busy": first_busy,
                          "second_free": second_free}), flush=True)
        assert first_busy[0] == 503, "unread body must retain the first owner's permit"
        assert second_free == (200, "second-independent"), "first owner's body exhausted second owner's permit"
    finally:
        connection.close()  # RST exercises actual connection/resource cancellation.
    ready(child, ports[0], "/one/7", 1007)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--exe", required=True, type=Path)
    parser.add_argument("--admission", action="store_true")
    parser.add_argument("--expect-external", action="store_true")
    args = parser.parse_args()
    # Hold both reservations together so the OS cannot select the same port.
    with socket.socket() as first, socket.socket() as second:
        first.bind(("127.0.0.1", 0))
        second.bind(("127.0.0.1", 0))
        ports = first.getsockname()[1], second.getsockname()[1]
    with tempfile.TemporaryDirectory(prefix="p890-isolation-") as temporary:
        root = Path(temporary)
        a, b = root / "first.at", root / "second.at"
        payload = b"owned-file\x00\xff"
        (root / "payload.bin").write_bytes(payload)
        if args.admission:
            # Larger than socket buffers, streamed by the real file executor;
            # a single fixed-size block avoids allocating the whole payload.
            with (root / "large.bin").open("wb") as large:
                for _ in range(64):
                    large.write(b"x" * (1024 * 1024))
        a.write_text('#[api(method = "GET", path = "/one/:n")]\nfn shared(n int) int { n + 1000 }\n'
                     '#[api(method = "GET", path = "/file")]\nfn resource() FileResponse { return http.file_response("'
                     + root.as_posix() + '", "payload.bin", "{}") }\n'
                     '#[api(method = "GET", path = "/large")]\nfn large_file() FileResponse { return http.file_response("'
                     + root.as_posix() + '", "large.bin", "{}") }\n'
                     '#[api(method = "GET", path = "/events")]\nfn events() ~Iter<int> {\n yield 7\n yield 8\n}\n'
                     '#[api(method = "POST", path = "/early")]\nfn early(req UploadRequest) ~UploadReceipt { return http.upload_error(418, "owner-one") }\n', encoding="utf-8")
        b.write_text('#[api(method = "GET", path = "/two/:word")]\nfn shared(word str) str { "second-" + word }\n'
                     '#[api(method = "GET", path = "/other-file")]\nfn resource() str { "second-file" }\n'
                     '#[api(method = "GET", path = "/other-events")]\nfn events() str { "second-events" }\n'
                     '#[api(method = "GET", path = "/other-early")]\nfn early() str { "second-early" }\n', encoding="utf-8")
        with (root / "stdout").open("w+") as stdout, (root / "stderr").open("w+") as stderr:
            child = subprocess.Popen([str(args.exe.resolve()), str(a), str(b), *map(str, ports)],
                                     stdin=subprocess.PIPE, stdout=stdout, stderr=stderr, text=True,
                                     env=dict(os.environ, AUTO_HTTP_MAX_INFLIGHT="1") if args.admission else None)
            after = None
            try:
                ready(child, ports[0], "/one/7", 1007)
                owner_resources(ports[0], payload)
                child.stdin.write("second\n")
                child.stdin.flush()
                ready(child, ports[1], "/two/hello", "second-hello")
                after = get(ports[0], "/one/7")
                owner_resources(ports[0], payload)
                if args.admission:
                    admission_isolation(child, ports)
                child.stdin.write("stop-first\n")
                child.stdin.flush()
                deadline = time.monotonic() + 10
                first_closed = False
                while time.monotonic() < deadline:
                    try:
                        with socket.create_connection(("127.0.0.1", ports[0]), timeout=0.2):
                            pass
                    except OSError:
                        first_closed = True
                        break
                    time.sleep(0.03)
                assert first_closed, "first owner's listener remained active"
                assert get(ports[1], "/two/still-active") == (200, "second-still-active")
            finally:
                if child.poll() is None:
                    child.stdin.write("shutdown\n")
                    child.stdin.flush()
                    try:
                        assert child.wait(timeout=20) == 0, "both owners must drain"
                    except subprocess.TimeoutExpired:
                        child.kill()
                        child.wait(timeout=5)
                        raise
                stdout.seek(0)
                stderr.seek(0)
                output, diagnostics = stdout.read(), stderr.read()
                print(output, end="")
                print(diagnostics, end="")
            assert child.returncode == 0 and "two owners drained" in output
            assert "isolation host=first activated" in diagnostics
            assert "isolation host=second activated" in diagnostics
            if args.expect_external:
                assert diagnostics.count("isolation runtime=external") == 2, diagnostics
            print(json.dumps({"scope": "actual-two-owner-metadata", "first_before": [200, 1007],
                              "second": [200, "second-hello"], "first_after": after,
                              "file_sse_headers_only_upload_preserved": True,
                              "first_shutdown_keeps_second_alive": True, "both_drained": True}))
            assert after == (200, 1007), "later program overwrote active owner's metadata"


if __name__ == "__main__":
    main()
