---
plan_id: PLAN-1097
status: drafting
feature_name: ACE AutoUI 原生 AC 宿主和 W2 交付
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-19
plan_revision: 4
current_step: 0
total_steps: 5
depends_on: [PLAN-980, PLAN-981, PLAN-978, PLAN-1071, PLAN-1078]
supersedes_spec_components: [docs/specs/ace-migration/m2-autoui.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [ace, auto-ac, autoui]
---

# [PLAN-1097] ACE AutoUI 原生 AC 宿主和 W2 交付

## 0. 变更摘要

2026-11-19。本计划独立交原生宿主与 W2。原 T-05/T-06 及对应 T-07/T-08 的 AGE 实施、验证、复审合入迁 [公共域](1102-age-ac-gallery-fusion-and-capabilities.md)、[G2](1104-age-ac-game-2d.md)、[G3](1105-age-ac-game-3d.md)。

### 2026-11-19 r3 · 接收AV-ONLY阶段与非AV检查恢复

用户已明确1078–1096只验AV，公共语料中AC/A2X等检查先以检查级#ignore暂缓。其未实现/未运行/失败不阻AV计划，不充PASS；本包进入对应后端开发/验证阶段后，逐check接手、移除本阶段标记并真实执行。共享源码、原oracle/条件/owner和本包完整验收门保持；不得要求AV计划先交非AVPASS。本轮仅修订规划。

### 2026-11-19 r2 · 公共语言语料按特性共源（仅规划）

按用户要求接[共享语料设计](../design/testing/ace-feature-organized-autodown-corpus.md)与1078的`IF-ACE-FEATURE-CORPUS/1`。AC所用公共语言切片复用language同Case、同source/project指纹，独立运行机器产物和诊断/资源检查。W2 完整源与 GUI/GPU/宿主负控保原门，实际 AV 子面已交可先行。 原任务ID/数量/进度与已有证据边界保持，本轮不施工、不迁实际测试。

拆分前，2026-11-19 从 [1068](1068-ace-m2b-three-backend-parity-and-aac-proof.md) 原 T-04 接过 AC 领域实现和独立交付。收据协议保留 `IF-1068-AC-DESKTOP`、`UI-W2-AC` 与 `IF-AGE-GFX-AC/<cap>`，各收据记录实际生产计划和版本。现为草案，所有验证待执行。

## 1. 目标

以真实 Windows AC 机器码运行同源 AutoUI/Widgets，通过已有 Iced 宿主验窗口、输入、资源和生命周期，交 IF-1068-AC-DESKTOP 与 UI-W2-AC。

## 2. 架构方案

同一 Node 源经 Schema/Checked HIR、MIR、AC 机器函数和 967 typed Rust FFI 到达同一 Iced 宿主。宿主事件按版本路由到机器码 handler，状态和绑定由 AC 业务执行。原有 Rust 宿主、renderer、资源和入口可复用。共享语言、普通标准库和通用机器 lowering 的问题回对应 AAC、标准库或 ABI 生产任务修复。

## 3. 技术栈

复用 ACE native/Cranelift、967 typed ABI、978 候选 SDK、980/981 宿主与 AGE renderer、现有 TestCase/AutoDown 语料。实现时使用 `D:/autostack/.wt/lang-1097/auto-lang` 和需要的同组 `auto-ui` 工作树、分支 `plan-1097-dev`。证据归 `D:/autostack/plan-artifacts/plan-1097/`。

## 4. 需求分析与背景调查

985/A 已在 `95c7fbad8` 正式交 M2A-08。980/981 已有 HELLO/AXR0/SOURCE，原窗口与源接口可复用。完整 W2 取正式 W1 的全分母。归档 [1071](archive/1071-autoui-w1-av.md) 提供已交 U01、原分组与生产入口子面。其余 U02–U06 和关联 Widgets 取 [AutoUI 104–108](../../../auto-ui/docs/plans/README.md) 的实际 AV/Iced 阶段收据，与 W1 原矩阵逐行核齐。各计划 A2Vue 的验收仍按各自合同。宿主接缝最小切片可先取 A 的接口和正式同源用例。1067 已交的调度路径按实际用例消费，所需后续调度切片按 [AAC 归属表](../design/59-aac-native-bootstrap-and-compiler-subject.md#现有-ac-差额的实施归属) 接收。扩展 AGE 域另取 [1075](1075-age-av-content-and-fusion.md) 对应同源 AV cap、原 981 类型及资源合同。进度以正式来源为准。

## 5. 详细设计

### AV-ONLY接收与本目标启用

1078的IF-ACE-FEATURE-CORPUS/1和1096的AV语言证书允许携带非AV#ignore/NOT_RUN行，接收时只核AV声明和迁移义务未丢，不以非AVpending回卡AV独立出口。本包实际验证自己的后端/代际时，由原生产任务逐check去掉AV阶段#ignore，保原平台/ignore与oracle，按当前同源/工具/SDK身份运行；失败属于本目标任务，必须修到本目标required齐才签其完整出口。1068/985的最终多端门及AAC独立门不接受忽略充PASS；AV阶段通过从不冒多端通过。

规范影响：公共作者/引用/选择/迁移协议由1078统一在拟`docs/specs/auto-test-case/feature-organized-corpus.md`沉淀；本消费者只在自身原SD/模块Spec记录实际Case引用、目标动作和正式证据，不再各写一套公共协议。AC-FC属于本包验收增量，原SD与目标义务全部保留。

### FC · canonical语言输入与旧义务承接

AC所用公共语言切片复用language同Case、同source/project指纹，独立运行机器产物和诊断/资源检查。W2 完整源与 GUI/GPU/宿主负控保原门，实际 AV 子面已交可先行。

由原T-01/T-02/T-03按其职责消费/迁移对应行，保原source/expected/check及条件，重复仅在完整输入闭包/语义等价时合并，原义务→canonical Case/check映射不得丢；既有固定组/suite/worker读者按1078接口切换，不新建按后端/代际维护的Auto副本。缺字段/目标接口由1078和实际producer在原框架有界补接，不要求本包先等全AV/全AAC完成。

| 交付 | 输入及边界 | 下游 |
|---|---|---|
| `IF-1068-AC-DESKTOP` | 980/981 已验接面、同版 ABI/SDK、同源最小中文函数与 Node Counter。冻结选择器、机器产物、宿主 ABI、typed 事件、资源和负控。 | W2、AGE 实际域、1068/985 |
| `UI-W2-AC` | 正式 W1 全 U01–U06、Widgets 加已验宿主切片，每行真机器业务、同窗操作、错误及资源。按阶段独立复审并合入，可在 W3 开工前交。 | 1074、OS064、1068/985 |

W2 独立签发。AGE 域由 [1102](1102-age-ac-gallery-fusion-and-capabilities.md)、[1104](1104-age-ac-game-2d.md)、[1105](1105-age-ac-game-3d.md) 消费正式宿主及 W2。宿主/W2 收据记录 source/Schema/compiler/ABI/SDK/机器产物/宿主 hash、oracle 和适用范围。

| delta_id | 操作 | 目标 | 设计变化 | 验收 |
|---|---|---|---|---|
| SD-AC-DESKTOP | modify | m2-autoui.md 的实际支持条款 | Windows W2 与 typed guest/source/host 按真实交付登记 | AC-HOST、AC-W2 |

## 6. 测试设计

### V-FC-CONSUMER（work / 原T-01/T-02/T-03，未执行）

同一canonical source变化必须使本包实际目标/代际依赖失效并用独立oracle重验；单目标oracle变异、遗留旧活入口、删原义务映射、零选中/缺required检查和emit冒run分别判红。公共语言输入去重不减少旧资源/错误/恢复/平台条件；typed IR/后端专项及外仓完整产品保原owner。实际结果按Case＋check＋编译器/目标/动作＋source/SDK/provider身份记录，不以AV或其它代结果回填。

| 验证 | 真实输入与判据 |
|---|---|
| V-AC-DESKTOP | 980 项目入口，原 Hello/function 中文、002 Node Counter，主线程真窗口和机器码事件，分别验 CPU/可见文本、0→1→0→0、两实例 2:1。拆 UI submit、宿主偷读 AV、切断 AV/A2R guest dispatch 或 rustc guest 编译进行判别。Rust 预构建宿主合法。 |
| V-AC-DESKTOP-OWNER | borrowed/owned 输入、错类型或 ABI、权限、代际、关闭重开及旧代迟到，错误前拒、保旧、复原、最后 owner 释放；保源位置。 |
| V-AC-WAVE2 / NOCYCLE | 正式 W1 全组和原 oracle、同版 ABI，全 U 组和 Widgets 机器业务＋同一 Iced 窗口/交互/错误/生命周期；删必需组或改为 AV fallback 必红。完整 W2 且 W3 待交时可签 W2。 |

## 7. 验收标准

- [ ] **AC-FC｜owner_stage: work**：V-FC-CONSUMER及§5 FC逐原义务满足，公共语言唯一作者/本目标真实动作与独立结果/迁移映射均可追溯；原独立review复核，merge只接真实落主来源。未交required目标保pending，不因共享目录、少文件或另一端通过而降门。

| AC | 阶段 | 通过条件 |
|---|---|---|
| AC-HOST | work | V-AC-DESKTOP 和 OWNER 真执行通过，正式交宿主切片。 |
| AC-W2 | work | 全 W1 源/行对应 AC 独立 oracle，正式 W2 释放 1074。 |
| AC-REVIEW | review | 独立核源、机器产物、宿主与运行证据、覆盖和负控。 |
| AC-LAND | merge | 已审变更与支持范围合入，保可定位收据、更新各下游交接。 |

## 8. 执行步骤

| task | owner_stage | depends_on | 工作及结果 |
|---|---|---|---|
| [ ] T-01 来源与矩阵 | work | M2A-08、已交 A/W1 子面；其余 W1 按组追加 | 冻结 W1/U 组/Widgets、SDK、ABI、prototype、独立 oracle。 |
| [ ] T-02 机器 guest 与 typed 宿主 | work | T-01 的最小切片 | 真实 Node→AC→Iced 与事件回路，V-AC-DESKTOP/OWNER，形成 AC-HOST 冻结证据。 |
| [ ] T-03 全 W2 | work | T-02、正式 W1 | 全原组件、示例/Widgets 的机器路径与交互，V-AC-WAVE2/NOCYCLE。 |
| [ ] T-04 W2 复核 | review | T-03 | 独立阶段复核 T-03 的全量结果。 |
| [ ] T-09 W2 合入与归档 | merge | T-04 | 落主并签发 AC 宿主与完整 UI-W2-AC，登记消费者版本，独立归档和清理。 |

## 9. 复审记录

- 2026-11-19 r3规划交接：明确AV路线只选AV，非AV#ignore由本目标实施阶段逐项恢复；原任务/状态/进度、完整门与原预期保持，本轮没有实现或运行。

- 2026-11-19 r2：`stage: new/revision`，`outcome: pass`（规划静态预检），`next: work`待后续指令/所需来源。新增FC/V-FC-CONSUMER/AC-FC，原T-01/T-02/T-03补接同源语言/迁移合同，任务/进度不变；未移动用例、修改expected、运行或合入。

2026-11-19 草案接手原 1068 T-04。W2 与三端最终验收分属不同计划，原检查尚待执行。

## 10. 待澄清事项

由 T-01 按当前 CLI 和 SDK 冻结真实 AC 选择器、产物及宿主 ABI，录入当时的正式来源。
