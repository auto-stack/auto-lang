"""Cold production generation, actual compiler/HTTP/media/stop/freshness oracle.

The explicit target is reused for this feasibility check. This is not a final
fresh SDK/relocation distribution certification.
"""
import argparse
import hashlib
import http.client
import json
import os
from pathlib import Path
import socket
import struct
import subprocess
import tempfile
import time
import urllib.parse
import uuid
import zlib


def png():
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))
    rows = b"\x00" + b"\xff\x20\x40" * 3
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", 3, 2, 8, 2, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(rows * 2)) + chunk(b"IEND", b"")


def jpeg_size(data):
    assert data[:2] == b"\xff\xd8"
    cursor = 2
    while cursor < len(data):
        assert data[cursor] == 255
        marker = data[cursor + 1]
        length = int.from_bytes(data[cursor + 2:cursor + 4], "big")
        if marker in (192, 194):
            return struct.unpack(">HH", data[cursor + 5:cursor + 9])[::-1]
        cursor += length + 2
    raise AssertionError("JPEG frame dimensions absent")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--exe", type=Path, required=True)
    parser.add_argument("--target", type=Path, required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[4]
    try:
        args.evidence.resolve().relative_to(repo.resolve())
        args.target.resolve().relative_to(repo.resolve())
    except ValueError:
        parser.error("generated source/build/evidence must stay inside this worktree")
    args.evidence.mkdir(parents=True, exist_ok=True)
    root = Path(tempfile.mkdtemp(prefix="generated-neutral-", dir=args.evidence.resolve()))
    project = root / "project"
    back = project / "src/back"
    back.mkdir(parents=True)
    (project / "pac.at").write_text('{ name: "plan890-neutral-service", scene: "workspace" }', encoding="utf-8")
    token = "generated_" + uuid.uuid4().hex
    api = back / "api.at"
    original = '#[api(method = "GET", path = "/value")]\npub fn value() str { return "' + token + '" }\n'
    api.write_text(original, encoding="utf-8")
    target = args.target.resolve()
    env = dict(os.environ)
    env.update(AUTO_LANG_ROOT=str(repo), AUTO_UI_SOURCE_ROOT=str(root / "missing-ui"),
               AUTO_RUST_WORKSPACE=str(root / "workspace"), CARGO_TARGET_DIR=str(target),
               CARGO_PROFILE_DEV_DEBUG="line-tables-only", TEMP=str(root), TMP=str(root))
    env.pop("AUTO_DELIVERY_SOURCE_ROOT", None)
    env.pop("AUTO_LANG_CRATE", None)
    env.pop("AUTO_A2R_BODY", None)
    env.pop("AUTO_API_LENIENT", None)
    def run(command, name, cwd=repo, expect=0, timeout=300):
        with (root / name).open("w", encoding="utf-8") as log:
            result = subprocess.run(command, cwd=cwd, env=env, stdout=log, stderr=subprocess.STDOUT,
                                    text=True, timeout=timeout)
        output = (root / name).read_text(encoding="utf-8")
        assert result.returncode == expect, f"{name}: exit {result.returncode}: {output[-6000:]}"
        return output
    generated = json.loads(run([str(args.exe.resolve()), "generate", str(project)], "generation.log").splitlines()[-1])
    member = Path(generated["member"])
    manifest = member / "Cargo.toml"
    assert "auto-ui" not in manifest.read_text(encoding="utf-8")
    main_rs = (member / "src/main.rs").read_text(encoding="utf-8")
    assert "auto_media_core::photo_service" in main_rs and "auto_ui_runtime::" not in main_rs
    command = ["cargo", "build", "--manifest-path", str(manifest), "--target-dir", str(target), "-j", "4"]
    run(command, "build.log", cwd=Path(generated["workspace"]), timeout=420)
    run(command + ["--locked"], "locked-build.log", cwd=Path(generated["workspace"]))
    metadata = json.loads(run(["cargo", "metadata", "--manifest-path", str(manifest), "--locked", "--format-version", "1"], "metadata.json", cwd=Path(generated["workspace"])))
    forbidden = [p["name"] for p in metadata["packages"] if p["name"].startswith(("auto-ui", "auto-man", "tauri"))]
    assert not forbidden, forbidden
    assert not any(p["name"] == "auto-lang" for p in metadata["packages"]), "unused facade entered selected closure"
    run([str(args.exe.resolve()), "freshness", str(project)], "fresh.log")
    with socket.socket() as reservation:
        reservation.bind(("127.0.0.1", 0))
        port = reservation.getsockname()[1]
    photos = root / "photos"
    photos.mkdir()
    photo_bytes = png()
    (photos / "fixture.png").write_bytes(photo_bytes)
    env.update(AUTO_HTTP_PORT=str(port), AUTO_PHOTO_ROOT=str(photos),
               AUTO_HTTP_SERVICE_JSON=json.dumps({"profile": "development", "listen": {"addr": "127.0.0.1", "port": port},
                   "allowed_hosts": ["127.0.0.1"], "features": {"photo_scan": True}, "limits": {"drain_timeout_ms": 1000}}))
    receipt = json.loads((member / "generation.json").read_text(encoding="utf-8"))
    baked = receipt["assembly"]["consumer_fingerprint"]
    def request(path, method="GET"):
        connection = http.client.HTTPConnection("127.0.0.1", port, timeout=2)
        try:
            connection.request(method, path)
            response = connection.getresponse()
            return response.status, dict(response.getheaders()), response.read()
        finally:
            connection.close()
    executable = target / "debug" / (generated["name"] + (".exe" if os.name == "nt" else ""))
    with (root / "stdout.log").open("w", encoding="utf-8") as stdout, (root / "stderr.log").open("w", encoding="utf-8") as stderr:
        child = subprocess.Popen([str(executable)], cwd=project, env=env, stdout=stdout, stderr=stderr)
        try:
            deadline = time.monotonic() + 30
            while time.monotonic() < deadline:
                assert child.poll() is None, (root / "stderr.log").read_text(encoding="utf-8")
                try:
                    status, _, body = request("/__auto/health/ready")
                    if status == 200 and json.loads(body).get("assembly_fingerprint") == baked:
                        break
                except (OSError, http.client.HTTPException):
                    pass
                time.sleep(0.05)
            else:
                raise AssertionError("actual generation identity never ready")
            status, _, body = request("/value")
            assert status == 200 and json.loads(body) == token, (status, body)
            status, _, body = request("/api/photos/scan")
            listing = json.loads(body)
            assert status == 200 and len(listing["entries"]) == 1, listing
            entry = listing["entries"][0]
            assert (entry["name"], entry["width"], entry["height"]) == ("fixture.png", 3, 2)
            status, _, full = request(urllib.parse.urlsplit(entry["full_url"]).path)
            assert status == 200 and full == photo_bytes
            status, headers, thumbnail = request(urllib.parse.urlsplit(entry["thumb_url"]).path)
            assert status == 200 and jpeg_size(thumbnail) == (3, 2), (status, thumbnail[:20])
            assert headers.get("content-type", "").startswith("image/jpeg")
            assert request("/api/photos/scan?dir=..")[0] == 400
            assert request("/api/photos/full/not-a-known-token")[0] == 404
            status, _, body = request("/__auto/shutdown", "POST")
            assert status == 202 and json.loads(body)["draining"] is True
            assert child.wait(timeout=15) == 0
            diagnostics = (root / "stdout.log").read_text(encoding="utf-8")
            assert "AUTO_SERVICE_STOPPED" in diagnostics, diagnostics
        finally:
            if child.poll() is None:
                # Failed oracle cleanup only; a forced exit is never green.
                child.kill()
                child.wait(timeout=5)
    api.write_text(original + "\n// mutation must invalidate receipt\n", encoding="utf-8")
    run([str(args.exe.resolve()), "freshness", str(project)], "stale.log", expect=1)
    api.write_text(original, encoding="utf-8")
    run([str(args.exe.resolve()), "freshness", str(project)], "restored-fresh.log")
    summary = {"scope": "production-neutral-generation", "evidence": str(root), "exact_business_value": True,
               "photo_scan_full_thumbnail": True, "traversal": 400, "unknown_token": 404,
               "normal_build_dev_packages": len(metadata["packages"]), "forbidden_packages": forbidden,
               "own_graceful_stop": True, "fresh_stale_restored": True,
               "photo_sha256": hashlib.sha256(photo_bytes).hexdigest()}
    (root / "result.json").write_text(json.dumps(summary, indent=2), encoding="utf-8")
    print(json.dumps(summary))


if __name__ == "__main__":
    main()
