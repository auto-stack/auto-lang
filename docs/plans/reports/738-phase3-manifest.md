# PLAN-738 Phase 3 / T-12 收据：manifest 共同身份与多消费者三角对拍

- 任务：T-12（manifest 共同身份与多消费者三角对拍）
- worktree：`plan-738-dev`，基线 `84bec29ef`（T-11 提交），本任务提交见 §5。
- 触面：`stdlib_assembly/manifest.rs`（双重身份 schema 4）、`auto-man/api_gen.rs`（收据身份切换 + workspace lock 记录）、`auto-man/rust_ui.rs`（新鲜度门双指纹 + lock 对拍）、`tests/plan738_stdlib_assembly_tests.rs`（t12 模块 2 测）、`rust_ui.rs` 真值表 1 测。

## 1. 设计决策：共同装配身份 × 消费者收据身份（schema 3 → 4）

rv3 §4.3 登记：`consumer` 名与 `consumer_input` 源参与指纹 → 不同消费者对同一装配内容得到不同"指纹"，三角对拍无共同基准。修复为**双指纹**（记录为 T-12 的身份规则）：

| 字段 | 构成 | 用途 |
|---|---|---|
| `fingerprint`（共同装配身份） | payload 消费者中立投影：consumer 名置空 + `consumer_input` 角色源剔除后哈希 | 跨消费者三角对拍：同 fixture/同 target/同 features/同来源闭包必相等 |
| `consumer_fingerprint`（消费者收据身份） | 全量 payload（含 consumer 名与业务输入） | 新鲜度门/收据对拍（= 旧 schema 3 单指纹语义，行为兼容） |

- `with_consumer_input` 只改收据身份，不改共同身份（业务输入不是装配差异）。
- 迁移规则：旧（≤3）收据的单指纹语义 = 现 `consumer_fingerprint`；新鲜度门全部切换到 `consumer_fingerprint()`（api_gen 烘焙常量/在途漂移核对、rust_ui 复用门）——**同消费者跨时比较语义零变化**；跨消费者断言一律用 `fingerprint()`。未知/旧不完整快照（无指纹/None）维持既有真值表语义（保守再生），lock 面新增同型真值表。

## 2. 内容闭包与身份缺项补齐

- public/实际选定层/缺失状态：freeze 既有（sources 角色 public/selected_target/native_declaration）✓。
- 依赖关系/provider schema/实现/target/environment/features：provider_inputs 按目标分叉（VM native 族/Rust a2r-std 全源/C trans）+ Cargo.lock + BUILD_INPUTS ✓（T-09 审计已核证包含关系）。
- 最终引用证明：references 携带 T-10/T-11 契约证明 ✓。
- **生成 workspace 实际 lock/features（§5.8.4 缺项）**：generation.json ready 记录新增 `workspace_lock`（生成产物运行时依赖输入的 FNV 身份；未建 lock 记 `absent`）——与生成器自身 Cargo 输入（manifest provider 面）分开记录，不互相冒称；复用门 `lock_freshness` 对拍（lock 出现/变化/旧收据缺字段 → 判陈旧走再生臂）。

## 3. 三角对拍与正负例（t12_manifest_identity，实际数据驱动，无手工拼 JSON）

| 测试 | 断言 |
|---|---|
| `triangle_consumers_share_assembly_identity_not_receipt_identity` | 同 fixture 同 target：CLI actual 路径（session→compile_actual_references→freeze("stdlib-inspect")）↔ 编译会话最终快照（真实 `trans_rust_with_session` + `freeze_assembly_manifest("compiler")`）↔ 生成收据形态（api-generation + 业务输入）——三者 `fingerprint()` 全等；`consumer_fingerprint()` 互异；schema=4 |
| `identity_invalidates_on_assembly_inputs_not_business_inputs` | 同内容异绝对根共同身份稳定（便携身份）；业务输入只改收据身份；target 变化改变共同身份 |
| `lock_freshness_truth_table`（rust_ui） | lock 双方一致才新鲜；缺字段/单侧缺失/漂移/absent↔实值均保守再生 |

## 4. 门禁

| 命令 | 结果 |
|---|---|
| `cargo t plan738` | 69/69 |
| `cargo test -p auto --bin auto stdlib -- --test-threads=1` | 10/10（CLI 三模式 + actual 闭包） |
| `cargo test -p auto-man --lib api_gen -- --test-threads=1` | 43/43（收据身份切换后生成链全绿） |
| `cargo test -p auto-man --lib freshness -- --test-threads=1` | 2/2 含新真值表；**`test_shell_pack_lib_freshness` 基线预存环境红**（stash 实证：T-11 基线树同红——本地 auto-os/shell 检出与入库 shell-pack 物不一致，与本轮改动无关，逐名在案） |
| `cargo t plan724` / `cargo check` 三 crate | 5/5 / 零错误；触面文件 rustfmt 干净 |

真服务完整链（生成→实编→serve→ready 含 workspace_lock 收敛）在 T-14 最终提交复跑。

## 5. 提交

- worktree `plan-738-dev`：`fix(stdlib): PLAN-738 T-12 manifest 双重身份——共同装配身份/消费者收据身份分离（schema 4）+ 生成 workspace lock 收据身份 (Plan 738)`。
- 映射：T-02/03/05/06/07、AC-01/02/04/06/07/08、SD-01/03/04/05/06。
