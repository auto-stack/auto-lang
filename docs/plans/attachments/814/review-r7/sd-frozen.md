## 5. 详细设计

1. **SDK与UI owner图**：逐模块列出源码、public项、normal/build/dev、feature、生成字符串/宏/include/build input依赖；解析/推断/编译/VM/非UI trans留同份Core，UI生成/运行移包；依赖语法名不等于UI实现依赖。现有UI兼容AST可在轻auto-ast暂留，明确R10退出，不把UI私有TypeStore/registry推回Core。
2. **默认组合与旧API**：旧auto-lang可为上层兼容包；SDK新名字公开且实测独立。对`Parser::with_session`、AutoVM/NativeInterface构造、外部Dialect、native callback、pub字段/trait/inherent与模块路径构造编译pin；旧UI场景仍默认可用，SDK的Core场景不加载UI实现，SDK UI解析须显式获得UI方言组合。SDK不提交UI注册项、不硬编码UI factory、纯SDK闭包零UI实现。**r3修订（2026-10-20用户批准）**：允许旧auto-lang上层兼容包及auto-ui-compiler经源码可见的`inventory::submit!`登记轻量UI文法函数表/宿主安装器函数指针，覆盖不经显式装配入口的旧直达调用——该登记经inventory启动构造器（0.3.25，Windows `.CRT$XCU`）实现，仅修改inventory自身注册表，启动登记阶段不调用UI文法/宿主工厂、不执行I/O、不初始化UI/VM业务状态；具名入口继续显式装配优先，未显式设置时SDK才收集已登记来源。完整文本=[§5.3](#53-r3-合同修订2026-10-20用户批准)。若无法兼容，T-01具体提出重规划，不自行砍API。
3. **UI compiler**：现有ui_gen四生成器/registry、AURA/a2ui与UI相关分析按闭合图迁移；通用trans/shared helper经SDK公开最小能力，不暴露Core全部私有结构。生成canonical runtime/SDK import由一份路径策略产生；旧路径门面保兼容。允许必要import/header基线变更须显式映射新旧路径，不能以更新golden掩盖行为差异。
4. **UI runtime**：ui目录runtime、renderer、editor/terminal/widget、desktop/MCP/宿主adapters及其状态/资源迁有归属的包；VM/SDK只保轻能力与非UI机制。i18n lookup/装载、a2r frame直连、config/UI依赖和其他反向边逐项验清；未消除边必须有本Plan内任务，不能在最终声明“纯Core”时留豁免实现。
5. **状态/并发**：registry/OnceLock/thread_local/global runtime/VM Rc、provider默认/实例覆盖/owner history、模板/语法session/cache不可跨新包产生第二真源；线程亲和、锁外回调、store额外文件drain、use诊断、终端/媒体生命周期/快照共享保持。移动字节不等于状态单源证据。
6. **消费者与测试**：旧路径消费者在兼容层继续可编译；至少一个真实非UI消费者（首选auto-vm，若其实际依赖需调整则T-01具名替代）直接切SDK。UI compiler/runtime及后续工具必需的语言adapter（如auto-gen可执行模板层）也要沿实际闭包切SDK，不能经旧auto-lang兼容包把UI实现重新拉回。auto-lsp等UI补全可暂用上层具名adapter，不谎称其整体纯Core。迁出的lib/test/integration/doc/schema测试保单份来源；cargo t/tv/tu/tt/tb实际选择面映射覆盖新包，UI tu保持合适单进程registry复用，不因包名变化丢测试。
7. **发行**：777精确域边/forbidden、792受管source与manifest/资源/producer/lock/重定位更新至真实包图；新SDK独立fresh/locked无UI checkout，UI实例工程可用新包，不靠父仓隐式path。SDK不含UI实现的证明覆盖normal/build/dev，UI部署profile仍保原功能而非删除默认重后端。

### 5.3 r3 合同修订（2026-10-20 用户批准）

本节为 r2→r3 的具名增量（来源=[R6 复审草案](attachments/814/review-r6/contract-r3-draft.md)，用户于 2026-10-20 批准；仅修订以下两项，其余原 AC、旧 API/类型/单份状态与 A/B 交付要求保持）。

**1. 具名的上层兼容注册例外**（替换 r2 AC-02/V-02/§5.2 中"无隐式链接装配"的绝对约束）：

- 纯 SDK 的 normal/build/dev 闭包不含 UI 文法实现、UI compiler/runtime/renderer 或 Man；SDK 不提交 UI 注册项，不硬编码 UI factory。SDK 未装配 UI 文法时的缺席/拒绝语义保持。
- 允许旧 auto-lang 上层兼容包及 auto-ui-compiler 经源码可见的 `inventory::submit!` 登记轻量 UI 文法函数表或宿主安装器的函数指针，覆盖不经显式装配入口的旧直达调用。该登记通过 inventory 启动构造器实现（当前锁定 0.3.25；Windows `.CRT$XCU`），会修改 inventory 自身注册表；不在启动登记阶段调用 UI 文法/宿主工厂、执行 I/O 或初始化 UI/VM 业务状态。
- 具名入口继续显式装配；未显式设置时才由 SDK 收集已登记来源。多个当前来源均构造自同一 auto-ui-dialect 实现，不依赖遍历顺序选择不同语义。不新增不同版本同时装配的支持承诺。
- 上层消费者允许按其功能依赖并链接 UI 实现；纯 SDK 的无 UI 源码消费者仍须成立。旧 Parser/VM API 与类型身份、单份状态、宿主默认/显式覆盖及缺席行为保持。
- 此例外明确承认链接时注册。仅删除 inventory 不能自动授权砍旧 API、改成硬错误或更改纯 SDK 缺席语义。

**2. 轻端口的归属修订**：

- 允许 `UiParseContext` 与 `UiGrammar` 的类型/函数表定义放入轻 `auto-parser-api`；UI 文法实现继续由 auto-ui-dialect 单源拥有，SDK 不反向依赖文法实现。auto-ui-dialect 保留原公开路径再导出及类型身份，通用 `ParseContext` 的既有实现方不被强制实现 UI 端口。
- 这是单源端口与适配器方案之间的设计取舍。canonical 目标（T-15 受管沉淀时落文）：`docs/specs/auto-parser-api/project.md`"UI 伴随端口不入本 Core 包"→"允许 UI opt-in 轻端口定义，不带 UI 实现依赖"；`docs/specs/auto-ui-dialect/design/full-grammar.md` trait 定义归属改 parser-api、方言 crate 保留再导出与实现 owner。SD-01/02/04/05 同步记录该窄例外及其验收，不提前发布 B 完整 runtime 事实。

### 5.1 A/B阶段的真实交付边界

**M814-A**不是空crate或接口草图：纯SDK完成Core parse→compile→VM与已有非UI消费者、必要trans出口；normal/build/dev无UI文法实现/编译/运行包或Man。T-02已经清除阻碍此性质的i18n/frame/config等边，不能把纯SDK所必需的去边留到T-06。UI compiler迁实际生成算法/资源与公开入口，真实消费者可生成其声明支持的产物。A中迁出的测试族和资源立即接入实际门/777/792及fresh/locked消费，不留到T-07/08才补。

旧auto-lang上层兼容可继续使用尚未搬迁的现有真实runtime，保原默认UI、Parser pub字段/方法/回调及单份状态；新SDK与compiler不可反依赖该兼容包。生成runtime import仍指向当前真实可用、版本受约束的实现，B再有控制地切换；不得将未完成的新runtime能力列为已支持。原四生成器各自支持面保持，依赖runtime的原完整正控保留给B，A至少验证生成产物和当期真实runtime消费路径。若依赖图不能这样切，T-01交具体needs_replan，不用伪实现凑阶段。

**M814-B**完成新runtime/renderer/宿主adapter、默认组合和全部原验收。A的局部合入不提前勾AC-01..05整体完成；仅新增AC-08..10描述阶段范围。SD-01/02与SD-04..08仅按A已发布事实沉淀，SD-03完整runtime规则等B，不把B目标写成现状。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/language-sdk/project.md | 原巨型包→纯SDK职责/API/feature/显式组合与正常闭包（r3：UiGrammar/UiGrammarSource/set_ui_grammar 装配端口语义+§5.3 具名登记例外+缓存身份时序——None→Some升级/首装优先/出生身份落盘/load比对失效） | 基础可独立消费 | AC-01, AC-02 |
| SD-02 | add | docs/specs/auto-ui-compiler/project.md | 原UI分析/生成→独立compiler包、公开接口/资源（r3：assemble_ui_grammar 显式装配入口+三生成入口时序+指纹 None=foreign build） | UI编译owner | AC-03 |
| SD-03 | add | docs/specs/auto-ui-runtime/project.md | 原UI运行/renderer→独立runtime与宿主/状态归属 | 运行边界 | AC-03, AC-04 |
| SD-04 | modify | docs/specs/auto-lang/project.md | old包标为上层兼容/组合，实际SDK位置明确（r3：上层经 inventory 具名登记的直达兜底例外+auto-parser-api 端口归属变更） | 避免纯Core虚报 | AC-01, AC-02 |
| SD-05 | modify | docs/specs/auto-lang/frontend/overview.md | Parser默认/显式UI组合与旧API承载（r3：Scenario::UI 内建方言双层装配来源——显式安装器首句+具名登记兜底；表缺席诊断文本） | 解析身份/装配 | AC-02 |
| SD-06 | modify | docs/specs/auto-lang/vm/overview.md | 无UI实现的VM能力/默认组合与余边清零 | 真正解耦 | AC-01, AC-04 |
| SD-07 | modify | docs/specs/auto-lang/ui/overview.md | UI包组/旧路径/资源/消费者事实 | 完整交接 | AC-03..05 |
| SD-08 | modify | docs/specs/repository-boundaries/generated-ui-distribution.md | 真实新包图/测试与资源输入进入发行profile | 迁包可分发 | AC-05 |


## 当前候选（包含明确的R7绑定及历史修正）
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

## 8. T-14 当前审定绑定（R7，r3；2026-10-20）

本节与主计划 r3 §5.3、[R7 复审记录](review-r7/review.md)共同确定当前 A 候选；
§1–7 保留早批历史，旧 SHA、SDK ui-dialect feature、旧计数及“B 清偿警告”
等叙述以历次复审闭合和本节为准，不作为当前发布事实。

- 冻结实现：`256613936b6a5e9baea450e5c545899ee7a661dc`，唯一树/分支不变；
  当前合同为 **PLAN-814:r3:M814-A**，两项具名修订已经用户批准。
- SDK `auto-lang-core 0.1.0`：normal/build/dev 无 UI 文法实现/compiler/runtime/
  Man/旧包清单边；无 `ui-dialect` feature。保留 `code-editor` 非 UI 行为开关与
 其他非 UI features；文法经轻 `auto-parser-api::{UiParseContext,UiGrammar}`
 端口，纯 SDK 缺席/显式拒绝保持。构建输入为具名 SDK 闭包，非全 crates 扫描。
- UI compiler `auto-ui-compiler 0.1.0`：真实四生成器/分析/资源及旧路径兼容；
  三个具名生成入口显式装配，直达路径可使用 r3 允许的轻表登记兜底。启动登记
  只改 inventory 自身表，不调用 UI 工厂/I/O 或初始化 UI/VM 业务状态。
- 当前 runtime 仍为上层真实实现；`auto-ui-runtime 0.1.0` 只有骨架，B 能力未发布。
- R3-01..06 与 R4-01 当前全部闭合；R6 严格九档逐 ID 映射、净新增警告 0/减少 7
  按已核实边界复用。r6-closeout 重验收据已核对，R7 另独立通过显式宿主/直达
  Parser/显式默认与快照/autodown、无 UI 源码 fresh+locked SDK 5/5、compiler 消费 4/4。
- T-14 只给 **A 阶段 pass**。规范子集绑定 r3 SD-01/02/04..08 及两份端口 Spec
  目标；SD-03 完整 runtime 留 B。T-15 冻结回归、实际 A 合入/规范与 ledger、版本/
  来源/consumer 读回收据仍 pending；A 未实际发布，原树保留继续 B，不归档。

