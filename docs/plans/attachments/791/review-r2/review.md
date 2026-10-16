# PLAN-791 final review R2 — needs_fix

2026-10-16，plan_revision=1；Codex本聊天独立于实施，复用同一实施树，不另派agent、不改实现。当前仅剩一项新发现的边界基线阻塞，R1两项关闭。

## 基线

- worktree `D:/autostack/.wt/lang-791/auto-lang`，branch `plan-791-dev`。
- reviewed_commit `b54cc4a2a1b95631c9e4726985615d66ba116c7e`；diff base `e4ac72a6265c85cdadeedbabc7a0c6fd93948e48`；修复比较点 `0cd3e3e9f7e448fc95a1ef61b13761ac02d7e65e`。
- 入场、测试后git status为空，HEAD未漂移；sibling auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- main Plan入场SHA256 `C155AF1C6D9B21B13E0E96A2836FCD0707803602EA77EC22BFD33BF133788D1A`。
- workspace Cargo.lock、module-boundaries配置与baseline内容均未改，hash同[R1](../review-r1/review.md)。

## Findings与闭合

| ID | 判定 | 证据、范围与后续 |
|---|---|---|
| F-791-R1-01 | closed | 新V02三例覆盖非空lib_files加载后函数不可见（报告内Undefined variable: greet、空输出树）、非空lib_paths及模板use行输出金样、缺库文件顶层TemplateLoadError/既有hello.txt哨兵字节不变/目录仅1文件。新旧阶段边界及成功控制均有断言；V02 14/14通过。work记录在旧实现9c4780c54上载入相同fixtures产生金样并重跑14/14，本轮将该旧臂回放作为已具名work证据复用，未宣称本轮另建旧树独立重跑。源码比对确认本次没有改语言渲染和库加载语义；所有新臂在新冻结HEAD独立重放通过。 |
| F-791-R1-02 | closed | 两crate fmt check=0；git diff --check base HEAD=0；6个只供V02使用的helper迁入V02，不再编进V03，定向完整编译/测试日志无auto-gen或core新增warning。旧auto-lang 411及shim-metadata 1 warning保留为依赖基线，不假称全仓零warning。 |
| F-791-R2-01 | P2 / 修复后的基线元数据遗漏，open | `python scripts/modularity-audit.py --mode strict --output <report>` exit=2，verdict=fail、violations=1、grandfathered=2；唯一违反为 `stale_baseline`，detail="锚定输入自 base_revision 起已变更: ['crates/auto-gen/Cargo.toml']；基线过期必须显式重采样，不能默默通行"。R1修复删除该manifest末尾空行，manifest字节变化而777冻结基线未更新。合同T-05明确包含metadata/777边界，不能以闭包无语言依赖、observe通过或语义只改空白代替strict。原树核对base_revision后所有锚定输入差异，按现有规范在干净提交上显式刷新基线，保留合法边与2项原grandfather，重新运行strict并提交新基线；不得跳过/放宽陈旧判定。受影响AC-04/05，T-05/06。 |

R1未独立运行strict（报告已注明，仅复核当时work收据），本轮扩大到该合同门发现此缺口。F-R1-02本身已关闭；不能把新增manifest指纹失配写成R1警告/格式仍未修复，也不能据R1旧PASS收据覆盖新manifest。

## 本轮实际验证

所有执行均在计划原树，无UPDATE_GOLDENS；完整测试原始输出：[targeted.log](targeted.log)。

| 检查 | 结果 |
|---|---|
| `cargo test -p auto-gen-core -p auto-gen`（本命令RUSTC_WRAPPER为空） | exit0；lib adapter 11 + V02 14 + V03 8 + core 12 + docs 5 = 50 passed，bin/core-doc 0 tests。 |
| `cargo test --manifest-path test/fixtures/modularity/gen-core-consumer/Cargo.toml` | exit0，4 passed（真实文件/guard/dry_run与失败哨兵/legacy字节）。 |
| Git Bash `test/fixtures/modularity/gen-core-consumer/verify-deps.sh` | exit0，PASS；actual normal/build闭包core→auto-val+叶依赖，无auto-lang/auto-man/VM/UI。 |
| `cargo fmt -p auto-gen-core -p auto-gen -- --check` | exit0。 |
| `git diff --check e4ac72a62 HEAD` | exit0。 |
| `python scripts/modularity-audit.py --mode strict --output <temp>/plan791-r2-boundaries.json` | exit2，唯一stale_baseline，原始[boundaries.json](boundaries.json)。 |

首次定向执行因本机sccache服务启动端口被Windows排除而exit101/os10013，未进入相关测试；第二次仅本次执行设置空RUSTC_WRAPPER后通过。未修改Cargo配置、未把该环境失败计为源码回归。

修复回归审查：生产代码变化只有格式/导入顺序，core只调整测试格式；common helper移动后golden比较仍对比完整文件集合和字节。新增`.empty-tree`提交文件解决Git无法跟踪空目录的复现缺口：marker分支同时断言实际零文件及marker为唯一golden条目。旧非空golden字节未重生。builder测试删去的builder.output_dir并非被测实际generator配置（后者仍独立指定tmp），输出名控制仍有效。

覆盖结论：AC-01/02/03 pass；AC-04生产调用/真实净删/未迁清单通过，边界strict部分fail；AC-05 fail；AC-06 merge-owned pending。生成器core、legacy两种guard差异、无解释器类型边界的R1源码证据可复用：修复diff未改变其语义、锁及依赖未变，而且其核心运行用例已重新执行。未遗漏未批准延期/功能workaround；本次新增缺口不可作为未来债豁免。

## Spec delta复审与指纹

SD-01新增core卡片保持R1内容；SD-02修正legacy措辞，明确执行仍在adapter解释器，只有guard feed由core单一定义，已与实际实现一致。`supersedes_spec_components=[docs/specs/auto-gen/project.md]`、`new_spec_components=[docs/specs/auto-gen-core/project.md]`及GOAL-003/016引用为真实目标，保持不变。main canonical auto-gen Spec和overview hash同R1，无并发规范冲突。review未发布canonical或改ledger。

冻结diff：[spec-delta.diff](spec-delta.diff)，SHA256 `15693CADF593700BDA6FF7D08793B92C459C02DD5E8294EDF2C28444E56F3F1D`，由base与完整受审SHA重建。

SHA256（实施树文件）：

- auto-gen/project.md：`45A666E13E450C50F604AF7A26FBF5C2E0C6A49F7D44B475F2F9FCB740F0C0D7`。
- auto-gen-core/project.md：`0AB2DE7A6D9EE5B474399D932869D60A06D6A732D6E9E6EC89ABA9B8FB7878BC`。
- v02_compat_golden.rs：`C5818E7296DD1251AE0C60418DEE5FBE7DFBBEA1F6E32BD6C99A508A8AE9426D`。
- fixtures/lib/util.at：`97E66F5304258C244475615577A35C43D9AEB12E51DBADA488416DB5D39A9CD7`。
- fixtures/templates/uses_lib.txt.at：`A672526A783E8C717E92B8718350763FF358AD26B53CE5578A6D6E724C121923`。
- fixtures/goldens/use_line_filtered/uses_lib.txt：`E6AB9FF065B2ACFB8FD8556669CA3047AE1202D63C3E48815B868BD3C4F705FC`。

## 路由

needs_fix，T-05/06重开，current_step=4/8（T-01..04完成），status=executing。原树修复F-R2-01并提交后重审，保留R1关闭事实。merge-owned裸cargo t、core/adapter定向全测、规范/ledger沉淀、归档/合入和guard清理全部pending；本轮未跑任何整档t/tv/tt/tb/tu/taa/tf，未merge、归档或清理。
