# Plan 798：初始化边界核查与热态重测

2026-10-20；work 定向测量，不是 R16 最终独立验收。用户明确保持 P95<1000ms，工程/开发环境初始化和第一次结果在计时外。

## 核查结论

最近 formal 的 edit 980.1ms / sig 1200.4ms 是跨进程请求计时，包含每次启动 worker、打开存储，以及父端实际身份/产物校验等开销。工具本身的首次构建在样本外，但这些重复初始化在样本内。因此不能用这些数字直接判定用户所定义的稳定环境热态周转失败。旧日志保留为跨进程端到端证据；先前单凭 sig 膨胀归因为环境的结论不足。

并非全部历史测量都混入相同初始化：既有 release `bench-turnaround` 的编辑/签名场景已经在同一 Driver 内，先打开存储、完成 base 请求，再计样本。本次实际重新执行该入口，没有从旧数字扣减估算开销。

## 冻结与执行

- worktree：`D:/autostack/.wt/lang-798/auto-lang`；分支 `plan-798-dev`。
- 前后 HEAD：`dbae2e634d2f65ac20c34221c28813ef2de6d2ee`，前后 clean；代码基线 `8e78502ba`，本次无代码修改。
- release 构建：`cargo build --manifest-path experimental/ac-av-probe/Cargo.toml --release -p auto-av-probe --bin av_probe_worker --offline --locked`。
- 执行：`experimental/ac-av-probe/target/release/av_probe_worker.exe bench-turnaround --samples 30 --gate-ms 1000 --out D:/autostack/auto-lang/docs/plans/attachments/798/warm-remeasure-20261020/raw.jsonl`。
- 2026-10-20 16:51:51–16:56:07（北京时间，含构建和全部六场景）；一次运行，exit 0。负载原样保存，没有挑窗重试。

## 结果

固定负载仍为单文件 273575 字节、115 个函数。每个场景 30 样本，六场景共 180 行，全部正确。

| 测量路径 | P50 ms | P95 ms | P99 ms |
|---|---:|---:|---:|
| 同进程预热后，连续修改函数体 | 834.647 | **874.5993** | 899.9341 |
| 同进程预热后，签名冲突诊断 | 35.694 | **51.1369** | 68.1972 |

编辑样本有 30 个不同源码哈希和 revision，30 次均实际执行并得到预期结果。签名路径是同一冲突输入重复 30 次，均返回 `frontend:source.argument`；不声称覆盖 30 种不同签名编辑。独立从 raw 重算分位数，采用现有 `round(p*(N-1))` 约定，与 summary 一致。A 前端瞬态 201.6133MiB、B 探针残差 5.5352MiB，均在当前 r5 预算内。

## 计时边界与状态

`bench.rs` 的两个场景在循环外创建 Driver、打开 store 并执行首个 base 请求。循环内调用 `Driver::request`，其内部请求时钟包含源码指纹、实际前端检查、缓存/依赖处理、必要 MIR/ABC 验证加载和 AV 执行/诊断等当前实现工作；没有逐样本进程启动或 store-open。输入生成在请求计时外。

Driver 常驻的是存储和项目上下文，尚未持有完整常驻 VM 的运行状态；每次请求仍执行当前完整必要编译/加载工作。本次证明**已实现热态请求链路在该固定负载、该次运行达到 P95<1s**，不是完整在线 VM 安装、活动帧迁移或编辑器到屏幕刷新延迟的验收。

原 r5 AC-06/V-11/T-07 仍有正式 adapter 计时要求。本次没有暗改合同、没有以独立 bench 替代正式 runner，也不据此复勾 T-07 或授 execution_done。后续须按用户澄清统一正式性能门的生命周期和计时边界，将跨进程启动指标单列；必要实际 IPC/排队/结果发布仍需计入，不能简单改名或扣减。旧 formal 失败不再作为用户热态目标失败的直接依据。

原始数据见 [raw](raw.jsonl)、[summary](summary.json.txt)、[独立重算](audit.json.txt)、[提交与退出状态](result.json.txt)、[输入哈希](baseline.json.txt)、[二进制身份](binary.json.txt)、[负载](load-during.jsonl)。merge-owned 整档与最终独立 review 保持 pending。

内存归因字段更正（同日架构核查）：B按实际门的采样总峰值217210880B减前端211406848B，为5804032B=5.5352MiB。先前6.3203MiB是端点private-bytes增量，另列信息项；延迟原始数据与各门PASS不变。
