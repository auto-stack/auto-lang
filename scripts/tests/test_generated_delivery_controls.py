#!/usr/bin/env python3
"""PLAN-930 generated-delivery-controls：生成工程交付回归的便宜负控。

只放四类确定性便宜控制（不发起重 Cargo/GUI、不编 ACE、不造新叶目录）：

1. 真身份定位：driver 的受管 member.json 成员解析（910 身份合同）——正确
   项目恰一 front 成员、同 basename 不同根互不碰撞、错 owner/旧无身份/
   多候选具名拒绝、旧 basename 布局不按名认领。
2. 产物比较：与 producer 源的逐字节对拍（单字节篡改/缺失判红）。
3. 坏来源：消费侧 env 隔离（开发根覆盖剥离 + RUSTC_WRAPPER 空串机器事实）
   与非法登记的拒绝面。
4. 子失败传播：python 固定组 runner 的发现完备性（删 case 判红）与子项
   失败整体非零传播。

runner=python 单一文件（scripts/test-groups.toml 注册 generated-delivery-
controls；cargo t generated-delivery-controls 显式运行）。任一必需控制失败
整体非 0；不默认发起重构建，真实 A/B 构建义务仍由原 driver 显式执行。
"""
from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts/test-governance"))

_DRIVER = ROOT / "test/fixtures/modularity/generated-ui-consumer/driver.py"
_spec = importlib.util.spec_from_file_location("gui_driver", _DRIVER)
driver = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(driver)

PROJ = _DRIVER.parent / "proj"


def _member_record(project_dir: Path, role: str = "front") -> dict:
    return {
        "schema": "generated-member/v1",
        "role": role,
        "project_path": driver.canonical_key(project_dir),
        "created_at_ms": 0,
    }


def _make_workspace_with_member(root: Path, proj_name: str, *, extra: list | None = None,
                                no_record: bool = False) -> tuple[Path, Path, Path]:
    """建 (ws_dir, project_dir, member_dir)；extra=额外成员目录名清单。"""
    ws = root / "rust-workspace"
    ws.mkdir(parents=True)
    proj = root / proj_name
    proj.mkdir(parents=True)
    member = ws / f"{proj_name}-12345678"
    member.mkdir()
    if not no_record:
        (member / "member.json").write_text(
            json.dumps(_member_record(proj), ensure_ascii=False), encoding="utf-8")
    for extra_name in extra or []:
        (ws / extra_name).mkdir()
    return ws, proj, member


class MemberIdentity(unittest.TestCase):
    """真身份定位：driver 实际解析函数 + 受管登记（member.json）形态。"""

    def test_correct_project_resolves_exactly_one_front_member(self):
        with tempfile.TemporaryDirectory() as temp:
            ws, proj, member = _make_workspace_with_member(Path(temp), "myproj")
            self.assertEqual(driver.resolve_front_member(ws, proj), member)

    def test_same_basename_different_roots_do_not_collide(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            resolved = []
            for tag in ("a", "b"):
                root = base / tag
                ws, proj, _ = _make_workspace_with_member(root, "shared-name")
                resolved.append(driver.resolve_front_member(ws, proj))
            self.assertTrue(all(r is not None for r in resolved))
            self.assertNotEqual(resolved[0], resolved[1])
            self.assertNotEqual(
                driver.canonical_key(base / "a" / "shared-name"),
                driver.canonical_key(base / "b" / "shared-name"),
            )

    def test_wrong_owner_record_not_claimed(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            ws = root / "rust-workspace"
            ws.mkdir(parents=True)
            proj = root / "myproj"
            proj.mkdir(parents=True)
            member = ws / "myproj-12345678"
            member.mkdir()
            rec = _member_record(root / "elsewhere")  # project_path 指向他处
            (member / "member.json").write_text(
                json.dumps(rec, ensure_ascii=False), encoding="utf-8")
            self.assertIsNone(driver.resolve_front_member(ws, proj))

    def test_legacy_no_identity_dir_not_claimed(self):
        with tempfile.TemporaryDirectory() as temp:
            ws, proj, _ = _make_workspace_with_member(Path(temp), "myproj", no_record=True)
            self.assertIsNone(driver.resolve_front_member(ws, proj))

    def test_multiple_candidates_rejected(self):
        with tempfile.TemporaryDirectory() as temp:
            ws, proj, _ = _make_workspace_with_member(
                Path(temp), "myproj", extra=["myproj-87654321"])
            (ws / "myproj-87654321" / "member.json").write_text(
                json.dumps(_member_record(proj), ensure_ascii=False), encoding="utf-8")
            self.assertIsNone(driver.resolve_front_member(ws, proj))

    def test_basename_only_layout_not_claimed_by_name(self):
        # 旧 basename 布局（无 member.json）与 910 新成员并存 → 只认登记成员。
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            ws, proj, member = _make_workspace_with_member(root, "myproj")
            legacy = ws / "myproj"  # 旧无身份目录
            legacy.mkdir()
            self.assertEqual(driver.resolve_front_member(ws, proj), member)

    def test_canonical_key_normalization(self):
        with tempfile.TemporaryDirectory() as temp:
            p = Path(temp) / "Proj Dir"
            p.mkdir()
            key = driver.canonical_key(p)
            self.assertEqual(key, key.lower())
            self.assertNotIn("\\", key)
            self.assertEqual(key, str(p).replace("\\", "/").lower())

    def test_back_role_not_front(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            ws = root / "rust-workspace"
            ws.mkdir(parents=True)
            proj = root / "myproj"
            proj.mkdir(parents=True)
            member = ws / "myproj-12345678"
            member.mkdir()
            (member / "member.json").write_text(
                json.dumps(_member_record(proj, role="back"), ensure_ascii=False),
                encoding="utf-8")
            self.assertIsNone(driver.resolve_front_member(ws, proj))


class ArtifactComparison(unittest.TestCase):
    """产物比较：与 producer 源逐字节对拍，单字节篡改/缺失确定性红。"""

    @staticmethod
    def _bundle_copy(root: Path, src_dir: Path, rels: list[str]) -> dict:
        bundle = root / "bundle" / "assets"
        bundle.mkdir(parents=True)
        manifest = {"resources": []}
        for rel in rels:
            src = src_dir / rel
            dst = bundle / rel
            dst.parent.mkdir(parents=True, exist_ok=True)
            dst.write_bytes(src.read_bytes())
            manifest["resources"].append({"rel": rel})
        return manifest

    def test_producer_byte_identity_green(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            rels = ["avatar.png", "app.at"]
            manifest = self._bundle_copy(root, PROJ / "src" / "front", rels)
            problems = [
                rel for rel in [r["rel"] for r in manifest["resources"]]
                if not (root / "bundle" / "assets" / rel).is_file()
                or (root / "bundle" / "assets" / rel).read_bytes()
                != (PROJ / "src" / "front" / rel).read_bytes()
            ]
            self.assertEqual(problems, [], "合法副本应逐字节一致")

    def test_single_byte_flip_detected(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            rels = ["avatar.png"]
            manifest = self._bundle_copy(root, PROJ / "src" / "front", rels)
            target = root / "bundle" / "assets" / "avatar.png"
            tampered = bytearray(target.read_bytes())
            tampered[len(tampered) // 2] ^= 0xFF
            target.write_bytes(bytes(tampered))
            problems = [
                rel for rel in [r["rel"] for r in manifest["resources"]]
                if not (root / "bundle" / "assets" / rel).is_file()
                or (root / "bundle" / "assets" / rel).read_bytes()
                != (PROJ / "src" / "front" / rel).read_bytes()
            ]
            self.assertEqual(problems, ["avatar.png"], "单字节篡改必须判红")

    def test_missing_resource_detected(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            rels = ["avatar.png", "app.at"]
            manifest = self._bundle_copy(root, PROJ / "src" / "front", rels)
            (root / "bundle" / "assets" / "app.at").unlink()
            problems = [
                rel for rel in [r["rel"] for r in manifest["resources"]]
                if not (root / "bundle" / "assets" / rel).is_file()
            ]
            self.assertEqual(problems, ["app.at"], "缺失资源必须判红")


class BadSource(unittest.TestCase):
    """坏来源：消费侧 env 隔离与非法登记拒绝面。"""

    def test_consumer_env_isolates_development_roots(self):
        # consumer_env 的 extra 是**追加**覆盖（交付侧注入），测剥离须从
        # os.environ 起点出发：把开发根覆盖放入环境，断言消费侧不携带。
        from unittest import mock
        with mock.patch.dict(os.environ, {
            "AUTO_RUST_WORKSPACE": "leak",
            "AUTO_LANG_CRATE": "leak",
            "AUTO_LANG_ROOT": "leak",
            "AUTO_DELIVERY_SOURCE_ROOT": "leak",
            "CARGO_TARGET_DIR": "leak",
            "RUSTC_WRAPPER": r"C:\Users\zhaop\.cargo\bin\sccache.exe",
        }):
            env = driver.consumer_env()
            for key in ("AUTO_RUST_WORKSPACE", "AUTO_LANG_CRATE", "AUTO_LANG_ROOT",
                        "AUTO_DELIVERY_SOURCE_ROOT", "CARGO_TARGET_DIR"):
                self.assertNotIn(key, env, f"开发根覆盖 {key} 不得泄漏进消费侧")
            self.assertEqual(env.get("RUSTC_WRAPPER", None), "",
                             "RUSTC_WRAPPER 须显式置空串（036 机器事实）")

    def test_foreign_canonical_key_never_matches(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            # member.json 指向基路径 A，解析以 B 为项目 → 具名拒绝
            ws, proj, member = _make_workspace_with_member(base / "one", "myproj")
            other = base / "two"
            other.mkdir(parents=True)
            self.assertIsNone(driver.resolve_front_member(ws, other))


class SubFailurePropagation(unittest.TestCase):
    """子失败传播：python 固定组 runner 的发现完备与失败整体非零。"""

    def test_discovery_enumeration_is_complete(self):
        # 钉住本文件全量 case 身份——删 case 即判红（V-04 必需项守恒）。
        probe = ROOT / "scripts/test-governance/python_tests.py"
        listed = subprocess.run(
            [sys.executable, str(probe), "--file",
             str(ROOT / "scripts/tests/test_generated_delivery_controls.py"), "--list"],
            capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=120,
        )
        self.assertEqual(listed.returncode, 0, listed.stderr[-500:])
        doc = json.loads(listed.stdout)
        self.assertEqual(doc["schema"], "python-unittest-discovery/1")
        ids = {t["id"].rsplit(".", 1)[-1] for t in doc["tests"]}
        expected = {
            "test_correct_project_resolves_exactly_one_front_member",
            "test_same_basename_different_roots_do_not_collide",
            "test_wrong_owner_record_not_claimed",
            "test_legacy_no_identity_dir_not_claimed",
            "test_multiple_candidates_rejected",
            "test_basename_only_layout_not_claimed_by_name",
            "test_canonical_key_normalization",
            "test_back_role_not_front",
            "test_producer_byte_identity_green",
            "test_single_byte_flip_detected",
            "test_missing_resource_detected",
            "test_consumer_env_isolates_development_roots",
            "test_foreign_canonical_key_never_matches",
            "test_sub_failure_propagates_nonzero",
            "test_discovery_enumeration_is_complete",
        }
        self.assertEqual(ids, expected, "删/加 case 必须显式修订本控制（整体非 0 守恒）")

    def test_sub_failure_propagates_nonzero(self):
        # runner 对子项失败必须整体非零（伪 summary/吞失败即红）。
        probe = ROOT / "scripts/test-governance/python_tests.py"
        with tempfile.TemporaryDirectory() as temp:
            failing = Path(temp) / "failing_probe.py"
            failing.write_text(
                "import unittest\n"
                "class T(unittest.TestCase):\n"
                "    def test_red(self):\n"
                "        self.assertTrue(False, 'must propagate')\n",
                encoding="utf-8",
            )
            listed = subprocess.run(
                [sys.executable, str(probe), "--file", str(failing), "--list"],
                capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=120,
            )
            self.assertEqual(listed.returncode, 0)
            case_id = json.loads(listed.stdout)["tests"][0]["id"]
            ran = subprocess.run(
                [sys.executable, str(probe), "--file", str(failing), "--case", case_id],
                capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=120,
            )
            self.assertNotEqual(ran.returncode, 0, "子项失败必须整体非零")
            report = json.loads(ran.stdout)
            self.assertEqual(report["failures"], [case_id])


if __name__ == "__main__":
    unittest.main()
