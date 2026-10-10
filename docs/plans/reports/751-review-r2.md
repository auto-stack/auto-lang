# PLAN-751 落地补充复验 R2 与 HTTP 分诊纠正（2026-10-10）

- stage: review；plan_id: PLAN-751；plan_revision: 1。
- outcome: **needs_fix（HTTP 能力后续修复/分诊证据；空 lock 修复单项 pass）**。不回退 738/751 的 archived 终态，不在本次复查中修改实现，也不把 HTTP 整体视为已修好。
- reviewed_commit: `d79282eb6d671ebd279fd823b48be4cba9e592ea`（实际 landing）；实现 `226ebfbbb637ade69e8d6b38ee05f400e9110452`；SD `ba9b4281b`。
- diff_base: `226ebfbbb^`；执行者旧基面 `0884add2aead438010bf20bca3c8aae08b896961`。依赖 auto-down=`895f8d0f9355c9f5ec3ce8fca268bdb768395846`；根 Cargo.lock/规范版本哈希见 baseline。
- 独立性：本上下文曾发现 R11，未实施/复审/合入 751；现在从真实工件与新反例重建结论。不是全新上下文全项目审计。
- 验证在独立 detached worktree 进行，生产源未改；仅临时 cfg(test) 探针，结束逐字节恢复。主检出 UI WIP 未触碰。

## 1. P738-R11-01 已闭合

逐 hunk 核证：生成端与复用门共同调用 `workspace_lock_identity`；`Ok(bytes)`（包括零字节）统一 FNV 身份；仅 NotFound→absent；其他错误生成端上抛、复用门保守陈旧。一次绑定与绑定失败拒绝逻辑保持。旧 `classify_lock_read` 已删除。

**复跑原 R11 真实生成/消费探针**，不是只信执行者新测试：

```text
read-error generator+unbound+bound+recovery PASS
empty-lock first_fresh=true first_receipt="811c9dc5"
           regenerated_receipt="811c9dc5" regenerated_fresh=true
test result: ok. 1 passed; 0 failed; ... finished in 1.55s
```

该反例此前在 738 落地树失败，现 cargo exit 0；证明空文件一次绑定、同空文件再生成、生成端读错误拒绝、未绑定/已绑定读错误和恢复可读均成立。树内 `review751_empty_lock_identity_matrix` 六步断言与修复相符。SD-01 三态澄清符合当前行为，无降低 fail-closed 规范。

正式生成服务全链复验结果：**1/1 pass，361.01s（含生成 crate 的冷编译），cargo exit 0**。最初编译因同机分页空间不足（os error 1455）中断；资源恢复后降并发重试，原夹具因硬编码工作树本地 target 不存在失败。第三轮仅把 cfg(test) 缓存路径临时适配为实际 `CARGO_TARGET_DIR`；全部生成、实编、ready、业务、stdlib-only 失效及再生断言保持。所有夹具修改最终恢复，不将前两次环境失败归为代码回归，也不冒称原夹具未适配就通过。

## 2. P751-R2-01（P1）：HTTP 公共能力仍被严格门拒绝

本轮使用 **原 plan730 的完整 upload_program** 和 **原 plan326 重定向测试的相同源字符串**，分别经过真实 `crate::run`（与原 start_server 相同入口）及 `create_vm_from_source`，不是静态名表推断。得到：

```text
actual crate::run: Err(...SIGNATURE_UNVERIFIED: http.upload_receive ... native #9937...)
compile: SIGNATURE_UNVERIFIED: http.upload_receive ... native #9937...
actual crate::run: Err(...SIGNATURE_UNVERIFIED: http.response_redirect ... native #3108...)
compile: SIGNATURE_UNVERIFIED: http.response_redirect ... native #3108...
test result: FAILED. 0 passed; 1 failed; ... finished in 0.10s
```

**确定的拒绝路径**：`Codegen` 核心 native 校验（codegen.rs:12643..12649）→ `NativeInterface::production()` → `verify_core_reference` → `reference.rs:184` 缺独立 adapter contract，返回错误。两个 shim 实际存在（stdlib.rs 注册），但 production 最终元数据没有相应独立合同；不是没有 HTTP 业务实现，也不是 751 的 lock 改动引入。错误发生在业务/服务器启动之前。

修复要求：在专用后续修复计划中审计 HTTP 公共 native 的**最终活 callee/独立签名/适配 ABI**，补齐真实 producer 契约并正确安排覆盖后声明次序；至少复验 upload_receive、response_redirect，再展开其余上传/Response 家族。保留严格门；不得以跳过核心校验、把已有能力改判 Unsupported 或修改成功断言来清绿。修复后必须验证真实服务启动、HTTP wire 行为和完整串行 th。

这是 P751-D1 的更直接调查入口，也是原 738「既有 HTTP 能力保持」的后续修复项；不要求把无关 HTTP 实现强塞回已归档的 751。

## 3. P751-R2-02（P2）：HTTP 收据将候选机制写成了已确证机制

`751-th-full-receipt.md` 的 102 selected / 86 pass / 2 fail / 14 timeout 与逐名表计数一致；它证明有执行报告，**不证明 HTTP 全绿或已修好**。本轮没有再跑完整 32.7 分钟全档，原 120 秒现象保留为历史观测。

但「服务端 accept 后不响应，60s×2 read timeout」缺中间证据，且无法统一解释源码：

- 重定向测试 `http_get` 实际为 **5 秒** read timeout；plan730 `read_response` 在首次 read 返回错误后按 0 字节退出并 panic，并不是明确的“两次 60 秒读取”路径。
- 两个 start_server 都是 `let _ = crate::run(&code)`，丢弃服务启动错误；ready 轮询结束也不检查成功。当前相同源在真实 run 入口 **0.10 秒内明确返回契约错误**，不能将它写成已成功启动的服务器。
- `http_server.rs` 最后被修改是归因候选；R2 对该文件的主要差异为格式，不能靠 last-touch 定因。`git log -S verify_core_reference` 显示编译器严格门引入锚点为 `d82eb02eb`，是更相关的候选；本轮尚未做旧树 good/bad 对拍，**不签署精确引入提交**。

**原测试另行复跑**：在未修改的落地树上，以 nextest 串行选中 `http_e2e_plan730_vm_plain_endpoint_untouched`，再次 **1 run / 0 pass / 1 timeout，120.046s，exit 100**，仅 nil 弃用警告。与 repo-root 的有界真实 run 诊断快返错误是两种已实测形态；本轮没有接通两者差异（进程 CWD、入口初始化/清理等须显式冻结观测），不能宣称缺契约已经解释全部超时。完整全档未再跑，但该原单项的确定性超时独立成立。

后续必须先让测试助手有界报告 `crate::run` 的 Err/panic/readiness，而不是把启动失败藏到客户端等待中；再用原测试和相同提交/依赖/stdlib 根追踪 120 秒等待的实际位置。未接通前，超时原因保持未确定，不能判负载 flake 或只凭固定端口归因。

## 4. 验收边界与下一步

| 751 验收 | 本轮结论 |
|---|---|
| AC-01/02 | pass：原 R11 探针转绿，空文件身份和一次绑定成立 |
| AC-03 | pass：生成端、未绑定/已绑定门读错误和恢复实证 |
| AC-04 | pass：生产调用点共用分类，旧函数移除，SD-01 一致 |
| AC-05 | partial：102 项计数历史报告在树；具体挂死机制须按新增证据纠正 |
| AC-06 | 两文件格式实查通过；执行者日常/auto-man 全档为历史证据，本轮不外推新全档 pass |

本轮不跑 tf/taa/tu/docs_gen，不在确定性缺陷出现后重复全库门禁。冻结规范与探针日志在 baseline 中；没有发布 canonical 或改账本。

收尾簿记另有可直接纠正项：计划缺 frontmatter revision、已有 spec 误填 new_component 且缺 docs/specs 前缀、归档后 current_step 仍 5/6；按原合同与已完成 merge 收据修正，不改变 AC。

批量回归 **DUE** 仍成立（750 落地触发）。截至 2026-10-10 16:08 CST，距旧收据 2026-10-08 11:20 UTC 约 44h48m，不能用「>48h」作依据；当时 753 仅批准/执行中。主检出仍有 020 示例 WIP，regress 要求主检出除簿记外干净，因此仍待 owner 收口；本轮未动 WIP、未擅跑批量档。

next：738/751 保持 archived；另立 HTTP 契约与启动错误专项修复合同，以本轮具名反例为入口，补齐 public provider 契约和有界启动诊断，再完成逐项真实 HTTP/完整 th 复验。空 lock 修复不需再次重开。
