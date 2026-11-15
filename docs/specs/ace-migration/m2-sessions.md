# ace-migration: m2-sessions — REPL持久会话daemon与在线更新

> Status: delivered (M2-A slice; PLAN-970, reviewed commit `295e93f7cf5d5aaa40c87278bf2f66b04752e6d0`,
> final review pass 2026-11-16). B-phase product/CLI migration (old-repl/serve/mcp
> commands in auto-man) stays with PLAN-985/987; the legacy
> `autovm_persistent`/`autovm_daemon`/`autovm_client`/`mcp::McpServer`
> surfaces are untouched until PLAN-987 retirement.

## 已验接口（Verified support）

| 面 | 接口/事实 | 来源 |
|---|---|---|
| 中立会话契约 | `auto_lang_core::ace_session`：`AceSessionHost` 注册表（单属主）、`ace_session_create/op/list/cleanup_expired`、`AceSessionOp{Eval,Edit,Inspect,Vars,Snapshot,Reset,Close,EvalAsync,Result,CancelEval}`、`AceSessionReport{revision,value,vars,snapshot,rebuilt,…}`；未注册host＝具名`ace.session/host-unregistered`，永不回退旧引擎 | `crates/auto-lang-core/src/ace_session.rs` |
| 会话协议身份 | `ac.session.daemon/1`（`SESSION_PROTOCOL_NAME/VERSION`）；请求缺`version`视为1，异版本＝具名`version/mismatch`列支持版本 | 同上＋`ace_daemon.rs` |
| 参考会话宿主 | `auto_lang_ace_host::session_bridge`：冻结声明集通道模型（`main`dispatch＋`__repl_eval_int`/`__repl_eval_bool`＋`__tail_anchor`锚；函数增量域为封闭声明集、仅body快速路径——顶层增删具名拒）；`Engine::{Direct,Resident,Taken}`单属主线程；`install_session_host[_with_store]` | `crates/auto-lang-ace-host/src/session_bridge.rs` |
| 同PID在线更新 | `FunctionIncrementalSession::prepare_edit/commit`（原子单发布、跨会话/陈旧base/关闭具名拒）＋`ResidentRunner`安全点提交（在途帧钉旧代、混代值）＋`ResidentRunner::into_session`（PLAN-970 additive：链终态/取消后交还Direct，二次async可再arm） | `session_bridge.rs`＋`experimental/ace/crates/av/src/resident.rs` |
| 变量持久 | 会话携带表（int/bool标量域，cap 64）＋prelude注入＋RHS纯捕获探针；非携带类型＝具名`session.var/non-carryable`边界 | `session_bridge.rs` |
| daemon | `AceDaemon`：JSONL over named pipe/unix socket＋stdio；旧7方法名保留（new-session/eval/inspect/reset/delete/snapshot/list/ping）＋新（edit/eval-async/result/cancel/close/vars）；幂等＝携带id请求的响应缓存（64条FIFO）逐字重放，无id请求不入缓存；坏行具名错误后继续；stdio EOF优雅关停、pipe逐连接EOF守护常驻；inspect/snapshot源文本入`value`（wire侧edit偏移可得）——`value`字段为多义（eval值/源文本/`"running"`标记） | `crates/auto-lang-core/src/ace_daemon.rs` |
| client | `AceSessionClient`：持久连接、10×200ms重连重试、30s收包超时、单调请求id | `crates/auto-lang-core/src/ace_client.rs` |
| MCP | `mcp::ace_server::AceMcpServer`：initialize按规范响应己方版本`2024-11-05`（不忽略客户端版本）；旧7工具名保留＋`auto_edit`/`auto_vars` | `crates/auto-lang-core/src/mcp/ace_server.rs` |
| 持久化 | `ac.session.state/1` op-log（eval/edit/reset）记录：shutdown flush（tmp+rename原子替换）、安装时op重放重验恢复（非反序列化Checked）、legacy `.at`快照按源显式重放、未知schema具名跳过且原文件不动、Close/过期删record；会话id掺pid+纳秒盐＋create碰撞跳过（恢复id不可静默覆盖） | `session_bridge.rs` |

## 边界（现值开放项）

- 会话求值域＝scalar profile int/bool（f64/i64/str/char/unit与表达式语句在此profile外＝具名拒；`return (E)`尾表达式重写、显式`return`语句求值、语句+sink三形态包装）。text/console/collections域不并入（各session域保留，1024 F05裁定）。
- 同步eval的cancel为入口快照（长同步eval由wall约束；中途取消＝async面）；Edit偏移以`Inspect`原源为基（捕获探针可能暂驻lane body，`Snapshot`为用户视图）。
- MCP/daemon的auto-man CLI接线（old-repl/serve/mcp命令换ACE）归PLAN-985/B；旧实现退役归987。
- 函数增量域约束（封闭声明集/最后区域编辑不收敛）为产品约束：条目定义输入走显式reshape重建（`rebuilt=true`用户可见）。

## 验证入口（permanent fixtures）

`crates/auto-lang-ace-host/tests/t970_sessions.rs`（12例：B01 14→17→20、隔离、变量携带、bool通道、reshape+保留名、错误后继续、异步混代、取消/关闭、重置、cap、快照、编辑负控）；`t970_consumers.rs`（3例：真实named pipe wire往返（wire侧偏移edit＋cancel→二次async→跑完）、MCP工具面、持久化记录三阶段）；`auto-lang-core`单测`p970_*`（14例：注册表纪律、协议版本/幂等/坏行、MCP协商/工具表）；T-01冻结probe P1..P10（38检查）证据已折入计划§5/§8。覆盖组：`ace-facade`、`ace-av`、`auto-lang-core`、`integration`。
