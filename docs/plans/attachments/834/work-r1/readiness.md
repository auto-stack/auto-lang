# PLAN-834 work r1 readiness — 2026-10-23

实施已提交，T-01..08、AC-01..07完成；独立review未执行，T-09/AC-08与T-10/AC-09 pending。唯一权威实施树 `D:/autostack/.wt/lang-834/auto-lang` / `plan-834-dev`。base `540c1e70f0a4682af245191ffb36fa0d8a636665`；实现 `0ea8bdb1582d05e0f39c6e5bfc6d349f9280473b`；交接 `ebeab12d2888b9c9fb7e00e3c24479fb3c028011`（仅追加180项源码身份清单，不改实现）；SDK `9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`。全部锁、原golden、资源原文件保持。身份文件区分LF规范化文本hash与二进制原始hash。附件日志仅规范化换行及末尾空行；evidence-sha256按LF文本/二进制原字节计算，checkout后核文本时先将CRLF还原LF。

## 逐AC与实际oracle

| AC / V | 执行结果与证据 |
|---|---|
| AC-01 / V-01 | `plan834-source.json`、`plan834-identity.json`、`834-source-consumer-summary-final.json`：SDK680文件/17本地包/29路径边/479 metadata包，禁入0；180 authored hashes。C无target起步，A的SDK与AGP移开期间C离线冷test/build/native双游戏成功，finally恢复A。source、player GLB、SDK锁、消费锁四项破坏分别非零，逐项原字节恢复后verify通过。C为生成消费包，不是第二权威Git树。 |
| AC-02 / V-02 | `numeric_probe`、`squash_numeric_replay`、`squash_supplements`：Auto有限f32 math→公开Session→3D九分量→原GLB；原45次spawn位置/yaw1e-4、velocity1e-3，raw yaw独立核验；错序RNG失败保留已接受stream；不是Rust代跑或通用SDK数学修复。 |
| AC-03 / V-03 | `squash_admission`六例与`834-sequence-init-counter.txt`；显式profile2独立于1，连续sequence/session/代际/角色/容量/有限旋转等使用前整包检查；consumer额外资源预检。实际GPU同设备接受、跨设备/漏mesh/错误字段拒绝前无上传；失败不改last-good/scene/HUD。 |
| AC-04 / V-04,05 | 当前C `squash_profile`两例：3600tick/45spawn/91events精确，位置与原容差；重力/碰撞变异被同一oracle拒绝。`squash_supplements`五例覆盖当前玩家aim、20个旋转detector sibling/body-free fatal、复用slot按出生序至多踩一个、两个轴向raw角反例。`834-axis-heading-closure.txt`和keyframes核0/60/200/995/2703图像、3/12及actor/shadow oracle；漏模型/阴影/Auto bob/roll/scale负控红。`squash_graphics`实际原OGG播放位置跨25.195s循环，原GLB全部三mesh/原font/HUD/Retry，mute使音频门失败。 |
| AC-05 / V-06 | C `native-positive/window.json` phase9/passed=true：真实winit D/Space、text focus下z+Space（面板确为`z `，游戏投递不变）、死亡Space+repeat/Enter fresh retry无旧jump、独立实例不受影响；negative同一oracle phase9/passed=false，精确报text focus keys/session变化。`squash_clock_input`四例：120Hz batch1/3结果相同、共享挂载单tick、focus/blur/unmount清键、无输入不自跳、init前排队tick只计一次。 |
| AC-06 / V-07,08 | `834-code728e-gpu-clock-stability.txt`：真实Runnable长计算触发预算且另一实例继续20tick/stock Pause仍响应；20 reset VM实际weak drop、缓存/管线/上传稳；mesh与music外部合法pin存续、末owner drop后weak失效；模型上传每实例137124B/16writes，文字/3D分项与总量核对，漏计变异红。`834-stability.json`三个连续60tick区间，selected owned57065320B、5个target9109600B；CPU wall P50/P95与进程工作集分列，非因果比较、非GPU时间。 |
| AC-07 / V-09, SD-01..04 | C `breakout_replay`原10800tick/210contacts及反控，原profile1/admission/graphics适用目标仍绿；两游戏native真实运行。工作树README、feasibility及5份拟议Spec全文可审：原4份canonical正文保留再追加，新增auto-3d-game-profile。I1只提供候选双游戏证据，仍需review/merge审定；I2/I3/Shader/CAP/Forge生产门未认领。 |

## 命令与版本绑定

在实验driver目录设置`RUST_MIN_STACK=16777216`。Windows原生构建显式`RUSTFLAGS=-C link-arg=/STACK:33554432`；C最终exe解析PE实际为33554432。全部cargo使用各实验workspace自己的manifest与锁，非根workspace。

- 当前实现0ea：C `python tools/verify_source.py`；`cargo test --locked --offline --test sdk_probe --test numeric_probe --test squash_admission --test squash_profile --test squash_supplements --test breakout_replay -- --nocapture`；`cargo build --locked --offline --bin autographics-auto-driver`。原生exe `--sample squash --window-probe --report ...`、加`--sabotage-focus`负控、`--sample breakout --frames 90 --report ...`。见834-c-isolated-*、native-*与summary-final。
- 当前实现0ea：A定向`--test squash_supplements --test squash_profile --test squash_visual_oracle`通过，见axis-heading-closure；两个轴向回归先在728上红，见axis-heading-red。
- 冻结728：A定向`--test squash_graphics --test squash_clock_input --test squash_stability --test squash_visual_oracle`以及其余profile/admission/numeric/旧graphics目标，见code728e-gpu-clock-stability、frozen-scoped、sequence-init-counter、fixed-resource-identity。0ea仅修改ag_heading数学与其两个永久回归、可行性说明：资源/音频/预算/计量/时钟代码、依赖/配置/输入未变，复用其对应证据；受影响Auto数值/图像/轨迹/真实输入全部在0ea重跑。元数据提交ebeab仅加入身份清单，无实现变化。
- 两workspace `cargo check --locked --all-targets`通过（AGP复用driver target），见frozen-check/final-check-r2/agp-all-target-check。SDK既有警告146条；未引入本次源码警告。两workspace `cargo fmt --manifest-path .../Cargo.toml -- --check`均通过；`git diff --check`通过。`fmt --all`会递归SDK内独立vendor workspace导致multiple roots，故采用各workspace自身格式检查，不改SDK。
- 定向门未skip，负控panic由测试捕获并断言，native错误焦点非零为预期；非实际产品错误。原生早期helper失败历史与修复在feasibility中保留，未放宽原玩法/图像容差。

## 内部闭合与边界

| 发现 | 不变量/修复/永久检查 | 剩余合同gap |
|---|---|---|
| +PI与-PI矩阵等价曾漏检raw yaw | Auto ag_heading零轴保留冻结-PI；movement/spawn两个raw value回归，加完整原轨迹和图像重新验证 | 无 |
| init前已排队tick统计会多计 | active_tick只在真实tick响应计数；permanent clock初始化排队用例 | 无 |
| RNG候选失败不能暗推进 | candidate stream仅整包通过后commit；错序及失败流回归 | 无 |
| slot复用可能改变碰撞次序 | birth-order列表；复用slot sibling与至多一次squash | 无 |
| consumer资源校验晚于发布 | 所有mesh/text/overlay/pipeline及device使用前预检；错资源保持last-good且无上传 | 无 |

开放合同finding：空；必须补测coverage gap：空；未决合同问题：空；未授权延期/workaround：无。原SDK通用数学转换问题仍属既有`SDK-MATH-CONVERSION`债务/本合同非目标，有限Auto算法已按本合同正反控核验，未以此延后任何834任务。预算为callback100000 instructions/120 resumes/250ms、slice1000/2ms、pump8ms加单slice/OS调度；65对象/64mob、130events、64input/64output有界。继承single-pass有限物理，非通用Godot/骨骼/任意模型承诺。

## 下一角色与保留物

独立review在唯一树冻结ebeab逐AC/SD与实际代码核验，勿并行写入；全部work改动已提交、tracked/untracked delivery clean。忽略的build/research产物保留。`consumer-b`历史包与`consumer-c`当前包保留供复现；它们是组内无Git生成物，最终merge清理须先确认路径/链接扫描，且任何worktree删除前强制wt-guard clean。

merge-owned pending：最终独立review pass后，在冻结提交统一完整driver test/build、AGP适用兼容/六surface/真实窗口音画输入与资源负控；合autographics-dev，Specs/索引/ledger沉淀、归档、guard clean收树。work未执行根cargo t/tv/tt/tb/tu/taa/docs_gen/tf，也未代跑完整driver/AGP门。本记录不是独立review pass或最终交付声明。
