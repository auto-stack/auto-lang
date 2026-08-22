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

### 5-B semantic tokens ✅ 2026-08-22 服务端完成(着色核验留待实机 F5)

- `semantic_tokens.rs`:词法扫描(注释/字符串/数字,关键字用 5-C 的
  `Token::all_keywords` 权威表)+ AST 符号分类(fn/参数/局部/type/enum/
  spec)+ 未分类启发式(`ident(`→调用、大写→类型);相对增量编码
  (UTF-16 列,多行字符串分段);8 类型 legend 顺序锁定。
- backend:能力声明(semanticTokensProvider)+ `semanticTokens/full`
  handler;extension 侧零改动(vscode-languageclient 在服务端声明能力后
  自动注册 feature)。
- 测试 ×4:token 序列锁定(关键字/fn/类型/参数/局部/数字/注释)、字符串
  +调用启发式、legend 顺序、协议级 round-trip(能力声明含 legend、
  线上扁平 5 元组形状)。
- **剩余**:VSCode 实机 F5 着色/闪烁核验(计划原定唯一实机点,待有桌面
  VSCode 会话时执行——服务端逻辑已由单测+协议测试锁定)。

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

### 6-B 集成测试扩容 ✅ 2026-08-22 完成

- 新 `tests/lsp_protocol_test.rs`(协议级,~250 行):直接驱动真实
  LspService(JSON-RPC Request + ClientSocket 排水任务)——生命周期
  (initialize→didOpen→completion)、didChange 增量编辑后 hover 不回旧
  快照、双文档 workspace documentSymbol、rename(实证当前为单文档作用
  域,虚拟 URI 无 resolver 根,边界如实记录)、signatureHelp/inlayHint。
- 覆盖率验收(llvm-cov 未装,按计划备选条款记录理由):未覆盖集中在
  bin.rs(stdio 主循环/传输胶水)、did_change 的 150ms debounce 任务分支
  (时序型)、resolver 的真实文件系统路径——三者均需进程级或磁盘环境;
  核心处理器(completion/hover/rename/symbols/signature/inlay)已全部
  有协议级用例。auto-lsp 13 测试全绿。

## 3. 执行顺序与联调安排

5-A ✅ → 6-A ✅ → 5-C ✅ → 6-B ✅ → 5-B ✅(服务端;VSCode F5 着色核验
待实机)→(6-C 全仓 rustfmt 决策项)。**416 至此仅剩两个手动/决策项。**5-A 落在 auto-vscode 仓(merge fc4cb9d,已推
gitee);6-A 落在 auto-lang(auto-lsp-ci.yml)。
