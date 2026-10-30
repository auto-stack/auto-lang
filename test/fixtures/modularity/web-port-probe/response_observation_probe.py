"""Real file/SSE terminal events, bytes, and independent owner counters."""
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
import uuid


def request(port, path, method="GET", headers=None):
    conn = http.client.HTTPConnection("127.0.0.1", port, timeout=5)
    try:
        conn.request(method, path, headers=headers or {})
        response = conn.getresponse()
        return response.status, {k.lower(): v for k, v in response.getheaders()}, response.read()
    finally:
        conn.close()


def ready(child, port, token):
    deadline = time.monotonic() + 20
    while time.monotonic() < deadline:
        assert child.poll() is None, "owner exited before ready"
        try:
            status, _, body = request(port, "/value")
            if status == 200 and json.loads(body) == token:
                return
        except (OSError, http.client.HTTPException):
            pass
        time.sleep(0.03)
    raise AssertionError("exact owner not ready")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--exe", required=True, type=Path)
    args = parser.parse_args()
    scratch = Path(__file__).resolve().parent / "target" / "response-observation-runs"
    scratch.mkdir(parents=True, exist_ok=True)
    with socket.socket() as a, socket.socket() as b:
        a.bind(("127.0.0.1", 0))
        b.bind(("127.0.0.1", 0))
        ports = a.getsockname()[1], b.getsockname()[1]
    with tempfile.TemporaryDirectory(dir=scratch) as temporary:
        root = Path(temporary)
        payload = (bytes(range(251)) * 523)[:131071]
        (root / "payload.bin").write_bytes(payload)
        (root / "large.bin").write_bytes(b"x" * (16 * 1024 * 1024))
        (root / "truncate.bin").write_bytes(b"y" * (16 * 1024 * 1024))
        sources, configs = [], []
        tokens = [uuid.uuid4().hex, uuid.uuid4().hex]
        for index in range(2):
            source, config = root / f"{index}.at", root / f"{index}.json"
            text = '#[api(method = "GET", path = "/value")]\nfn value() str { "' + tokens[index] + '" }\n'
            if index == 0:
                for route, filename in (("file", "payload.bin"), ("missing", "absent.bin"), ("large", "large.bin"), ("truncated", "truncate.bin"), ("timeout", "large.bin")):
                    text += '#[api(method = "GET", path = "/' + route + '")]\nfn ' + route + '() FileResponse { return http.file_response("' + root.as_posix() + '", "' + filename + '", "{\\"etag\\":\\"p890\\"}") }\n'
                text += '#[api(method = "GET", path = "/events")]\nfn events() ~Iter<int> {\n yield 7\n yield 8\n}\n'
                text += '#[api(method = "GET", path = "/forever")]\nfn forever() ~Iter<int> {\n loop { yield 9 }\n}\n'
            source.write_text(text, encoding="utf-8")
            config.write_text(json.dumps({"profile": "development", "listen": {"port": ports[index]}, "allowed_hosts": ["127.0.0.1"], "observability": {"log_sink_capacity": 128}, "limits": {"drain_timeout_ms": 1000}}), encoding="utf-8")
            sources.append(source)
            configs.append(config)
        expected = []
        with (root / "stdout").open("w+") as stdout, (root / "stderr").open("w+") as stderr:
            child = subprocess.Popen([str(args.exe.resolve()), *map(str, sources), *map(str, ports), *map(str, configs)], stdin=subprocess.PIPE, stdout=stdout, stderr=stderr, text=True, env=dict(os.environ, AUTO_EXECUTION_ENGINE="autovm", AUTO_HTTP_FILE_IDLE_MS="200"))
            second_sent = False
            try:
                ready(child, ports[0], tokens[0])
                child.stdin.write("second\n")
                child.stdin.flush()
                second_sent = True
                ready(child, ports[1], tokens[1])
                before = json.loads(request(ports[1], "/__auto/health/snapshot")[2])["counters"]
                status, headers, body = request(ports[0], "/file")
                assert status == 200 and body == payload
                assert "access-control-allow-origin" not in headers, "configured file response leaked legacy CORS"
                expected.append(("GET", "/file", 200, len(payload), "completed"))
                for method, extra, status_expected, body_expected in (
                    ("GET", {"Range": "bytes=3-9"}, 206, payload[3:10]),
                    ("HEAD", {}, 200, b""),
                    ("GET", {"If-None-Match": headers["etag"]}, 304, b""),
                    ("GET", {"Range": "bytes=999999-"}, 416, b""),
                ):
                    status, _, body = request(ports[0], "/file", method, extra)
                    assert status == status_expected and body == body_expected, (status, body)
                    expected.append((method, "/file", status, len(body), "completed"))
                status, _, body = request(ports[0], "/missing")
                assert status == 404
                expected.append(("GET", "/missing", 404, len(body), "completed"))
                status, _, body = request(ports[0], "/events")
                assert status == 200 and body == b"data: 7\n\ndata: 8\n\n"
                expected.append(("GET", "/events", 200, len(body), "completed"))
                for route in ("large", "forever"):
                    conn = http.client.HTTPConnection("127.0.0.1", ports[0], timeout=5)
                    conn.connect()
                    conn.sock.setsockopt(socket.SOL_SOCKET, socket.SO_RCVBUF, 4096)
                    conn.sock.setsockopt(socket.SOL_SOCKET, socket.SO_LINGER, struct.pack("HH" if os.name == "nt" else "ii", 1, 0))
                    conn.request("GET", "/" + route)
                    response = conn.getresponse()
                    assert response.status == 200 and response.read(1)
                    conn.close()
                broken = http.client.HTTPConnection("127.0.0.1", ports[0], timeout=5)
                try:
                    broken.connect()
                    broken.sock.setsockopt(socket.SOL_SOCKET, socket.SO_RCVBUF, 4096)
                    broken.request("GET", "/truncated")
                    response = broken.getresponse()
                    assert response.status == 200 and response.read(1) == b"y"
                    with (root / "truncate.bin").open("r+b") as changing:
                        changing.truncate(0)
                    try:
                        response.read()
                    except (http.client.IncompleteRead, ConnectionResetError):
                        pass
                    else:
                        raise AssertionError("truncated source must fail the advertised full-length body")
                finally:
                    broken.close()
                stalled = http.client.HTTPConnection("127.0.0.1", ports[0], timeout=5)
                try:
                    stalled.connect()
                    stalled.sock.setsockopt(socket.SOL_SOCKET, socket.SO_RCVBUF, 4096)
                    stalled.request("GET", "/timeout")
                    response = stalled.getresponse()
                    assert response.status == 200 and response.read(1)
                    deadline = time.monotonic() + 3
                    while True:
                        terminal = []
                        for line in (root / "stderr").read_text().splitlines():
                            try:
                                event = json.loads(line)
                            except ValueError:
                                continue
                            if event.get("event") == "request" and event.get("route") == "/timeout":
                                terminal.append(event)
                        if terminal:
                            assert len(terminal) == 1 and terminal[0]["outcome"] == "canceled"
                            break
                        assert time.monotonic() < deadline, "file watchdog did not terminate the still-connected stalled body"
                        time.sleep(0.03)
                finally:
                    stalled.close()
                ready(child, ports[0], tokens[0])
                after = json.loads(request(ports[1], "/__auto/health/snapshot")[2])["counters"]
                assert after["bytes_sent"] == before["bytes_sent"] and after["requests_completed"] == before["requests_completed"]
            finally:
                if child.poll() is None:
                    child.stdin.write(("" if second_sent else "second\n") + "shutdown\n")
                    child.stdin.flush()
                    try:
                        child.wait(timeout=20)
                    except subprocess.TimeoutExpired:
                        child.kill()
                        child.wait(timeout=5)
                        raise
                stdout.seek(0)
                stderr.seek(0)
                print(stdout.read(), end="")
                diagnostics = stderr.read()
                print(diagnostics, end="")
            assert child.returncode == 0
            events = []
            for line in diagnostics.splitlines():
                try:
                    event = json.loads(line)
                except ValueError:
                    continue
                if event.get("event") == "request" and event["route"] != "/value":
                    events.append(event)
            actual = [(e["method"], e["route"], e["status"], e["bytes_sent"], e["outcome"]) for e in events if e["route"] not in ("/large", "/forever", "/truncated", "/timeout")]
            assert sorted(actual) == sorted(expected), ("actual terminal events", actual, "required", expected)
            for route in ("/large", "/forever", "/truncated", "/timeout"):
                cancelled = [e for e in events if e["route"] == route]
                assert len(cancelled) == 1 and cancelled[0]["outcome"] == "canceled" and cancelled[0]["bytes_sent"] > 0, cancelled
                if route in ("/large", "/truncated", "/timeout"):
                    assert cancelled[0]["bytes_sent"] < 16 * 1024 * 1024, "RST must not count the unread full file"
            print(json.dumps({"scope": "actual-response-observation", "exact_terminal_events": len(expected), "rst_cancelled_once": 2, "source_truncation_cancelled_once": True, "connected_stalled_file_watchdog_cancelled_once": True, "other_owner_unchanged": True}))


if __name__ == "__main__":
    main()
