# PLAN-796 规范增量（spec-delta）

2026-10-18 重写冻结（R4 修复轮后）；锚定代码 = 本分支 HEAD（F-796-R4-01..13 全闭合）；
冻结决策出处：[contract-decision.md](contract-decision.md)（D1–D13，含 R4 期间的
D7 增补：called-sigs-fp 与 **src-fp 入键**——同键不容两容的 CAS 定律）。
**本文是评审用 delta 文档，不是已发布的规范**——canonical Spec 落盘由 merge 阶段执行。

## 状态总览（R4 修复后全部定稿）

| delta | 状态 | 要点 |
|---|---|---|
| SD-01..04 | 定稿并实现 | 新 auto-semantics 四件；预算/来源/保活按 R4 修复后事实绑定 |
| SD-05/06 | 定稿并实现 | auto-ac/auto-hir 追加段；不冒充旧 producer、不创建公共 Checked |
| SD-07/08 | 定稿并实现 | probe/run 接缝含 R4 的 typed 结局/生命周期/身份准入 |

## SD-01 add `docs/specs/auto-semantics/project.md`

三类数据分离（Facts/Work State/Runtime Projection）；独立 workspace
`experimental/ac-semantics`（contract/core/probe；core-ast feature 隔离后端依赖，
默认树零 auto-lang/UI/VM/Cranelift/auto-man/auto-val/auto-cache——sema_boundary_
cargo tree 断言）；支持矩阵 15 类（6 Implemented / 7 SchemaOnly / 2 Future）；
验证入口 `scripts/verify-ac-semantics-796.ps1 -Scope acceptance|closeout`
（acceptance 13 行：每行绑定最小执行计数，零选择=FAIL；closeout 列 merge-owned
门禁清单与 tf 收据在场检查）。

## SD-02 add `docs/specs/auto-semantics/semantic-contract.md`

LogicalId 名义身份（重启序无关/同名跨包不合并）；DeclRef 重导防篡改；envelope
`ac796/semaref/v1` admission；页 codec `ac796/sema-page/v1`（TypeDesc 13 变体、
递归组间接引用、≤512KiB、≤8192 深）；**R4 后**：record 页字段是**真实间接
TypeDesc::Record{DeclRef}**（非 Opaque 名串），发布经 Kahn 迭代序带 required-data
边（保留页跨 GC 存活；环=受控拒绝，组发布未实装）；CT 接缝 `ac796/ct-seam/v1`。

## SD-03 add `docs/specs/auto-semantics/inference-profile.md`

有限 i32/bool 双向推导+worklist；**R4 后**：直接调用含**实参数量准入**
（arity-mismatch 确定性诊断）；occurs/unsolved/歧义/unknown 均确定性诊断无 Any
兜底；差分对照合同（新旧管线独立运行，接受/拒绝与绑定类型程序序逐对一致——
手写 17+ 例与 LCG 240 例）；工作态永不持久化。

## SD-04 add `docs/specs/auto-semantics/query-cache.md`

查询键组件含 called-sigs-fp 与 **src-fp（R4 冻结）**：被读签名或源修订变化=
键变化（一键不容两容；位置漂移不得服务旧位置——漂移源走新键新算，spans 恒绑定
当前修订）；freshness 三态分类（Fresh/ProvenanceStale/Stale）；typed 结局互斥
（hit/reprojected/published-miss/published-stale/diags/resource/cancelled——
cache 词永不冒充成功）；**事实归属双向准入**（发布前 summary.fn_logical==target；
每次读取校验 query tag/target/variant/fn_logical）；预算账本接入真实查询流
（scratch 预留失败=typed resource 拒绝；有界 record 名注册表 8192 上限替代
无界 Box::leak；OS private-bytes 窗内采样）；受管隔离 namespace S796
（SnapshotRecord 载体，三重 admission+事实归属）。

## SD-05 modify `docs/specs/auto-ac/artifact-contract.md`（追加节）

796 语义 producer 的 A06 接缝：S796 namespace 用 783 通用块/图 API 作容器；
原 Future 登记与旧 producer 范围一字不改；A05/A12 仅 schema 冻结
（model_compute 独占 vm_execute）。

## SD-06 modify `docs/specs/auto-hir/stage-contract.md`（追加节）

before（现行）：verify 是 Checked 的唯一构造路径，语义事实无独立信任边界描述。
after（追加，不改现行）：InferFacts（无工作态）可作为 checked 准入的**只读输入**
被消费；资源/取消状态不得作为准入裁决；**本计划不创建公共 Checked、不取代也不
绕过既有 verify**——旧实现仅作差分对照 oracle 被只读引用。

## SD-07 modify `docs/specs/auto-test-case/architecture.md`（已实现）

adapter probe×run（v2-only）；注册配方表 [sema/query-v1]；声明式步骤
（query/edit-source/restart）；工具身份可信宿主绑定（SEMA_CASE_WORKER），
case 永不携带可执行；**R4 后**：worker `semaworker/1` 全生命周期（父墙覆盖
响应+退出+回收；stderr 越限即杀；4MiB 读上限；构建限时）；退出状态/版本镜像
准入；exe 内容 hash 入报告与 stale 轴（sema-worker-exe-sha256/version）；
restart=统一 runner 内真实跨进程持久化；**判定全在父侧**（typed 结局先于
oracle：ok 需零诊断、diags 须匹配 stage/code/message、resource/cancelled/infra
=Infrastructure Error 永非语言裁决）。

## SD-08 modify `docs/specs/auto-test-case/profile-v2-native.md`（已实现）

v2 追加 probe 面：atom-data 源类别；expect.probe 期望（cache 词表含
reprojected/diags）；probe 错误 completion 支持前端阶段（parse/resolve/typecheck）
且由父侧真实判定（类型错程序配 success 期望=FAIL，配 error completion=PASS）；
bindings 边车内容 hash 入 oracle 身份；v1 接受/拒绝集字节不变
（plan788_r3_ 兼容面全绿）。

## 依赖与并行审计（R4 后事实）

- 796 依赖（只读）：ac-core 公开 AST/parse（feature 门控）、783 store API。
- **T-06 起共享协议文件为授权写面**（auto-test-case contract/ir/suite、auto-lang
  test_cases/runner 测试/语料、plan788_r3_ 新字段编译兼容）；783/ac-core 源码、
  根 manifest/cargo 配置仍零触碰（sema_boundary_ diff 断言按阶段化允许清单）。
- 依赖 796：797 只读消费（D6 全集+ct-seam schema 冻结可读）。
- 冻结接口常量全部有 contract-decision 出处（sema_boundary 锁定）。

## touched_goals 说明

frontmatter `touched_goals: []`：本仓 specs 现行目标索引（docs/specs 各
project.md 的 goals 面）无"语义元信息/类型推导/统一缓存"对应目标条目——
PLAN-796 是 Design 42 的首个实施计划，其目标在本文 SD-01..08 沉淀后才进入
specs 体系；不虚构 goal ID，merge 时由 ledger 流程登记新目标映射。
