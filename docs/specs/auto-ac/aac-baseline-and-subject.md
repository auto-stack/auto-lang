# auto-ac: AAC 基线、主体与阶段契约

> **Status**: experimental（合同冻结）
> Schema：`autolang.aac-baseline/1`（PLAN-1044 T-04 准入校验器与数据矩阵统一身份；
> 修改合同列/词表/冻结值必须同步 `crates/auto-lang-core/src/test_cases/aac_baseline.rs`
> 的 frozen 集合并升版，否则准入拒绝）。
> 契约数据载体：`crates/auto-lang/test/cases/ace-m2/aac-baseline-1044/`
> （contract-profile / subject-map / bridge-contracts / cs-requirements /
> cl-requirements / supply-matrix / stage-test-contract 七个 TSV）。
> 本文件是 1044 冻结的 CS/CL/主体/桥/代际分账与版本化数据/资源合同的叙述性规范；
> 逐行细节与准入反控以 TSV 与 T-04 校验器为权威。

## 1. G-AAC 绑定事实

AAC 独立后续路线（沿用 v0.7 归属），开工门 **G-AAC** 已满足：PLAN-985 T-A05
正式 M2A-08 收据（delivery `95c7fbad8`），`docs/specs/ace-migration/m2-certification.md`
落主（SHA16 `045340642830E6B0`，M2A-01..08 门级与 B-pending 见该 Spec）。
本包只消费 985/A，不等 985/B、984 全产品、986/987 整包。

开工身份重绑（2026-11-19，见 contract-profile.tsv identity 行）：Design59
`B7A4013BEEB18A42`、AAC 路线 `32D55995FEC8A2DE`、`core/src/source/mod.rs`
`92B5CD1BADE1DD7F`、`core/src/hir.rs` `870D33B813AD5902`、
`av-core/src/mir.rs` `2A2C32D4746C4A17`、`source-query/src/lib.rs`
`E79BB8BCF94597D0`（未变）、`experimental/ace/grammars/auto-source.at`
`9E5248B2A6AEACD7`、`docs/specs/auto-ac/project.md` `0E9CA9D469698FA6`、
`docs/specs/auto-hir/stage-contract.md` `8BE44EA7473E1954`、
`docs/specs/auto-abi/project.md` `27333270014826A0`。任何已合入身份失效后必须
重验对应 AAC 证据，不拿旧 SHA 证明新源。

## 2. CS 与 CL 分账

两个轴分别计量，不能互相替代（Design59 §1）：

| 轴 | 冻结内容 | 可声明的结论 |
|---|---|---|
| CS：compiler-source-demand | AAC 自身源代码实际使用的语言、stdlib 操作和资源 profile（cs-requirements.tsv，16 行） | Rust AC 能编译这一份 AAC 源码；不证明 AAC 支持完整语言 |
| CL：compiled-language-support | AAC 对 M2 最终 AV/AC/A2R required 语言/库/领域矩阵及 A 后延增量并集的逐项支持（cl-requirements.tsv，46 行） | AAC 可编译对应用户程序；不能仅凭自编译或原 Rust 测试通过认领 |

- CL 分母从**最终 M2 三端 required 及 A 后延增量**导入，包含 ~T/?T/!T、泛型、
  CTE、required 标注、dynamic/闭包/资源/领域及组合；**CS 未用不删 CL 行**。
- 每行标 语法投影 → Auto 语义 → MIR → runtime → consumer 的实际任务/来源
  （producer_chain 列）与 owner（Design59 具名实施任务或真实库/领域 owner）。
- `NA` 不在 disposition 词表：把 required 改 NA、用 A 期 native 不支持豁免、
  无补齐 owner 的 deferred，准入一律拒绝并列出该行 ID。
- 早期阶段允许具名后继（deferred 仅限 stage 阶段且有 owner）；完整
  Gen1/代际/M2 最终签章不允许 required 缺项。

## 3. 主体归属与可信边界

subject-map.tsv 冻结各模块的主体权威（auto/auto-driver）与有界 Rust 保留
（grammar 数据工具、Rust reference、AV 执行器、MIR verifier 二次安全边界、
OS 原语、Cranelift/linker/FFI helper）。语义权威模块（syntax/bind/semantics/
cte/lower/incremental/native driver）被列为 Rust backend helper 属于信任越界，
准入拒绝。原 Rust 实现留作 reference，生产 AAC 路径不得静默回落。

bridge-contracts.tsv 冻结五类接缝的信任边界：

- 语法/语义/HIR：Auto 为唯一源码语义权威，拔掉 Rust 源服务后正例仍成功；
- Typed MIR → 后端：序列化对象不是可信 Checked，checked 标志必须拒绝；
- CT → AV：禁止传源文本/AST 让服务重绑定；CT timeout/禁用 IO/取消不冒普通
  语言错误或发布冻结值；
- 增量/恢复：坏缓存不升级为 Checked，失败 edit 不发布半新集合；
- FFI/OS/link：不退回旧 AutoVM/A2X，缺失依赖明确缺失。

## 4. 代际分账

`Rust AC(Gen0) + S → native AAC(Gen1)`；`Gen1 + 同一 S → Gen2`；
`Gen2 + 同一 S → Gen3`。每代绑定源码树/stdlib/schema/backend/runtime/link/
target 与实际可执行文件 hash。Gen1/2/3 不能调用 Rust 源码入口。完整代际
签章独立，不强加为 M2 前置；其暴露的 M2 三端能力缺口须在 M2 最终前关闭。

## 5. 版本化数据与资源合同

物理持久层与逻辑工件合同分离：SQLite/文件实现不是可序列化 Checked 凭证；
AAC 缓存 namespace/format 独立冻结，逻辑 identity 兼容与坏旧 cache 拒绝
由 1052 实现 Auto 策略并恢复重验。本决定只改变 AAC 缓存，不迁产品数据。

资源 profile（M2-A 已冻结值，更严上限保留，不因本包放宽；新增未覆盖项用
有界代表输入测量后按批准 profile 登记）：

| 维度 | 冻结值 | 来源 |
|---|---|---|
| 前端 work_bytes | 64 MiB（不得用 256MiB 默认档冒 64MiB 授权） | FrontBudget::WORK_OWNED_64MIB |
| 最大解析深度 | 1,048,576（1<<20） | 同 frontdoor.rs |
| 语义单任务步计数 | 1,000,000 | semantics/budget.rs DEFAULT_MAX_STEPS |
| 语义墙钟 | 5,000 ms | 同 DEFAULT_WALL |
| CT root 累积期限 | 10,000 ms（单 root 共享、不重置） | source-cte/schedule.rs（817 轴） |

执行模式：单编译请求/单 CT 服务串行（首版 CS），不承诺无限资源。拒绝需
区分源码错误、资源拒绝、外部工具/OS 失败，不能硬截断为成功。

## 6. 供应矩阵（F01–F10）

supply-matrix.tsv 沿 Design59 §9 F01–F10 建立 CS/CL/主体/服务逐行责任表：
build 工具/host/target/CT 四轴，每缺操作绑定实际供给 Plan 或 1056；1055
交最小 CT profile（基础 LowerCt），高级 CT 由 1050/T-09、1053/T-04 关闭。
每行必须落实际实现任务/输入/oracle/正式交付条件，仅列 owner 名字不满足
（准入拒绝空列行）。AAC 语义补齐与原 AC 机器/runtime 补齐区分：前者归
1046–1050 原任务，后者按 Design59 实施归属表回 1051/1052/1055/1056/1067/1068
或实际库/领域 owner。

## 7. 阶段测试与跨仓交接合同

stage-test-contract.tsv 冻结：

- AutoDown 文档格式 owner 是 auto-down；语言仓 TestCase/core adapter
  （crates/auto-test-case + auto-lang-core/test_cases + vendor/autodown-test-parser）
  与 ACE worker 才是语言用例执行方——把 auto-down 目录当语言 TestCase
  生产者必须拒绝；
- 每包首次使用某 Auto 阶段 driver 的 work 任务建立/登记实际入口（必要时
  注册 Cargo target/现固定组），1053 只汇总整机，不把早期 Auto 代码一直等
  到 1053 才测；
- 仅旧 Rust 组绿而未跑 Auto 阶段 driver 的报告必须拒绝；真实命令/退出码/
  输入与产物 hash/覆盖与缺口入计划；
- Gen0 编译 driver 可使用 Rust 前端；driver 运行并处理被测源码/IR 时主体
  对应算法必须 Auto 执行，不能用 Gen0 重新编译被测用户源码冒 Auto 主体；
- 跨仓仅消费现有规范/SDK/域 host/TestCase 格式，版本变动按失效规则重验，
  无外仓代码修改授权。

## 8. 后继交接（1044 → 1045/1046/1047/1051/1052）

本包发布已冻结合同与具体供给缺口/关闭 owner，后继最小输入与版本：

| 后继 | 最小输入（1044 交付） | 版本/身份 |
|---|---|---|
| 1045 hosted stdlib/SDK | CS 库操作逐行（CS-201..208）与资源预算（contract-profile.tsv） | autolang.aac-baseline/1 |
| 1046 语法/源码/模块 | grammar 身份（auto-source.at `9E5248B2A6AEACD7`）、CL 语法投影行 | 同上 |
| 1047 绑定/类型/受检 HIR | subject 权威（bind=auto）、HIR 身份（`870D33B813AD5902`）、CL-101..104 | 同上 |
| 1051 native backend/CTE 桥 | bridge 接缝（mir-backend/ct-av）、CS-206/208、ABI 身份 | 同上 |
| 1052 增量驱动/制品 | 身份分轴（incremental 接缝）、supply F04/F05、资源预算 | 同上 |

任何未供应 CS 行保持缺口并回 Design59 具名实施任务或真实 owner；1045 关闭
实际使用验证，本包不冒全部 AAC 源码可编译。1053/1054 消费本包 CL/供应/
代际合同做整机与独立验收。

## 9. 来源与关系

Design59（`docs/design/59-aac-native-bootstrap-and-compiler-subject.md`）、
AAC 路线（`docs/design/strategy/aac-delivery-roadmap.md`）、
`docs/specs/ace-migration/m2-certification.md`（985/A 正式收据）、
`docs/specs/auto-ac/project.md`（现行 Rust AC 事实，本包 SD-02 在其 review
通过后补 AAC 指针与已交事实）、`docs/specs/auto-hir/stage-contract.md`（HIR
阶段合同，AAC 主体沿用其 schema/版本化接口）。

## 10. 变更记录

- 2026-11-19 PLAN-1044 work：首版冻结（G-AAC 绑定、CS/CL 分账、主体/桥、
  资源、供应 F01–F10、阶段/跨仓合同、后继交接）。仅合同冻结，不代表任何
  AAC 编译器模块已实现或 CL 已支持；阶段/整机/代际验收分别在
  1045–1054/1056 与 1053/1054/1068 原门关闭。
