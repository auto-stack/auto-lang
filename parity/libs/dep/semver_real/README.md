# semver_real — 真三方 semver 动态加载绿面语料（PLAN-594）

动态加载真实 crates.io semver（类型面走 VM native_catalog）。

- 上游版本：semver 1.0.26（dep 声明与 oracle Cargo.toml 同版；锁定勿改版重跑）。
- 绿面：`Version.parse(..).unwrap()` + `.major/.minor/.patch`（u64 字段直读，
  数值比较形态；PLAN-594 T3/T9 探针实证）。
- 红面（登记 DIV-DEP-8+）：`print(v)`（VM=`<semver::Version>` 占位 vs a2r=Display
  "1.2.3"）、`VersionReq`/`Comparator` 面、`bump_*`（&mut self 接收者）。
  详见命中率报告。
