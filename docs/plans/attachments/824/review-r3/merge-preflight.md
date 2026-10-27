# PLAN-824 merge preflight（进行中）

- reviewed Plan r7：R3 pass `29b6eeea98d1b75e2ea18ea5f62eea96866e9d60`；工作树 `D:/autostack/.wt/lang-824/auto-lang` / `plan-824-dev`。
- 先检查主干 `5b65aa68c65bd33142e1198b73dfbd6e822fd5a6`：823 executing 3/11，未交付；824 占用 profile 5、AVAD v4、opcode 0x2F..31，主干止于 profile 4/v3/0x2E。后合入方仍需核实际并集/编号与 Closure × Dyn 拒绝。实际落地前还需重核主干未漂移。
- 已将 14 个实施提交 rebase 至该主干；[range-diff](merge-range-diff.txt) 14/14 `=`，`29b6eeea9 → e2d5c9e081c343f9237f97867e5a85f1e3db82ad`。代码冲突/改号均无；主干锁文件与根 workspace 配置更新了 ecow 的 serde 特性，故不能简单沿用旧依赖绑定。
- 新主干之上规范候选 `5e762131552f98eea8004e02a5776a4bd5961d97`：七项 SD 与三个模块导航/计划表；spec-index 重生无内容变更，13 个文档本地链接检查通过（未来归档链接在实际归档后核）。独立复核已指出三个计划表的行前空行，须修正并在新冻结提交复核。
- 新依赖定向：parallel/824 R2 五例、wire 五例、admission 三例；probe resources R2 六例、real adapter 一例、incremental body identity 一例，21/21；原 R2 原始复现回归/对照均符合预期。该验证由本实施会话执行，不冒称独立复审。
- 资源纪律：已观察其他会话 cargo 构建/计时测试；统一门及性能门须等无竞争单实例后执行。裸 cargo t、cargo tv、新 workspace 合同档、798 时序裁定、独立变更复审均 pending；无门禁或合入完成声称。
- 本地规范服务受限检查后以授权的只读连接确认 127.0.0.1:8080 连接被拒；ledger v101 可解析，符合六段。仅在代码落地后按 guarded offline projection 单写、备份、范围/并发/读回校验，不提前修改。

后续更新：首轮 cargo t 因 StepOut 漏字段及旧 UI/新 parser 不配对而编译失败；已修复并独立复审，通过后进一步同步仅文档的主干 ee06acf，17/17 =。当前候选 8da513aefe8885e28ec8716f054be5e8d639800d，UI 4903bef、down c3464d0；旧门禁不可复用，详细见 merge-gates/fix-independent-review.md。

