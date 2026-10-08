# stdlib 装配同源样例（PLAN-738）

一个模块名（`jsonx`），公共契约 + 三个目标层，演示 738 的装配语义：

- **VM 目标**（`auto run -r vm`，默认）：公共层 + 选定 `.vm.at` 层合并——
  `val()` 执行 VM 层 body（142）；公共 pure body `aux()` 共享（7）。
- **Rust 目标**（`auto run`）：公共层进类型上下文；发射走 a2r-std crate
  宿主 provider（`.rs.at` 是 candidate，不消费）。
- **C 目标**：同理，`.c.at` 为 candidate（C 发射走预生成产物）。

装配事实可机器核对：

```
auto stdlib inspect --actual examples/stdlib/assembly/witness.at --format json
# actual manifest：jsonx 的 context_file=jsonx.vm.at（VM 目标选定层），
# candidate_files 含 .rs.at/.c.at（未消费层只能报 candidate）。
```

同源 witness 的真编译实跑门：

```
cargo test -p auto-lang --lib plan738 -- --ignored
```

未在此样例展示的语义（见 docs/plans/738 计划 §5）：增/删/改选定层使缓存
失效；活 session 换目标/换 stdlib root 报 session_target_mismatch 须重建；
Browser 环境监听/本地 FS/native socket 三族 Unsupported。
