"""PLAN-738 T-09 scope audit: classify every file changed in 2c1b4a763^..2c1b4a763.

Method: for each modified .rs file, format the PARENT blob with the same rustfmt
(default config, edition 2021) and byte-compare with the CHILD blob.
  equal      -> pure_format_verified
  not equal  -> residual (semantic or mixed) -> saved for review
Non-.rs files are classified by path/type. Output: JSON to --out.
"""
import subprocess, sys, os, json, tempfile, hashlib

REPO = r"D:/autostack/.wt/lang-738/auto-lang"
COMMIT = "2c1b4a763"
PARENT = COMMIT + "^"
EDITION = "2021"

def git(*args, binary=False):
    r = subprocess.run(["git", "-C", REPO, *args],
                       capture_output=True, text=not binary)
    if r.returncode != 0:
        raise RuntimeError(f"git {args} failed: {r.stderr[:500]}")
    return r.stdout if not binary else r.stdout.encode() if False else r.stdout

def git_bytes(*args):
    r = subprocess.run(["git", "-C", REPO, *args], capture_output=True)
    if r.returncode != 0:
        raise RuntimeError(f"git {args} failed: {r.stderr[:500]}")
    return r.stdout

def sha256(b: bytes) -> str:
    return hashlib.sha256(b).hexdigest()

def main(out_path, residual_dir):
    files = git("diff", "--name-only", f"{PARENT}..{COMMIT}").splitlines()
    os.makedirs(residual_dir, exist_ok=True)
    results = []
    stats = {}
    for i, f in enumerate(files):
        status = git("diff", "--name-status", "--", f, PARENT, COMMIT).split("\t")[0]
        child = git_bytes("show", f"{COMMIT}:{f}")
        parent = git_bytes("show", f"{PARENT}:{f}")
        rec = {
            "path": f,
            "status": "M",
            "child_sha256": sha256(child),
            "parent_sha256": sha256(parent),
        }
        if not f.endswith(".rs"):
            rec["class"] = "non_rust"
            cls = "non_rust"
        else:
            with tempfile.NamedTemporaryFile(suffix=".rs", delete=False) as tf:
                tf.write(parent)
                tmp = tf.name
            try:
                fmt = subprocess.run(
                    ["rustfmt", "--edition", EDITION, "--emit", "stdout", "--quiet", tmp],
                    capture_output=True)
                if fmt.returncode != 0:
                    rec["class"] = "fmt_error"
                    rec["fmt_stderr"] = fmt.stderr.decode("utf-8", "replace")[:800]
                    cls = "fmt_error"
                elif fmt.stdout == child:
                    rec["class"] = "pure_format_verified"
                    cls = "pure_format_verified"
                else:
                    # residual: write formatted-parent and child for diff review
                    safe = f.replace("/", "__")
                    with open(os.path.join(residual_dir, safe + ".fmt-parent.rs"), "wb") as h:
                        h.write(fmt.stdout)
                    with open(os.path.join(residual_dir, safe + ".child.rs"), "wb") as h:
                        h.write(child)
                    with open(os.path.join(residual_dir, safe + ".parent.rs"), "wb") as h:
                        h.write(parent)
                    rec["class"] = "residual_semantic_or_mixed"
                    rec["residual_file"] = safe
                    cls = "residual"
            finally:
                os.unlink(tmp)
        stats[cls] = stats.get(cls, 0) + 1
        results.append(rec)
        if (i + 1) % 50 == 0:
            print(f"[{i+1}/{len(files)}] stats={stats}", flush=True)
    payload = {
        "repo": REPO, "commit": COMMIT, "parent": PARENT,
        "rustfmt": subprocess.run(["rustfmt", "--version"], capture_output=True, text=True).stdout.strip(),
        "edition": EDITION, "config": "default (no rustfmt.toml in repo)",
        "total": len(files), "stats": stats,
        "files": results,
    }
    with open(out_path, "w", encoding="utf-8") as h:
        json.dump(payload, h, indent=1, ensure_ascii=False)
    print("FINAL:", stats)

if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
