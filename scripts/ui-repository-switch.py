#!/usr/bin/env python3
"""ui-repository-switch.py — PLAN-831 T-08 三仓切换事务工具（FLR-5）。

按本批合同 §4（repo-delivery-parallel-order-831-832）实现
prepared → target landed → consumer switched → verified 四状态收据链：
多仓 Git 无原子事务，逐仓独立落状态与失败恢复指令；任何一仓失败不
宣称全套交付、不归档/删树；恢复通过新提交或受管交换，绝不 reset/
force-push/强移默认分支。

硬约束（受管判红，非越权动作）：
- land/switch 仅 `git merge --ff-only`（默认分支可快进才动）；默认分支
  tip ≠ 预期锚（base anchor）即拒绝——第三方推进/异 tip 不覆盖。
- 脏工作树（受管路径下有未提交变更）拒绝推进，给出恢复指令。
- 幂等重试：每步先对账收据链——已有 ok 事件且 ref 实际值与事件一致则
  跳过；失败事件不阻断重试（新增事件行追加，不覆写）。
- 收据 append-only（JSONL，递增 seq，无时间戳/绝对路径占位 <repo>）。

子命令：
  plan         prepared：锚定三仓身份（默认分支 base tip、迁移分支 tip、
               工作树状态），写 prepared 事件
  land-ui      target landed：UI 仓默认分支 ff-only 推进到迁移分支
  switch-lang  consumer switched：语言仓默认分支 ff-only 推进
  verify       verified：双仓 metadata 解析 + 五包成员/锁在位核验
  status       读收据链报告当前状态、下一步与恢复指令
  drill        scratch 合成三仓组故障注入演练（D1..D6，FLR-5 oracle）

用法示例（T-11 真仓执行形态）：
  python scripts/ui-repository-switch.py plan --ui <auto-ui> --lang <auto-lang> \
      --ui-branch codex/plan-831-dev --lang-branch codex/plan-831-dev \
      --receipts <dir>
退出码：0 成功/跳过；2 受管判红（含锚冲突/脏树/verify 失败——副作用
零或已记账）；1 运行错；3 用法错。
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

LEDGER_SCHEMA = "ui-repository-switch-ledger/1"
STEPS = ("prepared", "ui-landed", "lang-switched", "verified")
EXIT_OK, EXIT_ERROR, EXIT_MANAGED_RED, EXIT_USAGE = 0, 1, 2, 3


def log(msg: str) -> None:
    print(f"[ui-switch] {msg}", flush=True)


class ManagedRed(Exception):
    """受管判红：零副作用或已记账，附恢复指令。"""


def git(repo: Path, args: list[str], check: bool = True) -> subprocess.CompletedProcess:
    proc = subprocess.run(["git", "-C", str(repo), *args],
                          capture_output=True, text=True, encoding="utf-8",
                          errors="replace")
    if check and proc.returncode != 0:
        raise RuntimeError(f"git {' '.join(args[:2])} 失败: {proc.stderr.strip()[:200]}")
    return proc


def head_of(repo: Path, ref: str = "HEAD") -> str:
    return git(repo, ["rev-parse", ref]).stdout.strip()


def default_branch(repo: Path) -> str:
    proc = git(repo, ["symbolic-ref", "--short", "HEAD"], check=False)
    return proc.stdout.strip() if proc.returncode == 0 else "master"


def tracked_dirty(repo: Path) -> list[str]:
    proc = git(repo, ["status", "--porcelain"])
    return [l for l in proc.stdout.splitlines() if l.strip() and not l.startswith("??")]


class Ledger:
    """append-only 事件链：seq 递增，无时间戳；<repo> 占位保可移植。"""

    def __init__(self, path: Path):
        self.path = path
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self.events: list[dict] = []
        if self.path.exists():
            for line in self.path.read_text(encoding="utf-8").splitlines():
                if line.strip():
                    self.events.append(json.loads(line))

    def find(self, step: str, repo: str) -> dict | None:
        """该步该仓最近一次事件（ok 或 fail）。"""
        for event in reversed(self.events):
            if event["step"] == step and event["repo"] == repo:
                return event
        return None

    def append(self, step: str, repo: str, ok: bool, detail: str,
               from_sha: str | None = None, to_sha: str | None = None,
               recovery: str | None = None, extra: dict | None = None) -> dict:
        seq = (self.events[-1]["seq"] + 1) if self.events else 1
        event = {"seq": seq, "schema": LEDGER_SCHEMA, "step": step, "repo": repo,
                 "ok": ok, "detail": detail, "from": from_sha, "to": to_sha}
        if recovery:
            event["recovery"] = recovery
        if extra:
            event.update(extra)
        with self.path.open("a", encoding="utf-8", newline="\n") as f:
            f.write(json.dumps(event, ensure_ascii=False) + "\n")
        self.events.append(event)
        log(f"[{seq}] {'OK ' if ok else 'RED'} {step}/{repo}: {detail}"
            + (f" | 恢复: {recovery}" if recovery and not ok else ""))
        return event

    def step_ok_matches(self, step: str, repo: str, expect_ref: str) -> bool:
        """幂等对账：最近事件 ok 且其 to == 当前实际 ref 值。"""
        event = self.find(step, repo)
        return bool(event and event["ok"] and event["to"] == expect_ref)


# ---------------------------------------------------------------------------
# 事务步骤
# ---------------------------------------------------------------------------

class Ctx:
    def __init__(self, args):
        self.ui = Path(args.ui).resolve()
        self.lang = Path(args.lang).resolve()
        self.ui_branch = args.ui_branch
        self.lang_branch = args.lang_branch
        self.ledger = Ledger(Path(args.receipts) / "switch-ledger.jsonl")


def step_plan(ctx: Ctx) -> int:
    """prepared：锚定身份并写收据。异 tip 锚冲突=第三方推进过默认分支 → 红。"""
    for repo, name in ((ctx.ui, "ui"), (ctx.lang, "lang")):
        if not (repo / ".git").exists():
            print(f"[ui-switch] 配置错：{name} 仓不在场 {repo}", file=sys.stderr)
            return EXIT_USAGE
    ui_base, lang_base = head_of(ctx.ui), head_of(ctx.lang)
    ui_tip = head_of(ctx.ui, ctx.ui_branch)
    lang_tip = head_of(ctx.lang, ctx.lang_branch)
    for repo, name, base in ((ctx.ui, "ui", ui_base), (ctx.lang, "lang", lang_base)):
        dirty = tracked_dirty(repo)
        state = {"base": base, "dirty": bool(dirty)}
        if not ctx.ledger.step_ok_matches("prepared", name, base):
            ctx.ledger.append("prepared", name, True,
                              f"base={base[:9]} 迁移分支 tip="
                              f"{(ui_tip if name == 'ui' else lang_tip)[:9]} 脏树={'是' if dirty else '否'}",
                              extra={"base": base})
    # 快进可行性（不执行）：base 必须是迁移分支 tip 的祖先
    for repo, name, branch, base in ((ctx.ui, "ui", ctx.ui_branch, ui_base),
                                     (ctx.lang, "lang", ctx.lang_branch, lang_base)):
        anc = git(repo, ["merge-base", "--is-ancestor", base, branch], check=False)
        if anc.returncode != 0:
            ctx.ledger.append("prepared", name, False,
                              f"默认分支 {base[:9]} 不是 {branch} 的祖先（不可快进）",
                              recovery="准备可审路径：在迁移分支上 rebase/合并默认分支新进展后重跑 plan")
            return EXIT_MANAGED_RED
    log(f"prepared 完成：ui base={ui_base[:9]}→tip={ui_tip[:9]}；"
        f"lang base={lang_base[:9]}→tip={lang_tip[:9]}")
    return EXIT_OK


def _advance(ctx: Ctx, repo: Path, name: str, branch: str, step: str) -> int:
    """landed/switched：锚校验 + 脏树校验 + ff-only 推进（幂等）。"""
    base = head_of(repo)
    if ctx.ledger.step_ok_matches(step, name, base) and base == head_of(repo, branch):
        log(f"{step}/{name} 幂等跳过（已推进且与收据一致）")
        return EXIT_OK
    # 锚校验：prepared 记录的 base；第三方推进即拒（不覆盖）
    prepared = ctx.ledger.find("prepared", name)
    if prepared and prepared["ok"] and prepared["base"] and prepared["base"] != base:
        ctx.ledger.append(step, name, False,
                          f"锚冲突：默认分支 tip {base[:9]} ≠ prepared 锚 {prepared['base'][:9]}"
                          "（第三方推进过）", from_sha=base,
                          recovery=f"重跑 plan 重锚（确认新 tip 归属后再推进）；绝不 force/ reset")
        return EXIT_MANAGED_RED
    dirty = tracked_dirty(repo)
    if dirty:
        ctx.ledger.append(step, name, False, f"工作树脏（{len(dirty)} 项受管变更）",
                          from_sha=base,
                          recovery="由其 owner 处置 WIP（stash/patch 移交）后重跑本步；不覆盖工作区")
        return EXIT_MANAGED_RED
    tip = head_of(repo, branch)
    proc = git(repo, ["merge", "--ff-only", branch], check=False)
    if proc.returncode != 0:
        ctx.ledger.append(step, name, False,
                          f"ff-only 推进失败: {proc.stderr.strip()[:160]}", from_sha=base,
                          recovery="准备可审路径（迁移分支合并默认分支进展）后重跑")
        return EXIT_MANAGED_RED
    ctx.ledger.append(step, name, True, f"默认分支 ff {base[:9]} → {tip[:9]}",
                      from_sha=base, to_sha=tip)
    return EXIT_OK


def step_verify(ctx: Ctx) -> int:
    """verified：双仓 metadata 解析 + 五包成员/锁在位（结构判别 source-switched）。"""
    ui_head, lang_head = head_of(ctx.ui), head_of(ctx.lang)
    if ctx.ledger.step_ok_matches("verified", "both", ui_head):
        log("verified 幂等跳过")
        return EXIT_OK
    problems: list[str] = []
    for repo, name in ((ctx.ui, "ui"), (ctx.lang, "lang")):
        proc = subprocess.run(["cargo", "metadata", "--no-deps", "--format-version", "1"],
                              cwd=str(repo), capture_output=True, text=True,
                              env={k: v for k, v in os.environ.items()
                                   if not k.startswith("AUTO_") and k != "CARGO_TARGET_DIR"})
        if proc.returncode != 0:
            problems.append(f"{name} metadata 红: {proc.stderr.strip()[:120]}")
    if not problems:
        meta = json.loads(subprocess.run(
            ["cargo", "metadata", "--no-deps", "--format-version", "1"],
            cwd=str(ctx.ui), capture_output=True, text=True).stdout)
        names = {p["name"] for p in meta["packages"]}
        five = {"auto-ui-dialect", "auto-ui-compiler", "auto-ui-runtime",
                "auto-ui-tools", "shell-pack"}
        if not five <= names:
            problems.append(f"UI 仓缺五包成员: {sorted(five - names)}")
        if not (ctx.ui / "Cargo.lock").is_file():
            problems.append("UI 仓 Cargo.lock 缺席（受管锁丢失）")
    if problems:
        ctx.ledger.append("verified", "both", False, "; ".join(problems),
                          recovery="修复后在同 HEAD 重跑 verify（幂等）；verified 前不得宣称切换完成")
        return EXIT_MANAGED_RED
    ctx.ledger.append("verified", "both", True,
                      f"ui={ui_head[:9]} lang={lang_head[:9]} metadata 绿 + 五包成员 + 锁在位",
                      to_sha=ui_head)
    return EXIT_OK


def step_status(ctx: Ctx) -> int:
    log(f"收据链: {ctx.ledger.path}")
    for event in ctx.ledger.events:
        log(f"  [{event['seq']}] {'OK ' if event['ok'] else 'RED'} "
            f"{event['step']}/{event['repo']}: {event['detail'][:100]}"
            + (f" | 恢复: {event['recovery'][:80]}" if not event["ok"] and "recovery" in event else ""))
    # 下一步与整体状态
    chain_ok = all(
        ctx.ledger.find(step, repo) and ctx.ledger.find(step, repo)["ok"]
        for step, repo in (("prepared", "ui"), ("prepared", "lang"),
                           ("ui-landed", "ui"), ("lang-switched", "lang"),
                           ("verified", "both")))
    log("整体: " + ("verified（全套切换完成）" if chain_ok else "未完成——按链上 RED 事件的 recovery 指令处置后重跑对应步骤（幂等）"))
    return EXIT_OK if chain_ok else EXIT_MANAGED_RED


# ---------------------------------------------------------------------------
# drill：scratch 合成三仓组故障注入演练（FLR-5 oracle；不触真仓）
# ---------------------------------------------------------------------------

def _mk_repo(path: Path, name: str) -> None:
    path.mkdir(parents=True)
    git(path, ["init", "-q", "-b", "master"])
    git(path, ["-c", "user.name=drill", "-c", "user.email=drill@invalid",
               "commit", "-q", "--allow-empty", "-m", f"{name} base"])
    git(path, ["-c", "user.name=drill", "-c", "user.email=drill@invalid",
               "checkout", "-q", "-b", "dev"])
    git(path, ["-c", "user.name=drill", "-c", "user.email=drill@invalid",
               "commit", "-q", "--allow-empty", "-m", f"{name} migration candidate"])
    git(path, ["checkout", "-q", "master"])


def _receipts_ok(path: Path) -> bool:
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.strip() and not json.loads(line).get("ok", True):
            return False
    return True


def cmd_drill(args) -> int:
    import tempfile

    base = Path(tempfile.mkdtemp(prefix="ui-switch-drill-"))
    problems: list[str] = []
    checks: list[dict] = []

    def expect(cond: bool, label: str):
        log(f"{'PASS' if cond else 'FAIL'} [drill] {label}")
        checks.append({"check": label, "ok": cond})
        if not cond:
            problems.append(label)

    try:
        ui, lang = base / "grp" / "auto-ui", base / "grp" / "auto-lang"
        _mk_repo(ui, "ui")
        _mk_repo(lang, "lang")
        ui_base, lang_base = head_of(ui), head_of(lang)
        rcv = base / "receipts"
        common = ["--ui", str(ui), "--lang", str(lang),
                  "--ui-branch", "dev", "--lang-branch", "dev",
                  "--receipts", str(rcv)]

        def run(step: str) -> int:
            proc = subprocess.run([sys.executable, str(Path(__file__).resolve()), step, *common],
                                  capture_output=True, text=True, encoding="utf-8")
            sys.stdout.write(proc.stdout)
            if proc.stderr:
                sys.stderr.write(proc.stderr)
            return proc.returncode

        # D1 正常全链
        expect(run("plan") == EXIT_OK, "D1a prepared ok")
        expect(run("land-ui") == EXIT_OK and head_of(ui) == head_of(ui, "dev"),
               "D1b ui landed=dev tip（ff）")
        expect(run("switch-lang") == EXIT_OK and head_of(lang) == head_of(lang, "dev"),
               "D1c lang switched=dev tip")
        # verify 需要 cargo 工程——合成仓非 cargo：verify 判红（受管）+恢复指令，
        # 演练其失败面；D6 用真最小 cargo 仓复验 verified 绿路径。
        expect(run("verify") in (EXIT_OK, EXIT_MANAGED_RED), "D1d verify 执行（合成仓预期受管红或绿）")
        d1_chain = [json.loads(l) for l in (rcv / "switch-ledger.jsonl").read_text(encoding="utf-8").splitlines() if l]
        expect(all(e["ok"] for e in d1_chain if e["step"] in ("prepared", "ui-landed", "lang-switched")),
               "D1 全链 ok 事件（prepared/landed/switched）")

        # D2 锚冲突：第三方在 ui master 追加提交后**未重锚**直接 land →
        # 拒绝（prepared 锚 ≠ 当前 tip）、master 零覆盖；吸收后重锚可推进。
        git(ui, ["-c", "user.name=drill", "-c", "user.email=drill@invalid",
                 "commit", "-q", "--allow-empty", "-m", "third-party advance"])
        third_tip = head_of(ui)
        expect(run("land-ui") == EXIT_MANAGED_RED and head_of(ui) == third_tip,
               "D2a 锚冲突拒绝且 master 零覆盖（停第三方 tip）")
        chain_txt = (rcv / "switch-ledger.jsonl").read_text(encoding="utf-8")
        expect("锚冲突" in chain_txt, "D2b 锚冲突事件在案（附恢复指令）")
        git(ui, ["-c", "user.name=drill", "-c", "user.email=drill@invalid",
                 "checkout", "-q", "dev"])
        git(ui, ["-c", "user.name=drill", "-c", "user.email=drill@invalid",
                 "merge", "-q", "--no-ff", "-m", "absorb third-party", "master"])
        git(ui, ["checkout", "-q", "master"])
        expect(run("plan") == EXIT_OK, "D2c 吸收后重锚 ok")

        # D3 中断幂等：重跑已完成步=跳过（master 已在 dev tip → 零移动）
        expect(run("land-ui") == EXIT_OK and head_of(ui) == head_of(ui, "dev"),
               "D3a 吸收后 land 推进到 dev tip")
        after_d3a = head_of(ui)
        expect(run("land-ui") == EXIT_OK and head_of(ui) == after_d3a,
               "D3b 幂等：二次 land 零移动")

        # D5 取消态（新组）：prepared-only 全仓零副作用；随后在该组注入 D4。
        ui3, lang3 = base / "grp3" / "auto-ui", base / "grp3" / "auto-lang"
        _mk_repo(ui3, "ui3")
        _mk_repo(lang3, "lang3")
        b3u, b3l = head_of(ui3), head_of(lang3)
        rcv3 = base / "receipts3"
        run3 = lambda step: subprocess.run(
            [sys.executable, str(Path(__file__).resolve()), step,
             "--ui", str(ui3), "--lang", str(lang3),
             "--ui-branch", "dev", "--lang-branch", "dev", "--receipts", str(rcv3)],
            capture_output=True, text=True).returncode
        expect(run3("plan") == EXIT_OK and head_of(ui3) == b3u and head_of(lang3) == b3l,
               "D5 prepared-only：两仓零副作用（取消点=全保留）")
        # D4 交换失败保旧（lang **未切换**态）：脏树 → switch 拒绝、master 不动、
        # ui landed 保留（部分完成不回滚不误归档）。
        expect(run3("land-ui") == EXIT_OK, "D4 前置：ui landed")
        (lang3 / "WIP.txt").write_text("owner wip", encoding="utf-8")
        git(lang3, ["add", "WIP.txt"])
        before_lang3 = head_of(lang3)
        expect(run3("switch-lang") == EXIT_MANAGED_RED and head_of(lang3) == before_lang3,
               "D4a 脏树拒绝且 lang master 不动")
        chain3 = [json.loads(l) for l in (rcv3 / "switch-ledger.jsonl").read_text(encoding="utf-8").splitlines() if l]
        expect(any((not e["ok"]) and e.get("recovery") for e in chain3),
               "D4b RED 事件携带恢复指令")
        expect(any(e["step"] == "ui-landed" and e["ok"] for e in chain3),
               "D4c ui landed 保留（部分完成不回滚不误归档）")

        # D6 恢复重试（主组已全链切换）：二次 switch 幂等跳过=重试成功语义；
        # 阻塞解除场景由 grp3 收尾（D4 后清脏再 switch 至绿）。
        expect(run("switch-lang") == EXIT_OK and head_of(lang) == head_of(lang, "dev"),
               "D6a 二次 switch 幂等重试成功")
        git(lang3, ["rm", "-q", "--cached", "WIP.txt"])
        (lang3 / "WIP.txt").unlink()
        expect(run3("switch-lang") == EXIT_OK and head_of(lang3) == head_of(lang3, "dev"),
               "D6a2 阻塞解除后 switch 重试成功（grp3 恢复路径）")
        # verified 需 cargo 工程：最小五包 UI 仓 + 空 lang 仓（metadata 面绿）；
        # _mk_repo 已建 base(dev 空候选)骨架，工作面提交追加在 dev 上。
        mini = base / "mini"
        ui_m, lang_m = mini / "auto-ui", mini / "auto-lang"
        _mk_repo(ui_m, "mini-ui")
        _mk_repo(lang_m, "mini-lang")
        git(ui_m, ["checkout", "-q", "dev"])
        (ui_m / "Cargo.toml").write_text(
            "[workspace]\nmembers = [\"crates/*\"]\n", encoding="utf-8")
        five = ("auto-ui-dialect", "auto-ui-compiler", "auto-ui-runtime",
                "auto-ui-tools", "shell-pack")
        for pkg in five:
            d = ui_m / "crates" / pkg
            (d / "src").mkdir(parents=True)
            (d / "src" / "lib.rs").write_text("", encoding="utf-8")
            (d / "Cargo.toml").write_text(
                f"[package]\nname = \"{pkg}\"\nversion = \"0.0.0\"\nedition = \"2021\"\n",
                encoding="utf-8")
        (ui_m / "Cargo.lock").write_text("# lock\n", encoding="utf-8")
        git(ui_m, ["add", "-A"])
        git(ui_m, ["-c", "user.name=drill", "-c", "user.email=drill@invalid",
                   "commit", "-q", "-m", "mini workspace"])
        git(ui_m, ["checkout", "-q", "master"])
        git(lang_m, ["checkout", "-q", "dev"])
        (lang_m / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")
        git(lang_m, ["add", "-A"])
        git(lang_m, ["-c", "user.name=drill", "-c", "user.email=drill@invalid",
                     "commit", "-q", "-m", "empty lang"])
        git(lang_m, ["checkout", "-q", "master"])
        rcv_m = mini / "receipts"
        def run_m(step: str) -> int:
            proc = subprocess.run(
                [sys.executable, str(Path(__file__).resolve()), step,
                 "--ui", str(ui_m), "--lang", str(lang_m),
                 "--ui-branch", "dev", "--lang-branch", "dev", "--receipts", str(rcv_m)],
                capture_output=True, text=True)
            if proc.returncode != EXIT_OK:
                log(f"D6 step {step} rc={proc.returncode}: "
                    + ((proc.stdout + proc.stderr).strip().splitlines() or ["?"])[-1][:200])
            return proc.returncode
        ok_chain = (run_m("plan") == EXIT_OK and run_m("land-ui") == EXIT_OK
                    and run_m("switch-lang") == EXIT_OK and run_m("verify") == EXIT_OK)
        expect(ok_chain, "D6b 真 cargo 最小组全链 verified 绿")
        status_m = subprocess.run(
            [sys.executable, str(Path(__file__).resolve()), "status",
             "--ui", str(ui_m), "--lang", str(lang_m),
             "--ui-branch", "dev", "--lang-branch", "dev", "--receipts", str(rcv_m)],
            capture_output=True, text=True).returncode
        expect(status_m == EXIT_OK, "D6c status 报 verified（整体完成判定）")
    finally:
        def _force_rm(func, path, _exc):
            os.chmod(path, 0o700 | 0o200)
            func(path)
        shutil.rmtree(base, onexc=_force_rm)

    def _tool_repo_head() -> str | None:
        # F831-09-05：合成组工具收据绑定工具源仓（lang）HEAD——收据身份
        # admission（fixture driver）据此核对本工具自录制以来是否变更。
        proc = subprocess.run(
            ["git", "-C", str(Path(__file__).resolve().parent.parent), "rev-parse", "HEAD"],
            capture_output=True, text=True)
        return proc.stdout.strip() if proc.returncode == 0 else None

    receipt = {
        "schema": "ui-repository-switch-drill-receipt/1", "plan": "PLAN-831",
        "tool": "scripts/ui-repository-switch.py drill",
        "sources": {"lang": _tool_repo_head()},
        "checks": checks,
        "ok": not problems,
    }
    out = Path(getattr(args, "receipt_out", None)
               or Path(__file__).resolve().parent.parent.parent
               / "scratch-ui-repo-fixture" / "switch-drill-receipt.json")
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(receipt, ensure_ascii=False, indent=2),
                   encoding="utf-8", newline="\n")
    log(f"drill 收据 → {out}")
    if problems:
        log(f"drill 判红 {len(problems)} 项: {problems}")
        return EXIT_MANAGED_RED
    log("drill 全部通过（FLR-5 oracle：正常链/锚冲突/中断幂等/交换失败保旧/取消零副作用/恢复重试）")
    return EXIT_OK


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = parser.add_subparsers(dest="cmd", required=True)
    for name in ("plan", "land-ui", "switch-lang", "verify", "status"):
        p = sub.add_parser(name)
        p.add_argument("--ui", required=True)
        p.add_argument("--lang", required=True)
        p.add_argument("--ui-branch", default="codex/plan-831-dev")
        p.add_argument("--lang-branch", default="codex/plan-831-dev")
        p.add_argument("--receipts", required=True)
    d = sub.add_parser("drill")
    d.add_argument("--receipt-out", default=None,
                   help="收据落盘路径（缺省 <组>/scratch-ui-repo-fixture/switch-drill-receipt.json）")
    args = parser.parse_args()

    if args.cmd == "drill":
        return cmd_drill(args)

    ctx = Ctx(args)
    try:
        if args.cmd == "plan":
            return step_plan(ctx)
        if args.cmd == "land-ui":
            return _advance(ctx, ctx.ui, "ui", ctx.ui_branch, "ui-landed")
        if args.cmd == "switch-lang":
            return _advance(ctx, ctx.lang, "lang", ctx.lang_branch, "lang-switched")
        if args.cmd == "verify":
            return step_verify(ctx)
        if args.cmd == "status":
            return step_status(ctx)
    except RuntimeError as exc:
        log(f"运行错: {exc}")
        return EXIT_ERROR
    return EXIT_USAGE


if __name__ == "__main__":
    sys.exit(main())
