# PLAN-781 T-14 有限执行预算决策报告（合同冻结）

stage=work | task=T-14 | plan_revision=2 | 阶段 C（T-15~18）以本文为硬前置。
基线：worktree `D:/autostack/.wt/lang-781/auto-lang`，branch `plan-781-dev`，阶段 B 收口于 `292e8755a`。
探针：D:/p781-t14/{probe_budget.py,out/*.json}；CLI = worktree 重建 `auto.exe`（sha256 见 probe_report.json）。

## 0. 调查基础与现状定量（真实 whole-path，非估算）

### 现有机制（源码勘定）

- **驱动档**（engine.rs `drive_handler_segment`）：`DriveBudget::Legacy` = 每次驱动独立 **10M 指令**硬编码；`DriveBudget::CpuSlice`（UI 专用）= 每片 1M 指令/4ms 墙钟 + **跨片累计 10M 护栏**（`task.cpu_steps_total` 计）。源码自注：**「会 park 的 handler 每段重新计，比同步更宽」**——park/resume 段间预算重置，正是不变量 2 要消除的形态（当前 legacy 档 `cpu_steps_total` 恒 0）。
- **HTTP 入口三态**（http_server.rs）：① stdnet 串行同步 `call_fn_by_name`（:3903，非默认调用图保留位）；② Axum 段派生命名 `call_fn_by_name_segment`（:5369）；③ Axum closure `call_closure_segment`（:5367）。②③硬编码 `DriveBudget::Legacy`。SSE 生成器 next 走临时任务 `shim_iterator_next`（:3932 族）。
- **耗尽可观察**：`500 {"error":"internal server error","detail":"RuntimeError(\"instruction budget exhausted in '<fn>' at ip=… (…legacy budget 10M steps)\")"}` + stderr `[HTTP] POST … → 500` 行——**可识别资源拒绝**（非解析错/空成功/无返回假成功），v1 冻结沿用此形态。
- **Reader 导入算法面**（只读 consumer 实勘）：`import_book_json` = 头 64KB envelope 检查 → 全文 `read_text` 长度对账（第 2 次全文解码）→ `ph1` 哈希 → 去重 `content_equal` 逐字节比对 → `split("\n")` + 逐行 `cur_body + "\n" + raw` 拼接 → JSON 序列化落盘。**全程同步 CPU、零 park（fs.read_text 原生调用不 park）→ 单段单预算**。

### whole-path 探针定量（真实 Reader API /api/library/import，隔离 split-VM 实例，逐尺寸全新实例+隔离库）

| 尺寸 | 结果 | 墙钟 | 峰值 WS | 备注 |
|---|---|---|---|---|
| 256KiB | **成功**（`code:ok` + book_id） | 1.18s | ~206MB | 基线含 VM 自身 ~190MB |
| 512KiB | 预算耗尽 500 | 1.27s | ~192MB | 边界锐利 |
| 1MiB | 预算耗尽 500 | 1.4s | ~189MB | |
| 2/4/8MiB | 预算耗尽 500 | 1.5-1.9s | 213-291MB | |
| **10MiB TXT**（707 章） | **预算耗尽 500** | 1.6s | **295.7MB** | 目标档位 |
| **10MiB MD**（707 章） | **预算耗尽 500** | 1.7s | 294.5MB | 与 TXT 同界（分章路径共用） |

**换算**：成功/失败边界 ∈ (256KiB, 512KiB] → **20-40 指令/字节**；10MiB 需要 **≈210-420M 指令**；耗尽速率实测 ≈5-7M 指令/s（字符串重负载）→ 10MiB 成功路径预估 **35-70s 单线程忙时**、内存增量 ≈+90MB（10MiB 内容 ≈9× 放大，落 tf 轻池预算内）。内容漂移结论：非预算墙外的第二堵墙（256KiB 成功路径完整走通 decode/hash/落盘）。

## 1. 冻结决策

### D-1 policy 形态：可配置累计 Legacy 预算（不引入 HTTP CpuSlice）

- **v1 = 环境变量 `AUTO_HTTP_STEP_BUDGET`**（宿主进程启动期读取，u64 十进制）——**宿主可信配置**，.at 业务代码不可写、不入 pac.at（pac 是 app 声明面，非宿主面）。每进程各自读取，天然实例隔离（V-14 双实例不串线）。
- **语义 = 单 HTTP 请求 dispatch 生命周期的累计指令上限**：跨段（park/resume）、跨嵌套 fn 累计，不重置（不变量 2）；达到上限 = 既有「instruction budget exhausted」资源拒绝。**默认不设 = 10,000,000（逐字节保持现状）**。
- **合法域**：`1 ..= 4_000_000_000`（u64 十进制；0/负号/小数点/非数字/溢出 u64/超上界 = 非法）。非法 = **启动期指名配置错误 + 非零退出**（fail-fast，先于任何业务 dispatch——不变量 1「dispatch 前拒绝」取最严格解释：不进 serve 循环）。上界 4G 依据：10MiB 最坏 420M 的 ≥9× 余量 + u32 便携 + 防 u64 溢出类笔误；导入推荐档 **1_000_000_000**（最坏 420M 的 ≥2.4× 余量）。
- **为何不采 HTTP CpuSlice（v1 非目标）**：① 导入路径零 park（纯 CPU 单段），片化不改总量只加分页开销；② HTTP 侧 Runnable 恢复泵需改 http_server 全部 step 循环（6288/6402/6460 三处）——与在途 PLAN-776 写面正撞；③ 单 worker 线程持 35-70s 对多线程 axum 后端可接受（桌面 reader 档）。冻结为 v1 非目标；若未来公平性证据成立（多路由饥饿），经新计划引入。

### D-2 入口覆盖矩阵（V-20 面）

| 入口 | 消费 policy | 说明 |
|---|---|---|
| Axum 段派发·命名（`call_fn_by_name_segment`） | ✅ 累计 | 默认调用图主入口 |
| Axum 段派发·closure（`call_closure_segment`） | ✅ 累计 | Plan 383 fn-ref 路由 |
| stdnet 串行同步（`call_fn_by_name` @:3903） | ✅ 累计 | 非默认调用图保留位 |
| 段恢复（`resume_fn_by_name_segment` 及 http step 循环 6288/6402/6460） | ✅ 累计延续 | 不变量 2 的落点 |
| SSE 生成器 next（`shim_iterator_next` 临时任务） | ❌ v1 维持默认 10M/帧 | 流式帧每帧自界；业务级流预算另行计划 |
| UI bridge（vm_bridge CpuSlice 泵） | ❌ 不变 | PLAN-711 既有契约，UI 轨累计护栏独立 |
| back_proxy / run_module_init 同步驱动 | ❌ 不变默认 | 非 HTTP 请求面 |

### D-3 deadline/取消/终态

- **零新机制**：deadline/取消/断连/服务关闭沿现有 http request scope（PLAN-707 资源组绑定 + 既有超时路径）；**一次请求一个终态**——预算耗尽终态与既有 500 形态同源，不产生迟到成功。
- 预算耗尽后的任务回收沿既有 remove_task/cleanup_legacy_files 路径（探针实证：耗尽后实例继续服务后续请求，无泄漏性挂死）。

### D-4 计数实现面（T-15 接线合同）

- 载体 = `AutoTask` 既有 `cpu_steps_total` 字段扩展到 Legacy 档：段驱动每段结束把本段 steps 计入任务累计；段启动预算 = `min(policy, 10M_默认段帽)` 检查改 `累计 + 本段 > policy → 耗尽终态`。任务生命周期 = 请求 handler task（spawn→RET/终态→remove_task），天然请求级隔离。
- 配置解析：auto-man 启动链读 env → 逐 VM 实例快照（每实例一份，运行期不可变）；非法值 → 指名错误 + 非零退出（先于 serve）。
- **写面**：engine.rs（drive_handler_segment 预算源）+ http_server.rs（三入口传 policy）+ auto-man 启动链（env 解析）——**全部与 PLAN-776 在途修复正撞**（其 worktree 现持 engine.rs/http_server.rs 未提交改动，R7 needs_fix reopened T-01~08，next=work）。**T-15/T-16 必须等 776 合入后开工**（本计划 §4 纪律：不并行改共享热点）。

## 2. 负例与边界（V-14/15/16/18/19 判据）

- **V-14**：同机双实例（A: 1G 档 / B: 默认）——A 导 10MiB 成功、B 同文件预算耗尽；A/B 库互不可见（隔离 data 目录）；B 的 10M 默认边界不变（256KiB 成功/512KiB 耗尽复核锚点）。
- **V-15**：`AUTO_HTTP_STEP_BUDGET=0/-5/1.5/abc/99999999999999999999/5000000000` 各一进程 → 启动非零 + 指名该 env 的错误行；无请求被服务。
- **V-16**：探针级计量——`budget=N` 可控实例跑已标定负载（512KiB≈10-20M 步），N-1 耗尽/N 过/N+1 余量；park/resume 累计（handler 内置一次 HTTP await 后继续——同请求两段合计不重置）。
- **V-18**：断连/服务关闭沿用既有 scope 回收——预算路径零新状态机，以既有 http 生命周期测试面回归。
- **V-19**：非空库 + 低预算（10MiB 输入，N=1M）：耗尽 500、库索引 bytes 不变、无 managed 半写副本（既有 cleanup 路径断言）、无虚假成功条目。

## 3. 探针可重放命令

```
python D:/p781-t14/probe_budget.py            # 全尺寸 bisect（需 AUTO_BIN/worktree auto.exe）
# 单点：gen_book 10MiB → auto run -r vm --no-merge -B <free> （AUTO_READER_DATA=隔离目录）
#        → POST /api/library/import {"path":...,"author":"p781","force":false}
#        → 观测 500 detail("instruction budget exhausted") / stderr [HTTP] 行 / 峰值 WS
```

## 4. 修订记录

- r1（2026-10-14，T-14 初冻）。
