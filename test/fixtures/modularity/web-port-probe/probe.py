"""Real TCP compatibility baseline for PLAN-890; build the probe first.

This establishes the existing Core-only default. It does not certify a future
host port, upload/stream lifetime, or complete migration acceptance.
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
        with (root / "stdout.txt").open("w+") as stdout, (root / "stderr.txt").open("w+") as stderr:
            child = subprocess.Popen(
                [str(args.exe.resolve()), str(source)], env=env,
                stdin=subprocess.PIPE, stdout=stdout, stderr=stderr, text=True,
            )
            try:
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
                print(json.dumps({"scope": "core-only-http-default-baseline", "status": observed[0],
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
