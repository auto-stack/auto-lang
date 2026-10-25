# PLAN-832 R2 残余 findings 修复闭环

2026-10-25。范围仅为 R2 仍开放的 F832-R1-03/04；F01/F02 保持 CLOSED，不机械重开。连续两次 final needs_fix 的[覆盖诊断](diagnosis.md)已先于实施记录并用于本轮 oracle 调整。

## 冻结身份

- Man：`f52cd93b09401f358a89a19058fbf1fa7673fd16`。
- lang 生产源：`87f82d637ecd586c17a01c1c975e1456bc346991` 之后无生产 diff；本目录仅证据/Spec/交接。
- UI=`aee499e87093ff727f68ac8467c506e9ecf6ecdc`；down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- 原 R2 基线 Man=`ebfc88f8e66da05d192c88c11a8a946e8b5bbc7f`，lang=`87f82d637ecd586c17a01c1c975e1456bc346991`。

## Closure

| Finding | Invariant / affected states | Fix and permanent oracle | Current result / gap |
|---|---|---|---|
| F832-R1-03 | 普通开发 CLI 无 Cargo/AUTO 环境仍按 owner 发现；显式覆盖、安装邻接、缺件拒绝保持 | `discovery` 从实际 `current_exe` 的 `target/debug|release` 与 repo marker 推导 Man 同组 UI owner；无任意主检出搜索。CLI 永久测试删除继承的 `CARGO_MANIFEST_DIR`/AUTO 发现变量，且不再先解析再注入工具 | [直接生产七格](discovery-results.json.txt)：clean/explicit/manifest/UI-root/installed 全真实生成；missing-tool/missing-root 非零且零产物。[无注入 CI 1/1](ci-gen-no-injection.log.txt)。无剩余 gap |
| F832-R1-04 | 真实 backup 按文档恢复后 verify 且旧命令可运行；严格额外文件保护保持 | 文档把 owned `.man-backup.marker` 移到具名保存位置后再 verify，不忽略陌生文件。`man-distribution.py` FLR-6 永久加入真实 upgrade→backup→文档恢复→命令 | [Core 实际往返](restore-results.json.txt)：640 verify=0、Unicode 空格脚本 `restore-r2-ok`、stray extra exit2、移除后复绿。[full 永久 driver](permanent-restore.log.txt) PASS。自动回滚/installer 未改，无剩余 gap |

## 检查到不变量映射

- `dev_discovery.py`：R2 同一合法 `src/front` 输入，逐格删除旧 `gen/`，检查真实 CLI exit 与 `App.vue`，不再由父进程提供工具路径。
- `cargo test ... gen_runs_through_real_tool_process -- --exact`：永久测试自身执行无注入生产发现。
- `restore_verify.py`：真实 producer 生成带 marker 的 Core backup，按文档恢复，验证旧命令和陌生额外文件负控。
- `documented_restore` 永久 driver：full profile 同一共用机制的独立往返通过。
- `cargo build --locked -p auto`、`rustfmt --check`、Python AST、`git diff --check` 全过；编译仅有已归因上游/迁移警告。

未运行整档、六 profile、完整 14 项或无关 Core/owner 重放。统一门仍归 merge；下一责任角色为 T-10 受影响独立复审。
