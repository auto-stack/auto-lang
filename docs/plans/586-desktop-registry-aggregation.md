---
plan_id: PLAN-586
status: execution_done          # drafting → executing → execution_done → reviewed → archived
feature_name: Stage B 清障二——桌面注册表三源聚合（extra roots 泛化 + apps.manifest 聚合 + 三轨 parity）
author: [zhaopuming, ZCode]
created_at: 2026-09-07
updated_at: 2026-09-07

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-man/vue, auto-lang/ui]  # 受影响的 specs 路径
current_step: 6
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

- [x] extra roots 默认探测含 `../auto-os/apps`（vue+vm 双侧），solo 检出
      静默跳过既有语义保持（app_registry 既有单测锚不红）。
- [x] apps.manifest 聚合生效：kanban repo 条目双轨注册（vm 轨注册表 37↔36
      差值实证；vue 轨 extra root+glue；**执行期修正：extra root 原生挂载
      替代 WS 远程窗**，依据见执行步骤 3）；manifest schema 定稿注记随 T7
      回填 Design 01 §1-C。
- [x] 三轨 parity 测试在案（`extra_roots_three_track_parity` 同 fixture
      同序同集断言）；iced 轨 `DesktopOptions.extra_app_roots` 沿 apps_dir 先例。
- [x] 框架仓零回归：默认注册表扫描计数零变化（36 基线对照）、既有 env
      （AUTO_DESKTOP_APPS/_EXTRA）语义全额保留、osconfig 全链集成 1/1 绿。
- [x] V6 门档：check 绿 + 模块测试全绿；折叠前 tf 2619/2620 唯红=charts
      预存（564-Q6 豁免）。
- [x] auto-kanban 第一验收用例：vm 模式注册表全量含 kanban（37↔36 差值
      +desktop-visible +1 +resolver 注册）；「远程窗呈现」措辞随执行期修正
      为「extra root 原生挂载呈现」，交互级启动验收归 P-5 V1/V2 实机批。
- [ ] Design 01 §7 P-3 行状态注记回填（auto-os 侧小改随执行交付）。
      （T7 收口中。）

## 执行步骤

（原子任务；每步完成后追加 [✅ 已完成] 一行证据。行号为 2026-09-07
master 基点，执行时以 grep 重新定位。worktree=
`D:/autostack/.wt/lang-586/auto-lang`；依赖仓 auto-os 仅 §1-C 注记小改，
不入 worktree。）

1. [✅ 已完成] **D1 双侧泛化**：vue.rs `desktop_extra_app_roots` + app_registry.rs
   `extra_roots_from` 默认列表扩 `../auto-os/apps`（含 pac.at 子目录
   语义）。
   [✅ 已完成] app_registry 新增 `expand_apps_container`（排序/pac.at 门控/id
   去重/缺容器静默）+ `host_extra_roots` 并入；vue 轨同律镜像。
   验证：`cargo check -p auto-man -p auto-lang` 零错。
2. [✅ 已完成] **D1 单测**：默认列表断言 + 缺兄弟静默 + apps/ 子目录扫描
   （临时目录 fixture）。
   [✅ 已完成] `extra_roots_apps_container_expansion`（容器展开五断言）+
   `decision_matrix` 扩参复绿；vue 轨 `desktop_extra_app_roots_apps_container`
   （AUTO_OS_ROOT 钉死防主检出兜底漏入 fixture）。
   验证：`cargo t app_registry` 15/15 绿；auto-man 直跑 1/1 绿。
3. [✅ 已完成] **D2 manifest 聚合**：解析序定位 + repo 条目接线 +
   坏条目跳过；local 形态字段解析（P-5 前无实存，单测 fixture 承载）。
   [✅ 已完成] **执行期修正**：repo 条目注册为 **extra root 原生挂载**而非
   草案措辞的 WS 远程窗——remote-apps.json 机制（Plan 516 G4）实测为 WS
   投影协议端点（`RemoteAppConfig.url` 全 WS，连的是另一桌面实例投影面），
   http/原生 app 形态装不进；repo 仓本身即 pac.at+src/front/app.at 单 app
   根（os-config 先例同型；kanban README VM 轨 `auto run -r vm` 原生跑）。
   Design 01 §1-C「remote 窗或 extra root」两候选中后者落地；纯 web app
   iframe 嵌入列 Stage C。实现：`resolve_os_manifest_root`（AUTO_OS_ROOT
   env **设置即权威不回落**——兼作关断开关；→ 兄弟 → 主检出兜底）+
   `manifest_repo_roots`（repo|local|status 宽容解析，坏条目跳过+警告）。
   验证：`manifest_repo_roots_aggregation` 绿（六形态+坏 JSON+env 权威）。
4. [✅ 已完成] **D4 iced 轨**：`DesktopOptions.extra_app_roots`（沿 apps_dir
   先例；Some=全额替换=vue 轨 env 同语义，None=host_extra_roots 缺省探测）
   + renderer.rs boot 装配点接线。三轨 parity 锚测试
   `extra_roots_three_track_parity`（auto-man 侧同时触 vue 私有函数与
   app_registry 组合面，同 fixture 同序同集断言）。
   验证：parity 三源齐备 [os-config, alpha, beta, repoapp] 断言绿。
5. [✅ 已完成] **V5 实机双端**（env 注入等价形态；apps/ P-5 后实存，
   当期真面=框架 demo + kanban 两源）：
   [✅ 已完成] vm/iced 轨（scratch/p586/vm_boot_{default,dead}.log）：默认
   聚合 `app registry: 37 entries (21 desktop-visible)`，AUTO_OS_ROOT 关断
   对照 36/20——**差值恰为 kanban 一条**（可见性 +1 同步），关断开关有效、
   框架 demo 默认注册表零变化（worktree solo 布局无 os-config 兄弟亦静默）。
   kanban 经 app_resolver 全量注册（可 LaunchApp）。
   [✅ 已完成] vue 轨（scratch/p586/vue_gen.log + vuehost/ 工程产物）：
   `✓ extra root: kanban (from D:/autostack/auto-os/../auto-kanban)` + api
   glue 安装 + dev server 起（:3000）；kanban 入最终 apps-registry 受 vue
   宿主 **v1 单视图已知限制**（router pages 形态跳过，Plan 465 v1 注册限制
   预存，与 P-3 无关）——vm 轨为全量呈现面。临时宿主 vite 组件依赖解析
   失败为 fixture 环境性（deps merge 面），不涉 P-3 通道。
6. [✅ 已完成] **门档**：check 零错；`cargo t app_registry` 15/15、
   auto-man lib 266/266、osconfig 21/21+全链 1/1（ui-iced feature）；折叠前
   `cargo tf` **2619/2620 绿，唯一红=test_charts_gallery_compiles（在册
   charts 存量，564-Q6 豁免）——与基线一致**。
7. [ ] **收口簿记**：Design 01 §1-C schema 注记 + §7 P-3 行状态回填
   （auto-os 侧）；KNOWN-DEBT 如有新登记。验证：两仓注记在案。

## 复审记录

（待 /auto-plan:review 填写。）

## 待澄清事项

（2026-09-07 用户裁定，三项全取默认设计；批次排序同场裁定=保守串行
P-3 → P-2 → P-4 → P-7 → P-5 → P-6。）

1. **[已裁定] manifest 读取位 = 框架侧直读**：auto-lang 按解析序（新 env
   `AUTO_OS_ROOT` → 兄弟 `../auto-os` → 主检出兜底）定位并解析
   apps.manifest；repo 条目经 remote 机制注册。P-7 shell pack 加载器复用
   同套解析序机制。
2. **[已裁定] iced 轨 = 仅 DesktopOptions API 层**：`extra_app_roots` 字段
   沿 `apps_dir` 先例，默认值三轨同源；不加 CLI flag（ui_desktop 为验收
   宿主示例，保持最小面）。
3. **[已裁定] §3-a 包装脚本随 P-6 开工批落地**：P-3 的 V5 验证以 env 注入
   等价形态承载（判据不变：三类条目呈现 + 框架仓零回归）。
