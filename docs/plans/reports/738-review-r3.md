# PLAN-738 独立复审 R3 证据（2026-10-09，needs_fix）

- reviewed_commit：`b3a4d660e722cf9f1a81bb45bac9982548ce09c9`（worktree `D:/autostack/.wt/lang-738/auto-lang`，clean，只读）
- base：Phase 3 diff base `2c1b4a763`；merge-base(master, plan-738-dev)=`6d69dbdc7`；auto-down 兄弟 `895f8d0f` 只读
- 复审者未参与任何实现；全部结论由可复现工件重构（命令附下）

## 1. 门禁复跑（全部在 b3a4d660e 最终树，串行）

| 门禁 | 命令 | 结果 |
|---|---|---|
| 三 crate check | `cargo check -p auto-lang -p auto-man -p auto` | 零 error；api_gen/rust_ui 非 test 面 warning 均预存（Plan 061 `back_dir` 等在 2c1b4a763 同在） |
| 裸 t 全档 | `cargo nextest run -p auto-lang --lib --test schema_drift --test docs_gen --test component_registry_test --features ui-iced --no-fail-fast` | 5183 run：5166 pass / 17 fail；17=13 master 预存（musk p053/p054×6、desktop_protocol、iced×2、ash_leak、schema_drift×2、docs_gen）+4 flake（plan502/plan707_client/plan606_029 + plan484_024——后者隔离复跑 `cargo nextest run -p auto-lang --lib plan484_024` 2/2 绿，T-10 报告文档化 flake 族） |
| 语料档 | `cargo tv` | 162/162 |
| 转译档 | `cargo nextest run -p auto-lang --lib --features test-trans --no-fail-fast` | 5554 run：5541 pass / 13 fail——全部 ⊂ 裸 t 红名单（非基线红=0） |
| HTTP 档（缩减） | `cargo nextest run -p auto-lang --lib --features test-http-e2e -E 'test(back_proxy) \| test(plan730) \| test(vm_multipart) \| test(e2e_sse_chain) \| test(http_e2e_plan724) \| test(shell_pack) \| test(plan738) \| test(health_ready) \| test(back_provision) \| test(dep_fields)' --no-fail-fast` | 142 run：127 pass / 2 fail / 13 timeout——back_proxy corpora（R2 在档基线红）、plan730 interop FAIL@109s + 13 个 plan730 族级联 TIMEOUT（与执行报告 th 分诊同构；执行者临时 worktree @2c1b4a763 同命令复现在案；738 的 8 文件 diff 不触 plan730 路径；738 自身服务链同环境 1/1 绿；本机另有 musk-100 worktree `cargo run` 长驻服务 22:15 起在跑——固定端口族环境归因佐证）；e2e_sse_chain 本轮绿 |
| plan738 族 | `cargo t plan738` | 69/69 |
| plan724 族 | `cargo t plan724` | 5/5 |
| host strict 族 | `cargo nextest run -p auto-lang --lib plan738_host` | 10/10 |
| t12 双指纹 | `cargo nextest run -p auto-lang --lib t12_manifest_identity` | 2/2（triangle 14.3s 实跑） |
| CLI 档 | `cargo test -p auto --bin auto stdlib -- --test-threads=1` | 10/10 |
| api_gen | `cargo test -p auto-man --lib api_gen -- --test-threads=1` | 43/43（1 ignored=⑤腿） |
| freshness | `cargo test -p auto-man --lib freshness -- --test-threads=1` | 2 pass / 1 fail——**计数假象见 P738-R3-01**；test_shell_pack_lib_freshness 红与执行报告一致（stash 实证环境红在案） |
| Rust 实编 witness | `cargo test -p auto-lang --lib --features test-trans rust_host_provider_real_compile_witness -- --ignored --test-threads=1` | 1/1 |
| C stdio MSVC witness | `cargo test -p auto-lang --lib c_stdio_provider_bindings_compile_and_read_real_file -- --ignored --test-threads=1` | 1/1 |
| 服务完整链 | `cargo test -p auto-man --lib --features test-http-e2e http_e2e_plan738 -- --ignored --test-threads=1` | 1/1 @42.26s（生成→schema4+workspace_lock 收据→实编→serve→ready 指纹→业务 42→改 stdlib→陈旧→再生→复验） |

## 2. 独立探针

1. **plan724 改判理由独立证实**：`crates/a2r-std/src/http.rs:172` `pub fn post(url, body) -> Response`（2 参）；内嵌 `crates/auto-lang/src/a2r_std.rs:912` `pub async fn post(url, body, api_key) -> (i32,String,String,String)`（3 参）。Standalone 无 3 参 post producer——诚实拒绝改判成立。
2. **CLI 冒烟**（`target/debug/auto.exe`）：inventory → exit 3（status=partial、files_total=115、相对源 ID、verification 分级）；`--module auto.http --target vm --check` → exit 1；`--target wasm` → exit 2 + error JSON；exit 0=EXIT_OK 断言在 `crates/auto/src/cmd_stdlib.rs:512`（CLI 测试 10/10）。退出码与 SD-06 终稿一致。
3. **Resolved-only 残留 grep**：`Proof::Resolved`/`Resolved` 生产代码仅 `validate.rs:157`（inventory 验证等级枚举合法使用）与注释；`wrapped`/`mode_variant` 在 host.rs 零匹配——放行臂确认删除。
4. **ADAPTER_RULES 计数**（逐提交实测）：`c1579ed71`=12 条、`84bec29ef`/`b6cfc6df1`/`b3a4d660e`=15 条——T-10 报告"14 条"、T-11 报告"增至 17 条"均为计数失真（P738-R3-02）。

## 3. T-09 抽查（独立复验，不复用执行者输出）

- scope.json 分类计数 566 = 552 pure_format + 4 comment/whitespace + 2 golden + 8 functional ✓。
- 4 个 pure_format 文件独立执行「父 blob → `rustfmt --edition 2021 --emit stdout` → 与子 blob cmp」全部逐字节 MATCH（p025_scroll_render_tests / plan712_http_error_semantics_tests / plan442_musk_backend_probe_tests / bigvm_generic_integration_tests）。
- `lib.rs`（comment_or_whitespace_only 类）实际差异为 use/mod 语句重排+格式化——Rust 顶层 use/mod 顺序语义无关，"非语义变更"结论成立（分类命名不精确，P738-R3-04 观察）。

## 4. 健康扫描

- Phase 3 diff（`git diff 2c1b4a763..b3a4d660e`）零 `dbg!`/debug 残留。
- 触面 8 文件 `rustfmt --edition 2021 --check` 干净（exit 0）。
- **新增未处理编译器 warning**（P738-R3-01）：`cargo test -p auto-man --lib freshness` 编译输出 `warning: function assembly_freshness_truth_table is never used`（rust_ui.rs:5037）。
- P738-D1/P738-D2 在 KNOWN-DEBT-AND-RISKS.md 263/264 行在案（既有登记，非新延期）。

## 5. Findings 摘要

| ID | 严重度 | 摘要 |
|---|---|---|
| P738-R3-01 | major（必修） | `rust_ui.rs::tests::assembly_freshness_truth_table`（T-06 交付的真值表测试）自 `b6cfc6df1` 起丢失 `#[test]` 属性（新测试 `lock_freshness_truth_table` 插入时截走了属性，自身带重复 `#[test]` 被注册两次）——该函数现为死代码（dead_code warning 在案），T-06 回归保护丢失；执行报告"freshness 2/2"实为同一测试跑两遍的假象 |
| P738-R3-02 | minor（勘误） | ADAPTER_RULES 计数失真：T-10 报告 14 条（实际 12）、T-11 报告 17 条（实际 15） |
| P738-R3-03 | 观察（非阻塞） | 伴生 back 模块（fsys.at 等）逐字内容不在任何新鲜度面（`generation_source_snapshot` 只含 api.at+db.at，681/734 既有边界）；738 经 Embedded 引用证明实际扩大了可检测面，非回归 |
| P738-R3-04 | 观察（非阻塞） | T-09 的"comment_or_whitespace_only"4 文件实含 use/mod 语句重排（语义等价），分类命名不精确、结论成立 |
