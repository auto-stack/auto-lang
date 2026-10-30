"""Compile A, park its main, compile/activate B, then activate A on one thread."""
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


def get(port, path):
    conn = http.client.HTTPConnection("127.0.0.1", port, timeout=2)
    try:
        conn.request("GET", path)
        response = conn.getresponse()
        body = response.read()
        try:
            value = json.loads(body)
        except ValueError:
            value = body.decode(errors="replace")
        return response.status, value
    finally:
        conn.close()


def ready(child, port, path, expected):
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        assert child.poll() is None, "owner exited before ready"
        try:
            actual = get(port, path)
            assert actual == (200, expected), ("program metadata crossed the compile/main/activation gap", actual)
            return
        except (OSError, http.client.HTTPException):
            time.sleep(0.03)
    raise AssertionError("own exact response never became ready")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--exe", type=Path, required=True)
    parser.add_argument("--ordinary-first", action="store_true")
    args = parser.parse_args()
    nonce = uuid.uuid4().hex
    base = int(nonce[:7], 16) + 1000
    prefix = "second-" + nonce + "-"
    scratch = Path(__file__).resolve().parent / "target" / "metadata-gap-runs"
    scratch.mkdir(parents=True, exist_ok=True)
    with socket.socket() as a, socket.socket() as b:
        a.bind(("127.0.0.1", 0))
        b.bind(("127.0.0.1", 0))
        ports = a.getsockname()[1], b.getsockname()[1]
    with tempfile.TemporaryDirectory(dir=scratch) as temporary:
        root = Path(temporary)
        first, second = root / "a.at", root / "b.at"
        first.write_text('#[api(method = "GET", path = "/one/:n")]\nfn shared(n int) int { n + ' + str(base) + ' }\n'
                         '#[api(method = "GET", path = "/later")]\nfn later() ~int { delay_async(25) }\n'
                         'fn main() { delay_async(300).await }\n', encoding="utf-8")
        if args.ordinary_first:
            first.write_text('fn main() int { delay_async(300).await\n40 + 2 }\n', encoding="utf-8")
        second.write_text('#[api(method = "GET", path = "/two/:word")]\nfn shared(word str) str { "' + prefix + '" + word }\n'
                          '#[api(method = "GET", path = "/other")]\nfn later() int { 240 }\n', encoding="utf-8")
        with (root / "stdout").open("w+") as stdout, (root / "stderr").open("w+") as stderr:
            child = subprocess.Popen([str(args.exe.resolve()), str(first), str(second), *map(str, ports)], stdin=subprocess.PIPE, stdout=stdout, stderr=stderr, text=True,
                                     env=dict(os.environ, PLAN890_METADATA_GAP="1", PLAN890_ORDINARY_FIRST="1" if args.ordinary_first else "0", AUTO_EXECUTION_ENGINE="autovm"))
            try:
                ready(child, ports[1], "/two/hello", prefix + "hello")
                if args.ordinary_first:
                    deadline = time.monotonic() + 5
                    while "gap first result=42" not in (root / "stdout").read_text():
                        assert child.poll() is None and time.monotonic() < deadline, "ordinary owner inherited another program's routes or required its adapter"
                        time.sleep(0.03)
                    with socket.socket() as unused:
                        unused.bind(("127.0.0.1", ports[0]))
                else:
                    ready(child, ports[0], "/one/7", base + 7)
                    assert get(ports[0], "/later") == (200, 25)
                assert get(ports[1], "/other") == (200, 240)
                if not args.ordinary_first:
                    assert get(ports[0], "/one/not-int")[0] == 400
                assert get(ports[1], "/two/7") == (200, prefix + "7")
            finally:
                if child.poll() is None:
                    child.stdin.write("shutdown\n")
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
            assert child.returncode == 0 and "two owners drained" in (root / "stdout").read_text()
            assert diagnostics.count("isolation runtime=external") == (1 if args.ordinary_first else 2)
            print(json.dumps({"scope": "compile-main-activation-gap", "actual_main_parked": True, "ordinary_first": args.ordinary_first, "own_routes_params_async_returns": True, "both_drained": True}))


if __name__ == "__main__":
    main()
