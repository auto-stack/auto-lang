---
plan_id: PLAN-586
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: Stage B 清障二——桌面注册表三源聚合（extra roots 泛化 + apps.manifest 聚合 + 三轨 parity）
author: [zhaopuming, ZCode]
created_at: 2026-09-07
updated_at: 2026-09-07

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-man/vue, auto-lang/ui]  # 受影响的 specs 路径
current_step: 0
total_steps: 7
---

# [PLAN-586] Stage B 清障二——桌面注册表三源聚合（extra roots 泛化 + apps.manifest 聚合 + 三轨 parity）

## 变更摘要

Stage B 桌面域搬迁（[Design 01](
../../auto-os/docs/design/01-stage-b-desktop-migration.md)，auto-os 仓 Plan 584
定案）§4-P3 清障批次，§7 拆解表 **P-3**，主仓 auto-lang。现状三轨注册表
各自锚定框架仓 `examples/ui`，仓外两源未聚合：① auto-os `apps/`（local
形态，P-5 本体批落位）② auto-os `apps.manifest` repo 形态条目（kanban
17100/17101 首例）。本计划以「复用既有机制、零新概念」为纲三步走：

1. **extra roots 兄弟探测泛化**：默认探测列表由 `[../auto-os-config/auto]`
   扩为 `[../auto-os-config/auto, ../auto-os/apps]`——apps/ 每个含 pac.at
   的子目录即一个 local app root；缺失兄弟静默跳过（solo 检出不炸，既有
   语义）。
2. **apps.manifest 聚合**：按解析序定位 auto-os `apps.manifest`，repo 形态
   条目经既有 remote/URL 机制（`remote-apps.json`，Plan 516 G4）注册为
   远程窗（端口已在 manifest）。manifest 条目 schema 扩展（local 形态、
   apps/ 目录）在本计划定稿。
3. **三轨 parity**：vue（`vue.rs`）/ vm（`app_registry`）/ iced
   （`DesktopOptions` 传参链）注册表语义同源同步，缺一即红。

**排序依据**（Design 01 §7）：P-3 无前置、无窗口约束；P-7 shell pack 批
依赖 P-3 落地（同族解析序机制先行复用）；P-5 本体批前置之一（搬完即可跑）。

## 目标

1. **V5 三源聚合**：从 auto-os 启动双轨桌面（§3-a 包装等价形态——env 注入
   或解析序默认命中），注册表呈现三类条目：框架 demo（默认注册表）+
   本仓 `apps/`（local 形态）+ kanban（repo 形态远程窗）。
2. **框架仓零回归**：auto-lang 原样启动不变——examples/ui 默认注册表、
   028-launcher/教学 demo 启动方式、env 覆盖语义（`AUTO_DESKTOP_APPS`/
   `AUTO_DESKTOP_APPS_EXTRA`）全部保持。
3. **manifest schema 定稿**：条目形态 `kind: repo | local`、端口/状态字段
   与校验规则成文（落 auto-os 侧 Design 01 §1-C 注记或 schema 文件）。
4. **第一验收用例**：auto-kanban（repo 形态；583 已有 vm 模式 586 卡对账
   基线可复用）。

## 架构方案

| 件 | 落点（auto-lang） | 复用先例 |
|---|---|---|
| D1 extra roots 泛化 | `crates/auto-man/src/vue.rs` `desktop_extra_app_roots`（:5507）+ `crates/auto-lang/src/ui/app_registry.rs` `extra_roots_from`（:245）双侧 | Plan 559 W3 兄弟探测（`../auto-os-config/auto`）；Plan 501 多根聚合 |
| D2 manifest 聚合 | vue.rs / app_registry.rs 按解析序读 `../auto-os/apps.manifest` | `remote-apps.json` 远程窗机制（Plan 516 G4）；解析序约定（env → 兄弟 → 主检出） |
| D3 schema 定稿 | auto-os `apps.manifest` 条目扩展（跨仓交付物，注记入 Design 01 §1-C） | 伞形清单现状（AGENTS §4：id/name/repo/kind/ports/status/added） |
| D4 三轨 parity | iced 轨 `DesktopOptions`（renderer.rs:10953 `apps_dir` 同型）传参链 | `DesktopOptions.apps_dir` 先例（Plan 463 T7 `--apps-dir`） |

## 技术栈

Rust（auto-man + auto-lang/ui）+ auto-os `apps.manifest`（JSON）+ 既有测试
基建（app_registry 单测 / desktop_mcp / autoui-verifier）。

## 需求分析与背景调查

（2026-09-07 起草期实测锚点；执行时以 grep 重新定位。）

- **vue 轨**：`vue.rs:5484` `desktop_apps_dir`（`AUTO_DESKTOP_APPS` env
  覆盖 / 默认 `<root>/examples/ui`）；`:5507`
  `desktop_extra_app_roots`（`AUTO_DESKTOP_APPS_EXTRA` 路径表 / 默认兄弟
  探测 `../auto-os-config/auto`——`:5524` 注记 vm parity）。
- **vm 轨 parity**：`app_registry.rs:245` `extra_roots_from`（env 表 +
  兄弟探测同语义；:579-588 既有单测含「缺兄弟返回空」语义锚）。
- **iced 轨**：`ui_desktop.rs:17-37` `default_apps_dir` 编译期
  `CARGO_MANIFEST_DIR` 锚定框架仓 examples/ui + `--apps-dir` 覆盖；
  `renderer.rs:10953` `DesktopOptions.apps_dir`（装配 LaunchApp 注册表，
  :11284）。
- **消费端现状**：auto-os `apps.manifest` 已有 kanban repo 条目
  （17100/17101，2026-09-07 登记）；`apps/` 目录 P-5 前不存在——D1 泛化
  后缺目录静默跳过，不炸 solo 检出（先验证语义再等资产落位，故 P-3 先于
  P-5 的排序成立）。
- **远程窗机制**：`remote-apps.json`（Plan 516 G4）在案，repo 形态条目
  的注册通道现成。
- **边界**：§3-a 包装脚本（auto-os `scripts/desktop.ps1/sh`）非本计划硬
  交付——验证以 env 注入等价形态承载（见待澄清 3）；`auto desktop` 一等
  子命令 = Stage C 候选，不涉。

## 详细设计

### D1 extra roots 兄弟探测泛化

- 默认探测列表 `[../auto-os-config/auto]` → `[../auto-os-config/auto,
  ../auto-os/apps]`；apps/ 下每个**含 pac.at 的直接子目录** = 一个 local
  app root（与 auto-os-config/auto 单根形态同律，仅目录层级差一档）。
- vue（`desktop_extra_app_roots`）与 vm（`extra_roots_from`）**双侧同步**
  ——parity 测试锁两轨默认列表一致；缺失兄弟静默跳过语义保持
  （app_registry.rs:588「Z:/nowhere 返回空」既有锚不改语义只扩列表）。

### D2 apps.manifest 聚合

- **读取位（默认设计，待澄清 1 确认）**：框架侧按解析序读 auto-os
  `apps.manifest`——`AUTO_OS_ROOT`（新 env，可选）→ 兄弟 `../auto-os`
  → 兜底 `D:/autostack/auto-os`（对齐 AGENTS 解析序三段式）。manifest
  缺失 = 静默无聚合（框架仓 solo 完全不感知）。
- repo 形态条目 → 既有 remote/URL 机制注册远程窗（端口取 manifest
  `ports` 字段）；local 形态条目（P-5 后实存）→ 归并入 D1 apps/ 根扫描，
  manifest 仅作策展元数据（title/icon/可见性）。
- 解析失败（坏 JSON/未知 kind）行为：跳过该条目 + 日志一行，不炸启动
  （对齐「solo 检出不炸」精神）。

### D3 manifest schema 定稿

- 条目：`{ id, name, repo?, kind: "repo"|"local", ports, status, added }`；
  repo 形态必填 `repo` + `ports`；local 形态对应 `apps/<id>/` 目录存在性
  由扫描面自然裁定（manifest 不重复持有路径）。
- 定稿落点：auto-os 侧 Design 01 §1-C 注记更新（跨仓小改，随本计划执行
  一并交付）；schema 校验规则（必填/端口带）入 D2 解析代码的文档注释。

### D4 三轨 parity + iced 轨传参链

- iced 轨：`DesktopOptions` 增 `extra_app_roots: Option<Vec<PathBuf>>`
  （沿 `apps_dir: Option<PathBuf>` 先例）；`ui_desktop.rs` 默认值与
  vue/vm 同源（默认兄弟探测同样命中 auto-os/apps——三轨默认语义一致，
  iced 轨不再单独硬编码）。
- parity 测试：vue/vm/iced 三轨默认探测列表、env 覆盖、manifest 聚合
  结果同源断言（缺一即红）。

## 测试设计

（作用域：Category B——涉 auto-man + auto-lang/ui；`cargo check -p
auto-man -p auto-lang` + app_registry/vue 模块测试；折叠前 `cargo tf`。
不触 aavm/trans/book 路径，零 taa/tt/tb 触发。）

- 单测：extra roots 泛化双侧（默认列表断言 + 缺兄弟静默）；manifest
  解析（repo/local/坏条目三分支）；三轨 parity 同源断言。
- 集成/V5 实机：双轨桌面从 auto-os 视角启动（env 注入等价形态）——
  注册表三类条目呈现 + 框架仓内原样启动零回归（028-launcher + 教学
  demo 抽查）；auto-kanban 第一用例（vm 模式，583 的 586 卡对账口径）。

## 验收标准

- [ ] extra roots 默认探测含 `../auto-os/apps`（vue+vm 双侧），solo 检出
      静默跳过既有语义保持（app_registry 既有单测锚不红）。
- [ ] apps.manifest 聚合生效：kanban repo 条目在双轨桌面呈现为远程窗；
      manifest schema（repo|local 扩展）定稿注记入 Design 01 §1-C。
- [ ] 三轨 parity 测试在案（默认列表/env 覆盖/聚合结果同源，缺一即红）；
      iced 轨 `DesktopOptions` 扩展沿 apps_dir 先例。
- [ ] 框架仓零回归：examples/ui 默认注册表、028-launcher/教学 demo 启动
      方式、既有 env 覆盖语义全部不变（V5 后半句）。
- [ ] V6 门档：`cargo check -p auto-man -p auto-lang` 绿 + 模块测试绿；
      折叠前 `cargo tf` 与基线一致（预存红按 564-Q6 台账豁免）。
- [ ] auto-kanban 第一验收用例通过（vm 模式远程窗呈现）。
- [ ] Design 01 §7 P-3 行状态注记回填（auto-os 侧小改随执行交付）。

## 执行步骤

（原子任务；每步完成后追加 [✅ 已完成] 一行证据。行号为 2026-09-07
master 基点，执行时以 grep 重新定位。worktree=
`D:/autostack/.wt/lang-586/auto-lang`；依赖仓 auto-os 仅 §1-C 注记小改，
不入 worktree。）

1. [ ] **D1 双侧泛化**：vue.rs `desktop_extra_app_roots` + app_registry.rs
   `extra_roots_from` 默认列表扩 `../auto-os/apps`（含 pac.at 子目录
   语义）。验证：`cargo check -p auto-man -p auto-lang` 零错。
2. [ ] **D1 单测**：默认列表断言 + 缺兄弟静默 + apps/ 子目录扫描
   （临时目录 fixture）。验证：`cargo t app_registry` 绿。
3. [ ] **D2 manifest 聚合**：解析序定位 + repo 条目→remote 机制接线 +
   坏条目跳过；local 形态字段解析（P-5 前无实存，单测 fixture 承载）。
   验证：模块测试绿。
4. [ ] **D4 iced 轨**：`DesktopOptions.extra_app_roots` + `ui_desktop.rs`
   默认值同源 + 三轨 parity 测试。验证：parity 测试三轨全绿。
5. [ ] **V5 实机双端**：env 注入等价形态从 auto-os 启动 vue+vm 双轨——
   三类条目呈现 + 框架仓原样启动零回归 + auto-kanban 远程窗用例
   （autoui-verifier `test_vm_mcp.py`/`test_vue_playwright.mjs`）。
   验证：截图/快照证据落 scratch/p586/。
6. [ ] **门档**：`cargo check` + 模块测试 + 折叠前 `cargo tf`（预存红
   按 564-Q6 豁免口径）。验证：tf 与基线一致。
7. [ ] **收口簿记**：Design 01 §1-C schema 注记 + §7 P-3 行状态回填
   （auto-os 侧）；KNOWN-DEBT 如有新登记。验证：两仓注记在案。

## 复审记录

（待 /auto-plan:review 填写。）

## 待澄清事项

1. **manifest 读取位**：默认设计=框架侧按解析序直读（与 D1 同族机制，
   P-7 shell pack 加载器可复用同套解析序代码）；备选=§3-a 包装侧物化
   （wrapper 把 manifest 翻译成 env/remote-apps.json，框架零改动但每次
   启动多一层翻译）。倾向前者（Design 01 §4-P3 方案 2 原文「按解析序
   读本仓 apps.manifest」即框架侧口径），开工前请用户确认。
2. **iced 轨 CLI 形态**：`ui_desktop.rs` 是否补 `--extra-apps` flag
   （对齐 `--apps-dir` 先例）——默认只做 `DesktopOptions` API 层
   （ui_desktop 是验收宿主示例非产品入口，最小面），flag 列可选增强。
3. **§3-a 包装脚本归属**：Design 01 §4-P3 验收原文经包装脚本启动；本
   计划以 env 注入等价形态承载验证（脚本本体非聚合机制的一部分）。
   建议脚本随 P-6 开工批落地（apps/ 实存后包装才有完整意义）；若用户
   希望 P-3 即交付脚本，执行步骤 5 前插入 auto-os 侧小任务即可。
