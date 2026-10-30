"""Real borrowed-VM listener and direct-language missing-provider controls."""
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


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--exe", type=Path, required=True)
    parser.add_argument("--mode", choices=("installed", "direct-installed", "direct-missing", "unsupported", "missing"), default="installed")
    args = parser.parse_args()
    token = "blocking_" + uuid.uuid4().hex
    with socket.socket() as reservation:
        reservation.bind(("127.0.0.1", 0))
        port = reservation.getsockname()[1]
    address = f"127.0.0.1:{port}"
    source = ('#[api(method = "GET", path = "/api/value/:id")]\n'
              f'fn value(id int, suffix str) str {{ "{token}" }}\n'
              'fn main() {}\n')
    with tempfile.TemporaryDirectory(prefix="plan890-blocking-") as directory:
        path = Path(directory) / "main.at"
        path.write_text(source, encoding="utf-8")
        with tempfile.TemporaryFile(mode="w+", encoding="utf-8") as output, tempfile.TemporaryFile(mode="w+", encoding="utf-8") as errors:
            command = [str(args.exe.resolve()), str(path), address]
            if args.mode in ("installed", "direct-installed"):
                command.append("--install-adapter")
            if args.mode.startswith("direct-") or args.mode == "unsupported":
                command.append("--direct-native")
            if args.mode == "unsupported":
                command.append("--async-only")
            child = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=output, stderr=errors, text=True)
            try:
                if args.mode not in ("installed", "direct-installed"):
                    child.communicate(input="", timeout=20)
                    errors.seek(0)
                    diagnostics = errors.read()
                    assert child.returncode != 0, diagnostics
                    assert ("does not support blocking listeners" if args.mode == "unsupported" else "adapter is not installed") in diagnostics, diagnostics
                    with socket.socket() as released:
                        released.bind(("127.0.0.1", port))
                    print(json.dumps({"mode": args.mode, "child_exit": child.returncode, "no_listener": True}))
                    return
                deadline = time.monotonic() + 25
                def request(path):
                    connection = http.client.HTTPConnection("127.0.0.1", port, timeout=1)
                    try:
                        connection.request("GET", path)
                        response = connection.getresponse()
                        return response.status, response.read()
                    finally:
                        connection.close()
                observed = None
                while time.monotonic() < deadline:
                    if child.poll() is not None:
                        errors.seek(0)
                        raise AssertionError(errors.read())
                    try:
                        observed = request("/api/value/17?suffix=ok")
                        if observed == ((404, b"Not Found") if args.mode == "direct-installed" else (200, json.dumps(token).encode())):
                            break
                    except (OSError, http.client.HTTPException):
                        pass
                    time.sleep(0.05)
                if args.mode == "direct-installed":
                    assert observed == (404, b"Not Found"), observed
                else:
                    assert observed == (200, json.dumps(token).encode()), observed
                    assert request("/api/value/17")[0] == 400
                    assert request("/api/value/not-an-int?suffix=ok")[0] == 400
                    assert request("/not-registered")[0] == 404
                child.stdin.write("shutdown\n")
                child.stdin.flush()
                assert child.wait(timeout=15) == 0
                errors.seek(0)
                diagnostics = errors.read()
                assert "blocking runtime=external" in diagnostics, diagnostics
                assert "handler_tasks_delta=0 owner_thread_preserved=true" in diagnostics, diagnostics
                assert "blocking shutdown wake=true" in diagnostics, diagnostics
                output.seek(0)
                assert ("direct native returned=true" if args.mode == "direct-installed" else "blocking returned=") in output.read()
                print(json.dumps({"mode": args.mode, "exact_json": args.mode == "installed", "unknown_route": 404, "owner_returned": True, "handler_tasks_delta": 0}))
            finally:
                if child.poll() is None:
                    # Failure cleanup only, never counted as a successful return.
                    child.kill()
                    child.wait(timeout=5)


if __name__ == "__main__":
    main()
