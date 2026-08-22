# Plan 416: LSP/VSCode Phase 5-6 收尾（243 TS 迁移 + semantic tokens + CI）

> **状态**: 📋 已立项待实施（2026-08-22,源自审计 §5.2 B7 拆粒度;暂缓原因"需 VSCode 侧联调"仍成立——本计划明确联调环节的最低配置）
> **来源**: Plan 243 Phase 1-4 已 ✅(references/rename/code_action/signature_help/inlay_hint 真实实现,71acecc4);本计划只做 Phase 5/6 剩余

---

## 1. Phase 5 剩余(编辑器现代化)

### 5-A extension.js → TypeScript 迁移(预估 2 天)

- **现状**: `editors/vscode/extension.js` 单文件 ~309 行纯 JS,无类型。
- **拆解**:
  1. `src/extension.ts` + tsconfig(esbuild 打包回 dist/)——保持行为零变化
     的纯迁移,先不上新功能
  2. 语言客户端类型化(`vscode-languageclient` 官方类型)
  3. package.json main 指向 dist,vsce 打包验证
- **验收**: F5 扩展宿主里原有全部命令/激活行为不变;`npm run compile`
  (tsc --noEmit)零错。

### 5-B semantic tokens(预估 3 天,需 LSP 侧配合)

- **现状**: 只有 TextMate grammar(auto.tmLanguage.json,已覆盖 await/widget
  等);无 semantic token provider——变量/函数/类型同色。
- **拆解**:
  1. LSP server(`crates/auto-lsp`)实现 `textDocument/semanticTokens/full`:
     从现有符号表(Phase 1-4 的 workspace.rs 已有 references 数据源)分派
     tokenType(variable/function/type/keyword/enumMember)
  2. extension 侧注册 legend + SemanticTokensProviderStub
  3. **VSCode 联调点**(本计划唯一的实机环节):装扩展 → .at 文件验证
     着色正确、增量编辑不闪烁;最低配置 = 本机 VSCode + F5,无需发布
- **验收**: 对 `examples/ui/013-todo/src/front/*.at` 截图对比 TextMate-only
  前后差异;LSP 侧单测锁 token 序列。

### 5-C 生成关键字/类型/函数列表 + lsp-api-contract.md(预估 1 天)

- 243 Phase 6 遗留:从 parser 关键字表 + stdlib 目录生成补全数据源;
  `docs/lsp-api-contract.md` 固化 LSP 方法与能力矩阵。

## 2. Phase 6 剩余(持续维护基建)

### 6-A CI push 触发恢复(预估 0.5 天)

- **现状**: `.github/workflows/lsp-ci.yml` 仅 `workflow_dispatch`(push
  常红被禁)。恢复 push 触发前先修红:本地 `cargo test -p auto-lsp` +
  集成测试全绿为门槛;CI 加 `paths` 过滤(crontab 只盯 lsp 目录)。

### 6-B 集成测试扩容(预估 2 天)

- **现状**: 集成测试 70 行低覆盖。补:didOpen/didChange 增量、
  multi-file workspace、rename 边界(跨文件重命名)。
- **验收**: 覆盖率报告(llvm-cov)显示 lsp crate ≥60% 行覆盖,或明确
  记录剩余未覆盖模块的理由。

## 3. 执行顺序与联调安排

5-A(纯迁移,无联调)→ 6-A(CI 解红)→ 5-B(唯一需 VSCode 实机)→
5-C → 6-B。全部可在无 GUI CI 环境推进,仅 5-B 最后一步需要桌面 VSCode
会话(约半天)。分支:`plan-fix/416-<id>`。
