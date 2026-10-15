"""PLAN-785 T-03/T-04: UI-form external consumer replay (779 R3 method).

The independent fixture crate cannot resolve a fresh UI lockfile (wgpu-hal
version drift; same class as 779 R3 "独立fixture新UI锁解析冲突"). Boundary
(per plan §5, mirroring 779): UI external proof compiles the UNCHANGED fixture
source against the workspace-locked default-build (ui-iced) rlibs via
`rustc --test` + `--cfg feature="ui-host"` — workspace lock identity, NOT a
new independent lock release. Run from anywhere; paths are absolute.
"""
import hashlib, json, subprocess, sys
from pathlib import Path

root = Path("D:/autostack/.wt/lang-785/auto-lang")
out = Path(__file__).resolve().parent
scratch = root / "scratch/p785-ui-form"
scratch.mkdir(parents=True, exist_ok=True)
source = root / "test/fixtures/modularity/clipboard-provider-consumer/tests/clipboard_vm.rs"
frozen = out / "clipboard_vm.rs"
frozen.write_bytes(source.read_bytes())
(scratch / "clipboard_vm.rs").write_bytes(frozen.read_bytes() + (out / "extra-controls.rs").read_bytes())

with (out / "ui-build.jsonl").open("w", encoding="utf-8") as stdout, \
     (out / "ui-build.log").open("w", encoding="utf-8") as stderr:
    q = subprocess.run(
        ["cargo", "build", "-p", "auto-lang", "--lib", "--locked", "--message-format=json"],
        cwd=root, stdout=stdout, stderr=stderr, timeout=1800,
        env={**__import__("os").environ, "RUSTC_WRAPPER": ""},
    )
assert q.returncode == 0, "UI library build failed"

libs = {}
native = set()
for line in (out / "ui-build.jsonl").read_text().splitlines():
    try:
        r = json.loads(line)
    except json.JSONDecodeError:
        continue
    if r.get("reason") == "build-script-executed":
        native.update(r.get("linked_paths", []))
    if r.get("reason") == "compiler-artifact" and r["target"]["name"] in ("auto_lang", "auto_val"):
        for p in r["filenames"]:
            if p.endswith(".rlib"):
                libs[r["target"]["name"]] = p
assert set(libs) == {"auto_lang", "auto_val"}, libs

exe = scratch / "ui-consumer.exe"
cmd = ["rustc", "--edition", "2021", "--test", str(scratch / "clipboard_vm.rs"),
       "--cfg", 'feature="ui-host"',
       "-L", "dependency=" + str(root / "target/debug/deps"), "-o", str(exe)]
for name, path in libs.items():
    cmd += ["--extern", name + "=" + path]
for p in sorted(native):
    cmd += ["-L", p]
with (out / "ui-consumer-build.log").open("w", encoding="utf-8") as f:
    q = subprocess.run(cmd, cwd=root, stdout=f, stderr=subprocess.STDOUT, timeout=300)
assert q.returncode == 0, "probe compilation failed"

with (out / "ui-consumer.log").open("w", encoding="utf-8") as f:
    q = subprocess.run([str(exe), "--nocapture", "--test-threads=1"], cwd=root,
                       stdout=f, stderr=subprocess.STDOUT, timeout=120)

record = dict(
    head=subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip(),
    source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),
    frozen_source_unchanged=frozen.read_bytes() == source.read_bytes(),
    method="rustc --test against workspace-locked default(ui-iced) rlibs; --cfg feature=ui-host; NOT an independent UI lock resolution",
    compile_command=cmd,
    exit=q.returncode,
)
(out / "ui-consumer.json").write_text(json.dumps(record, ensure_ascii=False), encoding="utf-8")
print(json.dumps({k: record[k] for k in ["head", "source_sha256", "frozen_source_unchanged", "exit"]}, ensure_ascii=False))
print("\n".join((out / "ui-consumer.log").read_text(encoding="utf-8", errors="replace").splitlines()[-25:]))
sys.exit(q.returncode)
