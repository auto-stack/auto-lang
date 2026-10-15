# PLAN-782 验证收据（verification.md）

- 工作树：`D:/autostack/.wt/lang-782/auto-lang` @ `plan-782-dev`；
  基面 master `4568e4ea7`（770 实现合入后基线）。
- 本文件由 T-06 写定。门禁原始收据在
  `experimental/ac-core/target/ac-verify-receipts/text-record-782/`（gitignored，
  含 25 步逐项命令/exit/完整输出 `raw-*/`、案例目录 `cases/` 与最终
  `gate-20261015-093737.txt`）。设计冻结见 [design.md](design.md)、
  逐例索引见 [probe-cases.md](probe-cases.md)。规范增量提案由 T-07 另行
  写定（proposed-spec-delta.md，不在本步发布 canonical）。

## 1. 提交与产品清单

| 项 | 值 |
|---|---|
| T-01 提交 | `1bc7a1665`（design.md 冻结 + whole-path probe + 边界探针收据） |
| T-02 提交 | `697e3f811`（HIR 域/schema/binder/verifier + text_record_hir 13 测试） |
| T-03 提交 | `24171d462` + F-A 修复 `a2aa02a5b` + 收口 `3f0366f6d`（source 七模块组合面 + text_record_source 19 测试） |
| T-04 提交 | `3be6031d1`（text_resources 资源走查扩展 + text_record_resources 25 测试） |
| T-05 提交 | `a2fc74d6d`（合并 native lowering + text_record_native 11 测试） |
| T-06 修复提交 | `836635a2e`（gate 工作发现的 lowering 缺口修复 + 永久回归，§5） |
| T-06 收口提交 | `8558863ec`（门禁脚本 + verification.md 首版） |
| T-07 提交 | `0318bcb93`（V-04 两层 wrapper 反例闭合 +2 测试）+ `9726a412c`（proposed-spec-delta.md 冻结） |
| 依赖 | 无 auto-down / crates 改动；runtime/text 零源码改动（23 符号、合同 sidecar 原样） |

新增/修改（相对基面）：`experimental/ac-core/src/{hir,descriptor,verify,native,main,lib,text_resources}.rs`、
`src/source/{mod,parser,resolve,typecheck,adapter}.rs`；`schema/text-record.atom`；
`tests/text_record_{hir,source,resources,native}.rs`；`tests/record_source.rs`（F-A 回归 +1）；
`fixtures/text-record-{hir,source}/{valid,invalid}/`；`scripts/verify-ac-text-record-782.ps1`；
`docs/reports/782-text-record/*`。旧域（core/scalar/text/record）接受/拒绝集合零变化
（旧五门在最终基线实跑证明，§4）。

## 2. 测试设计执行（plan §6 Case 表 → 实际入口）

| Case | 实际入口 | 结果 |
|---|---|---|
| V-01 | `text_record_hir.rs` 13 测试（组合三元组、四旧域隔离、改一维拒、writer roundtrip）+ gate 步 14/18（profile isolation、receipt identity 五针） | ✅ |
| V-02 | `text_record_hir.rs`（Str/str/char 字段、传递分类、cycle、copyable 保持）+ `text_record_source.rs`（命名/位置构造、名义失配）+ `text_record_native.rs`（Token 四字段值/char、嵌套深克隆独立性）+ gate TP01 | ✅ |
| V-03 | `text_record_resources.rs`（use-after-move/double-move/循环重入/缺 .move 对称/ctor 裸转移拒、move 参数被调方清理计划、clone 不扰借用）+ `text_record_source.rs`（move/clone 源码腿、partial-move 含死分支） | ✅ |
| V-04 | `text_record_resources.rs`（view 参数字段返回合法；owned/mut/move-in 字段逃逸拒；替换/移根期间活跃借用 mut-conflict；悬挂视图使用点拒；**T-07 补：两层 mut-wrapper 同源拒/异源过**——770 R4-04 传播类组合闭合） | ✅ |
| V-05 | `text_record_native.rs`（r=r.clone()/transform/字段 concat/move 重建的逐步旧缓冲释放 trace）+ `text_record_resources.rs`（replace_frees 计划形状）+ gate TP02（重叠形态 PE exit 9 + 缓冲配对） | ✅ |
| V-06 | `text_record_resources.rs`（臂局部自臂出口闭、嵌套 return move、break/体末释放、直接 HIR 腿）+ `text_record_native.rs`（0 次循环零分配） | ✅ |
| V-07 | `text_record_resources.rs`（P3 双根双字段序、嵌套展开、owned_return 不排空；status/trap 双通道计划形状） | ✅ |
| V-08 | `text_record_hir.rs`（ctor/call binding 越界、clone 跨型、requires 删除受控拒） | ✅ |
| V-09 | `text_record_native.rs`（短路跳过臂零分配零释放、receiver/命名实参 marker 先于 FREE） | ✅ |
| V-10 | `text_record_native.rs`（256 层含资源嵌套、4096 迭代新鲜根、0 次循环零分配；前置深链在 771/770 套件内回归） | ✅ |
| V-11 | `text_record_native.rs` v11（缺 capability 拒绝且无制品）+ gate 步 16/17（capability 控制、SENTINEL 三件制品 hash 保持） | ✅ |
| V-12 | `text_record_native.rs`（Token 微程序 exit 57、新旧 profile 共存、直接 HIR 腿、**新增 tr01 fixture PE 腿**）+ gate 步 19..23（旧五门实跑）+ 步 24（scope diff） | ✅ |

新测试合计 **71**（hir 13 / source 19 / resources 27 / native 12；T-07 +2）；全套
ac-core `cargo test --locked` **433 通过 0 失败**（T-05 基线 430 + T-06 回归 1 +
T-07 反例闭合 2）；runtime/text **20/20**（零改动回归）。

## 3. 覆盖内省

- 整数/trace oracle 独立推导（design §4.5 布局冻结值、P3/P4 顺序、exit 码
  由源码语义手算后实跑对拍），无 golden 自动更新路径；无 skip 的 mandatory。
- gate TP01/TP02 的缓冲配对 oracle（LIT/CLONE/CONCAT id 多重集 == FREE id
  多重集）不依赖具体 trace 文本，杜绝泄漏/双释放；零长 Str 无 id 无分配
  （D-06）在配对规则内天然成立。
- 已知非阻断（§10 登记）：D-771-A/B/C 三项基线预存缺陷均非本计划引入，
  组合程序以既有语法规避，未扩面。

## 4. 门禁执行（verify-ac-text-record-782.ps1）

**最终基线 `9726a412c`（plan-782-dev，T-07 收口）：25 步全绿，0 failed，11 gate
案例**（2026-10-15 实跑；收据 `gate-20261015-100236.txt`，原始输出
`raw-20261015-094825/`。中间绑定：T-06 冻结基线 `836635a2e` 同 25 步全绿，
gate-20261015-093737；T-07 增量为 +2 组合测试与文档，基线重跑后本收据为
最终绑定）：

| # | 步骤 | 结果 |
|---|---|---|
| 1 | warning matcher controls（正/负对照） | PASS |
| 2 | context（HEAD/branch/rustc/cargo/rust-lld sha256 + raw dir） | PASS |
| 3 | ac-core `cargo check --locked --all-targets` 零警告 | PASS |
| 4 | ac-core `cargo test --locked`（433；四 text_record target 在场断言） | PASS 176.2s |
| 5 | ac-core `cargo fmt --check` | PASS |
| 6 | runtime/text `check --locked --all-targets` 零警告 | PASS |
| 7 | runtime/text `cargo test --locked`（20；故障注入/预算/trap/合同面） | PASS 10.8s |
| 8 | runtime/text `cargo fmt --check` | PASS |
| 9 | 构建工具二进制（sha256 入收据） | PASS |
| 10 | 构建 text runtime 支持库 + 合同 sidecar（api/revision/target/lib_sha256/symbol_count=23） | PASS |
| 11 | TP01 旗舰 PE：p01 构建+运行 exit 30、stdout 空、7 缓冲 id 全配对、obj+receipt 在场 | PASS |
| 12 | TP02 重叠形态 PE：p02 exit 9、5 缓冲配对 | PASS |
| 13 | HP01 直接 HIR 腿：tr01 fixture exit 2、精确 trace `LIT len=2/LIT id=1/TEXT-SCALARS n=2/FREE id=1` | PASS |
| 14 | 负例语料 8 例钉码拒绝（TN01..08，exit 1，码见 §2） | PASS |
| 15 | profile 隔离（四旧域拒 text/source.unsupported；未知 profile exit 2；域行在场） | PASS |
| 16 | **故意失败控制**：缺 capability → exit 1 + capability.missing + 零制品 | PASS |
| 17 | **SENTINEL 事务**：失败 build exit 1 ≠ 成功 0；exe/obj/receipt 三件 hash 保持 | PASS |
| 18 | 收据身份五行（source-profile/hir-schema/hir-revision/hir-profile/source-input-hash） | PASS |
| 19 | 旧 gate 771（record 面） | PASS 234.7s |
| 20 | 旧 gate 741（-SkipMainGates；主仓 t/tv 属批量回归档） | PASS 9.3s |
| 21 | 旧 gate 745（source core-i32） | PASS 35.6s |
| 22 | 旧 gate 767（scalar；含 post-772 固定计算面） | PASS 22.7s |
| 23 | 旧 gate 770（per-plan scope 失败归因，见下） | PASS 579.0s |
| 24 | scope 隔离（V-12：782 域清单外零改动；crates/test/ACC 绑定未触=strict acc_inventory 前提） | PASS |
| 25 | hashes（runtime lib+合同、24 项 gate 输入、三案例 PE、gate 脚本自身） | PASS |

### 4b. 770 gate 归因裁定（步骤 23 的语义）

770 gate 内置 per-plan scope 断言，其 allow-list 仅含 770 路径；在后继计划
树上按定义必然失败（782 自身的 18 个已提交报告/probe 文件不在 770 清单内）。
本 gate 不修改 770 已合入脚本（旧面字节不变），而是**实跑全程并精确归因**：
断言其唯一 `[FAIL]` 即 scope 步、被标文件集与同一 git diff 的重算结果一致、
且全体实质步骤（两 workspace check/test/fmt、matcher 对照、其内嵌四旧门）
全 `[PASS]`；被标集须 ⊆ 782 允许清单——**真实越界改动仍会落在本步红灯**，
等价于把 scope 不变式重基到 782 域（每计划的 scope 由其自身 gate 把关的
不变式链保持成立）。770 gate 的收尾 hashes 步在其 scope throw 后中止
（纯记录性、无断言），由本 gate 步骤 25 原样重算覆盖。

## 5. T-06 缺陷发现与修复（gate 工作抓获）

| 发现 | 修复 | 永久回归 |
|---|---|---|
| verify 的 FieldGet 兼容规则（T-04 冻结）同时接受 owned 字段读取的 `Str`（declared）与 `str`（retag）两种拼写，但合并 lowering 的 `TStep::Field` 只匹配 `str` 视图拼写——`Str` 拼写落入标量装载臂，消费方 text 操作触发 `native.rs unreachable!` panic（exit 101）：**Checked bundle 不可 lowering，违反后端合同**。T-02 冻结 fixture `tr01-combination.atom` 恰用 `type: Str` 拼写，gate HP01 案例首跑即暴露；T-05 测试的直连腿用内联 atom（str 拼写）未覆盖此面 | `native.rs` `TStep::Field` 视图装载臂扩为 `StrBorrow | StrOwn`——字段读取两种拼写都装载 (ptr,len) 借用视图（读取永不复制 owner，与 §5.2 语义一致） | `text_record_native.rs` `hir_fixture_str_typed_field_get_pe`：tr01 fixture 端到端 build→PE，exit 2 + 精确 trace；全套 431/0 |
| **V-04 案例缺口（T-07 核对发现）**：冻结案例表"两层 wrapper 同源/异源 view-mut"未物化（T-04 收据证据清单亦未声称，非测试退化） | `0318bcb93` 补 `wrapper_same_source_mut_view_conflict_rejects`（mut 链两层、同源→verify.mut-conflict）+ `wrapper_independent_sources_stay_legal`（异源过——按 SOURCE 不按形式参数号） | 前述两测试；全套 433/0；gate 第 4 轮 25 步全绿重绑定 |

门禁脚本自身迭代记录（非产品面）：第 1 轮在负例步骤暴露 `[string]` 参数
默认值经类型约束成为空串、`$null -ne` 守卫失效的 PS 5.1 陷阱（13 步后
中止，收据 gate-20261015-085429）；改 `IsNullOrEmpty` 守卫后第 2 轮 25 步
全绿；第 3 轮在提交 `836635a2e` 冻结基线上重跑全绿并作为绑定收据。三轮
原始输出均在 receipts 目录。

## 6. 输入指纹（节选自 gate-20261015-100236.txt）

| 输入 | SHA256 |
|---|---|
| head（最终绑定） | `9726a412cc3810bbf6f330565dd7b8c29f1cd6a7` |
| rust-lld | f436ddda2f519c5ad18b7c7b09a3eadc2d4fc1be88cd9280cc472740fc3b7389 |
| runtime lib（ac_text_runtime.lib） | 3d7cc13b4c92cbd3d35ebeed7930b3015638a1c2f1a73422d6273aa853b035b9 |
| runtime 合同 sidecar | a4529883ccb21429e0aee632939eb3935b12ba1c3f750c88988b952351eec87a |
| schema/text-record.atom | de5d4b955c331687f78cddef7511c5ab349ea658338e1e1f501c640c4f93aa91 |
| 工具二进制 | 7eab01d26f166969b6db9269d8712fc1fe1f8d5b2d0c9ce148f5fe0d3d1bc5d6 |
| gate 脚本自身 | fefaf6e721ca2d1e5746b44b21845275267923e46972b695d48ec6502fd8d38c |
| PE TP01/TP02/HP01 | 67f70f83…/（TP02 见收据）/75ac2903…（全文见收据） |

（收据内含全部 24 项 gate 输入 hash：src 九文件、tests 四、fixtures 三、
runtime 四、schema 一等；每案例 case.txt 附输入与 exe hash。）

## 7. 后续 owner（不在本步）

- T-08/T-09：独立复审与合入/沉淀/归档（T-07 已冻结
  [proposed-spec-delta.md](proposed-spec-delta.md)——SD-01..06 + 附A 反例
  组合闭合零缺口 + 附B 源文件 manifest；canonical 发布归 T-09）；主仓裸 `cargo t` 等 merge-owned
  门禁按 2026-10-14 时序裁定在冻结提交上由 merge 阶段执行（本计划不触
  `crates/`，根档门禁不适用）；批回归到期判定归 `/auto-plan:regress`。
- 真实 Token/lexer 迁移、enum/List/use、可变实例方法——后续计划另行立项。
