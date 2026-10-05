#!/usr/bin/env python3
"""PLAN-743 ACC 自举盘点工具：Auto 自举源码的保守词法扫描与可重跑 manifest。

扫描的是"词法观察"，不是语义解析结论：
- 产出的所有候选（声明、use 边、能力证据、运行时调用）都是 compiler-source-demand
  的观察证据，不等于 AC/ACC 已支持该能力（compiled-language-support 结论只能来自
  人工审定层 manual-decisions.json）。
- 未解析接收者、同名方法、动态调用一律标记 unknown，不以"扫描没找到"证明不使用。
- 注释与字符串内容经状态机屏蔽后再匹配，防止伪代码污染清单。

manifest 字节稳定性：相同输入 + 相同人工结论 => 输出字节一致。
不写入时间戳与机器路径；HEAD 仅作审计字段，漂移验证只用受管输入 hash。

用法（PLAN-743 §5.1）：
  python scripts/acc_inventory.py --root . --output docs/reports/743-acc-hir-contract --write
  python scripts/acc_inventory.py --root . --output docs/reports/743-acc-hir-contract --check
  python -m unittest discover -s scripts/tests -p test_acc_inventory.py

退出码：0 成功；1 漂移/校验失败（陈旧 manifest、注册表漂移、人工结论过期）；
2 用法/环境错误（缺参数、缺输入文件、输出目录不安全）。
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path, PurePosixPath

FORMAT_VERSION = 1
PLAN_ID = "PLAN-743"
MANIFEST_NAME = "source-manifest.json"
SUMMARY_NAME = "scan-summary.md"
DECISIONS_NAME = "manual-decisions.json"

# 受管输入：8 个源模块（七 lib + CLI）+ 注册表宿主。此清单是工具合同的组成部分；
# 扩大盘点范围须作为工具代码修订走复审，不是输出端静默扩展。
SOURCE_MODULES = [
    "auto/lib/token.at",
    "auto/lib/lexer.at",
    "auto/lib/parser.at",
    "auto/lib/typeinfo.at",
    "auto/lib/codegen.at",
    "auto/lib/engine.at",
    "auto/lib/a2r.at",
    "auto/aavm.at",
]
REGISTRY_HOST = "crates/auto-lang/src/lib.rs"
MANAGED_INPUTS = SOURCE_MODULES + [REGISTRY_HOST]
REGISTRY_SYMBOL = "AUTO_LIB_FILES_V2"
REGISTRY_RE = re.compile(REGISTRY_SYMBOL + r"\s*:\s*&\[&str\]\s*=\s*&\[(.*?)\]", re.S)
REGISTRY_ENTRY_RE = re.compile(r'"([^"]+)"')

# 运行时命名空间：接收者为可证明宿主内建时才标 native-runtime
# （List 容器、IO 行读、process 参数、File 文件读；engine.at nat#1015/1016
# File.read_text[_range] 等宿主内建证据）。
# QA-02（PLAN-743 r2）：不再按"方法名像内建"判定 native——本地类型可声明
# 同名方法（Meter.len/CG.new/Ar.new 实证），变量接收者一律 unknown 交人工。
NATIVE_NAMESPACES = {"List", "IO", "process", "File"}
BARE_NATIVES = {"print"}

TOP_DECL_RE = re.compile(r"^(pub\s+)?(fn|type|enum)\s+([A-Za-z_]\w*)")
METHOD_DECL_RE = re.compile(r"^\s+(pub\s+)?(static\s+)?fn\s+([A-Za-z_]\w*)")
FIELD_RE = re.compile(r"^\s+([A-Za-z_]\w*)\s+([A-Za-z_]\w*[<>]?(?:<[^{}]*>)?)\s*$")
ENUM_CASE_RE = re.compile(r"^\s*([A-Za-z_]\w*)\s*(\([^()]*\))?\s*$")
USE_RE = re.compile(r"^use\s+([A-Za-z_][\w.]*)\s*:\s*(.+)$")
QUALIFIED_CALL_RE = re.compile(r"\b([A-Za-z_]\w*)\.([A-Za-z_]\w*)\s*\(")
PIPE_METHOD_RE = re.compile(r"\|>\s*\.([A-Za-z_]\w*)\s*\(")
BARE_CALL_RE = re.compile(r"\b([A-Za-z_]\w*)\s*\(")

# 能力观察键：key -> (regex over masked line, 说明)。观察≠支持；状态只有
# observed/not-observed，implemented/required 判定属人工审定层（T-03）。
CAPABILITY_PATTERNS = [
    ("enum-declaration", re.compile(r"^\s*(pub\s+)?enum\s+\w+"), "enum 类型声明"),
    ("record-type-declaration", re.compile(r"^\s*(pub\s+)?type\s+\w+"), "record/type 声明"),
    ("free-function", re.compile(r"^\s*(pub\s+)?fn\s+\w+"), "自由函数声明"),
    ("type-body-method", re.compile(r"^\s+(pub\s+)?(static\s+)?fn\s+\w+"), "类型体方法声明"),
    ("generic-list-type", re.compile(r"\bList<"), "泛型容器写法 List<T>"),
    ("plain-list-value", re.compile(r"\bList\.new\("), "动态 List 值构造"),
    ("string-type-annotation", re.compile(r"\bstr\b"), "str 字符串类型注记"),
    ("integer-type-annotation", re.compile(r"\bint\b"), "int 整型注记"),
    ("boolean-type-annotation", re.compile(r"\bbool\b"), "bool 布尔注记"),
    ("char-literal", re.compile(r"'(\\.|[^'\\])'"), "字符字面量"),
    ("string-literal", re.compile(r'"'), "字符串字面量"),
    ("if-else", re.compile(r"^\s*(}\s*)?(if|else)\b|\belse\s+is\b"), "if/else 控制流"),
    ("while-loop", re.compile(r"^\s*while\b"), "while 循环"),
    ("for-loop", re.compile(r"^\s*for\b"), "for 循环"),
    ("is-match-statement", re.compile(r"^\s*is\b"), "is 模式匹配语句"),
    ("use-module", re.compile(r"^use\s+[\w.]+\s*:"), "use 模块导入"),
    ("mut-parameter", re.compile(r"\bmut\s+[A-Za-z_]\w*\s+[A-Za-z_]\w*"), "mut 参数"),
    ("return-statement", re.compile(r"\breturn\b"), "return 语句"),
    ("logical-operator", re.compile(r"&&|\|\|"), "逻辑运算符"),
    ("comparison-operator", re.compile(r"==|!=|<=|>="), "比较运算符"),
]
EVIDENCE_CAP = 400  # 每个 (能力, 文件) 最多记录的证据行数；超出记 truncated


def sha256_of(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def mask_comments_and_strings(text: str):
    """屏蔽注释与字符串/字符字面量内容，保留代码骨架与行结构。

    字符串内容整体替换为空格（保留引号占位），注释同理；换行始终保留以维持行号。
    QA-01（PLAN-743 r2）：跨行字符串是合法形态，字面量状态保持到真正闭合；
    转义换行（\\+换行）属字面量续行；EOF 仍未闭合的字符串/字符记异常，
    其起始行之后的全部内容按词法不确定屏蔽，不得生成候选。
    返回 (masked_text, anomalies, multiline_string_start_lines)。
    """
    out = []
    anomalies = []
    multiline_string_starts = []
    string_start_line = None
    state = "code"
    i, n = 0, len(text)
    line = 1
    while i < n:
        c = text[i]
        nxt = text[i + 1] if i + 1 < n else ""
        if c == "\n":
            line += 1
            out.append("\n")
            if state == "line_comment":
                state = "code"
            elif state == "char":
                # 字符字面量不跨行：非法/不支持形态，本行内容已屏蔽，
                # 记异常后在换行恢复，避免单个撇号吞掉后续真实代码
                anomalies.append({"line": line - 1, "kind": "unterminated-char"})
                state = "code"
            # 字符串跨越换行：保持字面量状态（QA-01）。
            # 内容保持屏蔽、行号照常推进；真未闭合由 EOF 兜底标记。
            i += 1
            continue
        if state == "code":
            if c == "/" and nxt == "/":
                state = "line_comment"
                out.append("  ")
                i += 2
                continue
            if c == "/" and nxt == "*":
                state = "block_comment"
                out.append("  ")
                i += 2
                continue
            if c == '"':
                state = "string"
                string_start_line = line
                out.append('"')
                i += 1
                continue
            if c == "'":
                state = "char"
                out.append("'")
                i += 1
                continue
            out.append(c)
            i += 1
        elif state == "line_comment":
            out.append(" ")
            i += 1
        elif state == "block_comment":
            if c == "*" and nxt == "/":
                state = "code"
                out.append("  ")
                i += 2
                continue
            out.append(" ")
            i += 1
        elif state == "string":
            if c == "\\":
                if nxt == "\n":
                    # 转义换行：字面量续行，保留换行维持行号对齐
                    out.append("\n")
                    line += 1
                    i += 2
                    continue
                out.append("  ")
                i += 2
                continue
            if c == '"':
                state = "code"
                if line > string_start_line:
                    multiline_string_starts.append(string_start_line)
                string_start_line = None
                out.append('"')
                i += 1
                continue
            out.append(" ")
            i += 1
        elif state == "char":
            if c == "\\":
                out.append("  ")
                i += 2
                continue
            if c == "'":
                state = "code"
                out.append("'")
                i += 1
                continue
            out.append(" ")
            i += 1
    if state != "code":
        # EOF 兜底：字符串未闭合=从起始行到 EOF 全部按字面量屏蔽
        # （词法不确定形态，禁止尾部内容生成候选），记异常供人工核对
        anomalies.append({"line": string_start_line or line,
                          "kind": "unterminated-%s-at-eof" % state})
    return "".join(out), anomalies, multiline_string_starts


class ModuleScan:
    """单个 .at 模块的词法扫描结果（全部为候选/观察，非语义结论）。"""

    def __init__(self, rel_path: str):
        self.rel_path = rel_path
        self.declarations = []       # {kind, name, line}
        self.methods = []            # {owner, name, line, static}
        self.fields = []             # {owner, name, type, line}
        self.enum_cases = []         # {owner, name, line}
        self.use_edges = []          # {to_module, imports}
        self.calls = []              # {kind, qualifier_or_receiver, name, first_line, count}
        self.capabilities = {}       # key -> [line,...]
        self.anomalies = []
        self.brace_balance = 0
        self.lexically_indeterminate = False

    def to_json(self) -> dict:
        return {
            "path": self.rel_path,
            "declarations": self.declarations,
            "methods": self.methods,
            "fields": self.fields,
            "enum_cases": self.enum_cases,
            "use_edges": self.use_edges,
            "call_candidates": self.calls,
            "capability_evidence": self.capabilities,
            "anomalies": self.anomalies,
            "brace_balance": self.brace_balance,
            "lexically_indeterminate": self.lexically_indeterminate,
        }


def scan_module(rel_path: str, text: str) -> ModuleScan:
    scan = ModuleScan(rel_path)
    masked, anomalies, ml_string_starts = mask_comments_and_strings(text)
    scan.anomalies = anomalies
    if any(a["kind"].endswith("-at-eof") for a in anomalies):
        # 词法不确定：EOF 兜底形态，观察只作部分证据，人工必须核对
        scan.lexically_indeterminate = True
    lines = masked.split("\n")

    # 预收集 use 导入与本地 fn 名，供裸调用分类（词法级，非语义解析）
    imported_symbols = set()
    for raw in lines:
        mu = USE_RE.match(raw.strip()) if raw.strip().startswith("use") else None
        if mu:
            for s in mu.group(2).split(","):
                if s.strip():
                    imported_symbols.add(s.strip())
    local_fns = set()
    for raw in lines:
        m = TOP_DECL_RE.match(raw) if raw else None
        if m and m.group(2) == "fn":
            local_fns.add(m.group(3))

    local_types = set()
    depth = 0
    type_body = None  # name of enclosing `type` for method/field attribution
    enum_body = None
    body_depth = 0

    for idx, raw in enumerate(lines, start=1):
        stripped = raw.strip()
        code = raw

        # ── 声明识别（在深度更新前，按当前深度归类） ──
        if depth == 0:
            m = TOP_DECL_RE.match(code)
            if m:
                kind, name = m.group(2), m.group(3)
                scan.declarations.append({"kind": kind, "name": name, "line": idx})
                if kind == "type":
                    local_types.add(name)
                    type_body, body_depth = name, depth
                elif kind == "enum":
                    local_types.add(name)
                    enum_body, body_depth = name, depth
        elif depth >= 1:
            mm = METHOD_DECL_RE.match(code)
            if mm and type_body is not None:
                scan.methods.append({
                    "owner": type_body,
                    "name": mm.group(3),
                    "line": idx,
                    "static": bool(mm.group(2)),
                })
            elif enum_body is not None and depth == body_depth + 1:
                mc = ENUM_CASE_RE.match(code)
                if mc:
                    scan.enum_cases.append({"owner": enum_body, "name": mc.group(1),
                                            "payload": bool(mc.group(2)), "line": idx})
            elif type_body is not None and depth == body_depth + 1:
                mf = FIELD_RE.match(code)
                if mf and mf.group(1) not in ("if", "for", "while", "return", "is"):
                    scan.fields.append({
                        "owner": type_body,
                        "name": mf.group(1),
                        "type": mf.group(2),
                        "line": idx,
                    })

        # ── use 边 ──
        mu = USE_RE.match(code.strip()) if code.startswith("use") else None
        if mu:
            imports = [s.strip() for s in mu.group(2).split(",") if s.strip()]
            scan.use_edges.append({"to_module": mu.group(1), "imports": imports})

        # ── 能力证据 ──
        for key, rx, _desc in CAPABILITY_PATTERNS:
            if rx.search(code):
                scan.capabilities.setdefault(key, [])
                if len(scan.capabilities[key]) < EVIDENCE_CAP:
                    scan.capabilities[key].append(idx)

        # ── 调用候选（masked 行；声明行的 fn 名剔除防自报） ──
        # R2-QA-01（PLAN-743 r3）：qualified/点/管道 归属全部推迟到
        # _post_classify——那里才有最终本地类型集、宿主优先级与 enclosing
        # owner 方法证据；本地同名（type IO/fn print）必须遮蔽宿主推断。
        code_scan = re.sub(r"\bfn\s+[A-Za-z_]\w*", "fn", code)
        for mq in QUALIFIED_CALL_RE.finditer(code_scan):
            recv, meth = mq.group(1), mq.group(2)
            scan.calls.append({"kind": "unknown-receiver", "receiver": recv,
                               "name": meth, "line": idx, "owner": type_body})
        for mp in PIPE_METHOD_RE.finditer(code_scan):
            meth = mp.group(1)
            scan.calls.append({"kind": "unknown-receiver", "receiver": "|>",
                               "name": meth, "line": idx, "owner": type_body})
        occupied = ([m.span() for m in QUALIFIED_CALL_RE.finditer(code_scan)]
                    + [m.span() for m in PIPE_METHOD_RE.finditer(code_scan)])
        for mb in BARE_CALL_RE.finditer(code_scan):
            if any(s <= mb.start() < e for s, e in occupied):
                continue
            name = mb.group(1)
            if name in ("if", "while", "for", "is", "return", "fn", "use", "enum", "type"):
                continue
            # 前导点=.m() 隐式 self 形态：携带当前 enclosing owner 供升级判定
            dotted = mb.start() > 0 and code_scan[mb.start() - 1] == "."
            if dotted:
                scan.calls.append({"kind": "unknown-receiver", "receiver": ".",
                                   "name": name, "line": idx, "owner": type_body})
                continue
            # 本地/导入符号先于宿主内建判定：本地 fn print 遮蔽内建 print
            if name in local_fns:
                kind = "local-fn"
            elif name in imported_symbols:
                kind = "imported-symbol"
            elif name in BARE_NATIVES:
                kind = "native-runtime"
            else:
                # 本扫描层无法解析（动态导入/宿主注入等）→ 保留 unknown，不丢弃
                kind = "unknown-bare-call"
            scan.calls.append({"kind": kind, "receiver": None, "name": name,
                               "line": idx, "owner": None})

        depth += code.count("{") - code.count("}")
        scan.brace_balance = depth
        if depth <= body_depth and (type_body or enum_body):
            type_body = None
            enum_body = None

    scan.calls = _post_classify(scan, local_types)
    scan.calls = _aggregate_calls(scan.calls)
    # 跨行字符串字面量（QA-01）：合法形态，掩码器保持字面量态到真正闭合；
    # 能力证据=每个跨行字面量的起始行（engine.at:314/333 实证）。
    # 未闭合-at-eof 才是异常（词法不确定），不再与合法跨行混记。
    for start_line in ml_string_starts:
        scan.capabilities.setdefault("multiline-string-literal", [])
        if len(scan.capabilities["multiline-string-literal"]) < EVIDENCE_CAP:
            scan.capabilities["multiline-string-literal"].append(start_line)
    # fn 参数/返回类型证据：从声明行向后拼接至首个 '{'，抽取 (...) 与返回段
    _extract_signature_types(scan, lines)
    return scan


def _post_classify(scan: ModuleScan, local_types: set):
    """主循环后的符号表升级：用本模块完整声明集重分类扫描期无法判定的候选。

    词法级名字匹配，仍属候选而非语义解析；跨模块同名歧义交人工审定层。
    R2-QA-01（PLAN-743 r3）：本地声明先于宿主（本地 type IO/fn print 遮蔽宿主
    推断）；dot/pipe 隐式 self 仅当调用点 enclosing owner 声明了该方法才升级
    （其它 owner 同名/无 owner 不猜）；变量接收者与无证据同名调用保持 unknown。
    返回排序后的候选列表（owner 字段仅判定用，聚合时剥离）。
    """
    local_enum_cases = {c["name"] for c in scan.enum_cases}
    methods_by_owner = {}
    for m in scan.methods:
        methods_by_owner.setdefault(m["owner"], set()).add(m["name"])
    for c in scan.calls:
        n = c["name"]
        recv = c.get("receiver")
        owner = c.get("owner")
        if c["kind"] == "unknown-receiver":
            if recv in (".", "|>"):
                # 隐式 self：仅当前 enclosing owner 声明该方法才升级
                if owner is not None and n in methods_by_owner.get(owner, ()):
                    c["kind"] = "implicit-self-method"
                    c["receiver"] = None
                # 其它 owner 同名/无 owner → 保持 unknown（形态标记保留）
            elif recv in local_types:
                c["kind"] = "type-qualified"
            elif recv in NATIVE_NAMESPACES:
                c["kind"] = "native-runtime"
            # 变量/无法解析接收者 → 保持 unknown-receiver
        elif c["kind"] == "unknown-bare-call":
            if n in local_enum_cases:
                c["kind"] = "enum-variant-construction"
            elif n in local_types:
                c["kind"] = "type-construction"
    return sorted(scan.calls,
                  key=lambda c: (c["kind"], c["name"], str(c.get("receiver"))))


def _aggregate_calls(calls):
    """同名同类别调用聚合计数，附首行证据；输出按 (kind, name, receiver) 稳定排序。"""
    agg = {}
    for c in calls:
        key = (c["kind"], c["name"], c.get("receiver"))
        if key not in agg:
            agg[key] = {"kind": c["kind"], "name": c["name"],
                        "receiver": c.get("receiver"), "first_line": c["line"], "count": 0}
        agg[key]["count"] += 1
    return [agg[k] for k in sorted(agg, key=lambda k: (k[0], k[1], str(k[2])))]


def _extract_signature_types(scan: ModuleScan, lines):
    """从每个 fn 声明行向后拼接至多 6 行，抽取参数类型与返回类型证据。"""
    sig_types = []
    for decl in scan.declarations:
        if decl["kind"] != "fn":
            continue
        start = decl["line"] - 1
        blob = "\n".join(lines[start:start + 6])
        m = re.search(r"fn\s+\w+\s*\(([^)]*)\)\s*([A-Za-z_][\w<>?, ]*)?\s*\{", blob, re.S)
        if not m:
            continue
        params_raw = m.group(1).strip()
        ret = (m.group(2) or "").strip()
        ptypes = []
        if params_raw:
            for p in params_raw.split(","):
                toks = p.split()
                if toks:
                    ptypes.append(toks[-1])
        sig_types.append({"fn": decl["name"], "param_types": ptypes, "ret": ret, "line": decl["line"]})
    scan.declarations = [
        {**d, "param_types": st["param_types"], "ret": st["ret"]}
        if d["kind"] == "fn" and (st := next(
            (s for s in sig_types if s["fn"] == d["name"] and s["line"] == d["line"]), None))
        else d
        for d in scan.declarations
    ]


def parse_registry(lib_rs_text: str):
    m = REGISTRY_RE.search(lib_rs_text)
    if not m:
        return None
    return REGISTRY_ENTRY_RE.findall(m.group(1))


def build_manifest(root: Path, head_commit: str | None):
    inputs = []
    scans = {}
    registry_parsed = None
    for rel in MANAGED_INPUTS:
        p = root / rel
        if not p.is_file():
            print("ERROR[missing-input] %s" % rel, file=sys.stderr)
            return None, 2
        inputs.append({"path": rel, "sha256": sha256_of(p), "role": _role_of(rel)})
    registry_parsed = parse_registry((root / REGISTRY_HOST).read_text(encoding="utf-8"))
    if registry_parsed is None:
        print("ERROR[registry-parse] %s: %s block not found" % (REGISTRY_HOST, REGISTRY_SYMBOL),
              file=sys.stderr)
        return None, 2
    expected = ["auto/lib/%s.at" % m for m in
                ["token", "lexer", "parser", "typeinfo", "codegen", "engine", "a2r"]]
    registry_drift = registry_parsed != expected
    for rel in SOURCE_MODULES:
        scans[rel] = scan_module(rel, (root / rel).read_text(encoding="utf-8"))

    known_types = set()
    for s in scans.values():
        known_types |= {d["name"] for d in s.declarations if d["kind"] in ("type", "enum")}

    manifest = {
        "format_version": FORMAT_VERSION,
        "plan_id": PLAN_ID,
        "generator": "scripts/acc_inventory.py",
        "source_identity": {
            "root_kind": "auto-lang-repository",
            "paths_are": "repo-relative, forward slashes",
            "head_commit_audit_only": head_commit,
            "note": ("head_commit is recorded for audit only; drift validation uses "
                     "managed input hashes, never HEAD"),
        },
        "inputs": inputs,
        "registry": {
            "symbol": REGISTRY_SYMBOL,
            "host": REGISTRY_HOST,
            "module_order": [PurePosixPath(p).stem for p in expected],
            "cli_entry": "auto/aavm.at",
            "parsed_from_current_lib_rs": registry_parsed,
            "matches_tool_contract": not registry_drift,
            "note": ("registry drift is a --check failure; calibration after an "
                     "intentional registry change is a tool-code revision, not a "
                     "silent output update"),
        },
        "scan_semantics": {
            "dimension": "compiler-source-demand observations",
            "not_a_claim": ("lexically observed constructs do not imply AC/ACC "
                            "compiled-language-support; implemented/required verdicts "
                            "live only in manual-decisions.json"),
            "unknown_policy": ("unresolved receivers, bare calls and dynamic dispatch "
                               "are recorded as unknown candidates, never dropped"),
        },
        "observations": {"modules": [scans[rel].to_json() for rel in SOURCE_MODULES]},
        "manual_layer": {
            "file": DECISIONS_NAME,
            "binding": ("each decision binds the hashes of the files it cites "
                        "(managed inputs and other repo files); stale bindings fail --check"),
        },
    }
    return {"manifest": manifest, "registry_drift": registry_drift,
            "scans": scans, "inputs": inputs}, 0


def _role_of(rel: str) -> str:
    if rel == REGISTRY_HOST:
        return "registry-host"
    if rel == "auto/aavm.at":
        return "source-module-cli"
    return "source-module"


def render_summary(manifest: dict) -> str:
    lines = ["# PLAN-743 盘点扫描摘要（工具生成，勿手改）", ""]
    lines.append("性质：词法观察（compiler-source-demand 证据），非语义结论、非支持声明。")
    lines.append("")
    lines.append("## 输入覆盖")
    for inp in manifest["inputs"]:
        lines.append("- `%s` `%s`" % (inp["path"], inp["sha256"][:16]))
    reg = manifest["registry"]
    lines.append("")
    lines.append("## 注册表 %s" % reg["symbol"])
    lines.append("- 顺序：%s" % " → ".join(reg["module_order"]))
    lines.append("- 与工具合同一致：%s" % ("是" if reg["matches_tool_contract"] else "**否（漂移）**"))
    lines.append("")
    lines.append("## 模块观察")
    for mod in manifest["observations"]["modules"]:
        ndecl = len(mod["declarations"])
        nmeth = len(mod["methods"])
        nuse = len(mod["use_edges"])
        ncalls = len(mod["call_candidates"])
        nunk = sum(1 for c in mod["call_candidates"] if c["kind"].startswith("unknown"))
        lines.append("")
        lines.append("### `%s`" % mod["path"])
        lines.append("- 声明 %d（方法 %d）；use 边 %d；调用候选 %d（unknown %d）"
                     % (ndecl, nmeth, nuse, ncalls, nunk))
        if mod["anomalies"] or mod["brace_balance"] != 0:
            lines.append("- 异常：%s；括号余额 %d" % (mod["anomalies"], mod["brace_balance"]))
        caps = ", ".join("%s×%d" % (k, len(v)) for k, v in sorted(mod["capability_evidence"].items()))
        lines.append("- 能力证据：%s" % (caps if caps else "（无）"))
        unknowns = [c for c in mod["call_candidates"] if c["kind"].startswith("unknown")]
        if unknowns:
            lines.append("- unknown 候选（交人工核对，逐条列出）：")
            for c in unknowns:
                lines.append("  - %s%s(%s) 首行 %d ×%d"
                             % (("%s." % c["receiver"]) if c["receiver"] else "",
                                c["name"], c["kind"], c["first_line"], c["count"]))
    lines.append("")
    lines.append("## 人工结论层")
    lines.append("- `%s`：每次 --check 绑定输入 hash 校验；过期即失败。" % DECISIONS_NAME)
    lines.append("")
    return "\n".join(lines)


def dumps_stable(obj) -> str:
    return json.dumps(obj, ensure_ascii=False, sort_keys=True, indent=2) + "\n"


DECISION_KIND_CONCLUSIONS = {
    "module-role": {"adapt", "adapt-core-rewrite-output", "reference", "non-subject",
                    "retire-for-acc", "new-subject", "keep"},
    "runtime-service": {"implemented", "required", "reference", "keep", "retire"},
    "capability": {"implemented", "required", "unknown", "not-required"},
    "unknown-resolution": {"resolved", "open"},
}
DECISION_REQUIRED_KEYS = ("id", "subject", "kind", "conclusion", "evidence",
                          "note", "bound_input_hashes")


def _evidence_file(ev: str) -> str:
    """'path.at:123' -> 'path.at'；无行号后缀则原样返回。"""
    s = str(ev)
    head, _, tail = s.rpartition(":")
    return head if head and tail.isdigit() else s


def _unknown_candidates_from_scans(scans):
    """聚合当前扫描的全部 unknown 候选：[(path, kind, name)] 去重排序。"""
    seen = set()
    for rel, s in scans.items():
        for c in s.calls:
            if c["kind"].startswith("unknown"):
                seen.add((rel, c["kind"], c["name"]))
    return sorted(seen)


def validate_decisions(dec_path: Path, inputs, root: Path,
                       unknown_candidates=None, require: bool = False):
    """校验人工结论层。返回 (状态, 消息列表)；状态 absent|error|stale|ok。

    绑定宇宙不限于受管扫描输入：决定可绑定仓库内任意文件，工具现场计算 hash。
    QA-03（PLAN-743 r2）：provided 层做完整结构校验（版本/必填字段/唯一 ID/
    合法结论/非空绑定/证据文件存在）；--require-decisions 严格完成态门额外要求
    人工层存在、非空，且覆盖全部受管输入与全部 unknown 候选（unknown_families
    双向闭环，open 族必须带 owner/探针/工作包）。早期 scan-only（absent 且未
    加严格门）保持可用，但消息明示"非完成态"，不显示为人工层 ok。
    """
    if not dec_path.is_file():
        if require:
            return "error", ["ERROR[decisions-required] 完成态要求 %s 存在"
                             % DECISIONS_NAME]
        return "absent", ["WARN[decisions-absent] %s 不存在：scan-only 模式"
                          "（仅校验扫描制品，非完成态，人工层未提供）" % DECISIONS_NAME]
    try:
        data = json.loads(dec_path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, UnicodeDecodeError) as e:
        return "error", ["ERROR[decisions-malformed] %s: %s" % (DECISIONS_NAME, e)]
    if not isinstance(data, dict):
        return "error", ["ERROR[decisions-malformed] %s: 顶层必须是对象" % DECISIONS_NAME]
    if data.get("format_version") != 1:
        return "error", ["ERROR[decisions-malformed] format_version 必须为 1，实际 %r"
                         % (data.get("format_version"),)]
    decisions = data.get("decisions")
    if not isinstance(decisions, list) or not decisions:
        return "error", ["ERROR[decisions-malformed] decisions 必须是非空数组"]
    current = {i["path"]: i["sha256"] for i in inputs}
    msgs, stale, structural_error = [], False, False

    seen_ids = set()
    covered_paths = set()
    for d in decisions:
        if not isinstance(d, dict):
            return "error", ["ERROR[decision-malformed] decisions 含非对象记录"]
        did = d.get("id")
        for key in DECISION_REQUIRED_KEYS:
            if key not in d or d[key] in (None, "", [], {}):
                structural_error = True
                msgs.append("ERROR[decision-malformed] %s 缺必填字段或为空: %s"
                            % (did, key))
        if not did:
            continue
        did = str(did)
        if did in seen_ids:
            structural_error = True
            msgs.append("ERROR[decision-duplicate-id] %s" % did)
        seen_ids.add(did)
        kind = d.get("kind")
        if kind not in DECISION_KIND_CONCLUSIONS:
            structural_error = True
            msgs.append("ERROR[decision-malformed] %s 非法 kind: %r" % (did, kind))
        elif d.get("conclusion") not in DECISION_KIND_CONCLUSIONS[kind]:
            structural_error = True
            msgs.append("ERROR[decision-malformed] %s conclusion %r 不属于 kind=%s"
                        " 合法集 %s" % (did, d.get("conclusion"), kind,
                                        sorted(DECISION_KIND_CONCLUSIONS[kind])))
        for ev in d.get("evidence", []):
            ev_file = _evidence_file(ev)
            if not (root / ev_file).is_file():
                structural_error = True
                msgs.append("ERROR[decision-evidence-missing] %s 证据文件不存在: %s"
                            % (did, ev))
            else:
                covered_paths.add(ev_file)
        subject = str(d.get("subject") or "")
        if subject and (root / subject.split(":")[0]).is_file():
            covered_paths.add(subject.split(":")[0])
        bound = d.get("bound_input_hashes") or {}
        for p, h in sorted(bound.items()):
            if p not in current:
                f = root / p
                if not f.is_file():
                    stale = True
                    msgs.append("ERROR[decision-stale] %s 绑定的输入已不存在: %s" % (did, p))
                    continue
                cur = sha256_of(f)
            else:
                cur = current[p]
            if cur != h:
                stale = True
                msgs.append("ERROR[decision-stale] %s 绑定的输入已变化: %s" % (did, p))

    if require:
        # 完成态覆盖闭环 1：全部受管输入被 ≥1 决定引用（subject/evidence/绑定）
        for inp in inputs:
            if inp["path"] not in covered_paths:
                structural_error = True
                msgs.append("ERROR[coverage-missing-input] 无决定覆盖受管输入: %s"
                            % inp["path"])
        # 覆盖闭环 2：unknown 候选 ↔ unknown_families 双向匹配
        families = data.get("unknown_families")
        if not isinstance(families, list) or not families:
            structural_error = True
            msgs.append("ERROR[coverage-missing-unknowns] 完成态要求 unknown_families"
                        " 覆盖声明（当前扫描 unknown 候选 %d 族未闭环）"
                        % len(unknown_candidates or []))
            families = []
        fam_index = []
        for f in families:
            if not isinstance(f, dict):
                structural_error = True
                msgs.append("ERROR[family-malformed] unknown_families 含非对象记录")
                continue
            fp, fk = f.get("path"), f.get("kind")
            if fp not in current:
                structural_error = True
                msgs.append("ERROR[family-malformed] family.path 不是受管输入: %r" % (fp,))
                continue
            if fk not in ("unknown-receiver", "unknown-bare-call"):
                structural_error = True
                msgs.append("ERROR[family-malformed] family.kind 非法: %r" % (fk,))
                continue
            names = f.get("names")
            if names != "*" and not (isinstance(names, list) and names):
                structural_error = True
                msgs.append("ERROR[family-malformed] family.names 必须为 \"*\" 或非空"
                            " 列表: %s/%s" % (fp, fk))
                continue
            disposition = f.get("disposition")
            if disposition not in ("resolved", "open"):
                structural_error = True
                msgs.append("ERROR[family-malformed] family.disposition 非法: %r"
                            % (disposition,))
                continue
            if disposition == "open":
                for req_key in ("owner", "probe", "work_package"):
                    if not f.get(req_key):
                        structural_error = True
                        msgs.append("ERROR[family-open-incomplete] open 族缺 %s: %s/%s"
                                    % (req_key, fp, fk))
            ref = f.get("decision") or f.get("owner")
            if ref and str(ref) not in seen_ids:
                structural_error = True
                msgs.append("ERROR[family-malformed] family 引用的决定不存在: %r" % (ref,))
            fam_index.append(f)
        if unknown_candidates:
            for (upath, ukind, uname) in unknown_candidates:
                hit = any(f.get("path") == upath and f.get("kind") == ukind
                          and (f.get("names") == "*" or uname in f.get("names", []))
                          for f in fam_index)
                if not hit:
                    structural_error = True
                    msgs.append("ERROR[coverage-missing-unknown] 未闭环 unknown 候选:"
                                " %s %s %s" % (upath, ukind, uname))
            for f in fam_index:
                fp, fk, names = f.get("path"), f.get("kind"), f.get("names")
                matched = any(p == fp and k == fk and (names == "*" or n in names)
                              for (p, k, n) in unknown_candidates)
                if not matched:
                    structural_error = True
                    msgs.append("ERROR[family-orphan] family 无对应候选: %s %s %s"
                                % (fp, fk, names))

    if stale or structural_error:
        return ("stale" if stale and not structural_error else "error"), msgs
    ok_msg = "OK[decisions] %d 条人工结论绑定全部新鲜" % len(decisions)
    if require:
        ok_msg += "；严格门覆盖闭环通过（输入 %d/unknown 候选 %d 族）" % (
            len(inputs), len(unknown_candidates or []))
    msgs.insert(0, ok_msg)
    return "ok", msgs


def resolve_output_safety(root: Path, out_dir: Path) -> str | None:
    """输出目录不得落在任何受管输入的目录子树内，也不得覆盖受管输入。"""
    root_res = root.resolve()
    out_res = out_dir.resolve()
    if out_res == root_res:
        return "output dir equals repository root"
    for rel in MANAGED_INPUTS:
        in_parts = PurePosixPath(rel).parts[:-1]
        out_rel = out_res.relative_to(root_res) if out_res.is_relative_to(root_res) else None
        if out_rel is None:
            continue
        out_parts = out_rel.parts
        if out_parts[:len(in_parts)] == in_parts:
            return "output dir %s is inside managed input source dir %s" % (out_dir, rel)
        if in_parts[:len(out_parts)] == out_parts:
            return "output dir would clobber managed input tree %s" % rel
    return None


def get_head_commit(root: Path) -> str | None:
    """审计用 HEAD；失败不致命（验证不依赖它）。"""
    import subprocess
    try:
        r = subprocess.run(["git", "rev-parse", "HEAD"], cwd=str(root),
                           capture_output=True, text=True, timeout=10)
        if r.returncode == 0:
            return r.stdout.strip()
    except Exception:
        pass
    return None


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description="PLAN-743 ACC inventory scanner")
    ap.add_argument("--root", required=True)
    ap.add_argument("--output", required=True)
    group = ap.add_mutually_exclusive_group(required=True)
    group.add_argument("--write", action="store_true")
    group.add_argument("--check", action="store_true")
    ap.add_argument("--require-decisions", action="store_true",
                    help="严格完成态门：要求人工结论层存在、合法且覆盖闭环"
                         "（仅 --check；QA-03，PLAN-743 r2）")
    args = ap.parse_args(argv)
    if args.require_decisions and args.write:
        print("ERROR[usage] --require-decisions 仅与 --check 组合使用", file=sys.stderr)
        return 2

    root = Path(args.root)
    if not root.is_dir():
        print("ERROR[usage] --root not a directory: %s" % root, file=sys.stderr)
        return 2
    out_dir = Path(args.output)
    if not out_dir.is_absolute():
        out_dir = root / out_dir
    unsafe = resolve_output_safety(root, out_dir)
    if unsafe:
        print("ERROR[unsafe-output] %s" % unsafe, file=sys.stderr)
        return 2

    built, code = build_manifest(root, get_head_commit(root))
    if built is None:
        return code
    manifest, scans = built["manifest"], built["scans"]
    summary = render_summary(manifest)
    manifest_bytes = dumps_stable(manifest).encode("utf-8")
    summary_bytes = summary.encode("utf-8")

    if args.write:
        if built["registry_drift"]:
            print("ERROR[registry-drift] lib.rs %s 不再匹配工具合同清单；"
                  "--write 拒绝生成（校准须先修订工具合同）" % REGISTRY_SYMBOL, file=sys.stderr)
            return 1
        out_dir.mkdir(parents=True, exist_ok=True)
        (out_dir / MANIFEST_NAME).write_bytes(manifest_bytes)
        (out_dir / SUMMARY_NAME).write_bytes(summary_bytes)
        dec = out_dir / DECISIONS_NAME
        if not dec.exists():
            print("NOTE[decisions-absent] 未生成 %s（人工结论由 T-03 审定产生，工具不代写）"
                  % DECISIONS_NAME)
        print("OK[write] %s + %s（%d 输入全记录）" % (MANIFEST_NAME, SUMMARY_NAME, len(manifest["inputs"])))
        return 0

    # ── --check ──
    failures = []
    notes = []
    if built["registry_drift"]:
        failures.append("ERROR[registry-drift] 注册表与工具合同清单不一致: %s" % built["registry_parsed"])
    man_path = out_dir / MANIFEST_NAME
    sum_path = out_dir / SUMMARY_NAME
    if not man_path.is_file() or not sum_path.is_file():
        failures.append("ERROR[artifacts-absent] 先运行 --write 生成 %s/%s"
                        % (MANIFEST_NAME, SUMMARY_NAME))
    else:
        # 审计字段（head_commit）不参与漂移判定：提交报告本身会推进 HEAD，
        # 属计划 §5.1 预期的非漂移变化；比较前双方都置空后再字节比对。
        try:
            on_disk = json.loads(man_path.read_text(encoding="utf-8"))
        except (json.JSONDecodeError, UnicodeDecodeError):
            on_disk = None
        if on_disk is None:
            failures.append("ERROR[manifest-stale] %s 不是可解析 manifest" % MANIFEST_NAME)
        else:
            on_disk.get("source_identity", {})["head_commit_audit_only"] = None
            regen = json.loads(manifest_bytes.decode("utf-8"))
            regen["source_identity"]["head_commit_audit_only"] = None
            if dumps_stable(on_disk).encode("utf-8") != dumps_stable(regen).encode("utf-8"):
                failures.append("ERROR[manifest-stale] %s 与当前输入重扫结果不一致"
                                "（输入 hash 或扫描观察漂移）" % MANIFEST_NAME)
        if sum_path.read_bytes() != summary_bytes:
            failures.append("ERROR[summary-stale] %s 与当前输入重扫结果不一致" % SUMMARY_NAME)
        covered = {i["path"] for i in manifest["inputs"]}
        missing = [p for p in MANAGED_INPUTS if p not in covered]
        if missing:
            failures.append("ERROR[coverage] manifest 未覆盖受管输入: %s" % ", ".join(missing))
    status, dmsgs = validate_decisions(
        out_dir / DECISIONS_NAME, built["inputs"], root,
        unknown_candidates=_unknown_candidates_from_scans(scans),
        require=args.require_decisions)
    if status in ("stale", "error"):
        failures.extend(dmsgs)
    else:
        notes.extend(dmsgs)
    for m in notes:
        print(m)
    if failures:
        for f in failures:
            print(f, file=sys.stderr)
        print("CHECK-FAIL %d 项" % len(failures), file=sys.stderr)
        return 1
    print("CHECK-OK 输入 %d 项全部一致；注册表一致；人工结论层 %s"
          % (len(manifest["inputs"]), status))
    return 0


if __name__ == "__main__":
    sys.exit(main())
