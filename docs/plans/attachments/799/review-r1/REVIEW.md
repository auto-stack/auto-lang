# PLAN-799 final review R1 — needs_fix

2026-10-17；plan revision 1；受审提交 `d35d4c0846d112ea661cb90fb950dff8c00d56d9`，实现提交 `ea291d3ab`；基面 `143aff0a258fdca8bda9a3b8068dc8a9cced623c`。
既有树 `D:/autostack/.wt/lang-799/auto-lang`，branch `plan-799-dev`，入/出净树、HEAD相同。审核人=本聊天 Codex（独立于实施上下文；未暴露模型/session ID，不虚构）。输入、依赖 tree 与文件 hash 见 baseline.json；冻结规范增量见 spec-delta.patch。未修改实现/规范/ledger，未合入。

## F799-R1-01 / P2 / 原合同兼容性回归

定位 `crates/auto-ast/src/ast.rs:75`，经 `crates/auto-lang/src/ast.rs:9` glob 门面泄漏。
影响 AC-02、AC-04/05；T-02/T-03/T-05/T-06。

基面 ast.rs 对 auto_val::AutoResult 为**私有 use**；新代码将其改成 pub use。
下游既有合法组合 `use auto_lang::ast::*; use auto_lang::error::*;` 中，
`fn parse() -> AutoResult<Code> { Code::parse("let x = 1") }` 现在产生
E0659（AutoResult 歧义）及 E0308（期望 Box<dyn Error>，得到 AutoError）。
这两个 AutoResult 是不同 item，不能用“同源再导出不歧义”证明兼容。
反例原文见 glob-compat.rs；以本树实际 auto_lang rlib 执行 rustc --edition=2021
--emit=metadata 复现 exit 1；只把 error::* 改成 error::AutoResult 的控制组 exit 0。
旧源码私有导入与无第二份公开 AutoResult 的 AST 模块面核对确认此为新增导出回归。

要求：保持旧门面的通配符导入兼容；例如 AutoResult 保持模型内部私有，或以不污染旧门面的名称/模块暴露轻值结果。为该反例及跨新旧路径的外部消费补永久编译回归；同步 SD/收据的导出说明。独立 consumer 无需公开 AutoResult 即可构造/调用当前模型方法。
此为原合同 AC-02 的缺口，不能登记为延期债后放行。

## 覆盖与实测

| AC / V | 结果 | 独立证据 |
|---|---|---|
| AC-01 / V01 | pass | cargo test -p auto-ast：74 unit + 1 projection + 5 doctest；独立 fixture cargo test --manifest-path test/fixtures/modularity/ast-model-consumer/Cargo.toml：3 unit + 1 projection 全绿；manifest 仅 auto-ast，normal/build/dev 无 auto-lang 反向边 |
| AC-02 / V02/03 | fail | F01；模型/fixture 36-case 固定 golden 重放通过，hash=1e74a8df…；ast_parse_ext 11/11；parse-identity.rs runtime exit0（双向 Code 互传，Code/Expr/Is/Arrow/OnEvents 坏输入拒绝，Name 正例） |
| AC-03 / V04 | pass | 32 子文件与旧文件逐字比较，仅 Pos/doctest 路径替换；根模型 diff 仅导入/移出 parsers/markdown_tests、doctest；旧目录物理删除、上层适配保留六 impl。真实消费者 parser 180、infer 107、vm::codegen 28、trans::gdscript 63、ui_gen::rust 83 全通过（2原有ignored） |
| AC-04 / V05/SD | partial | 777 strict pass：violations=0、grandfathered=2。792 DELIVERY_CRATES 增 auto-ast 与已交付接口吻合，work 25/25 driver 收据已读，794/ParserBridge work证据已读；本轮未独立重跑完整792 fresh/locked/重定位负控及794/ParserBridge fixture，不能声称全部独立通过。SD已冻结，F01出口说明需修正；metadata还需包含实际修改的 generated-ui-distribution.md 目标 |
| AC-05 | fail | final needs_fix |
| AC-06 | pending merge | 裸 cargo t + tv + tt + tu、必要合流重放、tf到期判定、规范/ledger/归档/guard清理未执行 |

cargo check -p auto-lang exit0（411 stock warnings）；触面 rustfmt --check exit0，git diff --check exit0。
auto-ast 测试 task.rs:1002 未使用变量警告在基面同一行已存在；没有归为新回归。
首次 consumer/check 因 sccache 端口 WSAEACCES 失败，临时 RUSTC_WRAPPER='' 后均成功；未修改仓库配置。
手工 rustc 初次探针选错依赖实例/缺 Windows native library 导致失败，已改为匹配 auto_ast-7bef1faa4eac8f93 与 native windows.0.52.0.lib，类型身份 metadata 与 runtime 均 exit0，不将工具接线失败报为产品缺陷。

## 可复跑方法

在既有799树执行上述cargo定向命令；临时清空 RUSTC_WRAPPER 可绕过本机缓存端口限制。
`glob-compat.rs`：rustc --edition=2021 <本附件源> --extern auto_lang=<本树当前匹配rlib> -L dependency=<本树target/debug/deps> --emit=metadata -o <临时rmeta>。
`parse-identity.rs`：同时 --extern auto_ast=<auto_lang实际依赖实例>，加 Windows native lib 搜索目录、-C linker=rust-lld.exe -C link-arg=/STACK:33554432 后运行。优先把这些输入做成由Cargo统一解析的永久consumer，避免手选缓存实例。

## 遗漏/延期/收尾扫描

未发现模型删减、Parser下沉、算法重写、重复模型或未授权UI拆分。唯一确认阻断为F01，规范“glob同源不歧义”也应限缩为实际同一item，不能覆盖两种AutoResult。新增规范内容仍在799树内，未发布到main。并行主检出已有800落地；合入前仍需同步/审查受影响新基线。保留未重放的V05项目，不用work banner代替独立验收。

下一步：原树 work 修复F01及相关SD/元数据、补永久反例，完成定向验证并提交，再复审；整档门保持merge-owned pending。
