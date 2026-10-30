"""T-02 R1/R2/R3/R4: actual launch identity, control and cancellation."""
import argparse
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

from t02_closeout_audit import ports, get, ready, stop


def configured(root, sdk, dynamic):
    declared, fallback = ports()
    if dynamic:
        declared = 0
    nonce = uuid.uuid4().hex
    source, config = root / "sdk.at", root / "sdk.json"
    source.write_text('#[api(method = "GET", path = "/identity")]\nfn identity() str { "' + nonce + '" }\n')
    config.write_text(json.dumps({"profile": "development", "listen": {"port": declared}, "limits": {"drain_timeout_ms": 200}}))
    with (root / "sdk.out").open("w+") as out, (root / "sdk.err").open("w+") as err:
        child = subprocess.Popen([str(sdk), str(source), "--install-adapter"], stdin=subprocess.PIPE, stdout=out, stderr=err, text=True,
                                 env=dict(os.environ, PLAN890_AUDIT_ACTIVE_CONFIG=str(config), PLAN890_EXPLICIT_LAUNCH="1", AUTO_HTTP_PORT=str(fallback), AUTO_EXECUTION_ENGINE="autovm"))
        try:
            deadline = time.monotonic() + 8
            bound = None
            while time.monotonic() < deadline:
                err.seek(0)
                diagnostics = err.read()
                match = re.search(r"transport listening on 127.0.0.1:(\d+)", diagnostics)
                if match:
                    bound = int(match[1])
                    break
                assert child.poll() is None, diagnostics
                time.sleep(0.02)
            assert bound and bound != fallback and (dynamic or bound == declared), diagnostics
            ready(child, [bound], "/identity", nonce)
            status, identity = get(bound, "/__auto/health/ready")
            expected_hash = re.search(r"expected config_hash=([0-9a-f]+)", diagnostics)[1]
            assert status == 200 and identity["bound"] == f"127.0.0.1:{bound}" and identity["config_hash"] == expected_hash, identity
            connection = http.client.HTTPConnection("127.0.0.1", bound, timeout=3)
            connection.request("POST", "/__auto/shutdown")
            response = connection.getresponse()
            assert response.status == 202 and json.loads(response.read())["draining"] is True
            connection.close()
            child.wait(timeout=8)
            assert child.returncode == 0
        finally:
            stop(child)
        err.seek(0)
        diagnostics = err.read()
        assert "probe logger joined instance=" in diagnostics, diagnostics
        print(json.dumps({"case": "explicit-launch-port0" if dynamic else "explicit-launch-priority", "bound": bound, "config_hash": expected_hash, "exit": child.returncode, "log_join": True}))


def failure_then_service(root, session, cancel):
    nonce = uuid.uuid4().hex
    occupied = socket.socket()
    occupied.bind(("127.0.0.1", 0))
    occupied.listen()
    second, _ = ports()
    sources, configs = [], []
    for index, port in enumerate((occupied.getsockname()[1], second)):
        source, config = root / f"session{index}.at", root / f"session{index}.json"
        source.write_text('#[api(method = "GET", path = "/value")]\nfn value() str { "' + nonce + '" }\n'
                          '#[api(method = "GET", path = "/slow")]\nfn slow() ~int { delay_async(4000) }\n'
                          '#[api(method = "GET", path = "/bad")]\nfn bad() int { delay_async(20).await\npanic("controlled async failure")\n0 }\n')
        config.write_text(json.dumps({"profile": "development", "listen": {"port": port}, "limits": {"drain_timeout_ms": 100}, "observability": {"log_sink_capacity": 128}}))
        sources.append(source)
        configs.append(config)
    pending = None
    with (root / "session.out").open("w+") as out, (root / "session.err").open("w+") as err:
        child = subprocess.Popen([str(session), *map(str, sources), *map(str, configs)], stdin=subprocess.PIPE, stdout=out, stderr=err, text=True,
                                 env=dict(os.environ, AUTO_EXECUTION_ENGINE="autovm", PLAN890_NETWORK_TAIL_MS="150"))
        try:
            ready(child, [second], "/value", nonce)
            status, _ = get(second, "/bad")
            assert status == 500
            if cancel:
                pending = socket.create_connection(("127.0.0.1", second), timeout=2)
                pending.sendall(b"GET /slow HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
                time.sleep(0.05)
            child.stdin.write("cancel\n" if cancel else "shutdown\n")
            child.stdin.flush()
            child.wait(timeout=10)
            assert child.returncode == 0
        finally:
            if pending:
                pending.close()
            if child.poll() is None:
                child.kill()
                child.wait(timeout=5)
            occupied.close()
        err.seek(0)
        out.seek(0)
        diagnostics, output = err.read(), out.read()
        if child.returncode:
            raise AssertionError(diagnostics + output)
        assert "session actual network/log joins and resource registries clean" in output, output
        events = []
        for line in diagnostics.splitlines():
            try:
                event = json.loads(line)
            except ValueError:
                continue
            if event.get("event") == "request":
                events.append(event)
        bad = [event for event in events if event["route"] == "/bad"]
        assert len(bad) == 1 and bad[0]["status"] == 500 and bad[0]["bytes_sent"] > 0 and bad[0]["outcome"] == "completed", bad
        slow = [event for event in events if event["route"] == "/slow"]
        if cancel:
            assert len(slow) == 1 and slow[0]["outcome"] == "canceled", slow
        with socket.socket() as released:
            released.bind(("127.0.0.1", second))
        print(json.dumps({"case": "failure-then-cancel" if cancel else "failure-then-shutdown", "exit": child.returncode, "async_error_event": bad, "cancel_event": slow, "actual_join_and_registries": True}))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--sdk-exe", type=Path, required=True)
    parser.add_argument("--session-exe", type=Path, required=True)
    args = parser.parse_args()
    scratch = Path(__file__).resolve().parent / "target" / "t02-session"
    scratch.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=scratch) as directory:
        root = Path(directory)
        configured(root, args.sdk_exe.resolve(), False)
        configured(root, args.sdk_exe.resolve(), True)
        failure_then_service(root, args.session_exe.resolve(), False)
        failure_then_service(root, args.session_exe.resolve(), True)


if __name__ == "__main__":
    main()
