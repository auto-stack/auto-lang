---
plan_id: PLAN-829
status: archived
feature_name: AutoGraphics独立调研与2D/3D游戏原型
plan_revision: 1
author: [Codex]
created_at: 2026-10-21
updated_at: 2026-10-22
current_step: 11
total_steps: 11
supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-graphics/project.md
  - docs/specs/auto-graphics/prototype-profile.md
  - docs/specs/auto-graphics/integration-contract.md
touched_goals: [GOAL-008]
affects: [auto-graphics, auto-lang/ui, repository-boundaries]
---

# [PLAN-829] AutoGraphics 独立调研与 2D/3D 游戏原型

## 0. 变更摘要

按用户 2026-10-21 授权制定独立并行的调研/原型计划。首版底层固定 **wgpu**，首批选 **AG-2D-01：Bevy Breakout**、**AG-3D-01：Godot Squash the Creeps**，以及 **AG-MIX-01：公开 Iced＋2D/3D 视图的共享设备探针**。

顺序为“共同基础先行，2D 先闭环，3D 提早探针、共同接口稳定后分轨推进”。不等全部 2D 完成才开始 3D，也不允许两轨各建一套共同基础。设计依据：[Design45](../design/45-autographics-wgpu-engine-and-game-prototypes.md)。

本 Plan 的完成范围是**独立引擎研究与 Rust driver 原型**、可复跑证据及迁移后接线合同。真实 Auto 游戏入口、生产 AutoUI 多层嵌套、RQHost 和 Forge 产品编辑仍为后续集成验收，不因本 Plan 完成冒称交付；v0.6 对它们的目标继续保留。本轮只制定计划，不启动实现。

## 1. 目标

1. 独立于三仓迁移和 ACE，在单独实验 workspace 建立可运行的 wgpu 图形/游戏基础，保持无语言/UI/Man 私有依赖。
2. 用两款明确来源的完整游戏样本盘点并补齐本轮能力，交付真实可玩的 Rust 驱动原型，玩法/视觉/音频及输入回放有证据。
3. 用一个真实 Iced 最小宿主证明同设备 UI 与图形合成路径，记录仍缺的生产接线，不复制现有 UI 实现。
4. 形成公开的轻量命令/资源/输入/生命周期合同及后继交接清单，明确迁移后的 owner、入口、前置和验收。

支持 profile：首轮 **Windows x64 原生窗口、键鼠**；记录实际 GPU/API/驱动、窗口尺寸与 DPI。手柄、浏览器、Linux 实机、移动端和生产发行非本 Plan 交付；原样本的其他控制形式列为已知 profile 差异，不伪称全平台/全外设复刻。

非目标：完整 Godot/Bevy API/ECS 兼容、运行原引擎、第二渲染后端、重写现有 AutoUI、生产 RQHost 接线、修改 ACE、完整物理/动画编辑器、Tilemap/骨骼/联网/XR 全能力、正式 AutoForge/自研国风游戏或工具链默认切换。选定样本实际需要的动画、碰撞、声音和阴影不因这份非目标被删除。

## 2. 架构方案

新 `experimental/autographics-prototype/`（以下简写 `AGP`）为独立 Rust workspace，有自身 manifest/lock、资源与测试。最终生产归属为 auto-ui 图形域；本 Plan 不搬仓、不改根 workspace/锁、不接当前生产 UI。

```text
Rust games / 固定动作回放         公开 Iced 最小探针
          ↓                             ↓
共享 model：对象/资源身份、时间、动作/事件、场景变更
          ↓                             ↓
2D renderer + collision     3D renderer + character/collision
          └───────── wgpu renderer / target / compositor ───────┘
               独立 winit 宿主 / Iced 提供的 Device/Queue
未来：公开 Auto SDK adapter、迁移后 AutoUI host、RQHost protocol adapter
```

共享 model/API 不暴露 wgpu/Iced、AST、Value 或 VM 栈类型。renderer/host 持有 GPU 资源；游戏逻辑仅提交场景/资源变更并消费动作/事件。2D/3D 保留维度专属算法，复用身份、时间、事件和资源机制。

有限选型只比较本轮所需场景存储、模型/动画、文本、音频和碰撞实现，不重新选择整体引擎。每类至多比较三个实际可用方案，以两款样本的 whole-path 验证作决定；阻碍原目标则携证据回 new，不能临时缩样本。

## 3. 技术栈

- 渲染固定 wgpu；按当前 Iced 适配依赖对齐同一个版本实例。规划读取的根锁为 wgpu 27.0.1、winit 0.30.13、Iced 0.14；work 进入时重核真实版本和 patch，只调整实验锁，不自动升级全仓。
- 原型语言 Rust；独立宿主 winit，真实 Iced GPU 探针复用公开 Primitive/Device/Queue 入口及必要既有 patch 来源，不复制 patch 源或 UI 代码。
- 数学、模型加载、文字、音频、碰撞候选在 T-02 的有限实验后选定并锁版本。glTF/GLB、关键帧/变换动画按样本真实资产决定；不预先要求骨骼系统或通用刚体引擎。
- 纯状态/碰撞/身份测试与真实 GPU/窗口/音频验证分开；Linux/Web/移动性能不从 Windows 外推。

## 4. 需求分析与背景调查

### 4.1 授权与当前状态

用户已明确：wgpu 与 Iced 的底层融合；以 Godot/Bevy 游戏示例复刻驱动图形/游戏功能；图形调研原型与迁移独立并行；本轮请求制定计划并判断 2D/3D 顺序。授权范围为本仓规划/设计与必要只读调查；未授权本轮开实现、建 agent/chat、外仓写入、发布或修改在途 Plan。

主检出 `D:/autostack/auto-lang`、master；调查 HEAD=`bf92f4585d8c23a0d26ef7bed26719d90b0168bd`，工作区已有其他文档/计划 WIP，未覆盖。计划通过仓库 `scripts/new-plan.sh` 在排他取号锁内创建，829 唯一，下一号 830。沙箱内 Git Bash 启动失败，重试脚本成功后复核编号；首次失败没有取号副作用。

### 4.2 本地证据

| 来源 | 本轮核实的事实 | 版本边界 |
|---|---|---|
| `docs/specs/overview.md` | 实验模块可采用独立 workspace；当前项目仍是多目标语言/UI 体系 | 页面总更新时间较旧且有滚动条目，不能据目录表推断迁移完成 |
| `docs/specs/auto-lang/ui/overview.md` | M814-A 只交编译/SDK 子集，runtime/renderer 尚有后继迁移；不存在 `ui/project.md`，现行入口为本 overview | 当前文件及 §M814-A；以实际收据重新取证 |
| `docs/specs/repository-boundaries/project.md` | 沿用旧仓身份、保全与受控迁移；禁止绕开迁移直接替换默认实现 | 现行治理合同 |
| `crates/auto-lang/src/ui/desktop_protocol/rqhost.rs`、`docs/design/autoui/rq-remote-renderer.md` | 当前有 CPU 记录/重放、rqhost 缺省软件绘制，与同进程 wgpu 组合不同 | 现行实现，生产接线不在本 Plan |
| `crates/auto-lang/src/ui/mpv/present.rs`、`widget.rs`、`crates/auto-lang/Cargo.toml` | 已有持久纹理/custom Primitive 先例，通过 iced_wgpu 使用同一 wgpu 版本 | 只读复用经验，不搬代码或增加生产耦合 |
| `experimental/ac-core/Cargo.toml` | 独立 `[workspace]` 原型先例 | 不复制 AC 逻辑或依赖其 WIP |
| `examples/godot/README.md` | 当前五例为 GDScript→Godot 项目，不证明自有 wgpu 游戏引擎已存在 | 保留旧资产，不以旧说明冒领新引擎能力 |
| `docs/design/strategy/autographics-forge-v06.md`、`docs/roadmap-v0.6.md` | 已确认目标态；不是功能完成证明 | 2026-10-21 月度基线；本 Plan 细化独立原型部分 |

相关来源内容 hash 见 [规划预检](attachments/829/planning/preflight.md)。未找到同目标的活动 Plan。`docs/specs/auto-graphics/` 尚不存在；新模块事实只能在实现验收后按 SD 表沉淀，不能现在写成 current Spec。

### 4.3 首批来源锁定

| 样本 | 不可变来源 | 本轮观察与 T-01 待盘点 |
|---|---|---|
| AG-2D-01 Bevy Breakout | `bevyengine/bevy@efe61cc5b9872f3f9e1ee7553325612c094ccc97`，`examples/showcase/breakout.rs`，blob `b9bfb8b4b2cd19aa15b460f1323ad5c2af0e5954` | 核实图元、输入、反弹/消砖计分与碰撞音效；底墙反弹，无原生掉球失败/生命系统。T-01 补全资产/字体与行为分母；不按常见打砖块玩法臆造原例规则 |
| AG-3D-01 Godot Squash the Creeps | `godotengine/godot-demo-projects@b761b4cd5718a7118f2c95eb55f181d0a5f004f1`，`3d/squash_the_creeps/`；README blob `8006f2783acf3ad8baceb63a80269b36b0c16138`，Main.tscn blob `105c56d67284326f5efdb5caf426c0f063f9f255` | 已核实移动/跳跃、生成敌人、计分/失败重开；场景有模型、光照/阴影与环境。T-01 补齐脚本/动画/资产及实际音频使用清单；不能仅看 README 版权表推断资源都在播放 |

来源：[Breakout 冻结源码](https://github.com/bevyengine/bevy/blob/efe61cc5b9872f3f9e1ee7553325612c094ccc97/examples/showcase/breakout.rs)、[Squash 冻结目录](https://github.com/godotengine/godot-demo-projects/tree/b761b4cd5718a7118f2c95eb55f181d0a5f004f1/3d/squash_the_creeps)。本轮只读源码/场景，没有运行原游戏。

## 5. 详细设计

### 5.1 当前成果与迁移后成果

| 阶段 | 本 Plan 是否负责 | 退出证据 |
|---|---|---|
| P0 调研/支持面与选型 | 是 | 冻结样本能力/资产、实际依赖、平台、差异及资源/测量策略 |
| P1 独立图形/游戏原型 | 是 | 两个 Rust driver 游戏真实运行与回放、渲染/音频及生命周期证据 |
| P2 最小 Iced 共享设备探针 | 是 | 真实 Iced 控件和两类视图共用实际设备/队列，输入与关闭正确 |
| I1 真实 Auto 源码游戏入口 | 后继集成 Plan，仍 pending | 两个 `.at` 示例通过公开 SDK 控制同一 core、处理真实动作/事件；Rust 原型不抵扣 |
| I2 生产 AutoUI/嵌套/RQHost | 后继集成 Plan，仍 pending | 迁移完成收据、真实宿主/协议与 UI 多层组合；独立 Iced 探针不抵扣 |
| I3 Forge 与发行 | 后继产品/集成 Plan，仍 pending | 实际运行场景的编辑/保存/撤销与相应发行闭包 |

I1–I3 不是本 Plan 的已完成项，也不是 merge 可以代做的实现；T-08 必须给出各自验收/owner 与入场条件，后继在迁移收据具备后按 new 取号，不能仅写“以后接一下”。

### 5.2 能力归属与并行门

| 工作面 | 需要补齐的能力 | 前置/写面 |
|---|---|---|
| 共用基础 | GPU/离屏目标、资源身份/缓存/释放、场景变更、动作/时间/事件、重置及回放 | 一个 owner；`AGP/src/core/`、`renderer/`、`host/`（均新）；设备与资源合同确认后才能分轨 |
| 2D Breakout | 正交视图/图元、碰撞与方向反弹、得分/文字、音效、附加 reset 宿主入口 | T-03 后推进；`AGP/src/render2d/`、`games/breakout/`（新）；reset 标为宿主扩展，无须制造原例没有的失败条件 |
| 3D 风险探针 | GLB/网格与材质、相机/深度、光照/阴影/环境、动画及角色碰撞可行性 | T-02 后即可开始，无须等待完整 2D 游戏；`AGP/src/render3d/`、`assets/`（新） |
| 3D Squash | 角色移动/跳跃与落地、敌人生命周期、踩踏/侧碰语义、计分与失败/重开、原例所需画面/动画/声音 | 共享合同及风险探针通过后；`AGP/src/games/squash/`（新） |
| Iced 接缝 | 同设备多 pass、图形目标与 UI 合成、颜色/alpha/裁剪/输入/resize/释放 | T-02 后可独立实验；`AGP/src/bin/iced_spike.rs`（新），不修改当前 AutoUI |

**G-base**：T-02/T-03 已形成实际设备/资源与动作/时间合同，3D 深度目标纳入同一设计；不是纯接口草图。G-base 后 2D 与 3D 独立模块可重叠推进；同一公共文件变更串行。同 Plan 同一树单写，不自动启动两个 agent；需要多人并行实现时先按已稳定的模块边界分解后继 Plan，而非多人同时改同一树。

### 5.3 资源与更新合同

对象/资源 ID 带会话或代际，不复用失效身份绑定新对象；销毁后拒绝迟到变更。局部属性/资源修改只更新相关对象/上传，未改资源身份保持，不能每帧重建所有纹理或 Device。
零尺寸/最小化、缺文件/坏模型/坏 shader、失效引用、重复释放及关闭中取消有明确结果；失败不覆盖最后可用场景，过期事件不影响新会话。

T-02 以实际样本确定支持的模型/纹理格式与规模、视图/帧/资源队列策略，以及可观察的拒绝和清理行为，写入 `research/support-profile.md`。它不能改变已选样本/平台或删除用户要求；如样本不可支持则带反例回 new，禁止把性能/显存未知改成无限支持。性能先记录基线，预算在 T-02 冻结后再开展后续负载验证，不预先编造 FPS/显存成绩。

### 5.4 迁移后集成合同（当前负责研究，不负责生产改造）

| 接缝 ID | 后继 owner/消费方 | 入场门及当前需交付的合同 |
|---|---|---|
| IH-01 GPU/Iced | auto-ui renderer/host owner，AutoGraphics renderer owner | 迁移完成及公开版本收据；同 Device/Queue 的借用/生命周期、纹理格式/alpha/深度、提交顺序、resize/退出；Iced 不持游戏业务状态 |
| IH-02 Auto SDK | 语言 SDK owner + UI/图形 adapter owner | 实际已发布 SDK/profile；动作输入、场景增量/资源引用、事件/错误、取消与更新；禁止 core 反依赖 VM/AST，不能预设 ACE 全语言完成 |
| IH-03 RQHost | 迁移后的 host/protocol owner | 明确场景状态、GPU、资源驻留和输入权威；版本/实例/背压/断连/资源退役；同进程 GPU 引用不直接跨进程。现有 CPU 路径兼容与图形启用策略单列 |
| IH-04 嵌套/Forge | AutoUI 输入/布局 owner + Forge owner | 实际 UI→3D→UI、3D→UI→2D；坐标/射线/UV、焦点/文本/IME及修改回退；保存后重开一致 |
| IH-05 发行/仓界 | auto-ui/auto-man 发行 owner | 源码/锁/patch/资源/producer来源、独立构建与重定位/失败恢复；不建 symlink/junction，不改旧仓历史或既有默认入口 |

“迁移完成”须由迁移负责人提供当前迁移路线的完成/Spec/冻结版本与写面交接收据；814-A 等局部发布不能替代。原型代码保留在独立分支直到交接门明确，再按审查模块合入，不一次性覆盖迁移后文件。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-graphics/project.md` | 当前无本模块；后续只登记 experimental 原型的真实入口、样本/profile、归属与未交付项 | 防止把旧 Godot/目标设计当成引擎实现 | AC-01、AC-03、AC-04、AC-08 |
| SD-02 | add | `docs/specs/auto-graphics/prototype-profile.md` | 新增真实支持面、资源/更新/失败与生命周期、复跑/测量方法 | 固定原型可用能力及证据边界 | AC-02、AC-05、AC-06 |
| SD-03 | add | `docs/specs/auto-graphics/integration-contract.md` | 记录已验证的独立 Iced 接缝与待实施 IH-01..05，明确生产 Auto/UI/RQHost pending | 后续有可消费交接，不伪报集成完成 | AC-07、AC-08 |

以上是 provisional 增量，review 冻结、merge 才写 current Spec/ledger。现有 UI/仓界规范不改语义，不把本稿变成生产协议；若实现需要改动它们，先修订合同。

## 6. 测试设计

下列均为**拟建验证入口**，由 T-02/T-03/T-04..07 建成；本次未执行。`AGP` 展开为 `experimental/autographics-prototype`，工作目录为唯一实施树根。命令只针对此独立 workspace，不调用根 `cargo t/tf/docs_gen`。

- `cargo check --manifest-path experimental/autographics-prototype/Cargo.toml --locked`
- `cargo test --manifest-path experimental/autographics-prototype/Cargo.toml --locked --test prototype_contract <test_filter>`：纯状态、身份/事件及反例；test_filter 使用 Rust 测试名（如 `shared_runtime`，不把含连字符的 runner case_id 原样当筛选器），预期至少命中一个具名案例且相关断言通过，零匹配不能作证据。
- `cargo run --manifest-path experimental/autographics-prototype/Cargo.toml --locked --release --bin graphics-probe -- --case <case_id> --replay canonical --report scratch/829/<case_id>`：真实绘制/游戏/资源验证，预期非 skip 的原始报告、关键帧与状态/设备事实齐全；失败非零。
- `cargo run --manifest-path experimental/autographics-prototype/Cargo.toml --locked --release --bin graphics-probe -- --case verifier-negative-controls --report scratch/829/negative`：T-07 建成，反控分别被实际 gate 拒绝时汇总成功；误绿、未执行或 skip 为非零。

| V ID / case_id | AC / 不变量 | 输入及入口 | 可区分的期望与证据 | 边界/任务 |
|---|---|---|---|---|
| V-01 / gpu-foundation | AC-02 | 两个独立更新目标、真实设备，resize/最小化/恢复 | 实际绘制和尺寸正确；同一宿主设备/队列身份一致、未改资源保持；不靠伪造的“shared=true” | Windows 实测；T-02 |
| V-02 / shared-runtime | AC-02、AC-05 | 固定动作/种子/步长，同一回放重复；重复/迟到/失效 ID | 相同输入产生约定相同状态/事件；失效 ID 被拒绝不改当前场景，局部编辑不重建未改资源 | 不承诺跨 GPU bitwise 像素相同；T-03 |
| V-03 / ag2d-breakout | AC-03 | 真窗口键鼠＋canonical 回放：墙/球拍/砖接触、重复 reset | 位置/反弹方向、消砖与计分、可见 HUD 和真实音效吻合源样本；reset 清理重建不保留旧事件 | 原例无掉球失败；Bevy stepping 工具不在玩法 profile；T-04 |
| V-04 / ag3d-squash | AC-04 | 移动/跳跃、踩踏和侧碰、失败/重开，固定敌人序列 | 角色/碰撞方向、计分只一次、失败停止/重开恢复；模型/动画/光照阴影与所需音频实际运行 | 纯 Rust driver，真实 Auto 尚未交付；T-05/T-06 |
| V-05 / lifecycle | AC-05 | 两游戏各重开20次、视图增删、缺模型/坏 shader、关闭中加载取消 | 除已登记有界共享缓存外，所属活对象/纹理/任务回到 warm 基线；退出无遗留 owned worker；失败保留可用场景。记录 GPU 完成点及资源/进程实际观察 | 不用 RSS 单点代替资源释放；T-07 |
| V-06 / iced-shared-device | AC-07 | 真实 Iced 控件＋2D/3D 目标，点击/裁剪/透明/DPI/resize及关窗 | 控件动作真实改变目标；实际共用 Iced 提供的设备/队列；未通过则不能用分开窗口/两设备冒充 | 非生产 AutoUI、非完整任意嵌套；T-08 |
| V-07 / metrics | AC-06 | 固定提交/锁/资产/设备与回放，cold/warm，多个视图 | 原始 CPU/GPU frame P50/P95、上传/重建量、RSS与资源计数、启动/重开/退出；GPU计时不可用明确标记，不报虚构数据 | 无跨平台/竞品性能结论；T-07 |
| V-08 / verifier-negative-controls | AC-05、AC-06、AC-07 | 向实际 runner 注入漏计分/错误踩踏、空白帧或未驱动交互、第二 Device、迟到事件四类反例；各有合法正控 | 对应正式断言确实失败且原因匹配；正控仍过。验证器不能仅信游戏自行打印 PASS/0 计数 | 仅冻结样本与接缝，不做通用引擎验证器；T-07/T-08 |
| V-09 / independent-consumer | AC-01、AC-08 | 独立 manifest 的实际依赖闭包、锁定构建和A→B重定位、生产文件差异 | 不需当前 auto-lang/ui/man 私有源码；根 workspace/锁和其他仓零改动；薄消费 model/renderer 分层成立 | Iced patch 通过记录的来源/外部位置解析，无链接绕路；T-09 |

每例保留实际 commit、锁/资产 hash、adapter/backend、原始 stdout/stderr/关键帧与事件 trace。玩法数值误差及图像容差在 T-01/T-02 的人工 source oracle 中冻结，不让实现自己改 oracle；音频同时取真实播放路径证据，触发计数不能代替实际音频链。
不要求本轮全平台像素一致，也不以此免除选定 Windows profile 的真实 UI/图形结果。强制案例 skip/环境不可用仍为缺口；GUI/GPU 全局资源与机器重型测试避开并行竞争。

角色：本轮仅 new 起草与规划预检；用户未指定实现/内部检查/最终复审人或模型，不创建其他会话/agent。后续 work 实现，独立 final review 在同一实施树按冻结提交验收，merge 执行冻结门与沉淀。当前自查不冒称独立 final review。

## 7. 验收标准

| AC ID | owner_stage | 可观察完成标准与验证 |
|---|---|---|
| AC-01 | work | 独立 workspace/锁/资产来源与依赖禁向明确，原样本冻结库存可追溯；V-09、样本 manifest 与来源核查 |
| AC-02 | work | 共用 GPU/离屏/资源/输入/时间基础实际成立，2D/3D 不各造设备与身份；G-base 与 V-01/V-02 |
| AC-03 | work | AG-2D-01 按冻结 profile 可游玩、原玩法/计分/音效和宿主 reset 正确；V-03 |
| AC-04 | work | AG-3D-01 按冻结 profile 可游玩，移动/踩踏/侧碰/失败重开及画面/动画正确；V-04；样本所需声音/阴影不得静默省略 |
| AC-05 | work | 局部更新、失效/取消、重开/关闭与资源清理通过，错误实现可被反控区分；V-02/V-05/V-08 |
| AC-06 | work | 固定环境性能/资源报告与能力缺口齐全，失败/skip不报通过，验证器有实跑正反控；V-07/V-08 |
| AC-07 | work | 真实 Iced 最小探针同设备合成/交互/resize/退出通过；V-06及第二Device/空画面反控 |
| AC-08 | work | P0–P2 与 I1–I3 分别报告；IH-01..05 有owner职责、实际/待交付入口、样例/验收及迁移门。I1–I3明确pending，不标记正式Auto/生产UI/发行已完成；V-09与交接文档审查 |
| AC-09 | review | 独立逐项验AC/SD、差异/未授权延期/警告/格式与调试残留；冻结revision/代码证据，merge-owned门pending明确，非仅复用作者PASS |
| AC-10 | merge | 最终review通过后在冻结提交重放本实验workspace适用门、交接前置核对后受控合入/沉淀/归档；无实现/依赖变更才复用证据，不把pending集成写成已实现 |

## 8. 执行步骤

所有新实现路径见下表，**现在均未创建**；不把拟议命令视为可执行证据。任务状态只维护本表；future review/merge 保持未勾选。work 完成 T-01..09 后可进入 execution_done，T-10/11 不作为 work 已完成的前置。

**当前状态（2026-10-22 affected final review R18）**：**PASS / reviewed**，冻结 `ddbde3e62857e6bbfc8b780c6182ee1bb7d4ef74`，详见 [R18报告/正常失败退出/当前正负控](attachments/829/review-r18/review.md)。F829-R14-01/02全部CLOSED：缺ID原生反例具名超时后正常返回exit1（不再panic101），旧冻结时钟/回调/已知ID关闭保持；result与E21历史更正一致。58定向全过，当前原CLI正控0/三时刻全绿、透明负控重跑1/三时刻全红（首轮点击未送达记录保留，不计完整负控）；fmt/check无warning。旧几何19反例/真实resize按完全未变函数与输入hash限定复用。T-01..10完成，`current_step: 10/11`；T-11冻结AGP全适用门/真实consumer、迁移写面放行、规范沉淀/归档/guard清理pending，尚未交付或合入。
**merge T-11 门禁执行（2026-10-22，merge 会话）**：冻结 `ddbde3e62857`（树 clean、127 文件 diff 全限 AGP、零根触面故不触发根档）上全部**可执行** merge-owned 门禁通过——fmt/check 全 0 无 warning；全适用测试 **117/117**（lib29+prototype16+squash17+composite_gate17+iced_seam5+delivery2+breakout11+threed10+cache3+verifier7+2 空 bin unittest）；十探针矩阵 **10/10 exit 0**（iced-shared-device 真窗口含真实输入 exit0/composite ok=true，independent-consumer 真实 A→B 重定位冷构建 `ok=true/mismatches=[]`），全报告指纹 `ddbde3e62857`、锁 `6cd34576…`；证据 [merge-t11/gates.md](attachments/829/merge-t11/gates.md)（报告存 `.json.txt`）。剩余唯一阻塞=迁移/写面交接放行（T-11 前置，用户/迁移负责人裁定，merge 不可自决）；放行前不 land、不沉淀、不归档、不清树，计划保持 `reviewed 10/11`。

| Task / 状态 | owner_stage | 前置 | 写面（AGP=新experimental/autographics-prototype） | 结果、AC及验证 |
|---|---|---|---|---|
| T-01 [x] | work | 用户确认本r1实施；核对实际主线/树与来源 | `AGP/research/sample-manifest.md`、`testdata/`、`assets/`（新），主检出本Plan/证据 | 运行/盘点冻结原例、键鼠/视觉/音频/规则/资产与许可，固定oracle/容差/回放；补新来源hash，不改样本目标；AC-01/03/04/08。[✅ 2026-10-21 work：commit 6c2448261，manifest+evidence-notes+testdata(22文本,SHA256全过)+assets(6二进制,~1.8MB)；三个blob hash全 MATCH；oracle冻结提案落 manifest（固定步长/种子/位置与图像容差/音效映射）；§4.3 修正 10 条录 evidence-notes（音乐为 autoload 实际播放、GLB 无动画/纹理、120Hz Jolt 物理等）。r2：F07 项下 manifest 就地勘误 seed 0x1B/60 tick 与冻结 trace 对齐（1fc3f6116）] |
| T-02 [x] | work | T-01 | `AGP/Cargo.toml`、`Cargo.lock`、`src/core/`、`renderer/`、`host/`、`src/bin/graphics-probe.rs`、`research/support-profile.md`（新） | bounded选型、same-device/离屏及3D深度whole-path探针，确定支持/资源/失败策略和预算；V-01，AC-01/02/05/06；依赖版本/patch可重放。[✅ 2026-10-21 work：commit 8fd36c7ac；版本与生产 iced_wgpu 对齐（wgpu 27.0.1/winit 0.30.13/iced 0.14.0 实测一致，glyphon 无 wgpu-27 线故 fontdue 0.9.5+自研 atlas，kira 0.9.6，gltf 1.4.1，自研碰撞，arena+代际ID 无ECS框架）；真实窗口+离屏双目标实测（RTX 4060 Ti/Vulkan/驱动617.14，同 device token，resize/最小化/恢复、缓存0重建、5类失败注入全按分类拒绝）；headless 回退关键帧哈希 bit-identical；probe 自抓一处相机上传 bug 已修；support-profile 含预算/失败分类/依赖决策表/GPU 事实。r2：F04 缓存 owner+代际绑定与 ID 回收（b5c1ed1af）；F09 去 vendoring、stock iced_widget 0.14.2 实测满足、AGP 零外部 path 依赖（1fc3f6116）] |
| T-03 [x] | work | T-02 | `AGP/src/core/`、`tests/prototype_contract.rs`、`testdata/`（新） | 对象/资源代际、动作/固定时间/事件、场景增量/回放；建立G-base，V-02/相关反例；AC-02/05。[✅ 2026-10-21 work：commit f4d537a6f；Runtime/Driver/ChangeLog/session 代际 RNG（xorshift64* 种子冻结）；16 具名契约测试全过（两次独立回放 194 事件精确一致、changelog 重放==现场状态、3 个反例守护实证会红、局部编辑不动未改资源 revision）；probe `shared-runtime` windowed exit 0，双回放 snapshot sha256 一致、0 重建、3 失效ID拒识、gpu_timing 明确 false；发现并修正 changelog 跨 session 重放语义 bug（符合 T-01 身份合同，oracle 未动）] |
| T-04 [x] | work | G-base | `AGP/src/render2d/`、`src/games/breakout/`（新） | 2D完整原型及reset、文字/音效，V-03；AC-03。可与T-05/06在模块层重叠，不抢公共写面。[✅ 2026-10-21 work：commit e09da38e1；冻结源逐项保真（布局/速度/条件反射/paddle clamp ±375/56砖计分/无底墙失败——零臆造）；golden trace 由独立 Python 参考实现生成冻结，Rust 回放 bit-exact（max diff 0.0，10800 tick 全清）；probe `ag2d-breakout` windowed exit 0，210 接触=210 音效（kira 真实播放链），4 关键帧，3 轮 reset 复现基线；字体偏差有据（冻结树无字体，用同树 FiraMono+OFL，E11）；修正 T-01 manifest paddle clamp 笔误；39 测试全过；live 模式可玩。r2：F02 LIVE 时钟改 elapsed-time 累加器（cadence 不变性测试，canonical 不动）（03f4999f4）；F07 内容金帧逐像素门（1fc3f6116）] |
| T-05 [x] | work | T-02；模型/输入/资源合同冻结 | `AGP/src/render3d/`、`src/assets/`、`research/3d-feasibility.md`（新） | 原模型/材质/相机/深度/光照阴影/动画/角色碰撞风险切片；不等T-04完成，失败不能降级立方体冒领；AC-02/04。[✅ 2026-10-21 work：commit e3b8df830；七项风险全 GO 且有真实冻结资产证据（GLB 精确网格/材质事实、正交相机 size24、2048² PCF 阴影 on/off 关键帧可视验证、Player/Mob.tscn 内联 bob 动画按冻结参数、sample 碰撞方案无物理引擎复现 TopLandingSquash/Fatal）；probe windowed exit 0（draws 20/uploads 8/0 重建）；关键帧目检抓出阴影相机朝向 bug 已修；诚实记录 3 项近似与 T-06 开放项（生成高度/WorldBoundary 单边语义/move_and_slide 多 pass/种子流/音乐/计分HUD）；mob GLB 隐藏旋转四元数以 full-TRS 烘焙修正] |
| T-06 [x] | work | G-base、T-05 | `AGP/src/games/squash/`（新） | 3D玩法/生命周期与真实视觉、动画及所需音频；V-04，AC-04。对公共核心变更经owner串行发布。[✅ 2026-10-21 work：commit 235ff1b58；冻结源逐项保真（移速14/跳跃20重力75/120Hz、0.5s 生成周界均匀 y=1 朝向±45°、屏幕出口 despawn、顶踩 exactly-once、死亡停表+Retry遮罩 ui_accept、死后世界继续运行、音乐 autoload 循环真实播放）；两处任务措辞与源冲突以源为准并记录（mob 不弹墙 mask=空层、无 squash 缩放动画）；golden trace 独立 Python 参考实现（种子扫描定 seed 0x8 恰好3踩后死亡），Rust 回放 3600 tick bit-exact、47/47 事件精确；probe windowed exit 0（3轮 retry 快照一致、4关键帧像素级验证含 sRGB 遮罩）；62 测试全过。r2：F01 LIVE 跳跃键按压边沿状态机（03f4999f4）；F03 生成瞄准实体位置+MobDetector 五路致命通道+朝向/动画速度/跳跃俯仰/mob 本地相位，golden 语义变更再冻结 seed 0x1B（03f4999f4）；F11 全量接入共享身份/场景增量/change-log+独立消费者重放编辑验证（03f4999f4）；F07 全 payload 事件/spawn 元组比对+Godot 4.4.1 原始引擎 3600 tick 地面真值（角色级为 R2 决策项）（1fc3f6116）] |
| T-07 [x] | work | T-04、T-06 | `AGP/tests/`、runner与`research/measurements.md`（新） | V-05/V-07/V-08；原始回放/画面/声音、资源/性能与反控；AC-05/06。未通过不转成仅统计完成。[✅ 2026-10-21 work：commit 52895fefe；lifecycle：20重开×2游戏 0 session 违例、视图10轮增删资源回基线、失败注入后 last-good 帧 bit-identical、关闭中加载取消 exit 0、线程17→17且自建 worker=0、RSS 215.5→223.1→215.5（三证据并用非单点RSS），模式 headless 已注明；metrics：cold/warm×双游戏 CPU P50/P95/上传/重建/RSS/启停原始数录 measurements.md，gpu_timing=false 明示；verifier-negative-controls：5 门正控过+注入必捕（漏计分/错误踩踏/空帧/第二Device/迟到事件，全部独立状态断言不信游戏自报），破坏验证器示范 exit 1 实证；8 probe case 全 exit 0，69 测试全过。r2：F06 全改实测（DropWitness/真实资源20轮/取消 awaited/teardown 后 clean_exit/mem::forget 永久红控）（b5c1ed1af）；F08 分路实测+原始序列+三指纹+缺观测不 PASS（b5c1ed1af）；F05 合成路径计数门+blank-blit/foreign-device 永久反控（1fc3f6116）] |
| T-08 [x] | work | T-02；独立Iced消费可用 | `AGP/src/bin/iced_spike.rs`、`research/integration-handoff.md`（新） | V-06及反控、IH-01..05与I1–I3后继验收合同；可提前研究，不等两游戏/迁移完成；不改生产UI；AC-07/08。[✅ 2026-10-21 work：commit b812222b5；iced 0.14.0/iced_wgpu 0.14.0 与 wgpu 27.0.1 同实例入 AGP 锁；V-06 windowed exit 0：真实 OS 点击走 iced 真实事件路径改变图形目标、同 Device/Queue 渲染合成、逐类视图 target（发现并修单 pipeline 共享缺陷）、DPI 2.0/resize/干净退出、空帧+第二 Device 反控必捕（wgpu27 跨设备 API 层不报错的事实如实记录，门以 token+架构守住）；交接文档 IH-01..05 全含 owner/入场门/已交证据/缺项，I1–I3 显式 PENDING；9 probe case 全 exit 0，73 测试全过。r2：F09 去 vendoring 后接线更新（1fc3f6116）；F10 handoff :23 伪称更正（1fc3f6116）] |
| T-09 [x] | work | T-01..08 | `AGP/README.md`、`research/result.md`（新），主检出本Plan与附件证据 | V-09、完整独立交付/重定位重放，能力/差异/原型完成与生产pending清单及Spec增量提案；AC-01/08。[✅ 2026-10-21 work：commit f7ea95f3b；V-09 四门全绿（依赖闭包机械扫描零越界、--locked 构建、A→B 重定位真实冷构建重放 26 个机器无关字段 0 差异+快照哈希 bit-identical、根 diff 为空/无链接/核心零 GPU 依赖）；README/result.md 交付（AC-01..08 逐条证据绑定、P0–P2 done/I1–I3 pending、E1–E11 偏差总账）；SD-01..03 提案稿落 AGP/research/spec-delta/（未写 docs/specs/）；73 测试全过，10/10 probe case exit 0。r2：全文档与实测事实对齐（E12/E13），闭合矩阵 F01–F11 全录，锁随去 vendoring 再生（1fc3f6116）] |
| T-10 [x] | review | T-01..09完成、冻结提交/树、作者停止写实现 | 本Plan §9、同树验收证据 | 独立final review核AC-01..09/SD-01..03；不运行根整档来替代验收、不直接修实现；AC-09。[✅ 2026-10-21 终审链：R1 nf(11项)→r2三批修复→R2 nf(5)→r3+E15→R3 PASS→**R4 nf(5，完整不变量缺口)**→r4(b84c6d91e,E16)→R5 PASS→**R6 nf(3：源生成高度y=0误读/截图门3D越界+错位/金帧manifest未入库)**→r5(814f9bf01,E17)→**R7 PASS**（逐项 throwaway 回退+复审者亲跑纯提交导出重放，104/104 测试，AC-01..09 全 PASS）。非阻塞债务：R5-01 无界点击等待、R5-02 僵尸手册、R5-03 SD-03 IH-01 措辞、R7-01 旧计数标注、Godot 角色级对比 deferred。merge-owned：交互会话 iced case 级双跑（唯一活证据缺口）+ 迁移交接放行授权 pending] |
| T-11 [x] | merge | T-10 pass；**2026-10-22 用户裁定**：以 `autographics-dev` 集成分支承接，替代原"迁移完成交接门已明确放行合入 master"前置 | 实验代码合入、Spec/ledger/归档与清理收据 | 冻结提交上跑独立workspace全适用测试/探针及真实consumer；未触根Rust/UI不触发根t/tv/tu/docs_gen，若新增触面须先修订合同。沉淀仅experimental事实、I1–I3仍pending；wt-guard clean后清理；AC-10。[✅ 2026-10-22 merge：门禁冻结 `ddbde3e62` 全绿（fmt/check 零 warning、117/117、十探针 10/10 exit0、consumer 真实重定位 mismatches=[]，证据 [merge-t11/gates.md](attachments/829/merge-t11/gates.md)）；**landed=merge `ab56c32ec`** 落 `autographics-dev`（基于 master tip，merge 保留原哈希不 rebase，AGP 子树与 `ddbde3e62` 逐字节一致 diff=0）；SD-01..03 沉淀 `docs/specs/auto-graphics/`（含 R5-03 counter/content 双门措辞修正）+overview/goals/INDEX 回写；ledger v64→v65 guarded offline projection（P829-AGP/R18/MERGE/AGP-TESTS 四条）；归档+wt-guard 清理收据见 §9 终收据；master 合入待 814/815/816 系迁移全部完成（用户裁定，非本计划范围）] | |

计划实施树：`D:/autostack/.wt/lang-829/auto-lang`，分支 `plan-829-dev`；基面由 work 核对届时 master 冻结提交，不复制当前其他WIP。当前不创建树/分支；root target 与实验 target 分开，不创建 junction/symlink、不写外仓。

**work r1 启动记录（2026-10-21）**：实施树已建 `D:/autostack/.wt/lang-829/auto-lang` @ `plan-829-dev`，基面 `fdb1fad2ce7afa124adfa3080323496df384e670`（master 冻结）；用户 "/auto-plan:work 执行" 即 r1 实施授权；T-01 完成见上表（commit 6c2448261）。
这是唯一实施/检查/复审树；未来若采用跨Plan图形集成分支，先明确定义来源、owner与交付合同，不另建同Plan第二实施树。原型可先完成独立review后等待迁移交接，不能为了合入擅自改约。

## 9. 复审记录

### 2026-10-22 merge 终收据 `PLAN-829:r1` — pass / delivered（autographics-dev 承接；当前）

- `stage: merge`；merge 会话按 auto-plan-merge 技能范式执行；未建 agent/聊天。前一条"blocked 于迁移交接放行"收据已被下述**用户裁定**取代。
- **dated 用户裁定（2026-10-22）**：AutoGraphics 不合入 master、不等待迁移完成——与 AC/AAC（ACE）路线同款开**长期集成分支 `autographics-dev`** 承接：PLAN-829 内容合并入该分支后即清理 worktree；后续 AutoGraphics 计划（830+）沿用此模式；**待 814/815/816 系列（及后续）迁移工作全部完成后再考虑合回 master**。此裁定替代 T-11 原"迁移完成交接门已明确放行合入"前置（对 master 合入而言仍保留等待语义）。
- **checkpoint `prepared` ✅**：R18 PASS 基线 `ddbde3e62857`、SD-01..03 冻结提案、锁 `6cd34576…`、资产 `983a78f540…`（同前收据）。
- **merge-owned 门禁 ✅（冻结 `ddbde3e62857` 实跑全绿）**：fmt/check 零 warning；全适用测试 **117/117**；十探针 **10/10 exit 0**（iced-shared-device 真窗口 composite ok=true；窗口用例前残留进程计数 0）；independent-consumer 真实 A→B 重定位冷构建 `ok=true/mismatches=[]`。证据 [merge-t11/gates.md](attachments/829/merge-t11/gates.md)（10 report+.json.txt 规避 `*.json` 忽略规则、30 关键帧）。
- **checkpoint `landed` ✅（目标=`autographics-dev`）**：`git checkout -b autographics-dev master` + `git merge plan-829-dev` → merge commit **`ab56c32ec4cb28e01fbcca74e9861164ffad43f0`**（parents=master tip+`ddbde3e62`；**merge 保留原哈希不 rebase**，`ddbde3e62` 为祖先；`git diff ddbde3e62 autographics-dev -- experimental/autographics-prototype/` **零行**=AGP 子树与复审提交逐字节一致）；合并树 `cargo check --all-targets --locked --offline` 绿。**master 零代码合入**（裁定），master tip 保持迁移系列簿记流。
- **checkpoint `ledger_refreshed` ✅（channel=guarded offline projection，store DOWN 已核实）**：`.autoos/specs.json` 备份 `.autoos/specs.json.bak-v65-829pre` → 外科投影（round-trip 格式锁定 indent=1/无尾换行，git diff 78+/1- 恰为新增条目+版本行）→ 回读校验（六段集合、+4 条、旧条目逐字段零变更）。version **64→65**；条目 `P829-AGP`（designs→docs/specs/auto-graphics/project.md）、`P829-R18`（reviews→attachments/829/review-r18/review.md）、`P829-MERGE`（reports→archive 本文件）、`P829-AGP-TESTS`（tests→experimental/autographics-prototype/tests/，tag 注明 branch:autographics-dev）。
- **canonical 沉淀 ✅（master，文档层）**：`docs/specs/auto-graphics/{project,prototype-profile,integration-contract}.md`（SD-01..03 落稿，含 R5-03 裁定措辞：counter 门=漏画检测、content 门=透明片元检测，两门并列不可互替，附当前三时刻证据）+`plans.md`；overview.md 外围实验表新增行、goals.md GOAL-008 关联补 829、INDEX.md 再生（spec-index.py GROUPS 登记"实验/沙盒"，58 projects）；spec-lint 1 error=预存 design/ 缺卡，21 warnings 均预存，本次零新增。知识文档显式记录：**代码在 `autographics-dev` 分支，master 侧仅文档**。
- **checkpoint `archived` ✅**：本文件 `git mv` 至 `docs/plans/archive/`，`status: archived`、T-01..11 全勾、`current_step: 11/11`；`completion_kind: delivered`（交付目标=autographics-dev 集成分支 + master 知识沉淀，**非 master 代码合入**）；plan/docs 工件随本收据提交 master（无未跟踪 docs/ 遗留）。文末 spec-sync 回写记录已附。
- **checkpoint `cleaned`**：wt-guard → worktree/分支/组目录清理，收据随后续提交补记（见本节末"cleaned 补记"）。
- **landing≠deployment 附注**：本计划为实验原型，无生产构件（backend release/daemon/web bundle）消费其代码；AGP 自有构建随分支。批量回归到期判定：收据 `last_covered_plan_id=817` @2026-10-21T06:27Z（<48h）且其后无计划落地（815–828 在途、829 非整 5 号且落集成分支）→ **不到期**。
- `outcome: pass`（delivered to autographics-dev；I1–I3 生产集成 pending 由 IH 合同承接；master 合回时机由迁移系列收口后再议）。

### 2026-10-22 merge T-11 门禁执行收据 `PLAN-829:r1` — blocked 于迁移交接放行（历史；已被同日用户裁定取代）

- `stage: merge`；merge 会话按 auto-plan-merge 技能范式执行；未建 agent/聊天。
- 基线：worktree `D:/autostack/.wt/lang-829/auto-lang`、分支 `plan-829-dev`、HEAD=`ddbde3e62857e6bbfc8b780c6182ee1bb7d4ef74`（=R18 reviewed_commit）、基面 `fdb1fad2c`；执行前后树均 clean；diff 127 文件全限 `experimental/autographics-prototype/`（纯 experimental 触面，按 T-11/R18 裁定不触发根 t/tv/tt/tb/tu/taa/tf/docs_gen）。窗口用例前 `graphics-probe`/`iced_spike` 残留进程计数 0。
- **checkpoint `prepared` ✅**：复审基线 R18 PASS（plan_revision 1）；SD-01..03 冻结提案与目标路径 `docs/specs/auto-graphics/{project,prototype-profile,integration-contract}.md` 不变（supersedes=[]、touched_goals=[GOAL-008]）；候选 delivery commit=`ddbde3e62`；锁 SHA256 `6cd34576…`、资产集 `983a78f540…`。
- **merge-owned 门禁（冻结提交实跑，全绿）**：fmt `--check` exit0；`cargo check --locked --offline --all-targets` exit0 无 warning；全适用测试 `cargo test --locked --offline` **117/117**；十探针矩阵 **10/10 exit 0**（gpu-foundation/shared-runtime `--mode auto`、ag3d-risk-slice、ag2d-breakout、ag3d-squash、lifecycle、metrics、verifier-negative-controls、independent-consumer、iced-shared-device 真窗口）；真实 consumer A→B 重定位冷构建 `relocation.ok=true/mismatches=[]`。逐例 exit/报告/关键帧与复现命令见 [merge-t11/gates.md](attachments/829/merge-t11/gates.md)（worktree `scratch/829/merge-t11/` 同源）。证据绑定 `ddbde3e62857`；实现/依赖/配置/输入变更即失效须复审重跑。
- **checkpoint `landed`/`ledger_refreshed`/`archived`/`cleaned` pending**：T-11 前置“迁移完成交接门已明确放行合入”（§5.4：迁移负责人收据，814-A 局部发布不替代；R3/R5/R7/R18 一致记为非复审/merge 可代决）**未放行**——本会话不 land、不写 canonical Spec/派生 ledger、不归档、不清树。SD-01..03 canonical 落稿、R5-03 措辞补齐（内容门 vs counter 门分工）等沉淀项随放行后一次性执行。
- `outcome: blocked`（publication-only；计划保持 `reviewed 10/11`、T-11 开放、`current_step: 10/11`）；无代码/依赖/配置变更，无根档执行，无越权合入。放行后余序：rebase 核对→ff-only land→Spec/ledger 沉淀→归档→wt-guard 清理（landing 非 deployment，本计划无生产构件重建项；批量回归到期判定按 fix-test-tiering 规则随收尾核对）。

### 2026-10-22 受影响独立 final review R18 — PASS（当前）

- `stage: review`；`review_scope: final, affected F829-R14-01/02 closeout`；`reviewer_identity: 当前独立复审聊天，与实施聊天独立，未创建agent/聊天`；R17为work收据，不作为独立判定依据。
- `plan_id: PLAN-829`；`plan_revision: 1`；`outcome: pass`；`worktree_path: D:/autostack/.wt/lang-829/auto-lang`；`branch: plan-829-dev`；`reviewed_commit: ddbde3e62857e6bbfc8b780c6182ee1bb7d4ef74`；`base_commit: fdb1fad2ce7afa124adfa3080323496df384e670`；入口/结论前HEAD一致、实现净树，修复diff仅AGP两文件。
- `dependency_revisions: wgpu27.0.1 / winit0.30.13 / iced及iced_wgpu0.14.0 / iced_winit0.14.1 / stock iced_widget0.14.2`；锁SHA256=`6cd345763c2dd44814db4a367fe636010478f9745577b8b06e3c2c573b08e988`；Plan入口SHA256=`607878C51DC8EE26C6346744AA8B42393425DA652F9AED37A5B23CA54298265A`。Spec/技能/SD原文与哈希、源码/可执行身份、当前raw见 [R18报告](attachments/829/review-r18/review.md)、[主复跑](attachments/829/review-r18/replay.ps1)、[原生失败入口](attachments/829/review-r18/native-replay.ps1)。
- `finding_closure`：**F829-R14-01 CLOSED**。原函数同状态副本boot缺ID302Tick具名到99，终态现为iced::exit任务units1，已知ID兄弟close units1、后600Tick不重复收尾；初始/点击后/冻结resized均181Tick超时，合法延迟回调到6保持。决定性原生扣留Resize反例现正常返回native_loop_returned，失败报告/verdict1及exit1，无panic、手动关闭或外部kill，关闭R16 exit101与零退出任务反例。永久单测真实原函数路径通过，非只信任务计数。**F829-R14-02 CLOSED**：result:117–121与E21的dated HISTORY/CORRECTED一致，旧1ff raw非commit-bound，实际36/ae40记录及本轮ddb证据各自成立；旧raw保留，不推定build.rs根因。
- `evidence/health`：**58定向全过**=17 composite+2 delivery+17 squash+5 seam+16 shared+1缺ID永久lib测试（另28 lib被filter，不冒领全档）；fmt/check--tests/diff-check全0、零warning。当前原CLI release正控exit0/三时刻content true；透明负控首轮点击未送达、901Tick具名超时导致后两图缺失，不计完整负控且raw保留；一次相同源/锁/资产/命令的定向重跑exit1、三时刻content false且click observed。完整正/负指纹均ddbde3e62857。未确定外部输入丢失根因，不把首轮当三时刻PASS、不指控exit修复导致输入问题。
- `evidence reuse`：仅退出分支/新增单测/result变，analyzer与ae40逐字归一化对照相同13414字符，输入hash重核无漂移；R16的19外围腐坏、全部R8/R10与真实resize正控按边界复用，未再执行/加入58计数。游戏/oracle/资源/音频/缓存/生命周期/计量/独立consumer原范围不变，先前已验收证据限定复用，不冒领全workspace/十probe当轮执行。
- `spec_inputs/delta`：现行overview/UI/仓界/goals/技能同R16；Design45新增§8已读且明确CAP/AutoShader不扩张829验收，本轮hash=`369576056DC07B2726C21DEAEAD42D5552EE60920C7010D230529CA860AF13D7`。SD-01..03语义PASS，三份提案已冻结；supersedes=[]，三个docs/specs/auto-graphics目标与GOAL-008不变，canonical模块尚未发布。SD-03旧counter/GDI句描述R1 skipped-draw门，不可解释为检测透明fragment；后者已用内容门实证。既有非阻塞R5-03沉淀文案补齐留merge，将当前截图/计数分工写清，不授权退回counter-only门；实验与I1–I3 pending边界不变。未写canonical/live ledger。
- `acceptance_results`：AC-01..04既有pass保持（delivery/shared/squash/seam新验），AC-05/06失败收尾与历史绑定现PASS，AC-07当前窗口正负/resize/缺ID正常失败退出PASS，AC-08交接边界与文档PASS，AC-09 PASS；AC-10 pending。无新增阻断finding、无未授权延期/缩面。T-01..10完成、T-11开放，`status: reviewed`、`current_step: 10/11`。
- `next: merge when handoff permits`；`pending_closeout_gates`：冻结提交上AGP适用完整测试/十probe矩阵/真实consumer重定位、迁移完成与写面交接放行、规范/派生ledger沉淀、归档及wt-guard清理均pending。纯experimental触面不触发根t/tv/tt/tb/tu/taa/tf/docs_gen；当前PASS不等于交付或绕过交接门合入。实现/依赖/配置/输入若变，受影响复审及新冻结门重跑。未改实施/提交/合入/归档/清树、未跑完整AGP或根整档、未建agent或跨聊天发信。

### 2026-10-22 受影响独立 final review R16 — needs_fix（历史；两项残项R18已独立关闭）

- `stage: review`；`review_scope: final, affected F829-R14-01/02`；`reviewer_identity: 当前独立复审聊天，与实施聊天独立，未创建agent/聊天`；R15为work收据，不重标独立PASS。
- `plan_id: PLAN-829`；`plan_revision: 1`；`outcome: needs_fix`；`worktree_path: D:/autostack/.wt/lang-829/auto-lang`；`branch: plan-829-dev`；`reviewed_commit: ae40d6052cdb7716c7cabe9e3f7e9edb1fe92d74`；`base_commit: fdb1fad2ce7afa124adfa3080323496df384e670`；入口/结论前HEAD一致、实现净树，修复diff仅AGP两文件。
- `dependency_revisions: wgpu27.0.1 / winit0.30.13 / iced及iced_wgpu0.14.0 / iced_winit0.14.1 / stock iced_widget0.14.2`；锁SHA256=`6cd345763c2dd44814db4a367fe636010478f9745577b8b06e3c2c573b08e988`；Plan入口SHA256=`BF24AF1BB979038A0B075003699D5B5CCA59D086C6CCC9A7C56CF61B06094CAE`。Spec/技能/设计输入哈希与R14相同，SD原文/哈希、源码副本/可执行身份、输入与raw已冻结到 [R16报告](attachments/829/review-r16/review.md)；[主复跑](attachments/829/review-r16/replay.ps1)、[原生失败复跑](attachments/829/review-r16/native-replay.ps1)。
- `finding_closure`：R14-01原冻结计时反例CLOSED：所有phase_age/resize-entry走wall_tick，initial/after_button/frozen resized均181 Tick到99且具名，冻结graphics保持120；真实延迟CompositeShot仍到phase6。boot缺ID现在302 Tick具名超时，ready正控能发首张任务。R14-02的E21更正CLOSED，旧1ff报告降work context、R14实际36收据引用正确；当前ae40指纹有效。
- `findings`：**F829-R14-01/P2 PARTIAL/OPEN**：新window-id-timeout进99后finalizer写report/verdict1，却在window_id=None时返回Task::none；finished=true后600 Tick仍无关闭/退出任务，已知ID同支对照有close任务。原生副本仅扣留Resize事件，使用同状态绑定的两份逐字原函数模块：超时后报告已写，但没有正常返回，Surface::present/Parent device is lost panic，exit101；不称原生20秒挂死，也不把GPU根因推定为TaskNone造成，确认的是该分支缺正常终止动作。**F829-R14-02/P3 PARTIAL/OPEN**：result.md:113–118仍认证E21/R13 live绑定1ff，与已更正的E21冲突，该文件本批零diff；需dated更正并引用实际36/ae40，不改旧raw头。
- `evidence/health`：57/57定向=17 composite+2 delivery+17 squash+5 seam+16 shared；fmt/check--tests/repair diff-check全0，无warning。19/19完整外围腐坏及R8/R10旧例仍拒，真实initial/button/两resize绿。当前**原CLI release**独立正控exit0、transparent_blit负控exit1，三时刻各自全绿/全红、指纹ae40；非复用R15 exit1（其negative仅initial完整）。source-copy前缀63788字节SHA256=`B881D3C9B7BFDDB87EEEC53CB3FF1B8BD8B2FCE738FD623B537ED97479DA8A28`，case副本与原件全字节相同；仅append测试入口。首轮native探针状态绑定错误及探针env/extern编译错误保留但不列产品finding，校正同状态后再复现并记录退出码。
- `acceptance_results`：AC-01/02/03/04原范围保持；AC-05/06 partial；AC-07正常live/resize通过，缺ID的失败收尾需修；AC-08边界保持、历史summary需改；AC-09 needs_fix；AC-10 merge-owned pending。未变路径/依赖/输入按先前收据限定复用，未冒领全AGP/根整档。SD-01..03目标/GOAL-008及实验/生产边界保持；携带的SD-03 counter/GDI措辞仍须沉淀前对齐，未发布canonical/live ledger。
- `next: work`：同一树补齐缺ID的正常失败退出和result历史更正；覆盖deadline→finalize→实际事件循环返回，保留所有已闭时钟/截图/几何/source正反控后提交送受影响复审。不仅用phase99/报告存在或exit非零作正常清理证据。T-01..06完成、T-07..10开放，`status: executing`、`current_step: 6/11`；T-11 pending，不授权延期/缩面。
- `pending_closeout_gates`：最终pass后冻结AGP全适用测试/探针矩阵及真实consumer重定位、迁移/写面交接放行、Spec/派生ledger沉淀、归档与guard清理。未改实施/提交/合入/归档/清树、未根t/tv/tt/tb/tu/taa/tf/docs_gen、未建agent或跨聊天发信。

### 2026-10-22 受影响独立 final review R14 — needs_fix（历史；计时与E21更正R16已验，终止路径及result更正仍开放）

- `stage: review`；`review_scope: final, affected R8-01/R10-02 plus repair regressions`；`reviewer_identity: 当前独立复审聊天，与实施聊天独立，未创建agent/聊天`；R13为work修复收据，不冒称独立PASS。
- `plan_id: PLAN-829`；`plan_revision: 1`；`outcome: needs_fix`；`worktree_path: D:/autostack/.wt/lang-829/auto-lang`；`branch: plan-829-dev`；`reviewed_commit: 36f520c104ad99e2eaed2e22a5a3f0bb93195391`；`base_commit: fdb1fad2ce7afa124adfa3080323496df384e670`；入口/结论前clean、HEAD一致，4修复文件仅AGP、根workspace/锁未改。
- `dependency_revisions: wgpu27.0.1 / winit0.30.13 / iced及iced_wgpu0.14.0 / iced_winit0.14.1 / stock iced_widget0.14.2`；锁SHA256=`6cd345763c2dd44814db4a367fe636010478f9745577b8b06e3c2c573b08e988`；Plan入口SHA256=`0242D68F9F9DE6D48225E98F58706F8A3987A7754F9EACF4D99EE36138AC3F97`。Spec/技能/SD原文哈希、源码/可执行身份、旧输入/新图及 [报告](attachments/829/review-r14/review.md) 与 [复跑入口](attachments/829/review-r14/replay.ps1) 已冻结到主检出附件。
- `finding_closure`：**F829-R8-01 CLOSED**，19/19完整外围腐坏输入当前消费函数均clip=false，初始/点击后/两张真实resize保持绿；R8/R10旧位移/侧渗/底渗重放皆拒且clipped-placement保留clip=true，R10-01 CLOSED保持。**R10-02原premature-finalize及正常live闭合已在36重新验证**：独立release正控exit0、transparent_blit负控exit1，三时刻全齐且内容逐时刻绿/红，实际指纹`36f520c104ad`。新有界失败路径不被该正控覆盖，见R14-01。
- `findings`：**F829-R14-01/P2 OPEN**：phase4先冻结GRAPHICS.tick再enter5，phase_age仍由该tick计算，phase5 pending resized时180Tick截止永不触发。冻结原源码副本仅append私有测试入口，62504字节前缀SHA256与原件相同`5366DE199E1DB39B2D5B525EB1145AA32814699C6D4932285A816816A7EA099A`；通过原update(Message::Tick)实跑，initial/after_button active对照181tick到99，resized frozen1000tick仍tick120/phase5/无失败；实际CompositeShot回调可解除等待到phase6；只切active-clock因果对照181tick触发resized-timeout。真实boot缺window-ID事件1000tick仍phase0/无失败，ready等待亦无界。不是声称当前GPU正常会丢callback，也不是新增硬wall-clock保证；新声明的有界失败路径未实现。**F829-R14-02/P3 OPEN**：R13两份live标1ff却含仅36新增f_top/pre2d/left/right格式，E21/result声称绑定旧冻结revision不成立；本轮36绑定的真实正负收据已另采，需保留原件并订正历史叙述。具体编译/复制根因未确定，不指控意图、不处方修改build.rs。
- `evidence/health`：57/57定向=17 composite+2 delivery+17 squash+5 seam+16 shared，fmt/check--tests/repair diff-check全0且无产品compiler warning。新failure-probe无GPU/native window/输入helper，止于首次phase99，不执行finalizer；是控制器Tick数加速重放，非模拟等待33秒。原始正常live pair本轮已成功结束，无90秒回收；外部截止仅作保护。两份小型独立Git夹具探索stale-fingerprint假说均正确报dirty，未复现该根因；失败探索输入/raw保留，不作产品finding，不纳入主复跑。只有夹具基线提交，实际Plan仓无提交/历史变更。
- `replacement-error disposition`：用户末尾替换失败的旧句已核查，result/E20实际已改为R11 dirty说明，无该旧clean-HEAD断言残留；编辑报错本身不列finding。R14-02指另行新增E21/1ff绑定事实，不混同已改旧句。
- `acceptance_results`：AC-01/02/03/04既有边界保持、shared/squash/seam新验；AC-05/06 partial；AC-07正常live与resize通过，但新超时/就绪失败路径需修；AC-08边界保留、历史收据叙述需改；AC-09 needs_fix，AC-10 merge-owned pending。未变游戏/oracle/资产/缓存/生命周期/计量/交付/配置范围按R8/R10/R12识别证据复用，不冒领完整116测试/全矩阵。SD目标/GOAL-008及实验/生产边界保持，原SD-03陈旧counter-only措辞沉淀前需对齐；未写canonical/live ledger。
- `next: work`：同树一次修齐两项，诊断deadline时钟所有权及active/frozen、pending/complete、window-ready/missing状态；保留冻结帧负控及所有已闭几何/source/交付修复，增加实际Tick更新路径的超时与回调正控，提交后受影响复审。`status: executing`、`current_step: 6/11`，T-01..06完成，T-07..10开放、T-11 pending；不授权延期/缩面或复用错绑收据。
- `pending_closeout_gates`：最终pass后冻结AGP门/consumer重定位、迁移/写面交接放行、Spec/派生ledger沉淀、归档与guard清理。未改实施/实际仓提交/合入/归档/清树/跨聊天发信，未建agent，未跑根t/tv/tt/tb/tu/taa/tf/docs_gen或完整AGP矩阵。

### 2026-10-22 受影响独立 final review R12 — needs_fix（历史；R14确认裁剪与正常live关闭，新增失败路径缺口）

- `stage: review`；`review_scope: final, affected R8-01/R10-01/R10-02 plus repair regressions`；`reviewer_identity: 当前独立复审聊天，与实施聊天独立，未创建agent/聊天`；R11为work修复收据，不视为独立PASS。
- `plan_id: PLAN-829`；`plan_revision: 1`；`outcome: needs_fix`；`worktree_path: D:/autostack/.wt/lang-829/auto-lang`；`branch: plan-829-dev`；`reviewed_commit: 1ff4ee67e06197180d4ab691c2d5764bdb6c5a61`；`base_commit: fdb1fad2ce7afa124adfa3080323496df384e670`；入口与结论前树clean/HEAD一致，修复4文件仍仅AGP、无根workspace/锁改动及reparse点。
- `dependency_revisions: wgpu27.0.1 / winit0.30.13 / iced及iced_wgpu0.14.0 / iced_winit0.14.1 / stock iced_widget0.14.2`；锁SHA256=`6cd345763c2dd44814db4a367fe636010478f9745577b8b06e3c2c573b08e988`。Plan入口SHA256=`E5C95AD68BF17A2DF0B147A787D64BF1F9F81F0A831579E1E4BB7B403CE39D3E`；Spec/技能输入与R10哈希一致；SD原文/哈希、关键源码/旧输入/所有新图片与构建可执行身份冻结在 [review-r12](attachments/829/review-r12/review.md) 同目录。
- `finding_closure`：R8-01/R10旧例实跑关闭保留：两视图各左右80px移位拒且clip=true，ground/model侧渗拒，R8三张旧变异仍拒；**R10-01 CLOSED**，两张真实resize图以独立受约束layout v3_h626通过、构造式resize改为313 logical，未扩大容差。R8-02源规则文档/高度/trace、R6-03纯提交闭包既有关闭保留；未变游戏/资产/缓存/生命周期/计量/配置范围沿用R8/R10具名证据。
- `open_finding`：**F829-R8-01/P2仍OPEN**。当前消费函数对真实2000×1600@2正控的完整背景区域矩阵19个变异，9个已观测区域正确拒，10个漏区仍`ok/clip/p2d/p3d=true`：① x20..1980,y82..98的2D上方间隙31360px×sky/model/ground；② x0..20与1980..2000,y20..82上侧外围2480px×三色；③ 同x、y1394..1474下角3200px×三色；④ x20..1980,y0..20顶padding39200px，用原图(1541,277)实际RGB(24,38,52)。原因：ctrl_bot82到v2_top98无人采样，frac只扫内部列、padding只扫两视图y范围，顶padding误归控制带仅认signature。先前已修的色谓词在被观测区有效，不能代替完整矩形补集。精确反例/正控/因果/关闭边界见 [报告](attachments/829/review-r12/review.md) 与 [复跑](attachments/829/review-r12/replay.ps1)。不声称渲染器实际发出这些腐坏图。
- `R10-02 evidence disposition`：完成屏障/窗口id就绪guard/Complete-Wait-Missing单测通过；本轮未证明新的premature finalize，但**完整live闭合仍partial**。所附R11正负报告指纹均为`1fa149873a57-dirty`，相对live目录链接缺失，无源快照hash将dirty二进制绑定1ff，不能据其称新SHA clean-HEAD绿；E20/R11叙述需更正并补绑定收据。本轮独立debug与fresh release均嵌`1ff4ee67e061`（可执行hash已存），各一次90秒正控仅有initial三图、无最终报告，自建进程树已回收；各自后续透明负控跳过，未冒领结果。具体阻塞phase/输入/GPU/环境根因未插桩，不新增猜测性产品finding，也不拿部分初始图当完整正控。
- `evidence/health`：53/53定向=13 composite+2 delivery+17 squash+5 seam+16 shared；fmt/check--tests/repair diff-check皆0、无compiler warning。独立完整perimeter探针19腐坏输入与4真实正控验证；原R8/R10已关闭输入重放成功。一个额外顶padding探针初尝试请求图中不存在的dark-blue像素而中止，raw作为无效选色尝试保留；已改为真实坐标像素并重跑exit0，不作产品缺陷。未跑全AGP矩阵、根整档或merge-owned门。
- `acceptance_results`：AC-01/02/03/04已验证边界保持；AC-08源profile文档修正保留、当前live叙述需准确绑定；AC-05/06 partial、AC-07 fail/partial、AC-09 needs_fix；AC-10 merge-owned pending。SD三目标路径/GOAL-008与实验/生产边界仍适当，当前验收缺口及原SD-03counter-only叙述收齐前不批准沉淀；未改canonical/live ledger。
- `next: work`：同树保留已闭修复，补完整外围背景mask及边界归属（不是再添几个颜色/条带），同步E20诊断；保留所有旧反例与真实初始/点击/resize正控，正确绑定新冻结SHA的完整live正负证据并修正dirty收据叙述，提交后受影响复审。`status: executing`、`current_step: 6/11`，T-01..06完成/T-07..10开放/T-11 pending。不授权延期/缩面或重冻腐坏输出。
- `pending_closeout_gates`：最终pass后冻结AGP门、consumer重定位、迁移/写面交接放行、Spec/派生ledger沉淀、归档与guard清理。未改实施/提交/合入/归档/清树/跨聊天发信、未建agent，未跑根t/tv/tt/tb/tu/taa/tf/docs_gen。

### 2026-10-22 受影响独立 final review R10 — needs_fix（历史；R12确认旧例及resize修复、外围仍有缺口）

- `stage: review`；`review_scope: final, affected R8-01/02 plus repair regressions`；`reviewer_identity: 当前R1/R4/R6/R8独立复审聊天，与实施聊天独立；未创建agent/聊天`。R9附件为work修复收据，不冒称新增独立PASS。
- `plan_id: PLAN-829`；`plan_revision: 1`；`outcome: needs_fix`；`worktree_path: D:/autostack/.wt/lang-829/auto-lang`；`branch: plan-829-dev`；`reviewed_commit: 1fa149873a578dcff44537eb383ee7ef05be6dc8`；`base_commit: fdb1fad2ce7afa124adfa3080323496df384e670`；修复链814→be00424c9→1fa。入口及结论前树clean、HEAD一致，7文件diff仅AGP。
- `dependency_revisions: wgpu27.0.1 / winit0.30.13 / iced及iced_wgpu0.14.0 / iced_winit0.14.1 / stock iced_widget0.14.2`；锁SHA256=`6cd345763c2dd44814db4a367fe636010478f9745577b8b06e3c2c573b08e988`。Plan入口SHA256=`B7928560D2AD7CE86577F1ED807A5FD50857D7B92F3E877D8F46FFAA8EE88EF9`。实际Spec输入及技能哈希与R8一致，SD-01..03逐份原文/哈希重新冻结；关键源码/输入/图片哈希与 [完整报告](attachments/829/review-r10/review.md)、[复跑入口](attachments/829/review-r10/replay.ps1) 同目录。
- `finding_closure`：**F829-R8-02 CLOSED**，sample-manifest/result/feasibility/risk_slice当前规则一致为local y0、当前trace death2703，历史y1/t1402均带日期；源脚本/trace未变，保留R8原Godot fixture/driver证据。**F829-R8-01三个原例 CLOSED、完整不变量 PARTIAL/OPEN**：原3D向下80、底边天空色、裁剪后仅3D错位均通过当前消费函数拒识；新独立横向×两视图×两方向四例仍`p2d/p3d/clip/ok=true`，两2D各93080像素变化、两3D各164683；25600个RGB224地面色或从真实角色采样的RGB(86,2,10)侧边越界也全绿。实现只查y extent，clip仍只认clear/sky色。
- `new_findings`：**F829-R10-01/P2** 修复回归：真实canonical resize=860×700@2，Iced约束第二视图为840×313，物理y754..1380；新门按名义320推底边1394，观测尾1378故`position_3d_ok=false`。旧真实图和本轮clean-HEAD新图均新门拒、814原样函数接受，永久synthetic resize把320画满而漏约束。**F829-R10-02/P2** 新观察到的既有状态路径缺口：两次真实window正控均正常结束exit1，真实OS click与resize完成；第一次initial/resized截图缺，第二次initial缺且resized被R10-01拒。phase发shot_task即递进，不等CompositeShot处理便finalize/close。具体callback漏送调度点未插桩、不推断GPU/环境根因；缺完成/ready屏障与实际缺图已证实，不是原无界click等待债务。各项完整既有AC/输入/正控/边界/关闭条件见报告。
- `evidence`：48/48定向=8 composite+2 delivery+17 squash+5 seam+16 shared，fmt/check--tests/repair diff-check全0且无warning。真实Iced两跑指纹`1fa149873a57`、Vulkan/RTX4060Ti/617.14，非90秒deadline；原初始/点击真实图通过，真实resize误拒。未跑transparent live负控（正控尚未完成通过），但原真实透明像素负控仍拒。未冒领107全档/10探针矩阵为本轮运行。冻结旧814 analyzer只用于相同字节新旧比较，没有修补实施或改变输入期待。
- `acceptance_results`：AC-01/08受影响文档项pass；AC-02/03/04保持既有边界pass（shared/squash/seam重测）；AC-05/06 partial；AC-07/09 fail；AC-10 merge-owned pending。未变游戏/缓存/生命周期/计量/交付/根边界的实现、锁、资产、trace、测试配置范围沿用R8具名证据；不以旧Iced PASS替代新门与真实canonical验证。SD目标/GOAL-008/实验与生产边界仍适当，验收缺口和原SD-03陈旧表述收齐前不批准沉淀；未写canonical/live ledger。
- `next: work`：同树一次修齐开放R8-01与R10-01/02，E18/E19补x/y两轴、实际受约束layout及截图任务完成/就绪矩阵；保留全部原例与真实初始/点击/resize正控，提交新SHA后受影响复审。R8-02和源高度/提交闭包关闭保留，T-01/T-05勾回、T-01..06完成，`status: executing`、`current_step: 6/11`；T-07..10开放、T-11 pending。不授权延期、缩面或用观察图反推期待布局。
- `pending_closeout_gates`：最终pass后冻结AGP门/真实完整Iced positive+transparent negative/consumer重定位、迁移及写面交接放行、Spec/派生ledger沉淀、归档与guard清理。未改实施/提交/合入/归档/清树/跨聊天发信，未跑根t/tv/tt/tb/tu/taa/tf/docs_gen。

### 2026-10-21 独立 final review R8 — needs_fix（历史；R10确认文档关闭与剩余几何缺口）

- `stage: review`；`review_scope: final`；`reviewer_identity: 当前R1/R4/R6复审聊天，与实施聊天独立；未创建agent/聊天，未虚构模型/会话身份`。
- `plan_id: PLAN-829`；`plan_revision: 1`；`outcome: needs_fix`；`worktree_path: D:/autostack/.wt/lang-829/auto-lang`；`branch: plan-829-dev`；`reviewed_commit: 814f9bf01aa14eadcd9e851bc4cce7ac8caa360b`；`base_commit: fdb1fad2ce7afa124adfa3080323496df384e670`。树入口/结论前clean、HEAD未漂移，diff仅AGP、无reparse点。
- `dependency_revisions: wgpu27.0.1 / winit0.30.13 / iced及iced_wgpu0.14.0 / stock iced_widget0.14.2`；锁SHA256=`6cd345763c2dd44814db4a367fe636010478f9745577b8b06e3c2c573b08e988`。Plan入口SHA256=`521D173A51B95F1AC80492DE96C49626057858E0B41CEF97365463D729D901F0`；关键源码/当前Spec/技能与旧输入hash及SD-01..03原文/hash冻结在 [review-r8](attachments/829/review-r8/review.md) 同目录。
- `findings`：**F829-R8-01/P2 OPEN**（R6-02原例关闭、完整不变量仍partial）：正式analyzer消费真实正控截图，原透明/侧边渗出/双视图整体错位均拒，但仅3D下移80px仍`position_ok/clip_ok/ok=true`；3D底边1960×80=156800天空色越界像素仍全绿；保持原矩形裁剪的仅3D错位也全绿。因position只观察2D顶行、clip不查3D以下区域。**F829-R8-02/P3 OPEN**：当前sample-manifest:206、result:224仍教生成y1，feasibility:74仍称当前death=1402，与源fixture/实现y0、trace death2703冲突。不是要求修改冻结源或恢复错误oracle。精确输入/实际入口/反例与正控/边界/关闭条件见 [完整报告](attachments/829/review-r8/review.md) 与 [复跑入口](attachments/829/review-r8/replay.ps1)。
- `closure/evidence`：R6-03 CLOSED，复审者亲跑git archive纯提交导出→离线锁定构建→双游戏headless exit0/trace bit-exact/真实金帧比对；清单已在HEAD且不ignored。required数组实际19项，R7收据24项计数不沿用。R6-01实现/生成器/trace CLOSED，真实Godot4.4.1原脚本四次生成y0、当前driver首spawn=(14,0,-0.35427094)，文档收口partial。R4-01..04独立原例重放保持闭合（死后6个存活实体全Mob、Space retry/无残跳、联合方向false/false、rotation snapshot及四元数符号等价）。45/45定向测试=5 composite+2 delivery+17 squash+5 seam+16 shared；不冒领104为本轮实跑。fmt与修复diff-check过、定向构建无warning；全base diff仅保留原样字体license既有尾空格。未跑新完整Iced窗口case，不推断环境/产品原因；新analyzer判别力已实证不完整。
- `acceptance_results`：AC-01/05/06/08 partial；AC-02/03/04在既有profile/证据边界pass（受影响3D/交付已新验）；AC-07/09 fail；AC-10 merge-owned pending。Breakout/cache/lifecycle/measurement等未变实现、锁与配置范围复用R4/R6/R7具名证据，不替代当前截图门验证。SD-01/02/03实验边界/目标路径/GOAL-008合理，当前验收缺口及R5-03陈旧SD-03文字未闭前不批准沉淀；未改canonical/live ledger。
- `next: work`：原树同批修齐两项、E17补每视图独立位置/全部外围边界和当前oracle消费者审计、保留正反控、提交新SHA再送审。保留T-02/T-03/T-04、确认T-06完成；T-01/T-05/T-07..10开放，`status: executing`、`current_step: 4/11`。修正入口Plan reviewed/10与旧R6状态段/任务勾选冲突，不抹历史收据；登记不授权延期/缩面或重冻腐坏画面。
- `pending_closeout_gates`：最终pass后冻结AGP门/完整Iced positive+transparent negative/真实consumer重定位、迁移及写面交接放行、Spec/派生ledger沉淀、归档与guard清理。未改实施/提交/合入/归档/清树/跨聊天发信，未跑根t/tv/tt/tb/tu/taa/tf/docs_gen。

### 2026-10-21 独立 final review R7 — PASS（历史；R8发现位置/底边与oracle文档缺口）

- `stage: review`；`review_scope: final confirmation`；`reviewer_identity: 独立 explore 子代理上下文（R2/R3/R5 复审者延续），与实施代理不同实例`。
- `plan_id: PLAN-829`；`plan_revision: 1`；`outcome: pass`；`worktree_path: D:/autostack/.wt/lang-829/auto-lang`；`branch: plan-829-dev`；`reviewed_commit: 814f9bf01aa14eadcd9e851bc4cce7ac8caa360b`（r5 修复批，基线 b84c6d91e）；树 clean、diff 仅 AGP。
- 范围：R6 findings F829-R6-01..03 关闭确认。**R6-03 CLOSED**：manifest 已入版本控制（嵌套 .gitignore 否定规则），delivery-closure 测试绑定 24 项必需输入清单并有判别力；**复审者亲自执行 R6 的关闭检查**：`git archive HEAD` 导出 → 独立 `--locked --offline` 构建 → 双游戏 headless 纯提交内容 exit 0、bit-exact、compare=Some；自审计无其他 required-but-ignored 输入。**R6-01 CLOSED**：复审者独立从冻结源重推导 y=0 合同（Main.gd:37 local position；Mob 直挂 Main）；新 trace 首生成 (14,0,−0.354) 精确符合；生成器/trace 带 dated provenance（非调参）；SPAWN_Y=1 回退→相关测试红；关联坐标路径核验边界如实记录。**R6-02 CLOSED**：几何锚定 layout×scale（图像搜索降级诊断）、position_ok 事实、双向 clip 签名；R6 两个精确腐坏输入（3D 天空色渗出/视图下移 80px）经真实 analyzer 均拒；削弱突变红；真实截图 analyzer 复跑独立重现（expected_top=98=observed_top=98）。
- iced 活证据处置：门逻辑判别力已由单元控制+真实截图+削弱突变+r4 sabotage 路径完全建立；剩余唯一 merge-owned 窄项=交互会话一次性完整 case 级 windowed positive（三时刻端到端，僵尸预清理）——输入投递/环境问题非门逻辑缺口；R5-01 无界等待债务仍在（r5 修复中直接撞上，有界化尝试已记录）。
- `evidence`：[review-r7/review.md](attachments/829/review-r7/review.md)；104/104 测试、fmt/warning 干净、全探针矩阵绿、consumer 干净态绿（指纹 814f9bf01aa1）、纯提交导出重放绿、result.md 完整（截断事故已修复重建，250 行）。
- 新非阻塞项：**F829-R7-01**（P3，result.md E15 叙述引旧"45 events"，沉淀时标注）；**F829-R7-02**（R5-01/R5-02 延续：无界点击等待 + 僵尸进程运行手册）。
- `acceptance_results`：AC-01..AC-09 **PASS**（AC-09=R7 通过）；AC-10 merge-owned pending。
- `pending_closeout_gates`：T-11 全套——交互会话 iced case 级 positive+transparent_blit negative（唯一活证据缺口，僵尸预清理）、真实 consumer 重定位、**迁移交接放行授权**、SD-01/02/03 沉淀（含 R5-03 内容门措辞、R7-01 标注、R5-02 运行手册）、债务登记（R5-01、Godot 角色级对比）、归档、wt-guard 清理。
- `next: merge`（前提：迁移交接放行授权）。

### 2026-10-21 work r5 修复移交（R6 → R7）

- `stage: work`；`outcome: pass`（修复批次）；`code_commit: 814f9bf01`（+641/−44，15 文件，AGP-only）。
- F829-R6-03：manifest 入版本控制（嵌套 .gitignore `!manifest.json`）+ 永久 delivery-closure 测试（24 项清单绑定：存在/被追踪/未被忽略/清单↔文件交叉核对）+ 纯提交导出重放双游戏绿（`scratch/829/r5/export`+`replay`）。F829-R6-01：SPAWN_Y=0 按冻结调用路径推导，生成器带 dated provenance 重冻 trace（45 spawns/91 events，JUMP_FACTOR 0.30 场景作者重同步录 meta），Rust 回放 bit-exact，3d-feasibility dated 条目重写。F829-R6-02：锚定几何（expected_top=49×scale=98 与真实截图精确一致）+ position_ok + 双向 clip 签名；5 个永久控制过真实 analyzer（含 R6 两个精确腐坏输入转红）；有界等待规则执行（三次有界尝试，真实初始合成被锚定 analyzer exact 接受）。
- 诊断扩展 E17：源 local/global 坐标、锚定几何+双向 clip、提交交付 vs 忽略文件（git status clean ≠ 提交闭包；导出重放成为立规检查）。
- 事故与恢复：result.md 曾被 write(read()) 截断，终审发现后从 git show 重建 re-amend，提交版完整。
- `next_responsible_role: review`（R7 PASS，见上条）。

### 2026-10-21 独立 final review R6 — needs_fix（已关闭）

- `stage: review`；`review_scope: final`；`reviewer_identity: 原R1/R4独立复审聊天，与实施聊天独立；按用户再次review请求执行，未创建agent/聊天`；`plan_id: PLAN-829`；`plan_revision: 1`。
- `outcome: needs_fix`；`worktree_path: D:/autostack/.wt/lang-829/auto-lang`；`branch: plan-829-dev`；`reviewed_commit: b84c6d91e83a00d4101e6e42403820127516c061`；`base_commit: fdb1fad2ce7afa124adfa3080323496df384e670`。实施树入口及结论前clean、SHA未漂移；但clean未包含ignored必需文件，不能代表冻结交付闭包完整。
- `dependency_revisions: wgpu27.0.1 / winit0.30.13 / iced及iced_wgpu0.14.0 / stock iced_widget0.14.2`；锁SHA256=`6cd345763c2dd44814db4a367fe636010478f9745577b8b06e3c2c573b08e988`。`spec_inputs`/Plan入口hash/源样本/实际本地清单与像素输入hash见 [冻结输入](attachments/829/review-r6/input-sha256.txt)；三份SD原文冻结在同目录，不批准本轮沉淀。
- `findings`：**F829-R6-01..03 OPEN**。P2/01：冻结Main.gd实际传SpawnLocation.position，Godot4.4.1直接执行原Main.gd/Mob.gd四次均生成y0（path y1但follower local y0/global y1）；AGP实际Spawned y1，原feasibility“匹配源样本”误读父/局部坐标。P2/02：原样提取的当前截图分析器，真实正控绿/透明负控红，但25,600个3D天空色越界padding像素仍`clip_ok/ok=true`；完整视图下移80px仍验绿（观测top178反推自身为期待top，实际固定布局top98）。P1/03：必需`testdata/golden_frames/manifest.json`被`.gitignore:161 *.json`忽略，不在HEAD；从git archive纯提交导出，真实committed frame_oracle读得`has_golden=false/compare=None`，仅在审查fixture加入本地ignored清单后立即`Some(0,0,true)`。实施未修，清单文本/错误/raw均留证。[完整报告](attachments/829/review-r6/review.md)含既有AC、精确入口、期待/实值、正控、因果与闭合边界。
- `closure/evidence`：97/97测试、无compiler warning、fmt0；R4-01角色身份贯穿death/replay已过，02死后Space retry无残留jump已过，03原joint-yaw对照false/false已过，04旋转snapshot与符号等价已过；本轮retry截图与R4旧截图独立比较恰3563px、bbox[195,408,295,474]，**接受该局部重冻为诚实期待修复**。3D正式headless probe exit0但依赖本地ignored清单，不证明源高度或纯提交可复跑。R4-05透明负控像素分类器有效，完整裁剪/位置仍partial（R6-02）。R1-F07源oracle与交付输入闭包仍partial（R6-01/03），不是新增全Godot兼容要求。
- `runtime limitation`：当前原版Iced/window正控90s外部截止，仅initial离屏帧，无合成截图/报告；已仅终止本轮owned进程树。该结果不被诊断为产品挂起或新的环境根因；不凭R5旧dirty收据冒领当前冻结live验证，修复后仍须实跑positive+negative。Godot反例为原source emitter最小fixture，未冒称全游戏模型/音频导入验收；像素反例执行exact当前函数，未冒称当前renderer实际输出该故意损坏图片。证据入口 [replay.ps1](attachments/829/review-r6/replay.ps1)，JSON报告以`.json.txt`保留。
- `acceptance_results`：AC-01/04/09 fail；AC-02范围pass；AC-03实现证据保留、完整交付复跑partial；AC-05/06/07/08 partial/fail（详报告）；AC-10 merge-owned pending。SD-01交付入口、SD-02源profile/replay、SD-03可见裁剪及新截图事实需修正并复审；目标路径/GOAL-008保持，无canonical/live ledger写入。
- `next: work`：原树同批修复并提交完整新SHA后再审。保留T-02/T-03/T-04（含R4已修snapshot），重开或保持T-01/T-05..10，`status: executing`，`current_step: 3/11`，T-11 pending。E16诊断应补原source local/global坐标、双视图裁剪与独立布局期待、committed-vs-ignored输入边界，不能只回退旧例证明测试红；不授权延期、削减样本或以补清单的审查fixture代替实际修复提交。
- `pending_closeout_gates`：修复最终验收；冻结独立workspace门禁与Iced正反控完成；仅提交内容的两游戏/真实consumer重定位；迁移/写面交接放行；受管Spec/派生ledger沉淀；归档/guard清理。未跑根t/tv/tt/tb/tu/taa/tf/docs_gen，未改实施、合入、归档、清树或向其他聊天发消息。

### 2026-10-21 独立 final review R5 — PASS（历史；R6发现源/门禁/交付缺口）

- `stage: review`；`review_scope: final confirmation`；`reviewer_identity: 独立 explore 子代理上下文（R2/R3 复审者延续），与实施代理不同实例`。
- `plan_id: PLAN-829`；`plan_revision: 1`；`outcome: pass`；`worktree_path: D:/autostack/.wt/lang-829/auto-lang`；`branch: plan-829-dev`；`reviewed_commit: b84c6d91e83a00d4101e6e42403820127516c061`（r4 修复批，基线 077de3b66）；树 clean、diff 仍仅 AGP。
- 范围：R4 findings F829-R4-01..05 定向关闭确认 + 无回归抽查。全部 CLOSED，且每项以 throwaway 副本回退实验证明永久测试判别力（非仅复跑正控）：R4-01 角色锚定（Scene::first_spawned + ChangeLog 重放一致；**金帧重冻结经复审者独立像素 diff 验证为诚实期待修复**——3563 px 单 bbox 橙玩家→深蓝敌人，Player.gd:68-70 die()=queue_free 合同依据，非 oracle 重定义）；R4-02 死后 Space/Enter retry（无残留跳边）；R4-03 detector 相对坐标系修复（canonical 仍 bit-exact 未重冻）；R4-04 snapshot 含符号规范化旋转；R4-05 iced 0.14 `window::screenshot` 合成内容门（非自镜像；R4 透明场景下 F05 计数仍绿而内容门三时刻全红）。
- `evidence`：[review-r5/review.md](attachments/829/review-r5/review.md)；97/97 测试、fmt/warning 干净、双模式游戏探针 0、negative-controls/metrics/lifecycle/shared-runtime 0、consumer 干净态绿（指纹 b84c6d91e83a 双侧一致）。
- 限制明示（merge-owned 重跑项）：复审者本会话 iced windowed 实跑环境受阻（compositor 截图回读在代理会话不返回；工作方同机录制 negative 红 + positive 绿 基于提交未变代码），交互会话重跑（positive + transparent_blit negative + 僵尸进程预清理）排队为 merge 门禁。
- 新非阻塞项：**F829-R5-01**（R3-01 延续，无界点击等待，留 KNOWN-DEBT）；**F829-R5-02**（运行手册：超时残留同标题僵尸进程毒化 FindWindow，重跑前 taskkill）；**F829-R5-03**（SD-03 IH-01 沉淀时需补内容门事实与 E16 规则引用——T-11 修正点）。
- `acceptance_results`：AC-01..AC-09 **PASS**（AC-09=R5 通过）；AC-10 merge-owned pending。
- `pending_closeout_gates`：T-11 全套（交互会话 iced 双跑含僵尸检查、sabotage 抽查、真实 consumer 重定位、**迁移交接放行授权**、SD-01/02/03 沉淀含 R5-03 修正、债务登记（R5-01/02、Godot 角色级对比）、归档、wt-guard 清理）。
- `next: merge`（前提同上：迁移交接放行授权，非本复审可代决）。

### 2026-10-21 work r4 修复移交（R4 → R5）

- `stage: work`；`outcome: pass`（修复批次）；`code_commit: b84c6d91e`（+1022/−44，11 文件，AGP-only）。
- F829-R4-01 角色身份：first_spawned 锚定贯穿死亡/复用/重放；retry 金帧按 Player.gd:68-70 期待修复重冻（E16 记录 bbox 分析，squash_moment/final_frame bit-identical 未动）。F829-R4-02 死后 Space/Enter retry 全键位审计。F829-R4-03 detector 相对系修复（rotation_y(player_facing−mob_yaw)），canonical bit-exact 未重冻。F829-R4-04 snapshot 含旋转（符号规范化，等价定向仍相等）。F829-R4-05 合成内容门：iced 0.14 `window::screenshot` 三时刻像素断言 + 永久 transparent_blit 反控（实测红：F05 计数绿 2→104→111 内容全红）。
- 诊断扩展（E16）：六个漏检不变量逐一点名+早期遗漏原因+永久控制绑定；立规"门必须命名不变量，复跑必须 exercising 完整不变量"。
- 验证：97/97 测试、双模式矩阵全 0、iced windowed pass、independent-consumer 干净态绿；报告 `scratch/829/r4/`。
- `next_responsible_role: review`（R5 PASS，见上条）。

### 2026-10-21 独立 final review R4 — needs_fix（已关闭）

- `stage: review`；`review_scope: final`；`reviewer_identity: 原R1复审聊天，与实施聊天独立；本轮未创建agent/聊天，未以R2/R3标签代替验收`。
- `plan_id: PLAN-829`；`plan_revision: 1`；`outcome: needs_fix`；`worktree_path: D:/autostack/.wt/lang-829/auto-lang`；`branch: plan-829-dev`；`reviewed_commit: 077de3b6697c69053c24f4067bf73188ca90ab62`；`base_commit: fdb1fad2ce7afa124adfa3080323496df384e670`。入口与结论前树clean、SHA未漂移，123文件均限AGP、无reparse点。
- `dependency_revisions: wgpu27.0.1 / winit0.30.13 / iced及iced_wgpu0.14.0 / stock iced_widget0.14.2`；AGP锁SHA256=`6cd345763c2dd44814db4a367fe636010478f9745577b8b06e3c2c573b08e988`。`spec_inputs`/Plan入口hash/关键源码与SD哈希见 [冻结输入](attachments/829/review-r4/input-sha256.txt)；三份SD原文冻结在同目录 `spec-delta-*.md`，本轮不批准沉淀。
- `findings`：**F829-R4-01..05 OPEN**。P2：① canonical tick1372玩家已销毁，`project_instances`却按首个存活对象将敌人index6/generation4投影为Player（真实retry截图仍有橙色玩家，官方门绿）；② LIVE Space死后不产生retry，Enter正控可重开；③ player/mob整体旋转90°后同几何detector false→true，实际driver由存活变死亡；④ 仅Y旋转180°，矩阵不同而Scene snapshot相同。P1：⑤ 最终Iced片元全部透明，真实click/resize与draw计数照常、官方 `exit=pass/composite.ok=true`。完整输入、路径、因果、正控、边界与复跑源见 [R4报告](attachments/829/review-r4/review.md)。
- `finding_closure`：R1-01自动跳/R1-02重绘计时/R1-04缓存/R1-09去vendor/R1-10健康的原问题关闭保留；R2-01..05定向修复保留。R1-03联合方向、R1-05可见合成、R1-11完整类型/状态不变量仅partial；**R1-07仍partial/open**：实现来源金帧把源样本禁止的死后玩家形象冻结并验绿，原始Godot参照缺角色资源不能补足该期待，R2/R3 reviewer接受/债务登记不能替代合同内验收。不是要求完整Godot兼容或同RNG逐像素对拍。
- `evidence`：90/90测试、零compiler warning、fmt0；36/36来源/资产/字体/金帧hash一致。当前真实runner breakout/headless、squash/headless、lifecycle、metrics、negative与原版Iced/window六例exit0；独立public-API probe四项反例全部复现。Iced独立标题正控与透明片元反控均exit0，透明例draw计数1→19→22。第一轮注入误选旧iced/image缓存库为无效基线尝试，已纠正；另一次同标题等待超时无画面结论，均留痕并只清理本轮自建进程。结论只使用Cargo JSON解析的当前依赖与成功的独立标题成对证据。报告/PNG/输入/源副本与 [复跑入口](attachments/829/review-r4/replay.ps1) 保存在主检出；JSON以`.json.txt`快照保留以避忽略规则。未重跑consumer/另三完整探针，不冒领新运行证据；R3同SHA收据限定保留。
- `acceptance_results`：AC-01/02/05/06/08 partial；AC-03保留当前2D定向与R3同SHA窗口证据的范围pass；AC-04/07/09 fail；AC-10 merge-owned pending。SD-01身份边界合理但原样本事实需修正，SD-02/03完整状态/可见合成等尚未验收；新Spec路径/GOAL-008元数据保留，未写canonical/live ledger。
- `next: work`：原树同批修复、提交新SHA、再审。重开T-01/T-03/T-06..10，保留T-02/T-04/T-05，`status: executing`，`current_step: 3/11`。多轮失败诊断需在E15模式维度之外补齐 alive/dead/leading-object删除/ID复用、Space/Enter、player/mob联合方向、旋转单属性及draw但不可见的矩阵，并将原例关闭与完整不变量关闭分开；先独立确立content期待再重冻金帧。
- `pending_closeout_gates`：修复后最终验收与冻结独立workspace门禁/真实consumer重定位、迁移/写面交接放行、受管Spec/派生ledger沉淀、归档与guard清理。未跑根t/tv/tt/tb/tu/taa/tf/docs_gen，未改实施、合入、归档或清树。R3-01无界输入等待债务历史保留，不以不稳定环境结果新增画面finding。

### 2026-10-21 独立 final review R3 — PASS（历史，R4发现完整不变量缺口）

- `stage: review`；`review_scope: final confirmation`；`reviewer_identity: 独立 explore 子代理上下文（R2 复审者延续），与实施代理不同实例`。
- `plan_id: PLAN-829`；`plan_revision: 1`；`outcome: pass`；`worktree_path: D:/autostack/.wt/lang-829/auto-lang`；`branch: plan-829-dev`；`reviewed_commit: 077de3b6697c69053c24f4067bf73188ca90ab62`；树 clean、diff 仍仅 AGP。
- 范围：R2 findings F829-R2-01..05 定向关闭确认 + 无回归抽查。**F829-R2-01 CLOSED**（离屏路径真实接入 frame_oracle，breakout/squash windowed+headless 四跑全 exit 0）；**F829-R2-02 CLOSED**（consumer A-side 在案内 isolated-target 真实重建，清洗态连跑两次全绿、零不匹配）；**F829-R2-03 CLOSED**（vendored 陈旧表述全清扫，第四处遗漏亦修）；**F829-R2-04/05 CLOSED**（60 tick 勘误、45 事件/90 测试实数对齐）；**E15 两次失败诊断核实**（不变量点名、 earlier-miss 解释、per-mode 调整实证执行）。
- `acceptance_results`：AC-01..AC-09 **PASS**（AC-03 windowed 模式缺陷已随 R2-01 关闭；AC-09=R3 通过）；R2 决策项裁定：(a) Godot 角色级引擎对比**可接受为替代 oracle 合同**（独立 Python 参考+冻结常量+引擎轨迹语义基线），角色级对比登记 deferred 债务；(b) 实现来源内容金帧**可接受**（provenance 声明+sha256 固定+腐化可判别）。AC-10 merge-owned pending。
- 新发现：**F829-R3-01（P3，预存非回归）**——iced phase-1 等待 `click_observed` 无界，点击重试耗尽时永久挂起而非带因 exit 1；建议 fail-phase 转换，登记 KNOWN-DEBT-AND-RISKS.md。iced windowed 实跑本轮环境受阻（超时残留僵尸窗口毒化 FindWindow + 活动桌面输入 flake，E14 族）；**决定性证据：`git diff 95ccfcc5c..077de3b66` 对 iced 相关文件为空**（与 R2 实测 windowed pass/blank-blit sabotage 红 的代码逐字节一致），已排队为 merge 门禁在交互会话重跑。
- `evidence`：[review-r3/review.md](attachments/829/review-r3/review.md) 及同目录原始报告；90/90 测试、fmt/warning 干净、negative-controls 5/5 门绿。
- `pending_closeout_gates`：T-11 merge-owned 全套（冻结提交全 AGP 测试+10 探针含交互会话 iced 重跑含僵尸检查+sabotage 抽查、真实 consumer 重定位、**迁移完成交接门放行授权**、SD-01/02/03 沉淀（R2-03 修正后 SD-01/03 可存）、债务登记（R3-01 + Godot 角色级对比）、归档、wt-guard 清理）。
- `next: merge`（前提：迁移交接放行授权——计划 T-11 前置，非本复审可代决；授权前原型保留 plan-829-dev 分支）。

### 2026-10-21 独立 final review R2 — needs_fix（已关闭）

- `stage: review`；`review_scope: final`；`reviewer_identity: 独立 explore 子代理上下文`。
- `reviewed_commit: 95ccfcc5c`；R1 findings F829-R1-01..11 **全部 CLOSED**（每条以再生突变/注入实证判别力，非仅复用旧证据）；AC-01/02/04/05/06/07/09 PASS，AC-03 PASS（附模式缺陷注记），AC-08 PASS 待 SD 措辞修正。
- 决策项裁定：(a) Godot 角色级 ground truth **可接受为 source-oracle 合同**（计划未强制引擎运行；替代 oracle 链满足独立性），角色级对比登记 deferred 债务；(b) 实现来源内容金帧 **可接受**（声明 provenance、sha256 固定、腐化可判别）。
- 新 findings（全部 r3 修复并经 R3 确认关闭）：**F829-R2-01（P2）** ag2d-breakout headless/离屏路径未接 F07 内容 oracle 确定性 exit 1（门接线缺陷，帧与金帧 bit-exact）；**F829-R2-02（P3）** consumer A-side 复用预建二进制在冻结提交上 false-red 非自足；**F829-R2-03（P3）** 三处"vendored patch"陈旧表述与 F09 去 vendoring 矛盾；**F829-R2-04（P3）** manifest §2.6 残留"30 physics ticks"；**F829-R2-05（P3）** 陈旧计数（47 事件/85 测试）。
- `evidence`：[review-r2/review.md](attachments/829/review-r2/review.md)；路由：`status: executing`，r3 小批修复 + 定向再复审；两次失败诊断按要求留档（E15）。
- `next: work r3`（已执行：077de3b66）→ review R3（PASS，见上条）。

### 2026-10-21 work r2 needs_fix 修复移交（R1 → R2）

- `stage: work`；`plan_id: PLAN-829`；`plan_revision: 1`；R1 结论 needs_fix（F829-R1-01..11 全开放），退回 executing 后修复完成，`outcome: pass`（修复批次移交，非最终 review）。
- `worktree_path: D:/autostack/.wt/lang-829/auto-lang`；`branch: plan-829-dev`；`base_commit: fdb1fad2ce7afa124adfa3080323496df384e670`；R1 冻结 `f7ea95f3b` → 修复链 `03f4999f4`（批1）→ `b5c1ed1af`（批2）→ `1fc3f6116`（批3）→ `95ccfcc5c`（E14 证据簿记）；`code_commit: 95ccfcc5c`。
- 修复内容（11/11 关闭，每条含不变量级修复+回归测试+破坏演示红）：
  - 批1 `03f4999f4`：F01 LIVE 跳跃键真实接通（按压边沿状态机，idle 不跳/按住不重复/死后门控）；F02 LIVE 时钟改 elapsed-time 累加器（cap+债务丢弃，30/60/120 cadence 不变性；canonical 路径不动仍 bit-exact）；F03 生成瞄准实体玩家位置+五路 MobDetector 致命通道+朝向/动画速度4/1/跳跃俯仰/mob 本地相位（golden trace 语义变更经独立生成器再冻结 seed 0x1B）；F11 squash 全量接入共享身份/场景增量/change-log，独立消费者可重放重建与编辑 3D 场景（含 restart 后失效 ID 拒绝）。
  - 批2 `b5c1ed1af`：F04 缓存键/项绑定已验证 owner+代际、同键异尺寸 miss-and-recreate、ID 随逐出/释放回收（reviewer 攻击序列与 64 周期泄漏环成永久反例）；F06 生命周期全改实测（DropWitness 派生 drop 计数、真实游戏资源 20 轮、取消回调 awaited、teardown 后 clean_exit 入 all_ok、去除提前 process::exit；mem::forget 20 目标永久红控）；F08 创建/重写字节/重建分路实测、原始序列留存、报告三指纹（agp_commit/lock sha256/asset sha256）、缺观测不 PASS（sabotage 红实证）。
  - 批3 `1fc3f6116`：F05 Iced 门改真实合成路径计数（BLIT_DRAWS 仅紧邻真实 composite draw 递增）+永久 blank-blit/foreign-device 反控（破坏 exit 1 实证）；F07 manifest 就地勘误（seed 0x1B/60 tick 与冻结 trace 对齐）、runner 全 payload 事件/全 spawn 元组比对、6 张内容金帧逐像素 mean/P99 门（非 distinct-hash 计数）、Godot 4.4.1 原始引擎 3600 tick 地面真值采集（角色因 headless 导入限制未渲染，决策项录 METHOD.md）；F09 去 vendoring——实测 stock iced_widget 0.14.2 满足 spike，删除整棵 patch 树，AGP 零外部 path 依赖，重定位复验（R1 锁 hash 已随锁再生更新）；F10 fmt/warning 清零、文档与 SD 提案全部对齐实测事实（含 handoff :23 伪称更正）。
- `evidence`: 90/90 测试；10/10 probe case exit 0（iced windowed、consumer 真实重定位重建）；批2/批3 sabotage 演示均红；闭合矩阵见 `AGP/research/result.md`，E12/E13 证据注记。
- 遗留决策项（不阻塞移交，R2 裁决）：Godot 原始引擎角色级画面对比 headless 受限未竟（METHOD.md 记录方法与差异）；内容金帧源自当前 bit-exact canonical 运行（来源已声明，R2 可裁定是否接受该替代 oracle）。
- 补充（E14，commit 95ccfcc5c）：用户报告 iced_spike LIVE 点击 "Step graphics" 后卡死——7 次带仪表实跑未复现（事件循环全点击模式下 30 tick/s 存活）、锁图审计无环、用户确认当时 press 为其手动且现不再卡死；归因为报告窗口期环境级 CPU/GPU 饥饿（并行重载），未改代码，基线不变；证据存档 `AGP/research/evidence-notes.md` E14 与 `scratch/829/live-freeze/`。
- merge-owned 门禁仍 pending（R2 pass 后冻结提交执行）；`next_responsible_role: review`（R2，同树冻结提交 `1fc3f6116`）。

### 2026-10-21 独立 final review R1 — needs_fix

- `stage: review`；`review_scope: final`；`reviewer_identity: 当前 Codex 复审聊天，与实施聊天独立；未创建其他 agent/聊天，未虚构模型/会话ID`。
- `plan_id: PLAN-829`；`plan_revision: 1`；`outcome: needs_fix`；`worktree_path: D:/autostack/.wt/lang-829/auto-lang`；`branch: plan-829-dev`。
- `reviewed_commit: f7ea95f3b71459914cc081d70dd62123c1303a6d`；`base_commit: fdb1fad2ce7afa124adfa3080323496df384e670`；入口与结论前实施树均 clean、HEAD 未漂移；174新增文件均在 AGP 内，生产 workspace/锁未改，reparse扫描为空。
- `dependency_revisions: wgpu 27.0.1 / winit 0.30.13 / iced及iced_wgpu 0.14.0 / iced_widget 0.14.2 path patch`；AGP锁 SHA256=`d9e1fdae2c2d6d299983796f766442599dac9482dd79575942f2db85c3bee235`。`spec_inputs` 与输入Plan hash见 [冻结输入](attachments/829/review-r1/input-sha256.txt)；SD-01..03逐份原文已冻结至同目录 `spec-delta-*.md`，不是可发布的已通过规范。
- `acceptance_results`：AC-01/02/03/08 partial；AC-04/05/06/07/09 fail；AC-10 merge-owned pending。完整 AC→任务→实现→证据及 SD审查见 [review.md](attachments/829/review-r1/review.md)。`supersedes_spec_components: []` / 三个 `new_spec_components` / `touched_goals: [GOAL-008]` 保留；未发布 canonical Spec/ledger。
- `findings`：**F829-R1-01..11 全开放**。P1：LIVE Squash 无真实跳跃键控制；Iced 最终合成完全不画仍 PASS。P2：LIVE 两游戏按重绘数推进导致真实时间倍率变化；Squash 原点瞄准/缺 MobDetector 与朝向动画状态；缓存旧key绑定新owner且资源ID不释放；生命周期计数/前后采样/关闭取消误证；原样本oracle与像素验收缺口；计量/指纹不完整；未经合同批准复制patch源码；3D绕过共用scene/身份/变更合同。P3：测试warning/格式和报告/规范提案事实冲突。精确路径、反例、修复边界均在报告；登记债务不豁免合同内缺口。
- `evidence`：独立实验workspace测试 **74/74**（非原移交73，4条warning）；30/30资产/来源/字体hash匹配；真实runner定向复跑 breakout/squash/lifecycle/metrics/negative/Iced 六例均exit0。另行实跑：玩家x=0与7.116672时敌人yaw相同；源detector相交但driver不死；缓存释放重用仍取旧纹理、64次create/release后allocator live3→67；**故意保留20个GPU目标仍报baseline恢复、关闭最终Iced图形绘制仍报all_gates_behaved=true**。反例副本、原始JSON/关键帧/stdout与 [复跑入口](attachments/829/review-r1/replay.ps1) 永久保留；未改实施源码。fmt exit1、diff-check exit2，raw在附件。其他四探针未重新执行，不授予本轮运行pass。
- `pending_closeout_gates`：修复后新冻结SHA的受影响复审；最终pass后独立workspace全适用测试/探针与真实consumer/重定位、迁移交接放行、Spec/派生ledger沉淀、归档与wt-guard清理。未触生产Rust，未跑根 t/tv/tt/tb/tu/taa/tf/docs_gen；不得用原型通过替代I1–I3。
- `next: work`：同一实施树一次修齐并补真实入口/故障反控；提交后按finding closure matrix再送审。T-01/T-02/T-04..09重开，保留T-03，`status: executing`，`current_step: 1/11`；T-10/T-11继续未完成。未合入、归档、清树或改语义合同。

### 2026-10-21 work r1 完成移交

- `stage: work`；`plan_id: PLAN-829`；`plan_revision: 1`；`outcome: pass`。
- `worktree_path: D:/autostack/.wt/lang-829/auto-lang`；`branch: plan-829-dev`；`base_commit: fdb1fad2ce7afa124adfa3080323496df384e670`（master 冻结）；`code_commit: f7ea95f3b`（T-01..T-09 共 10 提交，6c2448261…f7ea95f3b，全部仅触 `experimental/autographics-prototype/`，根 workspace/锁零改动）；`dependency_revisions: wgpu 27.0.1 / winit 0.30.13 / iced 0.14.0 / iced_wgpu 0.14.0（与生产 iced_wgpu 同实例，AGP 自有锁）`。
- `task_ids: T-01..T-09 全部完成`（T-10 review / T-11 merge 未启动，不占 work 前置）；`current_step: 1 / total_steps: 11`。
- `evidence`: 73/73 测试（lib+contract+breakout+squash+threed+verifier+iced_seam 全具名）；10/10 probe case exit 0（gpu-foundation、shared-runtime、ag2d-breakout、ag3d-risk-slice、ag3d-squash、lifecycle、metrics、verifier-negative-controls、iced-shared-device、independent-consumer；V-01..V-09 全覆盖，V-08 五门反控+破坏验证器 exit 1 实证）；双 golden trace（breakout 10800 tick / squash 3600 tick）对独立参考实现 bit-exact；报告与关键帧在 worktree `scratch/829/`，AC 绑定总表 `AGP/research/result.md`，交接合同 `AGP/research/integration-handoff.md`（IH-01..05 + I1–I3 显式 PENDING），SD-01..03 提案稿 `AGP/research/spec-delta/`。
- `blockers: 无`。merge-owned 门禁（冻结提交上重放本 workspace 全适用测试/探针 + 交接前置核对）按 2026-10-14 裁定在 review 通过后由 merge 阶段执行，此移交时 pending 明示，未冒称已跑。
- `next_responsible_role: review`（同树冻结提交验收；不运行根整档替代验收）。

### 2026-10-21 new r1 规划预检

- `stage: new`；`plan_id: PLAN-829`；`plan_revision: 1`；计划状态仍为 `drafting`。
- `outcome: pass` 仅指规划合同预检；`next: work`（用户确认本具体r1并授权执行后）。本轮用户只要求计划，不据此开始编码或运行原型。
- 新任务 T-01..11、验收 AC-01..10、规范增量 SD-01..03；各AC有work/review/merge owner及验证映射。
- 复核了共用基础/并行门、当前原型与后继Auto/UI边界、真实source规则/资产/实例与GPU身份、失败/释放及验证器反控；未知实现决策有T-01/T-02 owner和决策产物，后续生产入口具名IH合同不冒称已实现。
- 预检见 [receipt](attachments/829/planning/preflight.md)。这不是独立final review、功能pass或merge门通过；当前无实现证据。

## 10. 待澄清事项

本r1没有阻止完成计划的必答问题。以下由指定任务以证据解决，不能默默改变目标：

1. T-01：原例完整资产/实际播放音频与动画分母、参考运行环境；允许的键鼠profile已在§1明确，其他外设差异须披露。
2. T-02：局部ECS/组件存储、模型/碰撞/文本/音频库、wgpu/Iced依赖与patch、资源预算/失败分类；如果不可行，回new修订而不砍验收。
3. T-08：迁移后公开SDK/renderer/protocol的实际版本与符号、接口owner和迁移完成收据；目前不能编造不存在的生产API或承诺日期。
4. 真正多人并发实现需要另外确认模块owner和后继Plan分解；本轮未请求创建agent或chat，不默认分派。

## 11. spec-sync 回写记录（2026-10-22 merge）

- 新建 `docs/specs/auto-graphics/`：project.md（SD-01，含分支归属裁定事实）、prototype-profile.md（SD-02 实测支持面）、integration-contract.md（SD-03，含 R5-03 counter/content 双门措辞修正与 I2=master 合回裁定前提）、plans.md（本计划行）。
- 全局件：overview.md 外围与实验表 +auto-graphics 行；goals.md GOAL-008 关联 +829；INDEX.md 再生（spec-index.py GROUPS 登记"实验/沙盒"）。
- ledger：.autoos/specs.json v64→v65 guarded offline projection，P829-AGP / P829-R18 / P829-MERGE / P829-AGP-TESTS 四条（file/tags 全带，branch:autographics-dev 溯源）。
- 知识边界：代码唯一落点=`autographics-dev` 分支（merge `ab56c32ec`）；master 侧仅文档；I1–I3 pending 以 integration-contract.md 为准，后继计划只能以绑定收据替换 pending。

### cleaned 补记（2026-10-22）

- `bash D:/autostack/wt-guard.sh D:/autostack/.wt/lang-829/auto-lang` → **clean**（无 reparse point）；worktree 树 0 脏文件、全部提交已含于 `autographics-dev`（`git merge-base --is-ancestor plan-829-dev autographics-dev` 实证后才 `git branch -D`，删除值=ddbde3e62）。
- worktree 注销+目录删除+组目录 `D:/autostack/.wt/lang-829/` 移除；`git worktree list` 仅余主检出与 815/816/817/818/828 在途组。分支清单仅余 `autographics-dev`（829 相关）。
- 备注（终合并时注意）：`autographics-dev` 基于的 master 祖先含 f9f0cf04c（本计划 gate 簿记版计划文件于 docs/plans/），master 现已将计划文件改至 archive/ 并增终收据——终合并时按 rename 处理即可（分支侧未再改动该文件）。
- 批量回归到期复核：收据 last_covered_plan_id=817 @2026-10-21T06:27Z，<48h 且其后无计划落地（829 非整 5 号、落集成分支）→ 不到期，无需 /auto-plan:regress。
- `checkpoint cleaned` ✅；merge 五 checkpoint 全闭，`outcome: pass`。
