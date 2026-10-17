# PLAN-801 T-06 收据：迁位净删、单一实现、剩余边与 SHA

2026-10-17；stage: work。实施树 `D:/autostack/.wt/lang-801/auto-lang`，
branch `plan-801-dev`，base=043ca8567（master 分叉点），code=7c2c74ddc。

## 提交链（base → code）

| commit | 内容 |
|---|---|
| 8f3e8e9f8 | T-02 auto-parser-api crate + dialect-api-consumer fixture |
| ace61f7c4 | T-03 生产接入（端口实现/adapter/注册/水位清理/门面迁移） |
| 29ea71232 | T-04 model 单份迁入 + driver 转换 + V02–05 测试 |
| 643d5bdeb | T-05 777 域登记（lang-core） |
| 06e559a60 | T-05 基线重采样（4 条合法边，strict pass） |
| 7c2c74ddc | T-05 792 交付闭包接线（DELIVERY_CRATES 11→12） |

## 迁位净删/新增清单（27 files, +1654/−315 vs 043ca8567）

**迁出（auto-lang → auto-parser-api，门面保留）**：
- `session.rs` CompilerSession/Scenario+单测（211 行）→ API crate `session.rs`；
  auto-lang 门面余 25 行 re-export（−198/+18 含 DialectPriority）。
- `DialectPriority`（dialect.rs 枚举）→ API crate `dialect.rs`；旧路径 re-export。
- model 子集实现（ui-dialect-consumer/src/lib.rs 151 行）→
  dialect-api-consumer/src/lib.rs（轻闭包单份；桥面调用换端口映射，
  文法/护栏/恢复策略逐字保持）。

**新增**：
- `crates/auto-parser-api`（Cargo.toml + lib/session/dialect/context = 475 行 src）。
- `crates/auto-lang/src/dialect/external.rs`（ExternalDialectAdapter，48 行）。
- `parser_bridge.rs` +90 行（ParseContext impl + PortSnapshotSlot）；
  `parser.rs` +124 行（表字段/构造×3/实例计数/注册×2/port_checkpoint/
  port_restore/派发器水位×2）。
- fixture：dialect-api-consumer（Cargo.toml/src/verify-deps.sh/light_surface）；
  driver 新测试 v03_dispatch/v04_snapshot_lifetime/v05_legacy_api。
- 接线：根 Cargo.toml members+exclude、auto-lang Cargo.toml 依赖、
  module-boundaries 域映射+基线、rust_ui.rs DELIVERY_CRATES。

**净删边**：外部文法实现（dialect-api-consumer）→ auto-lang 实现依赖
（=778 时 ui-dialect-consumer 唯一依赖 auto-lang 的形态）被删除，换为
→ auto-parser-api（轻闭包 [api,ast,lex,diag,val]，V01 metadata+tree 双证）。

## 单一实现审计（V06）

- `parse_model_subset` 全仓唯一（dialect-api-consumer/src/lib.rs）。
- 内建 `parse_model_block` 全仓唯一（auto-lang/parser.rs）。
- 派发器 `try_dialect_stmt`/`try_token_dialect_stmt` 唯一（parser.rs:839/881），
  新旧方言同表，无第二派发表。
- `ExternalModelDialect` 实现唯一；driver 旧 trait 用例为 `probe <expr>`
  （LegacyProbeDialect），非第二份 model 算法。
- AST/词法/诊断沿 799/789/793 单一实现（auto_lang::{ast,token,lexer,error}
  门面 re-export，无镜像 DTO）。

## 剩余边（如实登记，不宣称 auto-ui 可搬仓）

1. 内建 `UiDialect` 经 `pub(crate) parser()` escape hatch 使用完整 Parser：
   widget/msg/component/actions/style/view/on/store 完整文法仍在 Parser 上
   （Parser→内建文法边未删；按族外置属后续合同）。
2. 生产入口/import 新建 Parser 不自动继承外部注册（778 冻结边界保持）。
3. widget 体内 model 由 `parse_widget_decl` 结构解析器直连内建，不经语句
   方言派发（v01_v05 v03 测试钉住该冻结边界）。
4. 旧 `Dialect` trait + by-value `BridgeCheckpoint` 兼容层保留（非债：778
   公开面承诺；终态随完整文法外置合同收敛）。

## SHA256（前 16 位，code=7c2c74ddc）

```
9cd1fdfe7a3b1b9f  crates/auto-parser-api/src/lib.rs
78d872a3d01cdb2d  crates/auto-parser-api/src/context.rs
aae8175450531389  crates/auto-parser-api/src/dialect.rs
63041f43ee8999b3  crates/auto-parser-api/src/session.rs
795ba18acf6e5f6e  crates/auto-lang/src/dialect/external.rs
b53036ec17752fc6  crates/auto-lang/src/parser_bridge.rs
482ea042aed6089a  crates/auto-lang/src/dialect.rs
45b98b8642187578  crates/auto-lang/src/session.rs
9706b9c326a8c421  test/fixtures/modularity/dialect-api-consumer/src/lib.rs
d822ab350f211fd9  test/fixtures/modularity/ui-dialect-consumer/src/lib.rs
```

## R1 复审修复轮（F-801-R1-01/02 闭合）

- **F-801-R1-01（快照过期失效漏洞）**：不变量=端口快照寿命由**签发
  作用域**决定，而非表长水位。根因：tombstone 复用可使递归内层调用签发
  的快照落在外层调用入口表长之下，`truncate(watermark)` 清不到；gen/
  实例 id 校验全匹配，过期凭据照常恢复游标。修复：`PortSnapshotSlot`
  增 `scope` 字段（签发调用作用域，派发器每次调用前递增、嵌套保存/
  恢复），调用退出 `port_release_scope` 按归属置 tombstone（尾部保守
  收缩），两派发器同改；`port_restore` 增存活位校验（原有）。受影响
  路径=Ident/token 两派发 × Some/None/Err 三态；复审 12 例反例收编为
  永久回归 `tests/r1_scope_expiry.rs`（6 无洞正控 + 6 复用槽位反例，
  全绿）；既有 V04 三例/跨 Parser/旧 BridgeCheckpoint 控制保持绿
  （driver 48/48）。
- **F-801-R1-02（交付验收缺失）**：本版 12 成员闭包经 792 正式入口
  `test/fixtures/modularity/generated-ui-consumer/driver.py` 完整重跑，
  **25/25 PASS exit0**：V01 fresh 独立锁+真实 build（generate crates=13
  含 auto-parser-api；resolve 3s patch 单源+pin 全命中；build 118s 产物
  在位）、V02 A→B 不带 target locked 重定位（112s，metadata iced_widget
  单源@B 内、越界 path=无，资源字节=producer）、V03 十四态负控（等长
  篡改/坏 hash/缺资源/未登记/漏 patch/不兼容 pin/外来目录/换装保旧/
  sidecar 外部与根相对/邻位保留/raw TOML/他应用存活）+恢复绿、V04 开发
  路由与模板同源。收据与日志：
  `attachments/801/delivery-driver-r2-receipt.json`、
  `delivery-driver-r2.log.txt`（scratch=组内 scratch-consumer，已留档）。
- **SD 增补（SD-04）**：canonical
  `docs/specs/repository-boundaries/generated-ui-distribution.md` 交付
  闭包清单 11→12 成员（+auto-parser-api 及其闭包内依赖说明）已在
  worktree 备稿；frontmatter `supersedes_spec_components` 增补该 target。

## 验证汇总（work 阶段定向面）

- V01：fixture 3/3 + verify-deps.sh PASS + metadata 闭包=[auto-parser-api,
  auto-ast, auto-diagnostics, auto-lex, auto-val]。
- V02：778 断言逐字保持经新接口注册（v01_v05 12 + r1 8 + r2 6）。
- V03：v03_dispatch 4/4（混排/同段顺序/with_session/Core 门控）。
- V04：v04_snapshot_lifetime 3/3（嵌套恢复/跨调用过期/跨 Parser 拒绝+表
  未污染）。
- V05：v05_legacy_api 3/3（旧 trait/泛型 syntax_error/by-value checkpoint/
  session 类型身份）。
- V06：单一实现审计（上）+ 777 strict pass（0 违规，2 grandfathered 存量）
  + 792 delivery 单测 9/9。
- 定向：auto-lang dialect 7/7、session 136/136、plan778 4/4；auto-parser-api
  单测+doctest 绿；cargo check -p auto-lang --all-targets 除预存 bench
  E0601（主检出同款，非本线引入）全绿；零新增编译警告（基线 414 → 本树 411）。

merge-owned 门禁 pending（按 2026-10-14 测试时序裁定）：冻结提交裸
`cargo t` + `cargo tv`（触 VM/编译器面：parser.rs 改动）；ui_gen 零触面
不跑 `cargo tu`；tf 到期归主检出批量档。
