---
plan_id: PLAN-590
status: execution_done         # drafting → executing → execution_done → reviewed → archived
feature_name: Stage B P-5——资产搬迁本体批（台账+launcher+apps+画廊+L8 指针+行数实测）
author: [zhaopuming, ZCode]
created_at: 2026-09-07
updated_at: 2026-09-07（execution_done：步骤 1–7 全 ✅，收口交接 review/merge）

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui, auto-man/examples]  # 受影响的 specs 路径
current_step: 7
total_steps: 8
---

# [PLAN-590] Stage B P-5——资产搬迁本体批（Design 01 §7 P-5）

## 变更摘要

Stage B 收口批：治理资产（A1 台账）+ 物理资产（A3 launcher + B 批 apps +
画廊两件）迁入 auto-os；框架仓引用清零（V7 后半）；框架层行数实测归档。
前置全就绪：P-2/P-3 ✅、P-4 ✅、P-7 ✅、541/582 已合并（唯一硬窗口满足）。

**枚举实测定案（2026-09-07，Design 01 §1-B「以实测为准」授权）**：

| 资产 | 源（auto-lang） | 去向（auto-os） | 实测依据 |
|---|---|---|---|
| A1 台账 | docs/plans/autos-desktop-program.md | docs/plans/（同名） | 活账整体迁移；auto-lang INDEX.md 追加台账指针行 |
| A3 launcher | examples/ui/028-launcher/ | apps/028-launcher/ | §1-A3 |
| B1 sys-monitor | examples/ui/025-sys-monitor/ | apps/025-sys-monitor/ | 541 终态主目录 |
| B1' 残骸 | examples/ui/025-dashboard/（仅 gitignored gen/） | 不迁（磁盘残留另清） | git 零跟踪实证 |
| B2 minesweeper | examples/ui/038-minesweeper/ | apps/038-minesweeper/ | §1-B |
| B4 画廊两件 | examples/{ui-gallery,widgets-gallery}/ | 顶层 {ui-gallery,widgets-gallery}/ | **os-008 定案口径**：578 变更摘要=两 examples 级画廊（非 029-photo-gallery——后者为教学 demo 留架 L7）；顶层落位保 578 的 `apps_dir.parent()/{ui-gallery,widgets-gallery}` 锚定语义自然延续 |
| B3/B5 | 无实物 | —— | B3（clock/tetris/klondike）计划已迁无目录；**B5 前提修正**：582 产出=website playground（packages/auto-playground-vue），非 examples/ui 资产——Design 01 §1-B B5 行注记修正 |
| common | examples/ui/common/（仅 settings/） | 视画廊引用实测 | 三 app（028/038/025-sys）源码零引用实证；画廊引用执行期查，有则抽 apps/common/ |

**测试面清单（执行期重锚主体）**：①examples manifest 不变量（api=manifest
计数，0cb047cc8 口径）——搬 4 app + 2 画廊后计数与断言更新；②
gallery_pages_compile_tests 锚 `examples/widgets-gallery/src/front/pages`
路径——迁移后改锚 auto-os 路径或迁移测试本体（后者为正：画廊测试随画廊
走，auto-os 侧承接）；③desktop_mcp/tests 随 app 目录走；④docs_gen/
kitchen-sink 若含画廊页需同步；⑤scan_examples_ui ≥34 锚（42−4=38 仍过）。

## 目标

1. 七项资产物理就位 auto-os（git mv 保历史）；auto-lang 活动区引用清零
   （V7：grep 028-launcher/038-minesweeper/025-sys-monitor/ui-gallery/
   widgets-gallery/autos-desktop-program 引用=0，INDEX 指针行除外）。
2. 框架测试面重锚后 `cargo tv`+`cargo tf` 基线口径绿（预存红台账豁免）。
3. 台账迁移后 auto-os 侧台账接棒（Design 01 §5 迁移机制 4）；INDEX 追加
   台账指针行。
4. 框架层行数实测归档（tokei/cloc，落 auto-os Design 01 §2 规模注记或
   本仓报告）。
5. L8 指针登记 + Design 01 §1-B 枚举修正注记（B5）+§7 P-5 行回填。

## 执行步骤

1. [✅ 已完成] common 引用终审（画廊两件源码 grep common/）→ 抽取或不迁裁定。
   实测：widgets-gallery 零引用；ui-gallery/pac.at:20 `dep settings { path:
   "../ui/common/settings" }` 唯一引用 → 裁定**抽 `apps/common/settings/`**
   （common/ 顶层无 pac.at，`expand_apps_container` 单层探测不误注册——
   app_registry.rs:273 实证），ui-gallery pac.at path 随迁改 `../apps/common/settings`。
2. [✅ 已完成] 物理迁移 git mv：A1/A3/B1/B2/B4 七件 → auto-os（一提交/仓）。
   auto-os `8a91761`（187 文件落位：apps/{028-launcher,025-sys-monitor,
   038-minesweeper,common/settings} + 顶层 {ui-gallery,widgets-gallery} +
   docs/plans/autos-desktop-program.md）；auto-lang `047743158`（七源 git rm，
   git 历史留档）。对账：源 blob vs 目的地 raw hash join——167 全等 + 19
   CRLF→LF 规范化（auto-os autocrlf=input，语义零变化）+ 1 预期改址
   （ui-gallery/pac.at dep path → `../apps/common/settings`），零意外漂移。
3. [✅ 已完成] auto-lang 引用清零：测试锚改址/迁移（gallery_pages_compile 随画廊
   迁 auto-os 或改路径锚）、examples manifest 再生、docs_gen 同步、INDEX.md 台账
   指针行。
   提交 `cf103b70d`。实测修正与裁定：① gallery_pages_compile **改路径锚**
   （非迁移测试本体——auto-os 无 Rust 测试设施，迁移=日常门禁防线消亡；
   P499-6 事故教训即门禁内防线）——新增 `os_paths::resolve_os_top_dir`
   解析器（env AUTO_OS_ROOT → 兄弟 → 主检出；无 feature 门，ui_gen/CLI
   无 ui 构建同源消费），九处画廊语料锚重锚（gallery_pages_compile/
   schema_drift 第 10 源+solo-skip/docs_gen 4 测+生成器/cmd_docs 脚手架/
   gallery_golden/plan370.409.412.502）；② scan 锚 36→33（计划「42−4=38
   仍过」判断失效——42 为目录数、扫描数实为 36，断言更新），C 档策展集
   20→17；③ playground manifest 不变量 `--check` OK（vm=465 aavm=158
   demo=28 parity=51）——采集源不含迁移资产，**无需更新**（清单①前提修正）；
   ④ gallery_apps_dir 补 `../auto-lang/examples/ui` 兄弟探测（画廊居 auto-os
   时收割框架示例，旧两臂零变化）；⑤ deploy-website.yml 画廊段改 checkout
   auto-os + dispatch `gallery` 输入（auto-os 侧自动触发随 P-6）；⑥ INDEX.md
   P-5 指针表 + README/overview/chart-components 去向注记。
   验证：cargo check 0 错；定向 lib 40/40；schema_drift 2/2 + docs_gen 4/4
   （core.md 字节一致=生成器重锚零漂移）；gallery_golden dump 主检出/工作树
   两位置全等（1609298B）——其红为 **master 预存基线漂移**（4 页 SFC，
   非 migration 所致；tf 档不含该孤儿目标），待澄清事项登记。
4. [✅ 已完成] auto-os 侧承接：apps/ 注册（P-3 容器探测自然命中验证=boot log
   entries +N）、画廊测试落位、台账接棒注记。
   auto-os `1b6a53e`：台账接棒注记（head 块，单一事实源本侧）+ README
   （Stage B 行/目录结构/Apps 表三 local app/台账链接翻转）。**容器探测
   端到端实证**：worktree 构建 ui_desktop（examples/ui=33）+ CWD 主检出
   boot → `[session] app registry: 38 entries (22 desktop-visible)`——
   33 仓内 + 3 迁移 app（apps/ 容器命中）+ os-config + kanban（manifest
   repo 形态）；desktop-visible 22 = 17 C 档 + 3 外根 opt-out 缺省 + 2。
   画廊测试落位：ui-gallery tests/（gallery_e2e + 截图）与三 app
   tests/desktop_mcp 随树物理迁移（在 8a91761 内）；widgets-gallery 的
   Rust 门禁防线（compile 冒烟/schema_drift/docs_gen/golden）留框架侧
   经解析序消费（见步骤 3 裁定注记）。
5. [✅ 已完成] 测试面重锚验证：tv/tf 基线口径（charts/lucide 预存豁免）。
   提交 `5525ccd16`（全量跑暴露的漏网锚收口：ui_gen/rust.rs PLAN-533 页级
   断言、plan492_m4 第三副本、component_registry_test 四测守卫+e2e 临时
   fixture 注入（旧 pkg_app.at 相对路径死资产移除）、a2ts 探针 fallback、
   plan503 launcher 语料、plan502 无门控助手）。
   **tv**：3618/3619，唯一红=test_charts_gallery_compiles（预存）。
   **tf（no-fail-fast）vs master b92d02316 同命令对账：零新增确定性红**。
   master 预存红实测定为 **22**（台账口径「唯 charts/lucide」偏窄，实为
   layout×14/plan370_015×3/plan055/desktop_protocol coverage/lucide/charts/
   plan492_m4 各 1——全部 master 同败在案）；clipboard_files_and_image
   并行 flaky（单测隔离过，环境敏感族非回归）。schema_drift 2/2、
   docs_gen 4/4、component_registry 7/7 全绿（解析序重锚生效）。
6. [✅ 已完成] V7 引用清零 grep + 行数实测归档（tokei/cloc）。
   提交 `0ca4eb9f4`。V7 六名 grep（活动区，排除 archive/reports/autoui
   历史设计/INDEX/本计划）残余定性：**路径依赖类清零**（rust-workspace
   025 陈旧 member 移除〔顺修全新检出 workspace 装载红，主检出靠
   gitignored 残骸目录掩盖的潜伏断裂〕+website 两页断链改 auto-os 仓链接
   +diagram/desktop-shell spec 载体注记）；余者=合法形态（解析序调用/
   兜底臂/名称键宿主模式/协议夹具字符串/schema 漂移记录串/生成物路由
   链接/历史叙述——aura.at 15 处为 drift 基线记录串，触碰即破
   schema_drift 再生成对拍，明确不动）。行数实测（tokei v15.0.0）归档
   Design 01 §2：crates/ Rust 527,646 行（code 439,453）——草案「约
   8 万行」系低估；UI/桌面子集 192,773；迁出资产 12,495 行。
7. [✅ 已完成] Design 01 §1-B（B5 修正+枚举定案表）+§7 P-5 行回填；KNOWN-DEBT
       P584 全结案核对。
   auto-os `1cfe35e`：§1-B 枚举实测定案注记（B1 实迁形态 sys-monitor、B4 两画廊
   顶层定案、common 抽取、os_paths 重锚注记、V1/V2/V3 移交 P-6）+ §7 P-5 行 ✅
   + P-6 行增补；worktree `d3a30f5c7`：KNOWN-DEBT P584 区终核注记（D1/D2/D3①
   全 ✅，D3② 另案维持）+ design/00-intro **L8 指针登记**（autoui 留架历史
   不回改，新增桌面设计落 auto-os docs/design）。
8. [→ 交接 /auto-plan:review → /auto-plan:merge] 收口：specs 沉淀 + plans.md
   回写 + INDEX 重生 + 归档。（此四项为 review/merge 技能承载：spec-impact
   元数据由 review 填写，specs.json 沉淀/plans.md 回写/INDEX 重生/归档
   `docs/plans/archive/` 由 merge 执行。执行侧已交付：worktree `plan-590-dev`
   六提交〔047743158/cf103b70d/5525ccd16/0ca4eb9f4/d3a30f5c7+物理迁移〕，
   auto-os main 四提交〔8a91761/1b6a53e/1cfe35e+立项注记〕。）

## 验收标准

- [x] 七件资产在 auto-os 就位（git mv 历史）；框架仓 git status 零残留。
  证据：auto-os `8a91761`（187 文件）↔ worktree `047743158`（187 删除，
  git 历史留档）；blob 级对账 167 全等 + 19 CRLF 规范化 + 1 预期改址，
  零意外漂移；worktree 提交后 status clean。
- [x] V7 grep 引用清零（INDEX 指针行除外）；examples manifest 不变量
  再生一致；tv/tf 基线口径绿。
  证据：路径依赖类清零（含 rust-workspace 陈旧 member 顺修、website 断链、
  spec 载体注记）；余者=合法形态（解析序调用/兜底臂/名称键/协议夹具/
  schema 漂移记录串/历史叙述——定性清单见步骤 6）。playground manifest
  `--check` OK（vm=465 aavm=158 demo=28 parity=51，采集面不含迁移资产）。
  tv 3618/3619 唯红 charts 预存；tf no-fail-fast vs master b92d02316
  **零新增确定性红**（master 预存实测定 22 红，clipboard 并行 flaky
  单测隔离过）。
- [x] 桌面 boot：registry entries 含迁移 apps（容器探测命中）——P-3
  机制端到端实证。
  证据：`[session] app registry: 38 entries (22 desktop-visible)` =
  33 仓内 + 3 迁移 app（apps/ 容器命中）+ os-config + kanban。
- [x] 台账在 auto-os 接棒；INDEX 指针行在案；行数实测归档。
  证据：auto-os `1b6a53e`（台账接棒注记 + README）；INDEX.md P-5 指针表；
  Design 01 §2 规模注记（tokei：crates/ Rust 527,646 行/code 439,453）。
- [x] Design 01 §1-B/§7 回填；两仓提交在案。
  证据：auto-os `1cfe35e`（§1-B 定案注记 + §7 P-5 ✅/P-6 增补）；两仓
  提交见步骤 8 交接注记清单。

## 复审记录

（待 /auto-plan:review 填写。）

## 待澄清事项

（枚举已按 §1-B 授权实测定案；B5 修正与 B4 顶层落位为执行期裁定，
依据在案可复核。）

- **gallery_vue_golden 预存基线漂移（执行期发现，2026-09-07）**：master
  b92d02316 上该测试即红（基线 TOTAL 1602304 vs 现生成 1609298，4 页
  SFC：drawer/hovercard/kitchen-sink/sheet）——非本迁移所致（两位置生成
  dump 字节全等实证）。该目标不在 cargo t/tf 门禁（孤儿集成目标）。
  处置建议：单独小批重采样（需人工复核 4 页 diff 并写明理由，P499-6
  拒绝洗白纪律）——不在 590 范围内吸收。
- **master 预存红面实测修正（2026-09-07）**：tf 档预存红实测定为 22
  （layout×14/plan370_015×3/plan055/desktop_protocol coverage/lucide/
  charts/plan492_m4 各 1）——台账口径「唯 charts/lucide」偏窄，全部
  master 同败实证在案（本计划零新增确定性红）；归档/修复另案。
- **V1/V2/V3 实机验收移交 P-6**（Design 01 §1-B/§7 注记在案）：五面
  交互/desktop_mcp 基线/双端一致性随 P-6 包装脚本批一次收拢。
