# Plan 857 R3 身份绑定修复交接

基线 `4d9fa8e8c4ad00f5c88d9d1c5be619f0c592b837`；代码提交 `81b0974ac2512126790b0ef2af79235edf45b485`；工作树 `D:/autostack/.wt/lang-857/auto-lang`，分支 `plan-857-dev`。本轮只修改验证工具及来源清单。此前复审的交互、性能和原始兼容证据依照本轮仅 Python 工具与清单变更复用，不视为重新执行了完整门。[入口身份矩阵](../repair-r3-entry.md) 与 [R3诊断](../../review-r3/diagnosis-r3.md) 在此收口，最终结论仍需独立复审。

- **F-857-R3-01**：锁收据必须恰好是 `autographics-auto-driver/Cargo.lock` 与 `autographics-prototype/Cargo.lock`；路径规范化后检查重复、别名和外来键，比较实际文件原始哈希、两份收据及856/857源码清单。两个实际锁 SHA-256 分别为 `2b131d3dd33a968b43d53664cb8649d6c24225fa0f2c28c1add603be98a358db`、`c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf`。双侧同错 README/Cargo.toml、双侧同错值、漏项和别名在真实比较器均非零且无成功摘要；单侧错值、JSON重复键通过专门来源控制拒绝。
- **F-857-R3-02**：受控构建从干净已提交树、固定 SDK/锁/配置和 Cargo 实际产物记录建立证明；比较器重新校验构建提交、产物、证明、当前源码和收据。不具证明的旧程序只标 `unverified`，不写入伪造的构建提交，实际旧 Home 报告也被比较器拒绝。跨提交复用须明确开启、核源码差异限于验证工具和清单，且保留真实构建提交；不声称这些本地收据具有数字签名或能抵御全面伪造。
- **原反例重现**：修复前复审脚本实际重放，双侧伪锁与旧程序重标均 exit 0，脚本按预期报失败；记录 `before-replay.log.txt`。本轮真实当前构建的双侧伪锁以及真实旧 Home 无证明输入均分别按正确原因拒绝。
- **真实当前构建正控**：干净提交在全新外部 target 上执行 `python experimental/autographics-auto-driver/tools/build_runner_proof.py --output D:/autostack/plan-artifacts/plan-857/repair-r3/final-controlled-build`，exit 0；实际输出 runner SHA `50d8e51bcd1c7fc92c90f5215f16fbfcacc144efdcb0d9b9f0dd3fe8e8a2e2d7`，构建提交和验证提交均为 `81b0974ac2512126790b0ef2af79235edf45b485`，来源清单 SHA `bd255025c495219aee746c79752519fe451f602ca1ff8dacb0d0f75fd6ab94af`。真实3D normal + partial 窗口通过；实际856冻结基线、当前默认与 Home RGB 逐像素正控 exit 0。比较器12项具名负控全部 exit 1，详见 [当前构建控制](current-controls.json.txt)。
- **真实合法复用及拒绝控制**：在修改前干净的 `4d9fa8e8c4ad00f5c88d9d1c5be619f0c592b837` 树运行 `cargo build --locked --offline --bin autographics-authoring --target-dir D:/autostack/plan-artifacts/plan-857/repair-r3/baseline-build-target`，核前后状态并留构建日志；随后以同 target 捕获 Cargo JSON 事件。其构建上下文收据依据当时保存的构建前后检查和记录整理，而非声称是签名证明。实际 runner SHA `1627acb2ed78dacab1e1bc0b5595a9ec0785b274d5d46439f26cff2d65166786`。使用 `--build-proof .../baseline-build.json --allow-reuse` 实际运行窗口并比较，保留旧构建提交 `4d9fa8e8c4ad00f5c88d9d1c5be619f0c592b837` 与新验证提交 `81b0974ac2512126790b0ef2af79235edf45b485`，复用范围仅 `3d-default-pixels-only`；同样12负控均非零及实际正控通过，见 [复用控制](reuse-controls.json.txt)。未开启 `--allow-reuse` 时真实比较器 exit 1 `cross-commit runner reuse requires explicit approval`；明确开启但改变 RUSTFLAGS 时 exit 1 `build configuration mismatch`，均不产生成功摘要。
- **额外来源与相邻路径**：`test_runner_provenance.py --output .../provenance-final2 --old-runner .../bc18c3aeab1ab107caebc431ecc040281fbf0f9ef3672e8a03b9ff7871ceef5e/autographics-authoring.exe --baseline-proof .../baseline-build.json` exit 0，12条结果包含真实旧 Home 无证明、旧程序与其他构建证明混配、构建提交伪改、未授权复用以及显式复用正控。固定 source verifier 480 项、SDK 681 文件验证、Python编译和 m2 自检通过。`check_editor_viewport2d.py .../current-2d --executable .../final-controlled-build/target/debug/autographics-authoring.exe --controls` 通过两次实际OS/GPU运行及变换/内容/像素检查，使用当前构建证明。

## 可重跑命令与环境

以下命令从工作树根目录执行；统一设置 `RUST_MIN_STACK=16777216`、`RUSTFLAGS=-C link-arg=/STACK:33554432`、`RUSTC_WRAPPER` 为空。`A=D:/autostack/plan-artifacts/plan-857/repair-r3`，`B=D:/autostack/plan-artifacts/plan-856/merge-r1/viewport2d/runner.json`；重跑时所有输出目录须换成新目录。

1. `python experimental/autographics-auto-driver/tools/check_editor_viewport3d.py A/current-native --executable A/final-controlled-build/target/debug/autographics-authoring.exe --build-proof A/final-controlled-build/build-proof.json --controls` → 0；旧 runner 用 `A/baseline-build-target/debug/autographics-authoring.exe --build-proof A/baseline-build.json --allow-reuse` → 0。
2. `python experimental/autographics-auto-driver/tools/test_default_3d_baseline.py --baseline-runner B --reset-dir A/current-native --commit 81b0974ac2512126790b0ef2af79235edf45b485 --output A/current-controls` → 0；对 `A/reuse-native` 增加 `--allow-reuse` → 0。
3. `python experimental/autographics-auto-driver/tools/check_default_3d_baseline.py --baseline-runner B --reset-dir A/reuse-native --commit 81b0974ac2512126790b0ef2af79235edf45b485 --output A/reuse-without-approval` → 1；增加 `--allow-reuse` 并将 RUSTFLAGS 栈值改为 `33554433` → 1。

`A`/`B` 在命令中是说明用路径缩写，执行时替换成完整路径。[证据索引](evidence.json.txt) 记录各原始文件绝对路径和 SHA-256，含构建输出、真实窗口、两个比较矩阵及失败日志；正控与负控输入是实际窗口报告/截图，非人为伪造的成功报告。`provenance-final` 第一次因调用时缩写了旧程序目录而 exit 1，随后以完整目录在 `provenance-final2` 成功；不计作通过。

工作修复结束，等待对新提交独立复审；状态未 reviewed、未合入。完整双端、受影响 AGP、原新 native/owner/content 与其他 merge-owned 门仍 pending；本轮未运行整档。主检出其他会话 WIP 和风险登记均保留。
