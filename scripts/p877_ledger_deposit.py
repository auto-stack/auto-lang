"""PLAN-877 ledger deposit: append review + merge entries, surgical, with backup.

Mirrors the pNNN_ledger_deposit.py convention: guarded offline projection,
explicit backup, staged diff asserted before write.
"""
import copy
import json
import pathlib
import shutil
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[1]
LEDGER = ROOT / ".autoos" / "specs.json"
BACKUP = ROOT / ".autoos" / "specs.json.bak-p877-merge"

entries = [
    {
        "id": "P877-GALLERY",
        "title": "PLAN-877 delivery: gallery basic six (GY-A)",
        "content": "本地 ag-examples 仓（master 6a223d9，自 4df37bd 锚点经 codex/plan-877-dev 合入）：六逻辑 ID/十子入口全 ready；固定引擎锁 8b12632d7/SDK 7b987ca8(860 副本 681 文件 PASS)/两锁 47176cc0+bc7287b5/runner 498ce814；gallery.py validate 严格契约（集合相等/路径安全/资产哈希/许可/Ready 门）+28 结构负控；66 断言验收（像素级 G01、GLB 双部件 G02、world 链式数值 G03、真实 OS 编辑探针 G04、独立 slab 对账 G05、角色探针+位移对照 G06）；六单项包离线 A→B 恒等+篡改拒绝。实测边界：2D 合成序=alpha 升序+数组序；仅 windows-native-x64。",
        "status": "verified",
        "file": "docs/specs/auto-graphics/gallery-basic-six.md",
        "tags": ["PLAN-877", "delivered", "content:017b959", "landed:6a223d9"],
        "module": "auto-graphics",
    },
    {
        "id": "P877-REVIEW-R1",
        "title": "PLAN-877 independent final R1 PASS (user-scoped gates)",
        "content": "复审同会话/同模型（已声明），从冻结工件重建：包内验证链重跑三段 PASS（443/681/479+AGP 锁）；validate+28 负控 OK；抽样门禁（seed 20261028 抽 g03）12 断言新鲜绿；六包无窗口 A→B 全 PASS。用户裁定 2026-10-28：(1) ComputerUse 式 OS 输入自动化验收暂停；(2) 门禁抽样约 1/10；(3) 门禁只测变更相关 case，全量回归留里程碑——探针面证据复用内容 017b959（工具修复零内容变更）。发现三项全闭：中断重跑曾写 passed:false 入 acceptance（恢复全过记录）、harness 无报告崩溃（补合成失败）、包冒烟改无窗口 --inspect。",
        "status": "verified",
        "file": "docs/specs/auto-graphics/gallery-basic-six.md",
        "tags": ["PLAN-877", "reviewed:c33a39b", "ruling:os-probe-pause", "ruling:sampled-gates"],
        "module": "auto-graphics",
    },
]

data = json.loads(LEDGER.read_text(encoding="utf-8"))
before = json.dumps(data, ensure_ascii=False, sort_keys=True)
version_before = data["version"]
reports = data["sections"][0]["items"]
reviews = data["sections"][5]["items"]
assert not any(i["id"] in {"P877-GALLERY", "P877-REVIEW-R1"} for i in reports + reviews), "already deposited"
reports.append(copy.deepcopy(entries[0]))
reviews.append(copy.deepcopy(entries[1]))
data["version"] = version_before + 1
after = json.dumps(data, ensure_ascii=False, sort_keys=True)

added = after.count("P877-") - before.count("P877-")
assert added == 2, f"unexpected diff shape: {added}"
shutil.copyfile(LEDGER, BACKUP)
LEDGER.write_bytes(json.dumps(data, ensure_ascii=False, indent=2).encode("utf-8"))
diff = subprocess.run(["git", "-C", str(ROOT), "diff", "--stat", "--", ".autoos/specs.json"],
                      capture_output=True, text=True).stdout
print(f"ledger v{version_before} -> v{data['version']}; +P877-GALLERY +P877-REVIEW-R1; backup {BACKUP.name}")
print(diff.strip()[:200])
