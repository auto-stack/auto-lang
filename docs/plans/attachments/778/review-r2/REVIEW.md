# PLAN-778 final review R2 — needs_fix

2026-10-15；plan_revision=1。独立于实施的当前评审会话；未修改实现，依据提交、源码、历史反例与新增边界输入重建结论。

审面：`D:/autostack/.wt/lang-778/auto-lang`，`plan-778-dev`，commit `a8a81aad4671ac8eae1728c4e71623c2628abd8b`；base `87f45dc4e2c592c31844e74824e67bf9e6cb419f`。auto-down 依赖与 frontend Spec 源 hash 见 [fingerprints.json](fingerprints.json)。实施输入前后 hash 一致，HEAD 不变，实施树及 auto-down 干净。

## 结论与 finding

**F778-R2-01 — P2 / blocking，原冻结子集与实际支持面不一致；AC-01/AC-03，T-03..05。**

主检出 Plan §T-01 第6条与 §T-05 已验证面均承诺 `model { [var] name [type] [= expr] }`，两个可选项可以同时省略。外部实现 `test/fixtures/modularity/ui-dialect-consumer/src/lib.rs:117..120` 仅在下一个 token 为 `=` 时省略类型；遇 `}` 或换行必调 `parse_type()`。

- `model { count }`：预期外部接管并生成 `ModelField(name=count, ty=Unknown, init=Nil)`；实际首错 `Expected type, got }`，offset14，整段恢复级联20错。级联是既有恢复机制，不另列新 finding。
- `model { count\nother int = 2 }`：预期两个字段；直接 `parse_stmt()` 实际 `Expected type, got \n`，offset13。此反例避开整段错误级联。
- `model { count int }`：成功，typed Int + Nil、external accepts=1。
- 内建 `model { count }` 同样拒绝：本 finding 是本 Plan 冻结支持面/验收报告的缺口，不是默认文法新回归，也不要求擅自扩大默认内建文法。

修复方向：明确冻结子集的真实规则并补永久边界正反控；按现有合同实现裸字段支持时保持默认内建路径不变。若原意仅复用内建的“类型或初值至少一个”规则，需通过 new 明确调整语义合同/plan_revision，不能在复审中静默收窄。当前记录“已完整验证”不能保留。

重放：运行 [run.py](run.py)（在 Plan 树 scratch 中复制 [independent.rs](independent.rs)）；为独立 harness 锁定 fixture 同一依赖版本，复制 fixture Cargo.lock 到 scratch/review778-r2/Cargo.lock，再运行 `cargo test --offline --manifest-path scratch/review778-r2/Cargo.toml --test independent -- --nocapture`；结果 [independent-pinned.log](independent-pinned.log)，6 pass/1 fail。复制 [boundary.rs](boundary.rs) 至 scratch/review778-r2/tests/boundary.rs 后运行 `cargo test --locked --offline --manifest-path scratch/review778-r2/Cargo.toml --test boundary -- --nocapture`；结果 [boundary.log](boundary.log)，2 pass/1 fail。两次均共享 fixture target，实施代码未改。

首次独立 harness 自动解析到较新的 rustls，见 independent.log；不以该轮作为最终依赖同基面证据。上述 pinned 重放保持 fixture 原依赖版本，重复得到同样6/1结果。

## 上轮闭合与审面

| Finding | R2结果 |
|---|---|
| F778-R1-01 | closed：Arc 表元素在 Ident/token 派发期间从不移出；model 闭包初值递归接管 accepts=2，兄弟块正控通过。 |
| F778-R1-02 | closed：派发 Err 恢复五个约定旗标；fn/on/watch/dot-RHS 保存前值并在 `?` 前恢复。新增独立断言在 harness restore 之前检查 in_fn_body=false，通过。 |
| F778-R1-03 | closed：新增 bridge、dialect 两文件、fixture src/两测试文件 rustfmt edition2021 check exit0；diff-check pass。parser/lib 的既有全文件格式债务不扩大本轮 scope。 |

同意联合修复期望 `[false,true,false]`：中位是在 fn 体内真实发生的递归派发，末位为恢复点上下文。旧 `[false,false]` 忽略了 P1 修复后的新增可见派发；无需回退实现。原永久控制的 checkpoint restore 现在同时恢复旗标，因此只检查 restore 后状态会遮蔽错误出口问题；本轮新增 pre-restore 观察通过，建议收编这一更强 oracle。

完整审计 bridge 公开面、私有 escape hatch、三段优先级、matches 场景门控、跨 with_session 保留、实例隔离、Ident/token None/Err、内建 UiDialect 调用、parser 创建点与 widget 直连边界。没有发现新的注册/恢复实现缺陷。生产入口/导入注入、完整 UI 文法和符号表事务均保持已声明未迁范围。

## 验收与验证

| AC | 结果/证据 |
|---|---|
| AC-01 | partial：typed model 基例、公开 API 隔离通过；冻结可选字段组合失败 F778-R2-01。 |
| AC-02 | pass：session/实例/冲突/Err/None/递归控制及更强 pre-restore 状态观察通过。 |
| AC-03 | partial：默认兼容定向检查通过，但支持面报告不准确；merge-owned 整档尚 pending。 |
| AC-04 | 本轮独立审计完成，整体不能 pass。 |
| AC-05 | merge-owned pending。 |

- 外部 fixture 18/18；[external.log](external.log)。
- `cargo nextest run -p auto-lang --lib --features ui-iced,test-cases -E 'test(parser::tests) | test(dialect) | test(model) | test(widget)'`：255/255，包括新4条、parser179；[scoped.log](scoped.log)。
- `cargo check -p auto-lang --locked` exit0；[check.log](check.log)。既有410 warnings仍存在，多数在未改源码；未发现本 Plan 新增未处理 warning，不声称全仓零 warning。
- 改动面格式/差异空白检查通过，[format.log](format.log)。未跑裸 t/tv 或其他整档。

SD-01 metadata 路径和 GOAL-007 可用；[spec-delta-reviewed.md](spec-delta-reviewed.md) 冻结为 partial，待 F778-R2-01 支持面契约闭合。merge 须同步替换 frontend 现有“mem::take 新方言照此模式”的过时坑点，而非仅追加新小节。本轮不发布 canonical Specs/ledger。

保留 T-01/T-02；重开 T-03/T-04/T-05（原通过证据保留，不抹历史），status=executing，current_step=2/7，T-06未通过，T-07 pending。下一步同树 work 闭合契约与永久边界控制、提交后复审。已连续两次 final needs_fix，下一 handoff 需附覆盖诊断/调整：原 oracle 覆盖 typed/带初值形态但未覆盖两个可选项同时缺失。

最终 review pass 后才由 merge 在新的冻结提交统一执行裸 `cargo t` + `cargo tv`；未触 trans/ui_gen/aavm，不要求 tt/tu/taa。本轮不修实现、不 merge/归档/清理。
