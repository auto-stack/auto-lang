# PLAN-833 r2 — final review R2

**Outcome: pass。** F-833-R1-01 / F-833-R1-02 独立确认关闭；无新增合同 finding。计划进入 `reviewed / 9 of 10`，T-09完成，T-10及所有merge-owned门仍pending。未修改实施、canonical Specs、ledger，未合入、归档或清树。

## 冻结与独立性

- stage: review; review_scope: final; plan_id: PLAN-833; plan_revision: 2; final review round: 2。
- reviewer_identity: 当前Codex审核会话，独立于实施会话，沿用R1审核上下文；具体模型/会话ID未暴露。按实际源码、旧反例和独立运行确认，不将实施侧readiness当作结论。
- worktree: `D:/autostack/.wt/lang-833/auto-lang`; branch: `plan-833-dev`; reviewed_commit: `77302d8552f074ebedd53e757acfcf3db0bde230`; implementation_commit: `6c47b79b13352eef44bf7e65084d2a8827a0b8a5`; base: `764e87559cff016d9e0d6b2141e5f1f9903398c5`。
- 当前30份实现/配置/输入hash全部匹配。相对R1清单，只有AGP `surface_runtime.rs` / `text.rs` 和driver `graphics_loop.rs` 3项改变；另外diff为README、handoff、身份与拟增量文档。无SDK/driver host/script/玩法/来源工具/依赖或测试配置改变。进入和结论前同HEAD、实施树clean。
- SDK `9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`，SDK原锁 `5192d08d502a8f17e277ad8912f5884175659bef642b614e25b394243d1ff091`；driver锁 `f01c231218b2f1534f971df128b9d188fa19648e5a4399dbe81ce37e5386063d`；AGP原锁 `c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf`。配置hash与R1相同。
- Windows x64 / RTX4060Ti / Vulkan / NVIDIA617.14。完整身份、技能hash、源规范hash、原反例hash及冻结拟Spec全文hash见 [baseline.json](baseline.json)。

## 旧例、完整不变量与修复回归

| Finding | 原始反例独立重放 | 完整不变量与合法控制 | 结论 |
|---|---|---|---|
| F-833-R1-01/P1 | 原R1三个输入/脚本逐字节不变。local/local接受；local/foreign、foreign/local、foreign/foreign全部DeviceMismatch。曾panic的矩阵现exit0，frame提前Err、frames仍0。独立replacement由原Ok改为DeviceMismatch。 | validate_device逐个检查quad/text；add_surface在目标分配前调用，frame在帧计数与GPU工作前对所有slot调用。新永久测试8组合×2入口状态：双向foreign、同foreign、raw、无overlay、local；错误分类、target数、frame数、0 upload和旧target hash均受断言。保留Iced/Scene3d单资源校验。 | CLOSED |
| F-833-R1-02/P2 | 相同原始upload探针：无overlay=1 rewrite/64B；5glyph=3/368B，实际增加2/304B；原两张GPU图片hash保持，文字确实绘制。 | 两个真实write_buffer点以实际bytes.len计数；统一content helper汇入per-content/content aggregate/runtime，未重复加数。永久测试无/5/空glyph=1/64、3/368、2/128；重复dirty frame精确，clean warm零写；共享/单挂载均只多2/304B。 | CLOSED |

重放源与原始输出：[provenance-probe.rs](provenance-probe.rs)、[矩阵](provenance-probe.txt)、[独立replacement](provenance-replacement.txt)、[upload-probe.rs](upload-probe.rs)、[实际计量](upload-probe.txt)、[重放脚本](replay-probes.ps1)。本轮复制到R2目录运行，未覆写R1历史输入/结果；脚本输出实际链接的AGP rlib。

计量省略负控复用本次修复的已识别实际mutation证据：[work-r3/repair-upload-negative.txt](../work-r3/repair-upload-negative.txt)，单个正式测试exit101，actual `(1,64)` / expected `(3,368)`，随后源码恢复并绑定当前hash。已检查正式断言读取per-content和runtime的精确数值，并独立运行相同输入的旧/新数值区别；本轮未再修改实施源码制造mutation，不将该负控写作审核新执行。

完整修复diff与相邻路径已核：无输出准入、调度、native输入或Auto玩法语义变化；设备检查与counter两条路径分别有拒绝和正常渲染控制。没有通过弱化原合同关闭finding，没有未授权延后/workaround或新生产写面。

## 本轮执行与证据复用

所有产品命令在唯一833树、显式manifest、locked/offline执行。未跑完整driver套件或根整档。

- driver `cargo test --manifest-path experimental/autographics-auto-driver/Cargo.toml --locked --offline --test graphics_loop -- --nocapture`：**6/6**，0 ignored/skip，[graphics.txt](graphics.txt)。包括原消费端回放/速度碰撞反控、2D/3D像素/源码修改、真实local/共享状态/单tick、Runnable故障公平性、20 reset/owner以及新归属/计量矩阵。
- 本轮V-07：compile26881us；shared P50/P95=2255/2758us，single=2128/2859us；5 targets /9109600B；reported upload **150720B**，warm extra pipeline=0。working set 20 reset前510513152B/后507691008B。旧89280B已失效；不声称GPU timestamp或resident memory可测。
- AGP `cargo test --manifest-path experimental/autographics-prototype/Cargo.toml --target-dir experimental/autographics-auto-driver/target --locked --offline --test breakout_contract --test squash_contract --test surface_contract`，dev debug0：**11+17+29=57/57**，[compatibility.txt](compatibility.txt)。不是完整merge兼容门。
- 两manifest `cargo fmt --check`、`git diff --check`均0；driver `cargo check --locked --offline --all-targets` 0，[check.txt](check.txt)。SDK原有146 warnings，无authored新warning。
- `verify_source.py` 0：680固定输入，17 local packages/29 edges，479实际SDK闭包，forbidden0，AGP原锁保持。

11项非graphics验收（2 unit +9 integration）复用独立R1的具体输入/断言与结果：SDK标量/数值、整包准入与owner、scheduler、10800tick/210contacts原golden等实现/输入/配置未变。故当前17项验收库存为 **6项本轮独立执行 +11项有界复用**，没有冒称17项全新重跑。native正反焦点、160帧Breakout/6次实际音效复用R1：host/window/router/script/source/audio/assets/dependencies不变；本轮GPU合法渲染控制再验。

SDK固定字节/镜像、SDK-only relocation与来源工具负控证据继续复用R1/work-r2；工具、图、manifest、锁、输入均未变，本轮重新verify_source及hash。历史外部包的AGP代码早于修复，**不作为当前修复后的图形交付证据**。完整当前源码交付、构建与适用重定位/原生窗口/资源等收口由T-10在此冻结提交执行。

## AC / SD审查与下一责任

| 条件 | 结论 / 依据 |
|---|---|
| AC-01 / T-01,03 / V-01 | pass；固定来源/锁/闭包、无根生产写面；复用和重新执行边界如上。 |
| AC-02 / T-02,04 / V-02 | pass；11项不变验收中SDK调用/数值/错误/owner证据复用；未碰SDK与VM。 |
| AC-03 / T-05 / V-03 | pass；本轮实际GPU创建/移动/颜色/销毁与输出反控重跑。 |
| AC-04 / T-06 / V-04 | pass；原完整golden证据不变，本轮消费端240tick、事件、速度碰撞/输入反控重跑。 |
| AC-05 / T-07 / V-05 | pass；共享/独立/local/单tick重跑，native焦点/音频证据有界复用。 |
| AC-06 / T-07 / V-06,07 | pass；两finding完整闭合，修复回归、57兼容、20 reset等通过；计量已更正。 |
| AC-07 / T-08 / V-08 | pass；README/handoff/SD-01/03补逐资源校验与真实文字写入，命令/限制和I1边界一致。 |
| AC-08 / T-09 | pass；独立final R2完成，无open合同finding。 |
| AC-09 / T-10 | **pending，merge-owned**；未交付/合入。 |

SD-01新增、SD-02 integration、SD-03 project/profile四份全文冻结为 `spec-delta-*.md`。三份现行829/830 canonical前缀全部保留，新增文字规则与实测吻合；I1双游戏分母、Auto Squash、生产I2/I3/RQHost/Forge、Shader/CAP和合master迁移门保持pending。supersedes三个路径、new一个路径、GOAL-007/008真实且不变。未发布canonical或ledger。

pending_closeout_gates：完整driver test/build，以及计划要求的AGP适用六surface/native/content/反控/resource/full compatibility等冻结门，均由merge在 `77302d8552f074ebedd53e757acfcf3db0bde230` 执行；必要当前源码消费/重定位收口也不得沿用旧包冒充修复交付。根t/tv/tt/tb/tu/taa/docs_gen不适用本实验写面，tf不在本树执行。

下一责任：T-10 / merge。门禁通过后才落autographics-dev、沉淀master知识/ledger/index、归档并guard清理。若门禁需要改代码，回work、冻结新提交并复审受影响内容；本pass不覆盖后续代码。当前只完成审查，保留实施树。
