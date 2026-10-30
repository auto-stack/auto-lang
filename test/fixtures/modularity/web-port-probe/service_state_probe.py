"""Real per-service identity/counter/policy/shutdown oracle for PLAN-890."""
from __future__ import annotations

import argparse
import http.client
import json
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import time
import uuid


def request(port, path, *, method="GET", headers=None):
    connection = http.client.HTTPConnection("127.0.0.1", port, timeout=3)
    try:
        connection.request(method, path, headers=headers or {})
        response = connection.getresponse()
        return response.status, {k.lower(): v for k, v in response.getheaders()}, json.loads(response.read())
    finally:
        connection.close()


def ready(child, port, token):
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        assert child.poll() is None, "owner exited before readiness"
        try:
            status, _, body = request(port, "/value")
            if status == 200 and body == token:
                status, _, identity = request(port, "/__auto/health/ready")
                if status == 200:
                    return identity
        except (OSError, ValueError, http.client.HTTPException):
            pass
        time.sleep(0.03)
    raise AssertionError("exact configured owner did not become ready")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--exe", required=True, type=Path)
    parser.add_argument("--shared-observation-negative", action="store_true")
    parser.add_argument("--idle-on-shutdown", action="store_true")
    parser.add_argument("--file-during-drain", action="store_true")
    args = parser.parse_args()
    with socket.socket() as a_socket, socket.socket() as b_socket:
        a_socket.bind(("127.0.0.1", 0))
        b_socket.bind(("127.0.0.1", 0))
        a_port, b_port = a_socket.getsockname()[1], b_socket.getsockname()[1]
    scratch = Path(__file__).resolve().parent / "target" / "service-state-runs"
    scratch.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="p890-service-state-", dir=scratch) as directory:
        root = Path(directory)
        payload = (bytes(range(251)) * (16 * 1024 * 1024 // 251 + 1))[:16 * 1024 * 1024] if args.file_during_drain else b""
        if payload:
            (root / "drain.bin").write_bytes(payload)
        sources, configs = [], []
        tokens = ["first-" + uuid.uuid4().hex, "second-" + uuid.uuid4().hex]
        for index, (port, token, origin) in enumerate(zip((a_port, b_port), tokens,
                                                       ("https://one.example", "https://two.example"))):
            source = root / f"program-{index}.at"
            source.write_text('#[api(method = "GET", path = "/value")]\nfn value() str { "' + token + '" }\n'
                              + ('#[api(method = "GET", path = "/drain-file")]\n'
                                 'fn drain_file() FileResponse { return http.file_response("'
                                 + root.as_posix() + '", "drain.bin", "{}") }\n' if payload else ""), encoding="utf-8")
            config = root / f"service-{index}.json"
            config.write_text(json.dumps({"profile": "development", "listen": {"addr": "127.0.0.1", "port": port},
                                          "allowed_hosts": ["127.0.0.1"],
                                          "cors": {"allowed_origins": [origin]},
                                          "limits": {"max_inflight_requests": 4, "drain_timeout_ms": 5000 if payload else 1000},
                                          "observability": {"log_sink_capacity": 64}}), encoding="utf-8")
            sources.append(source)
            configs.append(config)
        env = dict(os.environ)
        env.pop("PLAN890_SHARED_OBSERVATION_NEGATIVE", None)
        if args.shared_observation_negative:
            env["PLAN890_SHARED_OBSERVATION_NEGATIVE"] = "1"
        with (root / "stdout").open("w+") as stdout, (root / "stderr").open("w+") as stderr:
            child = subprocess.Popen([str(args.exe.resolve()), *map(str, sources), str(a_port), str(b_port), *map(str, configs)],
                                     stdin=subprocess.PIPE, stdout=stdout, stderr=stderr, text=True, env=env)
            idle = []
            download = None
            try:
                a_identity = ready(child, a_port, tokens[0])
                child.stdin.write("second\n")
                child.stdin.flush()
                b_identity = ready(child, b_port, tokens[1])
                assert a_identity["instance_id"] != b_identity["instance_id"], "two services share one instance identity"
                assert a_identity["bound"] == f"127.0.0.1:{a_port}"
                assert b_identity["bound"] == f"127.0.0.1:{b_port}"
                assert a_identity["config_hash"] != b_identity["config_hash"]
                assert request(a_port, "/__auto/health/ready")[2] == a_identity, "later startup changed first identity"
                before = request(b_port, "/__auto/health/snapshot")[2]["counters"]
                for _ in range(10):
                    status, headers, body = request(a_port, "/value")
                    assert status == 200 and body == tokens[0]
                    assert "access-control-allow-origin" not in headers, "configured no-Origin response leaked legacy wildcard"
                after = request(b_port, "/__auto/health/snapshot")[2]["counters"]
                assert after["requests_completed"] == before["requests_completed"], "A traffic changed B completion count"
                assert after["requests_total"] == before["requests_total"] + 1, "A traffic changed B request count"
                for port, origin, token in ((a_port, "https://one.example", tokens[0]), (b_port, "https://two.example", tokens[1])):
                    status, headers, body = request(port, "/value", headers={"Origin": origin})
                    assert status == 200 and body == token
                    assert headers.get("access-control-allow-origin") == origin
                    # Existing Spec: CORS is not authentication. Actual GET
                    # still executes without an allow header; preflight denies.
                    status, rejected_headers, body = request(port, "/value", headers={"Origin": "https://rejected.example"})
                    assert status == 200 and body == token
                    assert "access-control-allow-origin" not in rejected_headers
                    assert request(port, "/value", method="OPTIONS", headers={
                        "Origin": "https://rejected.example", "Access-Control-Request-Method": "GET",
                    })[0] == 403
                if args.idle_on_shutdown:
                    for port in (a_port, b_port):
                        conn = socket.create_connection(("127.0.0.1", port), timeout=3)
                        conn.sendall(b"GET /value HTTP/1.1\r\nHost:")
                        idle.append(conn)
                        deadline = time.monotonic() + 3
                        while request(port, "/__auto/health/snapshot")[2]["counters"]["connections_active"] < 2:
                            assert time.monotonic() < deadline, "idle connection was not admitted"
                            time.sleep(0.03)
                if payload:
                    download = http.client.HTTPConnection("127.0.0.1", a_port, timeout=10)
                    download.connect()
                    download.sock.setsockopt(socket.SOL_SOCKET, socket.SO_RCVBUF, 65536)
                    download.request("GET", "/drain-file")
                    in_flight = download.getresponse()
                    assert in_flight.status == 200
                    head = in_flight.read(1)
                    assert head == payload[:1]
                status, _, body = request(a_port, "/__auto/shutdown", method="POST",
                                          headers={"Expect": "100-continue", "Content-Length": "1000000"})
                assert status == 202 and body == {"draining": True}, "shutdown must precede body consumption"
                child.stdin.write("stop-first\n")
                child.stdin.flush()
                if payload:
                    time.sleep(0.15)
                    try:
                        assert request(a_port, "/__auto/health/ready")[0] != 200
                    except (OSError, http.client.HTTPException):
                        pass
                    assert head + in_flight.read() == payload, "in-flight file was truncated during drain"
                    download.close()
                    download = None
                deadline = time.monotonic() + 10
                closed = False
                while time.monotonic() < deadline:
                    try:
                        with socket.create_connection(("127.0.0.1", a_port), timeout=0.1):
                            pass
                    except OSError:
                        closed = True
                        break
                    time.sleep(0.02)
                assert closed, "A shutdown endpoint did not close A"
                assert request(b_port, "/__auto/health/ready")[0::2] == (200, b_identity), "A shutdown affected B ready/identity"
                print(json.dumps({"scope": "configured-service-isolation", "distinct_identity": True,
                                  "bound_config_stable": True, "counter_isolation": True, "cors_isolation": True,
                                  "headers_only_shutdown": True, "second_remains_ready": True}))
            finally:
                if child.poll() is None:
                    child.stdin.write("shutdown\n")
                    child.stdin.flush()
                    try:
                        assert child.wait(timeout=20) == 0, "owner drain failed"
                    except subprocess.TimeoutExpired:
                        child.kill()
                        child.wait(timeout=5)
                        raise
                stdout.seek(0)
                stderr.seek(0)
                print(stdout.read(), end="")
                diagnostics = stderr.read()
                print(diagnostics, end="")
                for conn in idle:
                    conn.close()
                if download is not None:
                    download.close()
            assert diagnostics.count("isolation runtime=external") == 2
            for port in (a_port, b_port):
                deadline = time.monotonic() + 3
                while True:
                    try:
                        with socket.socket() as rebound:
                            rebound.bind(("127.0.0.1", port))
                        break
                    except OSError:
                        assert time.monotonic() < deadline, "drained owner's port cannot be rebound"
                        time.sleep(0.05)


if __name__ == "__main__":
    main()
