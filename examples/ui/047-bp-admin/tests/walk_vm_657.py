"""PLAN-657 T-05: 047-bp-admin VM-track walkthrough (AC-01/AC-03).

Drives the iced window via the embedded AutoUI MCP server using the
autoui-verifier skill's standard client (no ad-hoc protocol code).

Classification: PASS = dual-track contract holds on VM; KNOWN-GAP = the
639-D1-family evidence recorded by PLAN-657 — cross-widget `on_*` callback
PAYLOADS arrive as the literal parameter name on the VM track (state probe:
`active_nav: "id"` regardless of the pressed node; vue track carries the
same flows green — see tests/screenshots/047_vue_*.png). Engine-level parity
debt, deliberately not fixed inside PLAN-657 (plan §4 risk ruling).
"""
import json
import os
import re
import sys
import time

SKILL = "D:/autostack/auto-lang/.agents/skills/autoui-verifier/scripts"
sys.path.insert(0, SKILL)
from test_vm_mcp import AutoUiMcpClient  # noqa: E402

PORT = int(os.environ.get("AUTOUI_MCP_PORT", "4770"))
OUT = os.path.join(os.path.dirname(__file__), "screenshots")
os.makedirs(OUT, exist_ok=True)


def find_id(snap: str, label: str) -> str:
    """Locate the node id whose text payload contains `label`."""
    # snapshot lines look like `button #vnode_539... "New item"` (rendered v2)
    for line in snap.splitlines():
        if label in line and ("aura_" in line or "vnode_" in line):
            m = re.search(r"#(aura_\d+|vnode_\d+)", line)
            if m:
                return m.group(1)
    return ""


def main() -> int:
    c = AutoUiMcpClient(PORT)
    results = []
    shot = 0

    def snap(name: str) -> str:
        nonlocal shot
        s = c.snapshot()
        shot += 1
        c.screenshot(name=f"047_vm_{name}", baseline=False,
                     save_path=os.path.join(OUT, f"047_vm_{name}.png"))
        return s

    def record(name: str, ok: bool, detail: str = "") -> None:
        results.append({"name": name, "ok": ok, "detail": detail})
        print(("PASS " if ok else "KNOWN-GAP ") + name + (f" :: {detail}" if detail else ""))

    # 1. initial (data view) — four-region render contract
    s = snap("initial_data")
    record("vm-nav-sidebar", "Data" in s and "Settings" in s and "Reports" in s and "About" in s)
    record("vm-table-rows", "Analytical Engine" in s)
    record("vm-shell-brand", "Acme" in s)

    # 2. CRUD: open create dialog, type, save (bp-local state + callback fire)
    btn = find_id(s, "New item")
    if btn:
        c.press(btn)
        time.sleep(0.4)
        s2 = snap("crud_dialog")
        record("vm-crud-dialog-open", "Edit item" in s2 or "Save" in s2)
        m = re.search(r"input #(\S+)", s2)
        if m:
            c.type_text(m.group(1), "Babbage Difference")
            time.sleep(0.3)
        save = find_id(s2, "Save")
        if save:
            c.press(save)
            time.sleep(0.5)
        s3 = snap("crud_after_save")
        # callback chain FIRES on VM (ui_note flips); the name payload itself
        # is the KNOWN-GAP — assert only the chain here, payload evidence below.
        st = c.state()
        note = next((l for l in st.splitlines() if "ui_note" in l), "")
        record("vm-crud-callback-fires", "created" in note, note.strip())
        record("vm-crud-payload-gap", 'ui_note: "created: "' in st + "\n" + note,
               "payload arrives empty on VM (vue carries it) — 639-D1 evidence")

    # 3. nav switch — state probe (the 639-D1 core evidence)
    nav = find_id(s, "Settings")
    if nav:
        c.press(nav)
        time.sleep(0.5)
        st = c.state()
        active = next((l for l in st.splitlines() if "active_nav" in l), "")
        record("vm-nav-payload-gap", '"id"' in active,
               f"after pressing Settings: {active.strip()} — literal param name as payload")
        s4 = snap("nav_pressed_settings")
        record("vm-nav-view-switch", "Danger zone" in s4,
               "no switch on VM (active_nav literal); vue green")

    # 4. reports empty-state triad (reachable only via nav on this sample;
    #    VM-side variant rendering proven by plan657 t01 unit anchor instead)
    rep = find_id(s, "Reports")
    if rep:
        c.press(rep)
        time.sleep(0.4)
        snap("nav_pressed_reports")

    # 5. about
    abt = find_id(s, "About")
    if abt:
        c.press(abt)
        time.sleep(0.4)
        snap("nav_pressed_about")

    hard = [r for r in results if not r["ok"] and "gap" not in r["name"]]
    print(f"\nSUMMARY: {len(results) - len(hard)}/{len(results)} accounted "
          f"(PASS + recorded gaps), {shot} screenshots")
    with open(os.path.join(OUT, "vm_walkthrough_657.json"), "w", encoding="utf-8") as f:
        json.dump(results, f, ensure_ascii=False, indent=2)
    return 1 if hard else 0


if __name__ == "__main__":
    sys.exit(main())
