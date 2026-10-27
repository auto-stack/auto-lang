# PLAN-857 独立最终复审 R4

- `stage=review`、`review_scope=final`；独立子agent（未参与实施），Plan revision 1。受审 R3 修复提交 `81b0974ac2512126790b0ef2af79235edf45b485`（`fix(autographics): bind runner builds and exact locks`），基线 `4d9fa8e8c4ad00f5c88d9d1c5be619f0c592b837`；同一工作树 `D:/autostack/.wt/lang-857/auto-lang` / `plan-857-dev`，复审首末 HEAD 一致、工作树 clean（含 untracked 检查）。依赖：SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`（source-lock `f5d7f1f8...` 在 base/受审提交间逐字节未变）、两把 Cargo.lock 原始 SHA 分别 `2b131d3dd3...`/`c3df168e72...` 且跨 e20b95e1→4d9fa8e8→81b0974a 三个提交未变。
- **结论：pass。** F-857-R3-01、F-857-R3-02 的身份不变量经静态审查、全量负控重放与 4 个复审自构对抗用例均闭合；无新发现。R1 行为/性能与 R2/R3 像素证据按无关差异复用（本轮 diff 仅 Python 验证工具+来源清单，Rust/AGP/两锁/SDK/场景/配置零变化），且本轮正控另以真实 856/857 双 80 帧 GPU 窗口逐像素重证默认/Home 等值。merge-owned 完整门仍 pending（见文末）。
- [baseline.json.txt](baseline.json.txt) 记录提交/依赖/锁/diff 清单/重放命令/全部证据 SHA-256；[spec-delta.md](spec-delta.md) 为冻结规范增量（与 R3 逐字相同，无合同变更）。被审方 29 个证据文件（repair-r3/evidence.json.txt）逐项重算 SHA-256 全部匹配。原始数据位于 `D:/autostack/plan-artifacts/plan-857/review-r4/`。

## F-857-R3-01 关闭核实（身份绑定·精确锁路径）

- **静态**：新增 `tools/runner_provenance.py::lock_map` 规范化反斜杠后强制恰好两个规范键（`autographics-auto-driver/Cargo.lock`、`autographics-prototype/Cargo.lock`），拒绝外来键/漏项/别名/重复；`check_locks` 对每个精确键做三方交叉：收据值↔实际文件原始字节 SHA↔856 与 857 两份来源清单的 canonical（LF 归一）hash，任一不符即抛错。比较器摘要 `locks` 字段改写为 `check_locks` 返回的已验证映射——摘要不再可能误标（R3 缺陷的直接闭合）。`m2_evidence.record_runner` 对无证明 runner 写 `locks={}`/`sdk_sha256=None`，不再自带锁声明。比较器 `read_json` 改用 `provenance.load`（拒绝 JSON 重复键、符号链接/junction）。
- **重放（被审方负控，全部真实执行）**：单测级 both-wrong-lock-names/missing-lock/aliased-locks/one-sided-wrong-lock/both-wrong-lock-values/duplicate-json 六例（`test_runner_provenance.py`，exit 0 整体、每例具名拒绝）；完整比较器级 double-wrong-locks/double-wrong-values/missing-lock/alias-locks 在 current 与 reuse 两矩阵各重放一次（`test_default_3d_baseline.py`，12 负控全 exit 1、stderr 含预期断言、无 summary.json）。修复前行为对照：`before-replay.log.txt`（SHA 见被审方证据）实录 base 提交上双侧错锁 exit 0/PASS——本轮同型输入全部非零。
- **复审自构对抗用例 AD-01**：单侧锁键伪造——基线收据保持真实两锁，仅把当前 runner 收据 `locks` 换成 `README.md`/`Cargo.toml` 的真实 hash（被审方矩阵只做过双侧同改与单侧错值，未做过"基线好+当前错键"的完整比较器组合）→ 比较器 exit 1 `unexpected, duplicate or aliased lock path`，无 summary（[ad01.stderr.txt](ad01.stderr.txt)）。**AD-04**：当前 runner 收据注入重复 JSON 键 → exit 1 `duplicate JSON key: sha256`（[ad04.stderr.txt](ad04.stderr.txt)）。
- 正控：两矩阵正控以真实当前受控构建 runner 独立重跑，summary `locks` 恰为两个真实锁路径与实际文件 hash（见 [current-controls-positive-summary.json.txt](current-controls-positive-summary.json.txt)）。

## F-857-R3-02 关闭核实（身份绑定·受控构建来源）

- **静态**：新增 `tools/build_runner_proof.py`（干净提交、verify_plan857+prepare_sdk verify-only、cargo JSON 事件留存、构建前后 commit/manifest 不变断言、外部新 target、proof+context 双文件互锁、按 binary SHA 注册表登记防冲突）；`runner_provenance.verify_build` 重校验 context 哈希与逐字段一致、binary↔proof↔cargo 事件产物三者绑定、构建提交的 committed identity↔归档 identity、SDK/RUSTFLAGS/RUST_MIN_STACK/RUSTC_WRAPPER/rustc/cargo 与当前环境一致、锁三方一致；跨提交分支强制 `allow_reuse`、identity 变更仅限 `tools/*.py`、git diff 仅限验证工具+清单，且 `build_commit` 恒为真实旧提交。`record_runner` 无证明时 `origin_verified=False`/`code_commit=None`/仅记 `observed_checkout_commit`；`verify_runner` 拒绝未验证收据。比较器要求 `origin_verified is True`、`verification_commit==expected`、`code_commit==validated.build_commit`、target 恰为 `bin/autographics-authoring`；摘要新增 `binary_build_commit`/`reuse`/`reuse_scope`/`build_proof_sha256`——旧 runner 不可能再被标为当前构建。
- **重放（被审方负控+正控，全部真实执行）**：old-runner-without-proof（真实旧 exe 录得 unverified、无构建归属）、actual-old-home-without-proof（真实旧 e20b95e Home 报告/PNG 重组收据 + 无证明旧 runner → exit 1 `current runner source identity missing`，无 summary）、wrong-binary-with-proof、forged-build-commit、cross-commit-without-approval、explicit-verified-reuse（保留 build_commit `4d9fa8e8c`、verification_commit `81b0974ac`）；完整矩阵 reuse-without-approval exit 1 `cross-commit runner reuse requires explicit approval`、改 RUSTFLAGS 栈值后 exit 1 `build configuration mismatch`（[reuse-without-approval.stderr.txt](reuse-without-approval.stderr.txt)、[reuse-config-changed.stderr.txt](reuse-config-changed.stderr.txt)）；reuse 正控 summary 显式 `reuse=true, binary_build_commit=4d9fa8e8c, reuse_scope=3d-default-pixels-only`（[reuse-controls-positive-summary.json.txt](reuse-controls-positive-summary.json.txt)）。"旧/新 Rust 源码未变不能替代显式复用证明"成立：base→受审提交 diff 零 Rust 变更，未开 `--allow-reuse` 仍被拒。
- **复审自构对抗用例 AD-02**：完全重签名的构建证明——复制真实当前 proof+context，把 `binary_path`/`binary_sha256` 一致改为旧 e20b95e runner（bc18c3ae）副本，重算 context SHA 并回填 proof 的 `build_context_sha256`（哈希自洽），保留真实 cargo 事件文件 → `verify_build` 直调与生产入口 `record_runner --build-proof` 均拒绝：`Cargo artifact does not bind runner`（[ad02-result.txt](ad02-result.txt)）——最后一道 cargo 产物绑定防线挡住了换芯证明。**AD-03**：真实 reuse 收据部分字段伪造——`code_commit` 改标当前提交、`build_provenance` 原样保留 → 比较器 exit 1 `runner build attribution mismatch`，无 summary（[ad03.stderr.txt](ad03.stderr.txt)）。
- **相邻路径 RF**：`check_editor_viewport2d` 所依赖的注册表回退路径实测——对当前 exe 无参 `record_runner`（无 env、走 build-proofs 注册表）得 `origin_verified=True, build_commit=verification_commit=81b0974ac, reuse=False`，`verify_runner` 通过（[registry-fallback-runner.json.txt](registry-fallback-runner.json.txt)）。被审方 2D 相邻窗口执行证据（`current-2d.log.txt`，哈希已验）实录 `PASS viewport2d: 2 completed OS/GPU runs; actual transform/content/pixels`。

## 重放与断言方向总核

- 独立重放矩阵与被审方收据**逐字节同结果**：provenance-final2/results.json、current-controls/results.json、reuse-controls/results.json 及两份 positive summary 的 SHA-256（547a310c/b14cca9f/e667f63d/d1dc6c41/5a1e3c72）与 repair-r3/evidence.json.txt 记录完全一致——重放命令、退出码、stderr hash 全同。负控断言方向全部为"非零退出+具名原因+无 summary.json"，正控为"exit 0+summary.json+逐像素相等"；`test_default_3d_baseline.py` 自身对每例强制这三条，复审复核其断言方向正确。
- 独立执行输出目录：`D:/autostack/plan-artifacts/plan-857/review-r4/{provenance-replay,current-controls-replay,reuse-controls-replay,adversarial,registry-fallback-runner}`。

## 范围、健康与复用边界

- `git show 81b0974ac --stat`：9 个文件全部位于 `experimental/autographics-auto-driver/tools/**` 与 `research/plan857-identity.json`（T-05/06 允许写面），无 Rust/AGP/锁/测试入口改动，无范围蔓延。对照 base 逐 hunk 复核：全部为新增检查或收紧（旧 `code_commit==expected` 单点比较被"构建提交+验证提交"双绑定取代；旧锁循环被精确路径三方校验取代；`read_json` 获得重复键防御），未发现删除检查、放宽断言或 workaround。
- 健康检查：8 个 Python 工具 `py_compile` 全过；`runner_provenance.py` 为无打印库，脚本仅在结论处单行 PASS/FAIL，无杂散调试输出、无 TODO/FIXME；比较器失败路径统一 `FAIL: <原因>` + 返回 1，3d 工具未验证 runner 以 `SystemExit` 非零退出，测试 harness 对意外接受抛错。来源清单与本轮 8 个触及文件树内 hash 逐项一致（canonical 约定），verify_plan857 480 项在每次重放中独立通过。
- 复用边界：R2/R3 已核的 R1 三项行为反例/正控、144 拾取点、126 轴回投、855/857 性能与 owner 证据——本轮 diff 不触 Rust/依赖/输入，按无关差异继续复用，不视为重跑；本轮另以真实窗口正控重证关键像素 oracle。两处既有限定保留：SDK 146 项历史 warnings 未新增；共面 zero-bias 单例结论未夸大。

## 验收与后续

- AC-01/06 身份链本轮闭合（来源/SDK/两锁/构建证明/复用审计/失败传播全部实测）；AC-02..05 实现与输入未变，证据按无关差异复用且关键像素正控重证；AC-07 复审项（清单核对/遗漏扫描/健康检查/规范增量核验）完成。SD-01..04 为待 merge 增量（冻结副本本目录）。`pending_closeout_gates`：完整双端、受影响 AGP、原新 native-owner-content 门仍归 merge（T-08），pass 不视为这些门已闭合；复审不合并、不清理工作树。
