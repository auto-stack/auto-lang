# PLAN-744 r2 重绑定证据（2026-10-14，work 续做会话）

工作记录 1 残留项（VM 冲突编排采证 + MCP 存活疑问）的闭环证据，全部为本会话在
当前 master 上的全新执行。

## 运行身份

- AutoLang 源：master docs-only 线（HEAD 会话内 284acbfb4 → 884a2876e 之间浮动，
  `crates/`+`stdlib/` 与 924273fa5 的 sqlite 状态逐字节一致——`git diff --stat
  284acbfb4..HEAD -- crates/ stdlib/` 为空）。
- CLI 二进制：`D:/autostack/auto-lang/target/debug/auto.exe`
  （`cargo build --features ui-iced --bin auto` 本会话全新构建）
  SHA-256 `6e30c87f5c3eca157973ff6e2d6e8da9ddc850b4a276e7579244f0000f3524c5`。
- Notes 应用：auto-notes main @1391bd7（split mode，`pac.at api:"rust"`）。
  仓内既存未跟踪 `src/front/tests/` 原样保留未动。
- 运行目录（gitignored，本附件为关键产物拷贝）：
  `D:/autostack/auto-notes/.auto/{r5-vm-1791936993144365500, p744-suite-1791938367827736100, …}`。

## AC#1 — sqlite 单测（scoped）

`cargo test -p auto-lang --lib sqlite`：

```
test vm::ffi::stdlib::sqlite_vm_tests::sqlite_vm_handles_are_vm_local_and_released_with_request_task ... ok
test vm::ffi::stdlib::sqlite_vm_tests::sqlite_vm_open_exec_query_roundtrip ... ok
test result: ok. 2 passed; 0 failed; 1 ignored(cookbook_vm_tests::cb_database_sqlite, needs sqlite——预存 ignore)
```

含 924273fa5（Plan 775）新增的任务生命周期句柄测试。

## AC#2 — VM 19 场景存量套件（净库单轮）

驱动：`auto.exe run -r vm -B <port>` + `python tests/run_autotest.py
015-notes.autotest --mode vm`（AUTOUI_MCP_PORT 私有端口隔离；MCP 78s 就绪——
rust 后端冷构建可超 r5 脚本的 90s 窗，本会话用放宽驱动重跑）。

**19 passed / 0 failed / 0 skipped，suite_exit=0**（场景逐项见
[vm-suite-scenarios.log](vm-suite-scenarios.log)，收据 [vm-suite-result.json](vm-suite-result.json)）。

## AC#3 — VM 冲突编排负例（净库单轮，真实 MCP + 外部写者 + fault 注入）

脚本：notes 仓 `tests/probe/evidence/r5-work/run_vm_negative.py`（NOTES-001 r5
交付，本会话原样重跑）。**13/13 [PASS]，EXIT=0，fails=[]**（收据
[vm-negative-result.json](vm-negative-result.json)）：

- R5-VM-1：外部写者推进后保存 → conflict=true 且 dirty 保留；导航入恢复槽 A；
  Recover 显示旧暂存（非自覆盖）；当前草稿保全于槽 B；槽 B 可再恢复。
- R5-VM-2：陈旧保存被拒（dirty+conflict）；Discard 后显示最新服务器标题
  （revision 跟进）。
- R5-VM-3：after_manifest fault 下提交已落服务器但 UI 保持 dirty；继续编辑的
  新意图经 revision 冲突裁决采纳服务器 revision 后落地；同内容重试幂等。

## 纯 VM 后端轨（--server vm）探测——新发现（744 范围外，已登记 P744-D1）

- `auto run -r vm --server vm`：后端 AutoVM HTTP 起服（26 路由，api.at 模块
  扁平化），**VM sqlite shims 实际执行**——fresh 目录下 inbox.db 落盘
  notes/requests/meta 三表 + meta 行（schema_v=3 / seeded=1），即 `db.exec`/
  `db.query` 方法形在 run 模式可链接可执行（见
  [vmserver-probe-boot-tail.log](vmserver-probe-boot-tail.log)）。请求处理被
  fs 奇偶缺陷阻断：`RuntimeError("copy_recursive failed: os error 2")` → 500
  （legacy 升级门）。
- `auto service --server vm`：链接失败 `Undefined symbol: SqliteDb.exec in
  module p2probe`（service 每-模块解析不含 Type.method 形映射；run 模式扁平化
  有——故上述 run 探测能建表。收据 [service-vm-link-failure.log](service-vm-link-failure.log)）。
- `fs.copy_recursive` 语义两轨分裂：`stdlib/auto/fs.at:58` 声明
  `fn copy_recursive(src str, dst str) void`（#[vm]，失败抛 RuntimeError），
  a2r 轨返回 bool——依赖 bool 语义的 .at 代码（notes repository.at fs_copy）
  在 VM 轨被 500。最小复现：VM 脚本对缺失源调 copy_recursive →
  `RuntimeError(copy_recursive failed: os error 3/2)`。
- 附带疑点（未隔离）：后端 VM handler 上下文中 `fs.exists` 疑似对不存在
  manifest.json 误报 true（fresh 目录 meta 恒缺 `upgraded` 行 + copy 500 佐证；
  独立脚本 VM 探针 exists 语义正确）——需源级插桩定位，与上面两项同族登记。
