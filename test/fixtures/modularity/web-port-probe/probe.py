"""Real TCP compatibility baseline for PLAN-890; build the probe first.

This checks explicit prototype assembly and missing-host/plain-language
controls. The old Core-only implicit default is retained as historical evidence
at commit 6d70b2267; this prototype is not a delivered Web implementation.
"""
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


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--exe", required=True, type=Path)
    parser.add_argument("--mode", choices=("installed", "missing", "plain"), default="installed")
    args = parser.parse_args()
    token = "p890-" + uuid.uuid4().hex
    with socket.socket() as reservation:
        reservation.bind(("127.0.0.1", 0))
        port = reservation.getsockname()[1]
    env = dict(os.environ, AUTO_HTTP_PORT=str(port), AUTO_EXECUTION_ENGINE="autovm")
    with tempfile.TemporaryDirectory(prefix="p890-sdk-") as temporary:
        root = Path(temporary)
        source = root / "api.at"
        source.write_text(
            '#[api(method = "GET", path = "/api/probe")]\n'
            f'fn probe() str {{ "{token}" }}\n', encoding="utf-8"
        )
        if args.mode == "plain":
            source.write_text("40 + 2", encoding="utf-8")
        with (root / "stdout.txt").open("w+") as stdout, (root / "stderr.txt").open("w+") as stderr:
            command = [str(args.exe.resolve()), str(source)]
            if args.mode == "installed":
                command.append("--install-adapter")
            child = subprocess.Popen(
                command, env=env,
                stdin=subprocess.PIPE, stdout=stdout, stderr=stderr, text=True,
            )
            try:
                if args.mode in ("missing", "plain"):
                    child.communicate(input="", timeout=20)
                    stdout.seek(0)
                    stderr.seek(0)
                    output, diagnostics = stdout.read(), stderr.read()
                    assert "transport listening" not in diagnostics
                    assert "explicit probe adapter:" not in diagnostics
                    if args.mode == "missing":
                        assert child.returncode != 0
                        assert "HTTP service host adapter is not installed" in diagnostics, diagnostics
                    else:
                        assert child.returncode == 0, diagnostics
                        assert 'value="42"' in output, output
                    print(json.dumps({"scope": "http-host-prototype", "mode": args.mode,
                                      "child_exit": child.returncode, "no_listener_start": True}))
                    return
                deadline = time.monotonic() + 30
                observed = None
                while time.monotonic() < deadline:
                    if child.poll() is not None:
                        raise AssertionError(f"SDK exited before HTTP response: {child.returncode}")
                    connection = http.client.HTTPConnection("127.0.0.1", port, timeout=0.5)
                    try:
                        connection.request("GET", "/api/probe")
                        response = connection.getresponse()
                        body = response.read()
                        # A listener belonging to another process cannot pass
                        # readiness: require this invocation's exact value.
                        if response.status == 200 and json.loads(body) == token:
                            observed = (response.status, body.decode())
                            break
                    except (OSError, http.client.HTTPException, ValueError):
                        pass
                    finally:
                        connection.close()
                    time.sleep(0.05)
                assert observed is not None, "Core-only default failed to serve the exact probe value"
                assert child.stdin is not None
                child.stdin.write("shutdown\n")
                child.stdin.flush()
                assert child.wait(timeout=20) == 0, "SDK must return successfully after drain"
                stderr.seek(0)
                diagnostics = stderr.read()
                assert "probe shutdown delivered=true" in diagnostics, diagnostics
                assert "explicit probe adapter:" in diagnostics
                print(json.dumps({"scope": "explicit-http-host-prototype", "status": observed[0],
                                  "exact_body": True, "exit_code": child.returncode,
                                  "shutdown_delivered": True}))
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
