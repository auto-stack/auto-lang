# PLAN-833 r2 — 独立最终复审 R1

**结论：needs_fix。** 冻结代码的现有 16 项具名验收与 57 项定向兼容均独立复现通过，但新增 TextOverlay 接入存在两项合同内缺陷，不能进入 merge。计划回到 `executing / 6 of 10`，重开 T-07、T-08；保留 T-01..06，T-09/10 未完成。没有修改实现、SDK、canonical Specs 或 ledger，没有合入或清树。

## 冻结身份

- stage: review; review_scope: final; plan_id: PLAN-833; plan_revision: 2。
- reviewer_identity: 当前 Codex 独立审核会话；未继承实施会话，依据主检出合同、提交、源文件与实际运行重建结论。工具未提供可验证的具体模型或会话 ID，不杜撰身份。技能及合同输入 hash 见 [baseline.json](baseline.json)。
- tree: `D:/autostack/.wt/lang-833/auto-lang`; branch: `plan-833-dev`。
- reviewed_commit: `b136129fed9a018f07ea49445868066449eaf5e9`; base: `764e87559cff016d9e0d6b2141e5f1f9903398c5`。进入与出具结论前 HEAD 相同、实施树干净，30 份实现/配置/输入的 LF 归一化 hash 全部匹配。
- SDK: `9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`; SDK 原锁 `5192d08d502a8f17e277ad8912f5884175659bef642b614e25b394243d1ff091`; driver 锁 `f01c231218b2f1534f971df128b9d188fa19648e5a4399dbe81ce37e5386063d`; AGP 原锁 `c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf`。
- SDK 镜像 678 份未投影源文件逐字节对照固定 Git archive，全部相同；另外两份为已记录的 workspace manifest 投影和原始 provider lock。verify_source 成功：680 输入、17 local packages / 29 edges、479 实际 SDK closure packages、forbidden=0。
- Windows x64 / RTX 4060 Ti / Vulkan / NVIDIA 617.14。构建继承仓内 `.cargo/config.toml` 的 32MB stack 链接选项；新 driver dev debug=0。无依赖升级或根生产写面。

## F-833-R1-01 — P1：本设备文字遮蔽外设备 quad，归属准入失效

- 违反的既有规则：AC-06 / V-06、T-07；§5.3 失败/资源归属；沿用 830 的所有内容 adapter 同设备、拒绝在 GPU 工作之前、失败保 last-good 合同。非新增跨设备支持要求，非恶意源码沙箱要求。
- 位置：`experimental/autographics-prototype/src/host/surface_runtime.rs:136..143`，同一 `device_ordinal()` 被 `add_surface` 和 `frame_with_cursors` 使用。
- 已确认原因：quad 与文字不同时，函数返回文字的 ordinal。如果 quad 来自 foreign、文字来自 runtime 自身，返回值恰好等于 runtime，隐藏了错误的 quad。
- 实际入口：公开 `Game2dContent` / `TextOverlay` → `SurfaceContent::device_ordinal` → `SurfaceRuntime::add_surface`；另走 `content_mut` 替换 quad → `frame`。没有修改实现、伪造 ordinal 或替换 oracle。

| quad | text | 期望 | 实际 |
|---|---|---|---|
| local | local | 接受 | 接受 |
| local | foreign | 拒绝 | DeviceMismatch |
| foreign | local | 拒绝 | **reported=Some(0)，admitted=true** |
| foreign | foreign | 拒绝 | DeviceMismatch |

host=0 / foreign=1。矩阵随后进入真实 frame，wgpu `storage.rs:135` panic：`Buffer[Id(5,2)] is no longer alive`，exit 101。独立的 replacement 反例先成功画出同设备控制帧，再只替换 quad；报告仍为 Some(0)，`frame=Ok(())`，exit 0，说明逐帧校验同样漏检。不能把矩阵中的 panic 误述为该独立 replacement 必定 panic；两条路径共同证明拒绝规则失效。

- 证据/重放：[provenance-probe.rs](provenance-probe.rs)、[首次准入矩阵与 panic](provenance-probe.txt)、[独立 replacement](provenance-replacement.txt)、[运行脚本](replay-probes.ps1)。先在意图受审提交构建具名验收，再运行脚本；脚本输出实际链接的 AGP rlib。探针字体路径是当前唯一实施树的冻结资产。
- 修复关闭条件：逐资源验证整个 content 的归属，不能用任一资源的 ordinal 覆盖另一个。初次加入与可变替换两条路径均覆盖 quad/text 的四种组合，以及原有 raw/unverifiable 控制；非法组合在目标分配/帧计数/GPU 工作之前返回分类错误，旧 target hash 保持，同设备文字正常呈现。补永久回归并冻结新提交后复审。

## F-833-R1-02 — P2：文字叠层上传未纳入实际上传计量

- 违反的既有规则：AC-06 / V-07、T-07；实际 upload bytes 计量与保留的 830 content aggregation 合同。现行 prototype-profile 明确总量等于真实分项；只豁免 Iced 内部上传。新叠层由 AGP `TextRenderer` 自己写 buffer，不属于该豁免。
- 位置：新接线 `surface_runtime.rs:902..906`；`content_uploads_of`、`content_uploads` 仍只读 quad counters。`render2d/text.rs:395,458` 两处真实 `write_buffer` 无对应计数。
- 实际入口：同设备 Game2d 无 overlay / 有 5 glyph overlay，分别通过正式 `add_surface → frame → capture`；没有 foreign adapter、没有构建期 atlas 上传混入。
- 结果：两者 `content_reported` 都为 1 rewrite / 64B，runtime delta 也均为 64B。overlay 的实际绘制路径另外写 camera 64B + 5×48B glyph = **304B**。图片 hash 从 `aefa198e…` 变成 `a86453a0…`，证明文字实际绘制。该场景正确总量应为 3 rewrites / 368B，当前少算 2 / 304B。现有 metrics/aggregation 检查只比较同一套不完整分项，仍然全绿。
- 证据/重放：[upload-probe.rs](upload-probe.rs)、[实际输出](upload-probe.txt)、[运行脚本](replay-probes.ps1)。304B 基于真实 layout 长度与执行的两个 write_buffer payload 长度，不声称测得 GPU 驻留内存。
- 修复关闭条件：在两个真实写入点记录 rewrite/字节，并让 per-content、content aggregate、runtime total 均包含叠层。正常无叠层/有文字/空 glyph/共享挂载与 warm 更新控制应精确；故意省略文字计量必须令同一正式门失败。重测并更正受影响的 V-07 数值与交接/SD 文本，保留 Iced 内部/GPU timestamp 不可测边界。

## AC → Task → 代码 / 运行证据

| AC / V | 结果 | 代码与独立核查 |
|---|---|---|
| AC-01 / V-01 | pass | T-01/03；固定 Git archive、30 hash、SDK closure/锁；仅 driver、必要 AGP 出口和四份 proposed Specs 的 diff，无 crates/root Cargo 改动。A→B 部分复用绑定证据，理由见下。 |
| AC-02 / V-02 | pass | T-02/04；Session 公开 CPU slice 调用、真实返回标签/栈与参数顺序；sdk_probe 3、numeric_probe 1、terminal_policy 2。未走高层 Nil 桩，未改 SDK。Parked 的 constructed-outcome 边界明确。 |
| AC-03 / V-03 | pass | T-05；Script → Publication → SceneConsumer → SurfaceRuntime。实测 2D 像素、3D hash、源码颜色变化、创建/移动/销毁；丢输出/错误 local/重复投递令实际门红。 |
| AC-04 / V-04 | pass | T-06；真实 Auto 执行 10800 tick / 210 exact contacts，原 1e-3 容差；实际消费端 240 tick+有序事件、速度与碰撞源码反控、碰撞后 pointer/key。Rust 路径只封送/呈现；冻结 sim/golden 未变。真实窗口160帧、161 ticks，6 requests=6 Kira plays、无错误。 |
| AC-05 / V-05 | pass | T-07 已通过部分；共享同 tick 去重、独立 VM、mesh surface 的真实 local；真实 OS 点击/键盘正控 pointer=1/key=1；focus bypass 反控 exit1，Auto x=-34、key=2。 |
| AC-06 / V-06/07 | **fail** | 两项新增 finding。既有11分支整包拒绝、代际/会话、Runnable/last-good/owner/20 reset 等控制仍通过；不能用其代替新叠层完整归属/计量。 |
| AC-07 / V-08 | partial | T-08；符号/来源/I1边界与原830增量保留正确；SD-01/03 的归属/计量当前行为尚不成立，交接必须随修复刷新。 |
| AC-08 | fail | 当前独立 R1 已完成调查，但仍有两项 open 合同 finding，T-09 不勾选。 |
| AC-09 | pending | merge-owned；未运行完整 driver test/build、六 surface/native/content/resource/full compatibility 等冻结门，未落 autographics-dev。 |

## 实际执行与证据边界

命令工作目录均为唯一833树，manifest 显式指定。未跑根 `t/tv/tt/tb/tu/taa/docs_gen/tf`。

1. `cargo test --manifest-path experimental/autographics-auto-driver/Cargo.toml --locked --offline --lib terminal_policy_tests ...`：最初附带的 integration targets 被该名称 filter 过滤为0；**只计2项 unit**，原记录 [unit-tests.txt](unit-tests.txt)。纠正后单独无名称 filter 运行下面六个具名 target，没有把0测试当通过。
2. 同 manifest `cargo test --locked --offline --test sdk_probe --test numeric_probe --test output_admission --test scheduler_lifecycle --test breakout_replay --test graphics_loop -- --nocapture`：14/14，结合上面的2项共16/16，[acceptance.txt](acceptance.txt)。其中 golden 26.11s，GPU5项9.18s。cold compile 28815us，shared P50/P95=2487/3264us，single=2238/3355us，warm pipelines增加0；reported upload89280B受F02影响，不认证为真实全量。工作集20 reset前502689792B/后444268544B，非GPU驻留值。
3. 两 workspace `cargo fmt --manifest-path ... --check` 均 exit0；driver `cargo check --locked --offline --all-targets` exit0，[check.txt](check.txt)。冻结SDK 146 warnings， authored driver/AGP无新增warning；不擅改上游SDK。`verify_source.py` exit0，完整 closure/hash核验见 baseline。
4. AGP manifest、`CARGO_PROFILE_DEV_DEBUG=0`、复用driver target，`cargo test --locked --offline --test breakout_contract --test squash_contract --test surface_contract`：11+17+29=57/57，[compatibility.txt](compatibility.txt)。这是指定兼容验收，未声称全量merge门。
5. driver `cargo run --locked --offline -- --window-probe --report .../window-positive`：exit0 / passed=true；加 `--sabotage-focus`：exit1 / 两条预期failure，[正控](window-positive/window.json)、[反控](window-negative/window.json)。真实Breakout `--sample breakout --frames 160` exit0，[报告](breakout/window.json)。
6. 外部包仍是 work-r2 指定的 `plan833-delivery-verified`，29/30 manifest条目相同；仅 `.gitignore` 少两行 receipt tracking例外，不影响构建/测试/源码依赖。外部 fresh target缺席、producer目录不可见、4项SDK/数值、30帧真实窗口与post-verify证据复用 work-r2/delivery-*：代码、依赖、测试输入未变，差异已显式审计。未为重复证明而再隐藏producer。原4项来源负控也作为不变工具的已识别证据复用；新review另核固定Git字节。

独立 rustc 探针首次链接缺 Windows native lib search path（LNK1181 `windows.0.52.0.lib`），属探针环境；加入registry下实际 x64 lib路径后重放。该次链接失败没有被计作产品缺陷。最终脚本保留相应搜索逻辑。生成的exe/pdb可由脚本重建，不入证据源码。

## 规范增量、遗漏与路由

SD-01 新文件、SD-02 integration、SD-03 project/profile 的四份全文已冻结为本目录 `spec-delta-*.md`，各 hash 与现行输入 hash 见 baseline。相对主检出，原829/830段落保持，未盲覆现行规范。SD-02 的 I1首片、Squash与I2/I3/Shader/CAP保持pending口径通过；SD-01/03 当前需要实现与实测闭合，不能把归属/计量承诺降成未支持。spec-impact metadata三个supersedes路径/一个new路径及真实GOAL-007/008保留。没有发布canonical规范或活ledger。

遗漏/延后/workaround扫描：真实Auto计算、原golden、原音画资源、两实例与共享单tick、生命周期、来源载体均有实际链路；未发现Rust代跑、改容差或新生产写面。SDK-MATH-CONVERSION已具名、有限Auto f32归一化位型与golden通过，保持独立上游候选。此次两项finding写入KNOWN-DEBT仅为追踪，**不是延期许可或验收豁免**。

下一责任：work在同树一次修复F01/F02，补反控与受影响定向验证、更新T-08交接后提交冻结，再独立复审。不要在finding迭代运行完整tier。最终review pass后，T-10才执行完整driver test/build、AGP适用六surface/native/content/resource/full compatibility等merge门；源码只落autographics-dev，master仅知识沉淀，之后再归档和guard清理。所有merge-owned门保持pending。
