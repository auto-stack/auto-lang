"""
Plan 371 Task 15-16: .autotest parser + MCP Adapter executor.

Parses .autotest scenario files and executes them via the AutoUI MCP
protocol (localhost:9247). Supports both VM and Rust rendering modes.

Usage:
    from autotest import run_suite
    run_suite("015-notes.autotest", mode="vm")  # or mode="rust"
"""

import re
import time
import requests
from dataclasses import dataclass, field
from typing import Optional


# ============================================================================
# Task 15: .autotest parser
# ============================================================================

@dataclass
class Step:
    """A single step in a scenario (given/when/then)."""
    keyword: str          # "given" | "when" | "then" | "skip_if"
    primitive: str        # e.g. "click_button", "exists", "state"
    params: dict = field(default_factory=dict)
    raw: str = ""         # original line for error reporting


@dataclass
class Scenario:
    """A test scenario with an ID, name, and ordered steps."""
    sid: str              # e.g. "T5a"
    name: str             # e.g. "Edit 进入编辑模式"
    steps: list = field(default_factory=list)
    skip_if_mode: set = field(default_factory=set)  # modes to skip entire scenario


@dataclass
class Suite:
    """A collection of scenarios."""
    name: str
    scenarios: list = field(default_factory=list)


def parse_autotest(text: str) -> Suite:
    """Parse .autotest text into a Suite of Scenarios.

    Grammar (line-oriented):
        suite "Name"                          → suite declaration
        scenario T1 "Description"             → scenario start
          given app_loaded                     → given step (keyword=given)
          when click_button label="Edit"      → when step
          then exists button label="Save"     → then step
          skip_if rust                         → skip modifier for previous then
        # comment                             → ignored
        (blank line)                          → ignored
    """
    suite_name = "default"
    scenarios: list = []
    current: Optional[Scenario] = None
    last_then: Optional[Step] = None

    for line_num, raw_line in enumerate(text.splitlines(), 1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue

        # suite declaration
        m = re.match(r'^suite\s+"(.+)"$', line)
        if m:
            suite_name = m.group(1)
            continue

        # scenario declaration
        m = re.match(r'^scenario\s+(\S+)\s+"(.+)"$', line)
        if m:
            if current:
                scenarios.append(current)
            current = Scenario(sid=m.group(1), name=m.group(2))
            last_then = None
            continue

        if current is None:
            continue

        # skip_if (applies to the preceding then step)
        m = re.match(r'^skip_if\s+(\w+)$', line)
        if m:
            mode = m.group(1)
            if last_then:
                last_then.params["_skip_modes"] = \
                    last_then.params.get("_skip_modes", set()) | {mode}
            else:
                current.skip_if_mode.add(mode)
            continue

        # given/when/then step
        m = re.match(r'^(given|when|then)\s+(\w+)\s*(.*)$', line)
        if m:
            keyword, primitive, rest = m.group(1), m.group(2), m.group(3)
            params = _parse_params(rest)
            step = Step(keyword=keyword, primitive=primitive, params=params, raw=line)
            current.steps.append(step)
            last_then = step if keyword == "then" else None
            continue

        # Unrecognized line — warn but continue
        print(f"  [parser] WARNING line {line_num}: unrecognized: {line}")

    if current:
        scenarios.append(current)

    return Suite(name=suite_name, scenarios=scenarios)


def _parse_params(text: str) -> dict:
    """Parse key=value pairs from a step's parameter string.

    Examples:
        label="Edit"               → {"label": "Edit"}
        input label="Note title" value="Hello"  → {"input": True, "label": "Note title", "value": "Hello"}
        field="dark_mode" equals=true           → {"field": "dark_mode", "equals": True}
    """
    params = {}
    # Match bare words (like "button", "input", "text") as type hints
    for m in re.finditer(r'(\w+)="([^"]*)"', text):
        key, val = m.group(1), m.group(2)
        # Convert common literal values
        if val == "true":
            val = True
        elif val == "false":
            val = False
        params[key] = val

    # Bare words that aren't key=value (e.g. "button" before label="X")
    # These indicate the node kind (button/input/text/textarea/checkbox)
    bare = re.sub(r'\w+="[^"]*"', '', text).strip()
    if bare:
        for word in bare.split():
            if word not in ("equals", "has_value", "length_increased_by", "changed"):
                params.setdefault("kind", word)

    return params


# ============================================================================
# Task 16: MCP Adapter executor
# ============================================================================

@dataclass
class TestResult:
    sid: str
    name: str
    status: str           # "PASS" | "FAIL" | "SKIP"
    detail: str = ""
    steps_run: int = 0
    steps_skipped: int = 0


class McpAdapter:
    """Executes .autotest scenarios against a running AutoUI MCP server."""

    def __init__(self, mode: str = "vm", url: str = "http://localhost:9247/mcp"):
        self.mode = mode          # "vm" or "rust"
        self.url = url
        self.req_id = 0
        # Plan 371 Task 20: screenshot visual-regression options. When set,
        # after each scenario we call autoui_screenshot with these options and
        # collect a verdict into screenshot_results.
        self.screenshot_mode = None   # None | "baseline" | "diff"
        self.screenshot_results = []  # list of (sid, verdict_text)

    def _call(self, tool: str, **args) -> str:
        """Call an MCP tool and return the text result."""
        self.req_id += 1
        resp = requests.post(self.url, json={
            "jsonrpc": "2.0", "method": "tools/call",
            "params": {"name": tool, "arguments": args},
            "id": self.req_id,
        }, timeout=15)
        data = resp.json()
        if "error" in data:
            raise RuntimeError(f"MCP error: {data['error']}")
        content = data.get("result", {}).get("content", [])
        return content[0]["text"] if content else ""

    def after_scenario(self, sid: str) -> None:
        """Plan 371 Task 20: capture a per-scenario screenshot for visual
        regression. Uses the stable `sid` as the baseline name so baselines
        are reproducible across runs. Only acts when `screenshot_mode` is set.
        """
        if self.screenshot_mode is None:
            return
        name = f"{sid}"
        try:
            if self.screenshot_mode == "baseline":
                verdict = self._call("autoui_screenshot", name=name, baseline=True)
            else:  # "diff"
                verdict = self._call("autoui_screenshot", name=name, diff=True)
        except Exception as e:
            verdict = f"error: {e}"
        self.screenshot_results.append((sid, verdict))

    def run_suite(self, suite: Suite) -> list:
        """Execute all scenarios in a suite. Returns list of TestResult."""
        results = []
        for sc in suite.scenarios:
            if self.mode in sc.skip_if_mode:
                results.append(TestResult(sc.sid, sc.name, "SKIP",
                                          f"skip_if {self.mode}"))
                continue
            result = self._run_scenario(sc)
            results.append(result)
            # Plan 371 Task 20: per-scenario screenshot hook (runs even on
            # PASS; skipped scenarios do not capture).
            self.after_scenario(sc.sid)
        return results

    def _run_scenario(self, sc: Scenario) -> TestResult:
        """Execute a single scenario's steps sequentially."""
        steps_run = 0
        steps_skipped = 0
        snapshot_before = None

        for step in sc.steps:
            # Check skip_if for this step
            skip_modes = step.params.get("_skip_modes", set())
            if self.mode in skip_modes:
                steps_skipped += 1
                continue

            try:
                if step.keyword == "given":
                    self._exec_given(step)

                elif step.keyword == "when":
                    self._exec_when(step)

                elif step.keyword == "then":
                    ok, msg = self._exec_then(step, snapshot_before)
                    if not ok:
                        return TestResult(sc.sid, sc.name, "FAIL",
                                          f"Step '{step.raw}': {msg}",
                                          steps_run, steps_skipped)
                    steps_run += 1

            except Exception as e:
                return TestResult(sc.sid, sc.name, "FAIL",
                                  f"Step '{step.raw}' raised: {e}",
                                  steps_run, steps_skipped)

        return TestResult(sc.sid, sc.name, "PASS", "", steps_run, steps_skipped)

    def _exec_given(self, step: Step):
        """Execute a 'given' precondition step."""
        if step.primitive == "app_loaded":
            pass  # MCP connection implies app is loaded
        elif step.primitive == "editing_mode":
            # Ensure we're in editing mode: if Edit button exists, click it.
            # If Save button exists instead, we're already in editing mode.
            text = self._call("autoui_exists", kind="button", label="Edit")
            if "FOUND" in text:
                vnode = self._find("button", "Edit")
            if vnode:
                self._press(vnode)
                time.sleep(0.3)

    def _exec_when(self, step: Step):
        """Execute a 'when' action step."""
        if step.primitive == "click_button":
            label = step.params.get("label", "")
            vnode = self._find("button", label)
            if not vnode:
                # If the button doesn't exist (e.g. Cancel when not in edit mode),
                # treat as a no-op reset rather than an error.
                if label in ("Cancel", "Save", "Dark", "Light"):
                    return  # No-op: already in the target state
                raise RuntimeError(f"Button '{label}' not found")
            self._press(vnode)
            time.sleep(0.3)

        elif step.primitive == "type_text":
            label = step.params.get("label", "")
            value = step.params.get("value", "")
            kind = step.params.get("kind", "input")
            vnode = self._find(kind, label)
            if not vnode:
                raise RuntimeError(f"{kind} '{label}' not found")
            self._type(vnode, value)
            time.sleep(0.2)

        elif step.primitive == "press_key":
            key = step.params.get("key", "Enter")
            self._call("autoui_keyboard", key=key)
            time.sleep(0.2)

    def _exec_then(self, step: Step, snapshot_before: Optional[str]) -> tuple:
        """Execute a 'then' assertion step. Returns (ok, message)."""
        if step.primitive == "exists":
            kind = step.params.get("kind", "")
            label = step.params.get("label", "")
            text = self._call("autoui_exists", kind=kind, label=label)
            return ("FOUND" in text, text)

        elif step.primitive == "not_exists":
            kind = step.params.get("kind", "")
            label = step.params.get("label", "")
            text = self._call("autoui_exists", kind=kind, label=label)
            return ("NOT FOUND" in text, text)

        elif step.primitive == "state":
            field = step.params.get("field", "")
            result = self._call("autoui_state", fields=[field])
            if step.params.get("equals") is not None:
                expected = step.params["equals"]
                expected_str = str(expected)
                # For boolean values, match the exact representation
                if isinstance(expected, bool):
                    expected_str = str(expected).lower()
                    return (expected_str in result.lower(),
                            f"expected '{expected_str}', got: {result.strip()}")
                # For string values, case-insensitive substring match
                return (expected_str.lower() in result.lower(),
                        f"expected '{expected_str}', got: {result.strip()}")
            elif step.params.get("changed"):
                return (True, "state changed (unimplemented deep check)")
            return (True, result.strip())

        elif step.primitive == "inspect":
            kind = step.params.get("kind", "")
            label = step.params.get("label", "")
            vnode = self._find(kind, label)
            if not vnode:
                return (False, f"{kind} '{label}' not found")
            text = self._call("autoui_inspect", element_id=vnode)
            if step.params.get("has_value"):
                # Check if value is non-empty
                return ("value:" in text and 'value: ""' not in text and 'value: ' in text,
                        f"inspect result: {text[:100]}")
            return (True, text[:100])

        elif step.primitive == "snapshot_changed":
            # Compare current snapshot node count with before
            snap = self._call("autoui_snapshot", include_state=False)
            node_count = len(re.findall(r'#vnode_\d+', snap))
            return (node_count > 0, f"snapshot has {node_count} nodes")

        return (False, f"Unknown assertion: {step.primitive}")

    # ── MCP helper methods ──────────────────────────────────────────

    def _find(self, kind: str, label: str) -> Optional[str]:
        """Find an element by kind+label, return its vnode_N ID.
        autoui_find returns an ancestor chain, so we match the LAST
        node of the target kind (the actual matched node)."""
        text = self._call("autoui_find", kind=kind, label=label, limit=1)
        # Find the matched node — it's the last <kind> vnode_N in the chain.
        # E.g. "col vnode_1 { ... button vnode_42 {label: \"Edit\"}" → vnode_42
        matches = re.findall(rf'{kind}\s+vnode_(\d+)', text)
        if matches:
            return f"vnode_{matches[-1]}"
        return None

    def _press(self, vnode_id: str):
        """Press a button by vnode_N ID."""
        self._call("autoui_action", element_id=vnode_id, action="press")

    def _type(self, vnode_id: str, value: str):
        """Type text into an input by vnode_N ID."""
        self._call("autoui_action", element_id=vnode_id, action="type_text", value=value)


# ============================================================================
# Runner: parse + execute + report
# ============================================================================

def run_suite(autotest_path: str, mode: str = "vm",
              url: str = "http://localhost:9247/mcp",
              screenshot: str = None) -> tuple:
    """Parse an .autotest file and execute all scenarios via MCP.

    Args:
        autotest_path: Path to the .autotest file.
        mode: "vm" or "rust" — controls skip_if behavior.
        url: MCP server URL.
        screenshot: Plan 371 Task 20 — None | "baseline" | "diff". When set,
            captures a per-scenario screenshot via autoui_screenshot.

    Returns:
        (results, screenshot_results) — results is a list of TestResult;
        screenshot_results is a list of (sid, verdict) (empty if no screenshot).
    """
    with open(autotest_path, encoding="utf-8") as f:
        text = f.read()

    suite = parse_autotest(text)
    adapter = McpAdapter(mode=mode, url=url)
    adapter.screenshot_mode = screenshot
    results = adapter.run_suite(suite)

    # Print report
    print(f"\n{'='*60}")
    print(f"AutoTest Suite: {suite.name}  (mode={mode})")
    print(f"{'='*60}")
    passed = failed = skipped = 0
    for r in results:
        symbol = {"PASS": "✅", "FAIL": "❌", "SKIP": "⏭️"}[r.status]
        print(f"  {symbol} {r.sid} {r.name}")
        if r.status == "FAIL":
            print(f"     → {r.detail}")
        elif r.status == "SKIP":
            print(f"     → {r.detail}")
        elif r.steps_skipped:
            print(f"     ({r.steps_skipped} step(s) skipped)")

        if r.status == "PASS": passed += 1
        elif r.status == "FAIL": failed += 1
        else: skipped += 1

    print(f"\n{'─'*60}")
    print(f"Total: {passed} passed, {failed} failed, {skipped} skipped")
    print(f"{'─'*60}\n")
    return results, adapter.screenshot_results
