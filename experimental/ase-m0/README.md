# ASE M0 — 应用解析与无窗口 AV 加载实验（PLAN-1021）

有界一次执行研究：严格解析两个已注册 `app://` 身份（`auto.demo.a` / `auto.demo.b`），
在允许根内读取已登记源闭包，经 **公开** ACE 包链
（`compile_package_product` → `PackageAvTarget::cold` → `run_value`）一次冷执行并真实返回
11/22；A 编辑为 12 不影响 B；坏源按解析/编译/执行/资源分类拒绝，修复后可重试。

边界：无 UI/后台服务/沙箱/安装器/持久会话/磁盘零编译缓存/warm body-edit；本目录是
独立 workspace（不加入语言/ACE workspace），只消费 ACE 公开 crate，不改 ACE 私有执行。
URI 不是执行任意文件的授权——只有显式注册的本地可信程序可加载。根包含检查是有界根
检查，不称完整沙箱。

CLI（实验入口）：

```
cargo run --manifest-path experimental/ase-m0/Cargo.toml -- \
  --app app://auto.demo.a/ --artifact-root <dir> [--registry <dir>] [--version <v>]
```

`--registry` 默认为随包 fixtures 目录。测试：`cargo test --manifest-path
experimental/ase-m0/Cargo.toml --test headless`（固定组 `cargo t ase-m0`）。
