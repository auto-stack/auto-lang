# PLAN-810 work 阶段交接（handoff）

2026-10-19。`stage: work | PLAN-810 | revision 1 | outcome: pass（work 自有
AC 完成，送独立 final review）`。

## 1. 身份

| 项 | 值 |
|---|---|
| 工作树 | `D:/autostack/.wt/lang-810/auto-lang`（组内兄弟 auto-down detached@c39ac7f，未触碰） |
| 分支 | `codex/plan-810-dev` |
| 基面 | master `ab08955c2` |
| work 提交 | `c8183ff73`（T-02..T-04 迁移）→ `605ed28d1`（T-05 逃逸口删除）→ `1900645c1`（T-06/T-07 测试与轻 consumer）→ `ccd024d72`（T-08 SD 备稿，**review 绑定此提交**；代码面冻结于 `1900645c1`） |
| 依赖 | auto-parser-api/auto-ui-dialect 均在本仓 worktree 内修改；无跨仓改动 |

## 2. 交付内容

- **端口（T-02）**：`auto-ui-dialect::grammar::UiParseContext`（对象安全子
  trait，`ParseContext` 零改动——既有外部 consumer 编译面不受影响）+
  `ParserBridge` 实现 + Parser 端口后端（`port_restore_raw`/`port_probe_
  next_token`/`port_advance_peek_kind`/`port_capture_raw_block`/诊断事务/
  `bind_var`/`define_handler_param`/`register_style_recipe` 等，复用 801
  句柄表后端）。五个模块私有 helper（expect_ident/cur_is_soft_ident/
  enter_scope/exit_scope/dot_item）升 `pub(crate)`——同一实现，非复制。
- **迁移（T-03/T-04）**：parser.rs 13223–17909（基面行号）完整 UI 文法
  约 4.9k 行迁 `grammar/{widget,view,scene,store,actions,helpers}.rs`；
  文件级自由函数 on_handler_event_name/damerau_levenshtein_1/
  stmt_expr_contains_await 随迁；Parser 留同签名薄委托门面（8 pub +
  looks_like_scene_decl 私有门 + 806 msg/model 委托收编）。迁移脚本
  `scripts/plan810_migrate.py` 留档（一次性工具，可复跑对拍）。
- **接线（T-05）**：`dialect/ui.rs` 重写为全端口薄 adapter；
  `ParserBridge::parser()` escape hatch **删除**（全仓零残留调用）。
- **验证（T-07）**：`crates/auto-lang/src/tests/plan810_full_ui_grammar.rs`
  （13 测，V-01..V-04）；`test/fixtures/modularity/ui-grammar-consumer/`
  （独立 workspace，7 测，V-06 轻闭包）；根 Cargo.toml exclude 注册。

## 3. 事务/行为冻结核对（contract-baseline §3 → 实测）

- computed object→body：raw save + 诊断 truncate + skip_check 三段复刻
  （v02_computed_body_fallback_rolls_back_speculative_diagnostics 绿——
  errors/warnings 零残留）。
- raw CSS：`capture_raw_block` 端口 + 文法侧前置条件保持（native_css 7 测
  字节级绿 + v02 多字节/括号注释绿）。
- 近失告警：raw cursor 投机 + warn 端口（v02_near_miss 绿，告警内容
  found/suggestion 断言）。
- scope：fragment/component 参数 bind_var、for var/index、on 参数
  define_handler_param（v03 绿 + 轻端口观察口断言）。
- style recipe：raw 投机 None 回退（v02 + toy None 反例）、docs 捕获
  （v02_style_recipe_takes_docs）、注册副作用经端口（toy recipes 观察口）。
- 事件族（on-events/arrow）**未迁**（Core 共享归 Core，非 UI 回退同源）；
  `parse_stmt`/`parse_store_stmt`（Core 表达式 store）未迁——与
  `parse_store_decl` 按调用点区分。

## 4. 验证收据（work 阶段，2026-10-19 实测）

| 门 | 结果 |
|---|---|
| `cargo check -p auto-ui-dialect -p auto-parser-api -p auto-lang` | 通过；新增 warning 清零（唯一残留 `widget.rs:566 events` 解构未用字段为基面同款 lint 的原样迁移，非新增） |
| plan810 模块 | 13/13 |
| plan806 / plan408 / plan367 / plan024 / plan051(+timer) | 18 / 20 / 6 / 16 / 25 全绿 |
| widget / store / view_ / actions / scene / style / recipe / native_css / plan339 / plan340 / plan370 / plan778 | 68 / 56 / 240 / 7 / 6 / 198 / 18 / 7 / 7 / 12 / 18 / 4 全绿 |
| 轻 consumer | `cargo test --manifest-path …/ui-grammar-consumer/Cargo.toml` 7/7；`verify-deps.sh` PASS（闭包无 auto-lang/auto-man/auto-vm/auto-ui）；806 declarations consumer 同 PASS |
| 777 | `python scripts/modularity-audit.py --mode strict --no-cargo-check` → **verdict=pass violations=0**（grandfathered=2 为既有） |
| 792 | `python test/fixtures/modularity/generated-ui-consumer/driver.py` **EXIT=0 全绿**（V01 fresh 真实 build/V02 A→B 重定位 locked/V03 全负控红/V04 dev 模式与逐字节同源）——交付闭包携带迁移后 auto-ui-dialect 源；本计划未增删闭包 crate/依赖（auto-ui-dialect 依赖面不变），受管 source 声明无增量 |

## 5. merge-owned 门（pending，最终冻结提交统一执行）

- 裸 `cargo t`（全日常面）+ `cargo tv`（语料三族）——必跑。
- `cargo tu`（UI 代码生成四生成器金样——完整语法对生成产物实际触面，
  计划 §6 明确最终统一跑）。
- `cargo tt` 仅当 trans 源被触（本计划未触 trans/，预期不适用）。
- tf 批量回归按主检出到期判定，非本计划门。

## 6. 遗留与勘误

- contract-baseline §1 勘误：`expect_ident` 最终**留 Core** 作具名端口能力
  （35 调用位单一实现，优于 806 的本地复刻形态）；§2 表已按实际实现
  微调（probe_next_token 失败上抛语义、advance_peek_kind 具名化、
  parse_ident/expect_eos 不入端口——迁移区零调用）。
- 未发现迁移闭包内无界 EOF 怪癖（全部定向面/交叉样例绿）；如 review
  发现，按合同隔离进程+外部截止证基线后再处置。
- mint 族事件参数解构 `events,` 未用字段 lint 为基面原样（见 §4）。
- View/On TokenKind 兼容基础 AST 保留（R10 范围）；`scan_use_statements`
  字符串级扫描留语言所有者（未迁）；UI 生成器/Man/renderer 未动
  （R04/R08 范围）。

## 7. R1 修复轮（F810-R1-01..06，2026-10-19）

复审 needs_fix（attachments/810/review-r1/REVIEW.md）后同树修复，代码提交
`55ce34200`（F01/03/04/06）→ `0f2995e5f`（F05 SD 重写 + 777 基线刷新）→
`9226c8af9`（toy consumer 端口面补齐）：

| Finding | 修复与证据 |
|---|---|
| F810-R1-01 快照生命周期 | `raw_cursor_release` 端口（Drop 语义 tombstone+尾部收缩）；computed 成功臂/style recipe 承诺后释放。`r01_snapshot_lifecycle_bounded_all_entry_paths`（公开门面 object/body、Core 直连 store、pub style、派发，n=1/8/64 存活恒 0）+ `r01_nested_release_keeps_outer_credential`（外层凭据内层解析后仍兑现） |
| F810-R1-02 777 基线 | 干净树 `--refresh-baseline`（绑定新 HEAD，declared_edges=63）→ strict `verdict=pass violations=0`；self-test 正反控 **15/15**；docs 提交后复验仍绿 |
| F810-R1-03 测试 oracle | None 回退真反例（`style = 1`→Bina 表达式+后续 msg 照常+零诊断）；双 session（Core msg=确定性 Err=隔离证据）；store 护栏（on 体让行 + fn 体**基线怪癖**如实断言——gate 只查 in_on_body，不修）；凭据跨上下文/重复拒绝；owned AST 源码释放；shadow；computed 空表达式确定性 Err；**三层 use-import 链**（grandchild 状态 7→8 经传递注册）。V-05 代表矩阵（work 期定向重放，全量 tu 归 merge）：VM=plan370/plan339/plan340/plan051+vm_bridge inline_lambda 双测；Vue=plan024/plan367/plan408/native_css+vue.rs inline/dialog_vmodel；Rust=rust.rs test_simple_counter×2/timer_block/image_surface/inline_lambda；Ark=ark/generator inline_lambda；Jet=jet/generator inline_lambda——全部定向绿。**导入再导入**=v04_import_of_import_chain |
| F810-R1-04 scene prev | `advance_peek_kind_resync`（scene/subresource 旧臂双 push+next 重同步，prev=被观察 token）与 `advance_peek_kind`（stream 手动恢复，prev=advance 前 cur）**区分**；`r04_scene_gate_prev_matches_old_two_push`（prev==6 + 旧臂手工重放对照）+ scene 让行/subresource/stream 代表对照 |
| F810-R1-05 SD 冲突 | overview 两处/parser-api/dialect-system/ui-dialect（parse_on_events 归属）/full-grammar（两前瞻形态+release 生命周期）按现行单一事实重写，历史另述 |
| F810-R1-06 触面卫生 | rustfmt edition 2021（crate 实际 edition；复审的 --edition 2024 无法解析 `gen` 标识符）触面全净；parser.rs `}impl` 拼接与 4+ 连空行残渣修复；新增 warning 清零（仅余 widget.rs `events` 解构 lint——复审已认定旧臂同款迁移，非新增） |

**R1 期间发现的基线怪癖（如实冻结，不修）**：① UI 场景 fn 体内
`store.x = 1` 确定性 Err（store 直连门只查 in_on_body 不查 in_fn_body，
gate 未迁出零改动）；② 未闭合 fragment（`if x {` 后 EOF）无界 EOF 栈溢出
（fail-fast 0xc0000409，verbatim 迁移同文本同行为——806 spec 在案的
msg-block 同族怪癖；不进进程内测试，需要时按合同隔离进程+外部截止）。

修复后回归：plan810 **20/20**、plan806 18、plan408 20、plan778 4、
native_css 7、plan370 18、plan367 6、plan024 16、scene 9、recipe 19、
轻 consumer 7/7、verify-deps PASS、777 strict pass+负控 15/15、fmt 净。
merge-owned 门不变（handoff §5 pending）。

## 8. 下一步

- [x] 792 driver 收据回填（§4：EXIT=0 全绿）
- [x] R1 needs_fix 六 findings 同树修复（§7）
- [ ] 独立 final review 第二轮（T-09，同树，绑 R1 修复后提交）
- [ ] merge：冻结提交统一 §5 门 → 合入 → 沉淀 → 归档（T-10/T-11）
