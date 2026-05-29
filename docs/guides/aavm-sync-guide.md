# AAVM ↔ Rust 同步操作手册

## 触发条件

以下任一情况触发同步：
- Rust 侧新增了语言特性（新语法、新 OpCode、新类型）
- AAVM 需要支持新的测试场景
- 距上次同步超过 2 个月
- 手动决定需要追赶

## 操作步骤

### 1. 检查差异

```bash
# 逐文件检查 Rust 侧的变更
cd d:/autostack/auto-lang
git log --oneline fe666a6f..HEAD -- crates/auto-lang/src/vm/engine.rs crates/auto-lang/src/vm/opcode.rs
git log --oneline ddbb161a..HEAD -- crates/auto-lang/src/parser.rs
git log --oneline 7cc484b1..HEAD -- crates/auto-lang/src/trans/rust.rs
git log --oneline fe666a6f..HEAD -- crates/auto-lang/src/eval.rs crates/auto-lang/src/interpreter/
git log --oneline fe666a6f..HEAD -- crates/auto-lang/src/lexer.rs
git log --oneline fe666a6f..HEAD -- crates/auto-lang/src/type_inference/
git log --oneline fe666a6f..HEAD -- crates/auto-lang/src/vm/generic_registry.rs
```

Baseline commit 记录在各 .at 文件头部的 `AAVM Sync Snapshot` 注释中。

### 2. 评估变更

对每个组件的变更，判断：
- **需要同步**：新语言特性、新 OpCode、bug 修复、行为变更
- **不需要同步**：UI/debugger/FFI/数据库/多文件支持（AAVM 暂不追踪）

### 3. 实施同步

对需要同步的组件：
1. 阅读 Rust 侧的新增代码
2. 用 Auto 语法重写对应功能到 `auto/lib/*.at`
3. 添加 bootstrap 测试验证
4. 更新文件头部的 Baseline commit 和 Coverage/Missing

### 4. 验证

```bash
# 运行所有 bootstrap 测试
cargo test -p auto-lang --lib -- test_aavm_99_bootstrap --include-ignored

# 运行 a2r 测试确认无回归
cargo test -p auto-lang --lib -- a2r_tests
```

## Claude 提示词模板

需要同步时，发送以下提示词：

```
请执行 AAVM 同步检查。

1. 读取每个 auto/lib/*.at 文件头部的 AAVM Sync Snapshot，提取各组件的 Baseline commit
2. 对每个组件运行 `git log --oneline <baseline>..HEAD -- <rust-ref-paths>` 检查变更
3. 列出每个组件的新增 commit，标注哪些是 AAVM 需要同步的（排除 UI/debugger/FFI/DB/多文件）
4. 对需要同步的变更，生成实施计划
5. 更新完成后，更新各文件头部的 Baseline 和 Coverage
```
