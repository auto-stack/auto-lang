# PLAN-798 AutoDown sources/oracles for the ac-av probe（T-07 正式接线）

D-796/D-788 已合入：本目录用例经统一 runner（TestCase v2）的 probe/run 接缝
由 **`ac-av/core-run-v1`** recipe 正式选择/执行（T-07 落地）。worker
（`av_probe_worker case-serve`，envelope `avworker/1`）只接收声明式 Atom 参数
与 source 引用，不读取 expected；父 runner 校验 actual 来源/工具身份（协议/
recipe/exe-sha256/worker-version 四轴入报告与 stale 判定）并判定 oracle。
Rust 定向覆盖（`experimental/ace/crates/av/tests/**`）继续并行保留。

## suite 说明

- recipe：`ac-av/core-run-v1`（T-07 冻结；单 `run` 步 + `av` 绑定 entry/caps，
  与 sema/query-v1 同合同追加，不改共享协议）。
- oracle 形态：success+`exit`（AV Success 值）、error-run+message（trap code）、
  error-parse/verify+code（诊断拒绝）——均为独立预期，非两后端互证。
