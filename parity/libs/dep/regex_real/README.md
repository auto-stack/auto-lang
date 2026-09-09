# regex_real — 真三方 regex 动态加载绿面语料（PLAN-594）

与 flat 复刻库 `libs/regex` 区分：本目录动态加载真实 crates.io regex
（VM methods pack 类型面走 native_catalog，自由函数/构造器走 pack）。

- 上游版本：regex 1.11.3（dep 声明与 oracle Cargo.toml 同版；锁定勿改版重跑）。
- 绿面：`Regex.new(..).unwrap()` + `is_match`（bool，TAP 可断言；PLAN-594 T3 探针实证）。
- 红面（登记 DIV-DEP-8+）：`find`/`captures`（Match/Captures 句柄导航）、迭代器面、
  replace 系（&str/Cow 借用返回）。详见命中率报告。
