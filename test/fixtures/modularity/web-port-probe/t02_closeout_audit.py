"""Bounded T-02 closure audit. A nonzero result leaves the technical gate open."""
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


def ports():
    with socket.socket() as a, socket.socket() as b:
        a.bind(("127.0.0.1", 0))
        b.bind(("127.0.0.1", 0))
        return a.getsockname()[1], b.getsockname()[1]


def get(port, path):
    conn = http.client.HTTPConnection("127.0.0.1", port, timeout=0.5)
    try:
        conn.request("GET", path)
        response = conn.getresponse()
        return response.status, json.loads(response.read())
    finally:
        conn.close()


def ready(child, candidates, path, value):
    deadline = time.monotonic() + 8
    while time.monotonic() < deadline:
        assert child.poll() is None, "owner exited before exact readiness"
        for port in candidates:
            try:
                if get(port, path) == (200, value):
                    return port
            except (OSError, ValueError, http.client.HTTPException):
                pass
        time.sleep(0.03)
    raise AssertionError("exact service did not become ready")


def stop(child):
    if child.poll() is None:
        try:
            child.stdin.write("shutdown\n")
            child.stdin.flush()
        except BrokenPipeError:
            pass
        try:
            child.wait(timeout=12)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait(timeout=5)
            raise


def configuration(root, sdk):
    declared, fallback = ports()
    nonce = uuid.uuid4().hex
    source, config = root / "configured.at", root / "configured.json"
    source.write_text('#[api(method = "GET", path = "/identity")]\nfn identity() str { "' + nonce + '" }\n')
    config.write_text(json.dumps({"profile": "development", "listen": {"addr": "127.0.0.1", "port": declared}}))
    with (root / "config.stdout").open("w+") as out, (root / "config.stderr").open("w+") as err:
        child = subprocess.Popen([str(sdk), str(source), "--install-adapter"], stdin=subprocess.PIPE, stdout=out, stderr=err, text=True,
                                 env=dict(os.environ, AUTO_HTTP_PORT=str(fallback), PLAN890_AUDIT_ACTIVE_CONFIG=str(config), AUTO_EXECUTION_ENGINE="autovm"))
        try:
            observed = ready(child, [fallback, declared], "/identity", nonce)
        finally:
            stop(child)
        err.seek(0)
        diagnostics = err.read()
        print(diagnostics, end="")
    return {"case": "configured-launch-handoff", "passed": observed == declared and child.returncode == 0,
            "declared_port": declared, "observed_port": observed, "env_fallback_port": fallback,
            "active_config_unconsumed": "active_config_unconsumed=true" in diagnostics, "child_exit": child.returncode}


def bind_failure(root, sdk):
    source = root / "occupied.at"
    source.write_text('#[api(method = "GET", path = "/value")]\nfn value() int { 17 }\n')
    with socket.socket() as occupied:
        occupied.bind(("127.0.0.1", 0))
        occupied.listen()
        result = subprocess.run([str(sdk), str(source), "--install-adapter"], input="", capture_output=True, text=True, timeout=10,
                                env=dict(os.environ, AUTO_HTTP_PORT=str(occupied.getsockname()[1]), AUTO_EXECUTION_ENGINE="autovm"))
    print(result.stderr, end="")
    return {"case": "owned-startup-failure-result", "passed": result.returncode != 0,
            "child_exit": result.returncode, "bind_failure_diagnostic": "bind failed" in result.stderr.lower()}


def async_observation(root, isolation):
    a, b = ports()
    nonce = uuid.uuid4().hex
    sources, configs = [], []
    for index, port in enumerate((a, b)):
        source, config = root / f"async-{index}.at", root / f"async-{index}.json"
        text = '#[api(method = "GET", path = "/value")]\nfn value() str { "' + nonce + '" }\n'
        if index == 0:
            text += '#[api(method = "GET", path = "/later")]\nfn later() ~int { delay_async(25) }\n'
        source.write_text(text)
        config.write_text(json.dumps({"profile": "development", "listen": {"port": port}, "observability": {"log_sink_capacity": 64}, "limits": {"drain_timeout_ms": 1000}}))
        sources.append(source)
        configs.append(config)
    with (root / "async.stdout").open("w+") as out, (root / "async.stderr").open("w+") as err:
        child = subprocess.Popen([str(isolation), *map(str, sources), str(a), str(b), *map(str, configs)], stdin=subprocess.PIPE, stdout=out, stderr=err, text=True,
                                 env=dict(os.environ, AUTO_EXECUTION_ENGINE="autovm"))
        second = False
        try:
            ready(child, [a], "/value", nonce)
            child.stdin.write("second\n")
            child.stdin.flush()
            second = True
            ready(child, [b], "/value", nonce)
            wire = get(a, "/later")
        finally:
            if not second and child.poll() is None:
                child.stdin.write("second\n")
                child.stdin.flush()
            stop(child)
        err.seek(0)
        diagnostics = err.read()
        print(diagnostics, end="")
    events = []
    for line in diagnostics.splitlines():
        try:
            event = json.loads(line)
        except ValueError:
            continue
        if event.get("event") == "request" and event.get("route") == "/later":
            events.append(event)
    valid = len(events) == 1 and (events[0]["status"], events[0]["bytes_sent"], events[0]["outcome"]) == (200, 2, "completed")
    return {"case": "parked-async-response-observation", "passed": wire == (200, 25) and valid and child.returncode == 0,
            "wire": wire, "events": events, "child_exit": child.returncode}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--sdk-exe", required=True, type=Path)
    parser.add_argument("--isolation-exe", required=True, type=Path)
    args = parser.parse_args()
    scratch = Path(__file__).resolve().parent / "target" / "t02-closeout-audit"
    scratch.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=scratch) as directory:
        root = Path(directory)
        results = [configuration(root, args.sdk_exe.resolve()), bind_failure(root, args.sdk_exe.resolve()), async_observation(root, args.isolation_exe.resolve())]
    print(json.dumps({"scope": "T-02-bounded-closure-audit", "results": results}, ensure_ascii=False))
    raise SystemExit(0 if all(result["passed"] for result in results) else 1)


if __name__ == "__main__":
    main()
