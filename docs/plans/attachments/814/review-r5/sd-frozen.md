### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/language-sdk/project.md | 原巨型包→纯SDK职责/API/feature/显式组合与正常闭包 | 基础可独立消费 | AC-01, AC-02 |
| SD-02 | add | docs/specs/auto-ui-compiler/project.md | 原UI分析/生成→独立compiler包、公开接口/资源 | UI编译owner | AC-03 |
| SD-03 | add | docs/specs/auto-ui-runtime/project.md | 原UI运行/renderer→独立runtime与宿主/状态归属 | 运行边界 | AC-03, AC-04 |
| SD-04 | modify | docs/specs/auto-lang/project.md | old包标为上层兼容/组合，实际SDK位置明确 | 避免纯Core虚报 | AC-01, AC-02 |
| SD-05 | modify | docs/specs/auto-lang/frontend/overview.md | Parser默认/显式UI组合与旧API承载 | 解析身份/装配 | AC-02 |
| SD-06 | modify | docs/specs/auto-lang/vm/overview.md | 无UI实现的VM能力/默认组合与余边清零 | 真正解耦 | AC-01, AC-04 |
| SD-07 | modify | docs/specs/auto-lang/ui/overview.md | UI包组/旧路径/资源/消费者事实 | 完整交接 | AC-03..05 |
| SD-08 | modify | docs/specs/repository-boundaries/generated-ui-distribution.md | 真实新包图/测试与资源输入进入发行profile | 迁包可分发 | AC-05 |


## 当前候选（完整冻结，历史记录与当前增量并存）
# PLAN-814 M814-A 候选（T-13 冻结 @ `abe9c9900`）

2026-10-20。stage: work（T-13 完成冻结）。792 交付 driver 已跑通（25/25，
EXIT=0），候选 SHA 冻结于 `abe9c9900`。**T-13 完成不等于 AC-08 pass**——
AC-08/AC-09 由 T-14 独立阶段审查绑定本 SHA 判定；AC-10 归 T-15。

## 1. 冻结对象

| 项 | 值 |
|---|---|
| worktree / branch | `D:/autostack/.wt/lang-814/auto-lang` / `codex/plan-814-dev` |
| **冻结 SHA** | `abe9c9900`（交付合同更新至新包图——792 driver 25/25 恢复全绿的收口提交） |
| base | master `203e2ce91` |
| A 发布包 | `auto-lang-core 0.1.0`（纯 SDK）、`auto-ui-compiler 0.1.0`（UI 编译域）、`auto-ui-runtime 0.1.0`（骨架——**能力未发布**，B/T-04 迁入） |

## 2. A 阶段已发布事实（API/支持清单）

### auto-lang-core（纯 SDK）
- **公开面**：Parser（Core 场景/显式 session 装配；UI 文法门面签名恒在，
  feature off=显式诊断）、VM 全链（Codegen/AutoVM/VirtualFlash/
  native_registry/loader）、trans 出口（trans_rust/trans_c + with_session）、
  run 族入口（run_with_capture/execute_autovm 等）、config/i18n(hook)/
  frame(779 端口)/a2r_std、stdlib_assembly/database(UICache 指纹注入)、
  use_scanner/dep_scanner/os_paths。
- **特性**：default=[with-file-history]；`ui-dialect`（显式 UI 文法组合）、
  `code-editor`（a2r diff 双轨）、`python`/`streaming-http`/`mem-profile`/
  `test-cases`。
- **闭包**：normal/build/dev 零 UI 文法/UI 编译/UI 运行/Man/旧巨包
  （cargo tree 三边断言 + 消费 fixture 钉住）。

### auto-ui-compiler（UI 编译域）
- **公开面**：`ui_gen::api::generate_component_from_file`（.at→Vue SFC
  真实生成出口）+ ComponentGenOptions/GeneratedComponent；四生成器
  （vue/jet/ark/rust）+ widget registry/bp；aura 模型/提取/校验；a2ui；
  style DSL 分析（iced 适配器留上层运行时）；action_config；lucide 单源
  表；route 分析；`generator_fingerprint()`（build.rs 指纹公开）。
- **闭包**：SDK + 显式 ui-dialect + 轻前端 crate；零旧巨包反依赖。
- **测试选择**：ui_gen 862（tu 单进程 libtest）+ nextest 215（日常档含
  t/tf 的 `-p auto-ui-compiler`）。

### 上层兼容（auto-lang）
- default 转发 `ui-dialect`/`code-editor`——Scenario::UI 默认装配、旧
  Parser 公开 UI 门面、`auto_lang::{aura,a2ui,ui_gen,route,ui::style,
  ui::action_config,ui::iced::lucide_generated}` 旧路径经再导出全保持。
- A 期真实 runtime 仍在本包（ui/ 运行时未迁——B/T-04）；生成物 import
  指向当期真实可用实现（vue 生态路径策略）。

## 3. 证据矩阵（V-06 + A 涉及 V-01..05）

| Case | 状态 | 证据 |
|---|---|---|
| V-01 纯 SDK/单源 | ✅ | `language-sdk-consumer` 5/5（Core VM 执行 42 输出/手工字节码管线/trans_rust 产物）；auto-vm 真实消费者切换+冒烟；closure 三边 0 UI；SDK lib off 1627/1627 |
| V-02 兼容/装配 | ✅（A 范围） | 旧路径再导出编译 probe（上层 --tests 全编译）；plan806/dialect/scene 24/24；SDK on 1713/1713；文法拒绝负控（feature off 显式诊断不静默） |
| V-03 真生成出口 | ✅（A 范围） | `ui-package-consumer` 4/4（真实 SFC/script setup/状态/vue 生态 import）；ui_gen 862/862 金样；**当期真实 runtime 消费=A 期上层路径（旧行为保持），B 切新 runtime** |
| V-04 余边/寿命 | ✅（A 已知面） | frame 779 端口单源（plan716 探针）；i18n 路由恢复；store_extra/strip 下沉单源；autodown 残边已归因记 T-04 |
| V-05 选择面/发行 | ✅ | t/tf/tu 别名扩包（池 6851，ui_gen 862 保持选中）；777 strict pass ×4（基线 `d91948ee1`）；fresh（path+独立 lockfile）+ `--locked` 双 fixture 消费通过；**792 交付 driver 25/25（EXIT=0 @ `abe9c9900`）**——V01 fresh resolve+build/V02 A→B 重定位 --locked/V03a..j 全负控（资源缺失/坏 hash/漏 patch/不兼容 pin/外来目录/sidecar 外部 path/raw TOML）/V04 dev 同源；SD-08 三接缝修复：DELIVERY_CRATES 16→19、投影按 manifest 分 spec（含 SDK spec）、ui_gen 指纹源路径更新 |
| V-06 A 真实边界 | ✅（除 792） | 上表合成：禁止边确实消除+非 UI 消费执行成功+旧 API 同源+A 公开生成出口可用；缺新 runtime 能力处显式拒绝（ui_grammar_unavailable） |

## 4. 后续（T-14/T-15 归属）

1. ~~792 交付 driver~~ **已完成**（25/25，EXIT=0；首轮受管红暴露的三处结构
   漂移即 `abe9c9900` 修复面——投影 4 缺 1/ui_gen 源根/runtime 未 vendored，
   红不回退主检出语义保持）。
2. **T-14（next）**：同树独立阶段审查——绑定 `abe9c9900` + 本文件 + r2 §5.1
   阶段性质，验 AC-08/V-06 与 Spec 子集（SD-01/02/04 按 §2 事实备稿）；
   merge-owned 门禁（裸 t+tv/tu/tt）保持 pending 记录（2026-10-14 时序裁定）。
3. **T-15（T-14 pass 后）**：冻结门（裸 cargo t + 触面 tv/tu/tt）→ 实际
   master 合入 → Spec/受管 ledger 增量与发布收据；保原树/分支继续 B。

## 5. 已知限制（A 边界内的诚实声明）

- auto-ui-runtime 仅骨架：B/T-04 迁入真实 renderer/宿主 adapter 前不列
  任何已支持能力。
- desktop.at 等 shell 资产家族留上层（T-04 归属定夺）；vue.rs 测试经
  MANIFEST_DIR 引上层单源副本。
- autodown natives 在 SDK 为 fallback 拒绝形（跨仓依赖不进 SDK；autodown
  消费者迁移=T-04）。
- `code-editor`/`ui-dialect` 为 SDK 侧显式组合 feature：纯 SDK 默认闭包
  不含其实现，属设计内边界而非缺口。

## 6. R3 修复批增量（2026-10-20 追加，绑定分支头）

R3 阶段审查 needs_fix 后三批修复（`d1fbae132`/`d905e796c`/`6f9daa680`+`6418f99ef`）
对本文件的证据矩阵更新：

- §5 限制项"`ui-dialect` 为 SDK 显式组合 feature"**已失效**——R3-03b 起
  SDK 清单零 auto-ui-dialect（UI 文法经 UiGrammar 函数表端口装配；
  no-UI 源码快照消费探针 PASS）。纯 SDK 边界从"optional 组合"升格为
  "清单级不存在"。
- §3 证据矩阵的绑定刷新：777 基线 → `6418f99ef`（violations=0）；
  新增绑定 [R3-05 测试选择映射表](r3-05-test-selection-mapping.md)（86 项
  随迁 auto-ui-dialect）、[R3-06 告警归因](r3-06-warning-attribution.md)
  （unexpected_cfgs=0）、[R3-03b 收口探针证据](r3-03b-closeout/)
  （host/cache/autodown 三探针 + no-ui 快照全 PASS）。
- reviewer 三探针原始反例（R3-01 宿主装配/R3-02 指纹顺序/R3-04 autodown
  实装）批 1 修复后回放全 PASS，批 3 后复跑保持 PASS。

## 7. R4 修复批增量（2026-10-20 追加，绑定 `189ca0f77`）

R4 复验 needs_fix 后修复批（`81c8a5743`+`189ca0f77`）对本文件的更新：

- §3 证据绑定刷新：777 基线 → `189ca0f77`；新增 [R4-01 装配合同对账
  与 SD 增量正文](r4-01-assembly-contract.md)（显式装配优先 + inventory
  兜底边界 + AC-02 澄清提案 + UiParseContext 归属变更理由 + 缓存身份
  时序规范——回应 R4"实际规范 delta 正文尚缺"）。
- [映射表](r3-05-test-selection-mapping.md) R4 修订：基面回溯
  203e2ce91，九档（含 tb/tu/taa/ta/t3）未匹配 0/选择缺口 0；aavm 档
  T-02 编译红、ui-iced 残门、T-02 被丢测试三项修复。
- [告警归因](r3-06-warning-attribution.md) 203e2ce91 口径：净新增≈14
  条全归因 T-02/T-03 残留（B 清偿域），R4 批零新增。
- SDK build 输入：全 crates 扫描 → 具名 17-crate 编译闭包（零 UI/Man
  条目 + 双负控测试）——§2 的 SDK 事实清单相应更新。
- 缓存身份语义（SD/PLAN-698 精确化）：None→Some 升级、首装优先、
  save 按出生身份落盘、load 比对失效。

## 本轮新增合同及规范提案（待授权部分不视为已通过）
# F814-R4-01 对账：UI 文法/宿主装配来源与合同边界（PLAN-814 R4 修复批）

## 问题陈述

R4 判定：R3-03b 采用的 `UiGrammarSource` inventory 自注册（上层被链接即
submit 生效，SDK 侧 build_dialects 惰性收集）与交付合同 r2 §5.2/V-02/
AC-02 的"不靠 inventory 偶然性""无隐式链接装配"字面冲突——尽管行为实测
通过（host/parser-only 正控均绿，且 R4 明确不声称当前样例发生随机链接
失败），该机制未经合同修订授权。

同一合同条款下，R3-01 的 `UiCompatInstaller`（宿主能力装配，覆盖不经
run 族漏斗的直达调用）已被 R3/R4 两轮复审接受并关闭——其语义边界在
SDK lib.rs 有具名注记（SDK 零 submit；上层链接即生效=确定性自注册；
纯 SDK=零提交、缺席=显式拒绝/回退）。

## 两条路线（R4 给出）

- **A. 完全显式装配**：删除一切 inventory 参与。后果：不经任何装配入口
  的直达 `Parser::from(..).with_session(UI)` 构造失去文法——这正是 R4
  自己列为**通过证据**的 parser-only 正控（"只使用旧 Parser 路径的新
  进程可解析 widget"），也是 AC-02 的旧行为保持面。路线 A 与该正控
  互斥，需砍旧 API 行为——超出 work 权限，须 needs_replan。
- **B. 显式优先 + 边界对账的兜底**（本批落地）：全部具名装配入口显式
  注入；inventory 仅作为直达构造的兜底，语义边界与已被接受的
  UiCompatInstaller 完全同构。

## 本批落地（路线 B）

1. **显式装配优先**：
   - 旧 auto-lang `install_ui_host_capabilities()`（run 族漏斗、旧公开
     入口包装、SDK 钩子回调的全部汇点）首句显式
     `set_ui_grammar(auto_ui_dialect::ui_grammar())`——含无 renderer
     构建形态（文法属语言面，与 renderer feature 无关，旧
     `ui-dialect` default 组合语义如此）。
   - auto-ui-compiler 公开生成入口（`transpile_aura`/
     `transpile_vue_aura`/`generate_component_from_file`）经
     `assemble_ui_grammar()` 显式装配。
2. **inventory 兜底保留**，SDK 侧注记改写为与 UiCompatInstaller 同构的
   语义边界声明（SDK 零 submit；纯 SDK 消费者零提交、表缺席=显式拒绝/
   回退；上层被链接即其 submit 生效——确定性注册，非 linker 偶然拉入：
   提交物是上层 crate 的显式 `inventory::submit!` 语句，不依赖任何
   静态构造器/弱符号偶然性）。
3. **回放证据**：host/parser-only/cache/autodown 四探针复跑保持 PASS
   （显式路径与兜底路径行为一致）；language-sdk-consumer 纯 SDK 负控
   （表缺席=显式拒绝）保持。

## 合同修订提案（供 T-14/T-15 裁定；不默降 AC）

**提案**：AC-02/V-02 的"无隐式链接装配"澄清为——"SDK 不依赖链接期
偶然性拉入 UI 实现（无静态构造器副作用、无 Core 硬编码 UI factory、
纯 SDK 闭包零 UI）；上层兼容包允许经显式 `inventory::submit!` 语句
注册装配器，覆盖不经装配入口的直达调用——与 PLAN-814 R3-01 已验收的
UiCompatInstaller 同一机制与同一边界"。

理由：(a) 与 R3-01 已接受的机制同构，无双重标准；(b) 直达构造正控是
AC-02 自身的旧行为保持面，路线 A 无法同时满足两者；(c) 提交物是上层
crate 源码中的显式语句，可审计、可负控（纯 SDK fixture 证明零提交）。

若裁定不接受该澄清：needs_replan 路线的具体变更是"直达
`Parser::from(..).with_session(UI)` 在未装配时从『空方言回退』改为
『硬错误』或删除该直达面"——两者都是旧 API 行为破坏，需用户授权。

## SD 增量正文（A 子集，绑定本批）

- **SD-01（language-sdk）补**：SDK 公开 `set_ui_grammar(UiGrammar)`/
  `UiGrammarSource`（inventory 收集点，SDK 零提交）与
  `set_generator_fingerprint`（None→Some 升级、首装优先、save 按出生
  身份落盘）两个装配端口的精确语义。
- **SD-02（auto-ui-compiler）补**：`assemble_ui_grammar()` 公开入口与
  三个生成入口的显式装配时序；`generator_fingerprint()` 的 None 语义
  （foreign build）。
- **SD-04/05（auto-lang/frontend）补**：Scenario::UI 内建方言的装配
  来源双层（显式安装器首句 + inventory 兜底）；表缺席=空方言回退+
  公开门面显式拒绝的诊断文本（"not assembled ... UI grammar table"）。
- **auto-parser-api 归属变更**：`UiParseContext` 自 auto-ui-dialect 迁入
  （PLAN-810 §2"不入 Core 包"裁定的修订：孤儿规则下 impl 只能落在
  trait 或类型所属 crate，SDK 无清单边要求 impl 留 SDK——唯一合法位置
  是双侧共同依赖的轻端口 crate；裁定理据保持——独立 opt-in 子 trait，
  ParseContext 实现方零强加，旧路径 re-export 不变）。canonical
  `auto-parser-api/project.md` 与 `auto-ui-dialect/design/full-grammar.md`
  的相应正文随 T-15 受管沉淀（本批不写 canonical Specs）。
- **runtime 缓存规范（PLAN-698 语义在分包后的精确化）**：指纹槽
  None→Some=升级；已装配=首装优先；save 按缓存对象出生身份落盘（不
  重盖）；load 经 cached-vs-current 比对判失效（一侧缺席=整缓存失效）；
  真实身份消费方（auto-man vue/jet、compiler api）在 load 前注入。

