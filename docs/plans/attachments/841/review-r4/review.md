# PLAN-841 r1 — R4 affected final review

2026-10-27，outcome: **pass**，review_scope: **final**。当前 Codex `/root` 延续 R1/R2/R3 独立审查上下文，未参与实施、未启动子代理或额外审查 checkout。本结论覆盖代码/验收/候选规范；**merge-owned 统一门禁仍 pending，未合入、未交付完成**。

## Baseline

同组既有树 `D:/autostack/.wt/lang-841/{auto-lang,auto-ui,auto-man}`，分支均 `codex/plan-841-dev`；进入及退出 clean，HEAD 未漂移：

- UI `eb50ce67930179f51db041b74faa6d1fbc72c211`，repair base `60011b3029a7e8a7b9f95aeecff821fbd448e962`。
- lang `ecfa187f72d3fcf7ba662ae6cd3e1170aad75735`，repair base `6892bad1ae9556f93da743bb1af265625330d809`（本轮仅文档/候选 Spec，无 Rust 变化）。
- Man `ebfbb37903d24bcea3cbd81c6a0a5602da605982` 未变；down clean clone `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` 未变。

主计划从语言主检出读取，revision 1；[baseline.json](baseline.json) 冻结主计划输入、三仓提交、依赖/配置、当前 canonical 与候选 Spec hash、probe hash。完整候选规范差量：[spec-delta-reviewed.diff](spec-delta-reviewed.diff)，SHA256 `cf1a23cd0120152df7be2ed11890336a51620023979dc614b9bc0b562b5a0c54`。

检查范围从 [R3 finding](../review-r3/review.md) 与 [work 诊断/关闭矩阵冻结副本](repair-r3-closeout.md) 出发：新共享预检的导入初始化、文件目录归属、冷态/残留/拒绝后恢复，以及被影响的准入/配方合法控制。未重放无关全档、完整发行矩阵或已闭 codec 家族。

## F841-R3-01 — CLOSED

`auto-ui-compiler/src/ui_gen/api.rs::validate_ui_file_with_modules` 现在在主文件目录下先执行 `prepare_style_recipe_imports`，再 parse/lower；`validate_use_modules` 对每个直接导入模块按其自身目录执行相同步骤。与正式组件入口的初始化顺序一致。没有删除领域校验、启用 skip_check 或依赖先前 registry 预热。

原输入逐字重放成功：local/named/wildcard 的 cold full、wildcard cold incremental 均通过，实际输出 recipe class `rounded-full`；warm 对照保持。原始 [original-cold-controls.log](original-cold-controls.log) 为两项 R3 probe 的重放。

独立补充了共享不变量的两组控制，完整代码 [review841_r4_compatibility.rs](review841_r4_compatibility.rs)，最终记录 [independent-state-controls.log](independent-state-controls.log)：

1. **同名 stale、拒绝后恢复**：真实组件入口先生成并证明 registry 存在 `chip = rounded-full`；下一独立文件没有导入、仍引用同名 chip，file validator 正确报 undefined variable。为第二文件添加自己的合法导入后恢复成功，实际 class 为它自己的 `border`，不再含旧 `rounded-full`。不是用不同名字 missing 假充同名残留控制。
2. **直接模块按自身目录取 recipe**：宿主 `styles.at` 和 `parts/styles.at` 都导出 chip，但分别为 `bg-red-500`、`bg-blue-500`；宿主导入 `parts.card: Card`，Card 在自身目录 `use styles: *`。冷态联合 preflight 成功；真实宿主/子件生成分别含红/蓝 class。证明使用了真实模块上下文，未把宿主目录误用于子模块。

两项补充与两项原 probe 组成的最终独立 4 测全绿。私有/缺失配方、同文件/跨包/传递配方及 name collision 由本轮永久控制继续验证。没有发现新的 repair regression 或未处理合同内 gap。

## Check → invariant evidence

| Check | Result | Evidence / boundary |
|---|---|---|
| 独立原 cold + 新状态/目录 probe | **4 pass** | [log](independent-state-controls.log)；named/wildcard/local、full/incremental、实际 class、同名残留、失败后恢复、主/子目录。早先2测日志保留，不重复累计。 |
| tools `ui_domain_validation` | **6 pass** | [log](permanent-tools.log)；四 backend 的合法/四领域错误家族/导入/零输出、incremental、cold 配方和 private/missing/stale；已阅读循环和断言。 |
| compiler `design_tokens_recipe` | **9 pass** | [log](recipe-controls.log)；named/wildcard/non-pub/transitive/cross-package/collision 与既有 desugar/lint/semantic 控制。 |
| compiler `ui_domain_validation` | **3 pass** | [log](compiler-controls.log)；本轮改变共享文件/模块预检，故复核原领域拒绝与合法导入，未因配方修复放宽准入。 |
| rustfmt / repair diff-check | **pass** | 本轮两个 Rust 文件以既定 `rustfmt --check --edition 2021 --config skip_children=true` 通过，[format.log](format.log)；UI/lang repair diff 无 whitespace error。 |

最终四组共 **22 项具名定向测试通过**。没有依靠编译成功或总数代替实际断言。构建仍有先前已归因的 warning（含链接器提示）；本修复六条生产代码未引入新的未处理 warning 或 debug 输出。

## Prior evidence and AC map

本轮 production diff 仅 file/module helper 的两处导入初始化（六行），测试新增101行；Cargo manifests/locks、Core、runtime/SDK 实现、codec 均未改。相关依赖/配置与输入 hash 已冻结，因此按受影响范围复用既有证据：

| AC | R4 result and evidence chain |
|---|---|
| AC-01 | **pass**：R1/R2 已审的 Core/UI 分类、低层零专属承载、依赖闭包/独立 Core 消费证据保留；本轮只调用 UI owner 已有 recipe 函数，Core 端口与边界未改。 |
| AC-02 | **pass**：原全部 UI 专属臂退出/领域模型/合法 Core 对照保持；R2/R3 已闭工程准入，本轮 cold 合法样式恢复与无效数据拒绝共同通过。 |
| AC-03 | **pass**：R1/R2/R3 的字段/span/source_lines/跨文件 owner、Weak、codec 深度/完整输入/语义/payload、持久缓存 schema 证据按实现/配置/输入不变复用，本轮不重复 codec9 或 cache1。 |
| AC-04 | **pass，统一交付 closeout pending**：四生成器/VM/Iced/CLI/独立消费者的既有具名证据保留；工具/合法导入受影响面由本轮6+9+独立控制关闭。最终冻结来源的统一回归、发行/777/792与三仓实际合流仍由 merge 承担。历史39/39不冒充新提交的整套发行结果。 |
| AC-05 | **pass**：测试 ID/owner/golden、旧实现退出及842上层适配清单沿用R1已审记录；所有历次finding已关闭，本轮SD与实际初始化顺序一致，健康检查通过。 |
| AC-06 | **pass**：独立最终复审绑定本轮实际提交、依赖/配置、输入和候选SD；T-10完成。 |
| AC-07 | **pending，merge-owned**：T-11/T-12 未完成，不据复审通过宣称交付或后继U841放行。 |

历次结论：F841-R1-01/02/03、F841-R2-01 保持 CLOSED；**F841-R3-01 本轮 CLOSED**；没有新增 blocking finding。所有历史失败输入、日志、诊断和关闭证据均保留，不改写历史 verdict。

## SD review and bookkeeping

SD-01 的分类/旧AST退出、SD-02 的中立端口/UI owner、SD-03 的真实消费者、SD-04 的SDK/来源/版本与发行合同、SD-05 的领域模型/codec/寿命规则均结合当前实现与已审证据核对。本轮 `auto-ui-compiler/project.md` 的“按各文件自身目录先 prepare imports 再 parse/lower、不借前一文件残留”描述获实际状态/目录控制支持。

复审补齐 Plan metadata 与 SD-02/04 的目标映射：增加实际候选中已存在的 `docs/specs/auto-ui-dialect/design/full-grammar.md`、`docs/specs/auto-lang/project.md`。其含义已在原 SD、handoff 与历史差量内，不新增功能/兼容承诺，不改 revision；本轮仅修正漏列的沉淀导航。全部11个 modify + 1个 add Spec 目标与冻结差量一致。未改 canonical 文件或 ledger。

## Pending gates / next

Plan **reviewed 10/12**，T-10勾选；接 merge，在最终冻结三仓输入上做并集合流影响复核、裸 cargo t 与适用 tv/tt/tu（按迁后owner/alias）、完整发行/777/792及三仓交付检查，随后 canonical/受管ledger/索引沉淀、归档与逐树fresh wt-guard清理。cargo tf 仍仅按到期规则在主检出单实例批量执行。

若实现、依赖、测试配置或输入改变，受影响证据随之失效并按范围复审；不能将本次 pass 扩张到未来代码。当前没有运行整档、合入、归档、清理工作树或放行 U841。

临时 probe 已从原实施树撤去，持久代码/日志在本目录；最终 clean/HEAD/hash 见 final-state.json。
