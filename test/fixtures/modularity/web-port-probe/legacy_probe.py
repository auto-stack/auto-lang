"""Actual compatibility SDK -> explicit external legacy stdnet grade."""
import argparse
import http.client
import json
import os
from pathlib import Path
import re
import socket
import subprocess
import struct
import tempfile
import time
import uuid


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--exe", type=Path, required=True)
    parser.add_argument("--mode", choices=("installed", "missing", "unsupported", "grade-swap"), default="installed")
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[4]
    scratch = Path(__file__).resolve().parent / "target/legacy"
    scratch.mkdir(parents=True, exist_ok=True)
    with socket.socket() as reservation:
        reservation.bind(("127.0.0.1", 0))
        port = reservation.getsockname()[1]
    with tempfile.TemporaryDirectory(prefix="owned-legacy-", dir=scratch) as directory:
        root = Path(directory)
        token = "legacy_" + uuid.uuid4().hex
        original = (repo / "crates/auto-lang-core/src/vm/ffi/http_server.rs").read_text(encoding="utf-8")
        generator = re.search(r'fn e2e_sse_generator_handler\(\).*?start_server\(\s*r#"(.*?)"#', original, re.S)
        assert generator, "original SSE program changed; audit instead of substituting"
        program = ('#[api(method = "GET", path = "/value/:id")]\n'
                   f'fn value(id int, suffix str) str {{ "{token}" }}\n'
                   '#[api(method = "GET", path = "/unsupported-file")]\n'
                   f'fn file() FileResponse {{ return http.file_response("{root.as_posix()}", "not-opened.bin", "{{}}") }}\n'
                   + generator[1] + '\nfn collision_seed() ~Iter<int> { yield 707; yield 808 }\n'
                   '#[api(method = "GET", path = "/ordinary-zero")]\nfn ordinary_zero() int { return 0 }\n'
                   '#[api(method = "GET", path = "/stream-as-int")]\nfn stream_as_int() int { return counter_handler() }\n'
                   '#[api(method = "GET", path = "/forever")]\n'
                   'fn forever() ~Iter<int> { while true { yield 1 } }\nfn main() {}\n')
        source = root / "main.at"
        source.write_text(program, encoding="utf-8")
        command = [str(args.exe.resolve()), str(source), f"127.0.0.1:{port}"]
        if args.mode in ("installed", "grade-swap"):
            command += ["--install-adapter", "--legacy"]
            if args.mode == "grade-swap":
                command += ["--swap-legacy-grade"]
        else:
            command += ["--direct-legacy"]
            if args.mode == "unsupported":
                command += ["--async-only"]
        def request(path, method="GET", headers=None):
            connection = http.client.HTTPConnection("127.0.0.1", port, timeout=2)
            try:
                connection.request(method, path, headers=headers or {})
                response = connection.getresponse()
                return response.status, {key.lower(): value for key, value in response.getheaders()}, response.read()
            finally:
                connection.close()
        with (root / "stdout").open("w+", encoding="utf-8") as stdout, (root / "stderr").open("w+", encoding="utf-8") as stderr:
            child = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=stdout, stderr=stderr, text=True,
                                     env=dict(os.environ, AUTO_EXECUTION_ENGINE="autovm"))
            try:
                if args.mode not in ("installed", "grade-swap"):
                    child.communicate(input="", timeout=20)
                    stdout.seek(0)
                    stderr.seek(0)
                    output, diagnostics = stdout.read(), stderr.read()
                    # Original compatibility signature is unit, with diagnostics.
                    assert child.returncode == 0, diagnostics
                    assert "legacy compatibility returned=unit" in output, output
                    expected = "adapter is not installed" if args.mode == "missing" else "does not support blocking listeners"
                    assert expected in diagnostics, diagnostics
                    assert "Server listening" not in diagnostics, diagnostics
                    assert "blocking runtime=external" not in diagnostics, diagnostics
                    with socket.socket() as free:
                        free.bind(("127.0.0.1", port))
                    print(json.dumps({"mode": args.mode, "legacy_unit_return": True, "explicit_diagnostic": True, "no_listener": True}))
                    return
                deadline = time.monotonic() + 25
                while time.monotonic() < deadline:
                    assert child.poll() is None, "legacy owner exited before exact readiness"
                    try:
                        status, headers, body = request("/value/7?suffix=ok")
                        if status == 200 and json.loads(body) == token:
                            break
                    except (OSError, http.client.HTTPException):
                        pass
                    time.sleep(0.05)
                else:
                    raise AssertionError("exact legacy value never ready")
                assert headers.get("access-control-allow-origin") == "*", headers
                status, zero_headers, zero_body = request("/ordinary-zero")
                assert status == 200 and zero_body == b"0", (status, zero_headers, zero_body)
                assert zero_headers.get("content-type") == "application/json", zero_headers
                assert request("/value/7")[0] == 400
                assert request("/value/bad?suffix=ok")[0] == 400
                assert request("/missing")[0] == 404
                # Header-only preflight must not read the promised body.
                status, headers, body = request("/value/7", "OPTIONS", {"Content-Length": "1000000"})
                assert status == 204 and not body and headers.get("access-control-allow-origin") == "*", (status, headers, body)
                status, headers, frames = request("/api/counter")
                assert status == 200 and headers.get("content-type") == "text/event-stream", (status, headers, frames)
                assert frames == b"data: 1\n\ndata: 2\n\ndata: 3\n\n", frames
                status, int_headers, int_frames = request("/stream-as-int")
                assert status == 200 and int_headers.get("content-type") == "text/event-stream", (status, int_headers, int_frames)
                assert int_frames == frames, int_frames
                for _ in range(3):
                    status, headers, body = request("/unsupported-file")
                    assert status == 500 and json.loads(body) == {"error": "file responses require the default HTTP transport"}, (status, body)
                assert json.loads(request("/value/9?suffix=after")[2]) == token
                # Observe an actual infinite generator frame before resetting
                # the owned peer; the serial owner must recover for another call.
                reset = socket.create_connection(("127.0.0.1", port), timeout=2)
                try:
                    reset.sendall(b"GET /forever HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
                    observed = b""
                    while b"data: 1\n\n" not in observed:
                        chunk = reset.recv(4096)
                        assert chunk, "infinite generator closed before first frame"
                        observed += chunk
                    reset.setsockopt(socket.SOL_SOCKET, socket.SO_LINGER,
                                     struct.pack("HH" if os.name == "nt" else "ii", 1, 0))
                finally:
                    reset.close()
                deadline = time.monotonic() + 6
                while True:
                    try:
                        if json.loads(request("/value/11?suffix=after-reset")[2]) == token:
                            break
                    except (OSError, http.client.HTTPException):
                        pass
                    assert time.monotonic() < deadline, "legacy iterator owner did not recover after peer reset"
                child.stdin.write("shutdown\n")
                child.stdin.flush()
                assert child.wait(timeout=15) == 0
                stderr.seek(0)
                diagnostics = stderr.read()
                assert "legacy terminal iterators_delta=0 file_descriptors=0 request_credential_owners=0" in diagnostics, diagnostics
                assert "handler_tasks_delta=0 owner_thread_preserved=true" in diagnostics, diagnostics
                assert "legacy owner returned credentials=0 uploads=0" in diagnostics, diagnostics
                assert "legacy collision actual_key=0 caller_generator_untouched=true" in diagnostics, diagnostics
                assert "blocking controller_joined=true" in diagnostics, diagnostics
                print(json.dumps({"mode": args.mode, "exact_json": True, "cors": "*", "preflight": 204,
                                  "sse_handler_forms": 2, "exact_sse_frames_per_handler": 3, "file_explicit_500": 3, "real_iterator_collision_stays_json": True, "peer_reset_recovered": True, "owner_and_controller_returned": True,
                                  "handler_tasks_delta": 0, "service_iterators_delta": 0,
                                  "file_descriptors_before_vm_drop": 0, "request_credential_owners_before_vm_drop": 0,
                                  "return_credentials_after_owner_drop": 0, "uploads_after_owner_drop": 0}))
            finally:
                if child.poll() is None:
                    try:
                        child.stdin.write("shutdown\n")
                        child.stdin.flush()
                        child.wait(timeout=3)
                    except (OSError, subprocess.TimeoutExpired):
                        if child.poll() is None:
                            child.kill()  # Failure cleanup only, never green.
                            child.wait(timeout=5)


if __name__ == "__main__":
    main()
