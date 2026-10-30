"""Actual native policy -> external owner oracle, with two concurrent VMs."""
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


def request(port, path):
    conn = http.client.HTTPConnection("127.0.0.1", port, timeout=3)
    try:
        conn.request("GET", path)
        response = conn.getresponse()
        return response.status, json.loads(response.read()), {k.lower(): v for k, v in response.getheaders()}
    finally:
        conn.close()


def ready(child, port, path, token):
    deadline = time.monotonic() + 20
    while time.monotonic() < deadline:
        assert child.poll() is None, "owner exited before readiness"
        try:
            result = request(port, path)
            if result[:2] == (200, token):
                return result
            raise AssertionError(result)
        except (OSError, http.client.HTTPException):
            time.sleep(0.03)
    raise AssertionError("exact policy owner never became ready")


def limited(port, path):
    status, body, headers = request(port, path)
    assert status == 429, ("native rate_limit must reach external owner", status, body)
    assert body["error"] == "rate limit exceeded" and 0 < body["retry_after_ms"] <= 60000
    assert 0 < int(headers["retry-after"]) <= 60 and headers["x-request-id"]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--exe", required=True, type=Path)
    parser.add_argument("--plain-exe", required=True, type=Path)
    args = parser.parse_args()
    scratch = Path(__file__).resolve().parent / "target" / "policy-runs"
    scratch.mkdir(parents=True, exist_ok=True)
    token = uuid.uuid4().hex
    with socket.socket() as a, socket.socket() as b:
        a.bind(("127.0.0.1", 0))
        b.bind(("127.0.0.1", 0))
        ports = a.getsockname()[1], b.getsockname()[1]
    with tempfile.TemporaryDirectory(dir=scratch, prefix="native-") as temporary:
        root = Path(temporary)
        first, second = root / "first.at", root / "second.at"
        ordinary = root / "ordinary.at"
        ordinary.write_text("http.rate_limit(2, 60000)\n40 + 2\n", encoding="utf-8")
        plain = subprocess.run([str(args.plain_exe.resolve()), str(ordinary)], input="",
                               text=True, capture_output=True, timeout=20)
        assert plain.returncode == 0 and 'value="42"' in plain.stdout, (plain.stdout, plain.stderr)
        assert "explicit probe adapter:" not in plain.stderr and "transport listening" not in plain.stderr
        first.write_text('http.rate_limit(4, 60000)\n'
                         '#[api(method = "GET", path = "/first")]\n'
                         f'fn first() str {{ "a-{token}" }}\n', encoding="utf-8")
        second.write_text('http.rate_limit(0, 60000)\n'
                          '#[api(method = "GET", path = "/second")]\n'
                          f'fn second() str {{ "b-{token}" }}\n'
                          '#[api(method = "GET", path = "/configure")]\n'
                          f'fn configure() str {{ http.rate_limit(2, 60000)\n "b-{token}" }}\n',
                          encoding="utf-8")
        with (root / "stdout").open("w+") as stdout, (root / "stderr").open("w+") as stderr:
            child = subprocess.Popen([str(args.exe.resolve()), str(first), str(second), *map(str, ports)],
                                     stdin=subprocess.PIPE, stdout=stdout, stderr=stderr, text=True,
                                     env=dict(os.environ, AUTO_EXECUTION_ENGINE="autovm",
                                              PLAN890_EMBEDDING_RATE_DEFAULT="1",
                                              AUTO_CORS_ORIGIN="https://p890.example"))
            second_sent = False
            try:
                initial = ready(child, ports[0], "/first", "a-" + token)
                assert initial[2]["access-control-allow-origin"] == "https://p890.example"
                for _ in range(3):
                    assert request(ports[0], "/first")[:2] == (200, "a-" + token)
                limited(ports[0], "/first")
                child.stdin.write("second\n")
                child.stdin.flush()
                second_sent = True
                ready(child, ports[1], "/second", "b-" + token)
                for _ in range(6):
                    assert request(ports[1], "/second")[:2] == (200, "b-" + token)
                limited(ports[0], "/first")
                assert request(ports[1], "/configure")[:2] == (200, "b-" + token)
                for _ in range(2):
                    assert request(ports[1], "/second")[:2] == (200, "b-" + token)
                limited(ports[1], "/second")
                limited(ports[0], "/first")
            finally:
                if child.poll() is None:
                    child.stdin.write(("" if second_sent else "second\n") + "shutdown\n")
                    child.stdin.flush()
                    try:
                        child.wait(timeout=25)
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
            assert diagnostics.count("isolation runtime=external") == 2
            assert "isolation embedding rate default=1" in diagnostics
            print(json.dumps({"scope": "native-policy-external-owners", "initial_quota": 4,
                              "second_disabled_then_dynamic_quota": 2, "isolated_buckets": True,
                              "legacy_env_cors_preserved": True, "both_owners_drained": True}))


if __name__ == "__main__":
    main()
