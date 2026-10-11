# serde_json_real — 真三方 serde_json 动态加载绿面语料（PLAN-594）

与 flat 复刻库 `libs/serde_json`（手写复刻）区分：本目录经
`dep serde_json(version) + use.rs` 动态加载**真实 crates.io serde_json**
（三轨：VM methods pack / a2r 转译 / 手写 Rust oracle）。

- 上游版本：serde_json 1.0.145（dep 声明与 oracle Cargo.toml 同版；锁定后勿改版重跑，
  见 KNOWN-DEBT P592-D1 wrapper 缓存键弱失效）。
- 绿面：`from_str` + Auto 类型标注 `let data Value` + `.unwrap()`（VM native 面与
  a2r 发射双侧实证，PLAN-594 T3 探针）；`print(data)` 直出为紧凑 JSON，三轨逐字节
  一致（Display；比较器只对齐 TAP 行，该行作附加输出人工核对）。
- 红面（不进 TAP，登记 DIV-DEP-8+）：`data.to(str)`（VM=紧凑 JSON vs a2r=Debug）、
  `let n i64 = from_str(..)`（VM 窄槽 4294967295 vs a2r 42）、`is_err()`（VM 静默
  None vs a2r true）、`to_string(42)`（VM String 无 to_string / a2r 缺 & 借用）。
  详见 docs/plans/reports/p594-dep-skip-hit-rate.md。
