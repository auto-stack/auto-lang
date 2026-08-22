# Plan 416: LSP/VSCode Phase 5-6 收尾（243 TS 迁移 + semantic tokens + CI）

> **状态**: 📋 已立项待实施（2026-08-22,源自审计 §5.2 B7 拆粒度;暂缓原因"需 VSCode 侧联调"仍成立——本计划明确联调环节的最低配置）
> **来源**: Plan 243 Phase 1-4 已 ✅(references/rename/code_action/signature_help/inlay_hint 真实实现,71acecc4);本计划只做 Phase 5/6 剩余

---

## 1. Phase 5 剩余(编辑器现代化)

### 5-A extension.js → TypeScript 迁移 ✅ 2026-08-22 完成(auto-vscode `fc4cb9d`)

- `src/extension.ts` 全量类型化移植(309 行 JS → strict TS;languageclient
  v9 走 /node 子入口);webpack → esbuild 构建链;`npm run typecheck`
  (tsc --noEmit)零错;宿主外 require 冒烟——新旧 bundle 在同一 vscode-stub
  桩点以同一方式失败(等价性证据);真实扩展宿主 F5 联调归 5-B 批次。

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

### 5-C 生成关键字/类型/函数列表 + lsp-api-contract.md ✅ 2026-08-22 完成

- `Token::all_keywords()`(lexer 权威关键字表,56 词;and/or 显式 None、
  grid/route 注释臂剔除,round-trip 单测锁定)→ keyword_completions 并入
  (精选 snippet 优先);`stdlib_index.rs` 惰性解析 stdlib/auto/<mod>.at
  (fn 签名 + type 声明)→ `json.`/`fs.` 模块成员补全 + 类型位模块名补全;
  `docs/lsp-api-contract.md` 能力矩阵/补全数据源/传输约定/已知边界。
  测试:stdlib 索引 + complete() 端到端(json 成员/yield 关键字)×2。

## 2. Phase 6 剩余(持续维护基建)

### 6-A CI push 触发恢复 ✅ 2026-08-22 完成

- 常红根因定位:fmt 作业检查**全仓**(9000+ 文件非 rustfmt-clean)。
  修复:auto-lsp 包 `cargo fmt -p` 清零(测试 8/0 保持);fmt 作业收缩为
  包级;push/pull_request 触发恢复并加 `paths: crates/auto-lsp/**` 过滤。
  全仓 fmt 属独立决策(登记 Plan 416 后续 6-C)。

### 6-B 集成测试扩容(预估 2 天)

- **现状**: 集成测试 70 行低覆盖。补:didOpen/didChange 增量、
  multi-file workspace、rename 边界(跨文件重命名)。
- **验收**: 覆盖率报告(llvm-cov)显示 lsp crate ≥60% 行覆盖,或明确
  记录剩余未覆盖模块的理由。

## 3. 执行顺序与联调安排

5-A ✅ → 6-A ✅ → 5-C ✅ → 5-B(唯一需 VSCode 实机)→ 6-B →(6-C
全仓 rustfmt 决策项)。5-A 落在 auto-vscode 仓(merge fc4cb9d,已推
gitee);6-A 落在 auto-lang(auto-lsp-ci.yml)。
