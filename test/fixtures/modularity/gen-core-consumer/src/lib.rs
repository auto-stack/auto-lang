//! PLAN-791 外部生成核心消费方探针。
//!
//! 本 crate 无产品代码——全部断言在 `tests/gen_core.rs`（V01 反控集：
//! fake renderer 真实落文件 / guard 保留 / dry_run 零写入 / 失败不覆写 /
//! legacy feed）。构建/门禁形态见同目录 Cargo.toml 头注与 verify-deps.sh。
