# PLAN-755 独立复审 R2（2026-10-10）

- stage: review；plan_id: PLAN-755；plan_revision: 1；outcome: **needs_fix**。
- reviewed_commit: `4ce6455167fd55cee3514789f5d461daf08cebe2`；base_commit: `d6e4819ba686298238237ae6f473da77b22478bf`；3 提交、6 文件 +485/-8。
- dependency_revisions: auto-down=`895f8d0f9355c9f5ec3ce8fca268bdb768395846`；Cargo.lock SHA256=`30fdffc6c8d8642b765340ff1238a4e84bcfbf119428e3ed7617579408c5ec06`。
- spec_inputs: 主检出 canonical 与工作树 SD-01 的哈希、完整 patch 见 baseline；SD blob=`383d2dabeca541647be1444022b31b9bef63e0fd`。
- 独立性：本会话未实施 755；从计划、提交差异和实际测试重建结论。本会话曾复查 751，不宣称从未见过相关问题。
- 工作树起止均干净。只在 cfg(test) 夹具临时注入标记 panic 并添加负向验收探针，结束逐字节恢复；不修改实现、不合并、不发布 canonical 或账本。现有工作树保留供执行者修复。

## 验收映射

| AC / tasks | 本轮结果 | 证据与边界 |
|---|---|---|
| AC-01 / T-02 | **fail** | 原 Err 诊断的历史时间线有明确 connect 阶段；新独立负向测试证明 panic 原因没有通过结果通道转发，P755-R2-01 |
| AC-02 / T-01,T-03 | pass（当前守卫覆盖） | 两个正式守卫 2/2，0.223s；逐 hunk 核实尾部独立契约声明与泛型方法变体；13 个无 shim 项按已登记边界不冒称契约可证 |
| AC-03 / T-03,T-06 | pass | 完整 diff 仅声明侧、测试与 SD；严格门、CORE_MODULES、Unsupported、stdlib.rs、http.at 均无改动；原 wire 断言未删弱 |
| AC-04 / T-04 | pass（当前 SHA） | 原上传、重定向 2/2，分别 0.596s / 0.629s，串行，真实启动与原 wire 断言 |
| AC-05 / T-05 | 历史全档收据保留；修后待重绑 | 102/101/1 红/0 超时的首轮与同 SHA 二次采样记录在案；本轮未重跑全档，不将它称为本轮独立全档 pass |
| AC-06 / T-06 | **fail** | 恢复原文件后 rustfmt 四文件检查 exit 1，P755-R2-02；日常档/tv 历史记录保留，确定失败后不重复扩大测试 |
| AC-07 / T-07 | 元数据纠正；修后待定稿 | 已存在的规范文档增补属于 modify，不是新组件；SD 冻结，未发布；P755-R2-03 |

## P755-R2-01（P2）：panic 结果通道丢失原始载荷

三处新增分支均为 `Err(panic) => format!("... panicked: {panic:?}")`。`catch_unwind` 的载荷是 `Box<dyn Any + Send>`，这样格式化只得到 `Any { .. }`，没有提取字符串 panic 信息。

独立复现直接使用已提交的 plan730 `start_server` 助手；只把测试线程的 run 调用临时替换为带标记的 panic（助手的 catch、格式化、通道、ready 逻辑保持原样），再检查父线程报告是否保留标记：

```text
FAIL [2.153s] review755_panic_payload_is_forwarded
子线程默认 hook: REVIEW755_ORIGINAL_PANIC_PAYLOAD
父线程失败: test server failed to start on port 2500: crate::run panicked: Any { .. }
Summary [2.195s] 1 test run: 0 passed, 1 failed
exit 100; restored_exact_bytes=true
```

默认 panic hook 仍打印原始消息，不把它误述为整个日志完全没有消息；缺陷是设计要求的**载荷经通道回传与启动失败信息**没有保留原因，不能依赖未重定向的子线程 hook 来补齐该合同。正常 Err 分支与已恢复的 HTTP wire 结果不受本发现否定。

修复：三处助手统一提取 `&str` / `String` payload，未知类型给明确兜底；补真实助手的负向验证，确认父线程启动失败信息携带原因。落点：plan730_http_upload_tests.rs:274、http_server.rs:1789/1846。不要只改报告或断言。

## P755-R2-02（P2）：已提交树格式不通过

在探针逐字节恢复、工作树 clean 的真实 reviewed_commit 上执行：

```text
rustfmt --edition 2021 --check --config skip_children=true \
  crates/auto-lang/src/vm/native.rs \
  crates/auto-lang/src/tests/plan755_http_contract_tests.rs \
  crates/auto-lang/src/tests/plan730_http_upload_tests.rs \
  crates/auto-lang/src/vm/ffi/http_server.rs
exit 1
```

四文件均出现格式 diff；例如 native.rs:394 两个契约声明拼在同一行。此前“fmt 干净”不覆盖该提交工件，AC-06 不能签 pass。修复后提交格式变化并对新 SHA 检查，日志已完整保存。

## P755-R2-03（P2，簿记已纠正）：既有规范误分类为 new

`docs/specs/stdlib/design/assembly-manifest.md` 在基面已存在，SD-01 为 modify。此前 frontmatter 以“新增小节”理由填 `new_spec_components`，会给 merge 错误沉淀分类。现按技能纠正为 supersedes 中的该既有精确路径、new=[]；affects 列精确源码/规范路径。不改规范语义、不增加 plan_revision；历史错误记录保留并由本记录覆盖当前路由结论。

## 证据与下一步

baseline、四项日志、panic 探针、SD patch 均在本报告同目录持久保存，哈希绑定；不依赖工作树内临时日志存续。

**755 返回 executing**：T-02/T-06 必修；T-04/T-05/T-07 的最终证据在修后重新绑定。保留当前契约与真实业务通过记录，P751-D1 仍是闭合候选，尚不最终销案。无需新计划，保持严格门；修复提交后复验 panic/Err/readiness、原两反例、串行 th、格式及按改动触面要求的日常/tv 门禁，再复审与 merge。

本次不重跑 tf/taa/tu/docs_gen，也不动主检出他方 WIP。整项 pass 被具名反例否定，不以同会话自审声明作为阻塞理由。
