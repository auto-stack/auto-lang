# PLAN-829 r1：new 阶段规划预检

日期：2026-10-21。范围：本仓规划/设计和只读代码、官方样本调查；没有创建实现树/分支、安装依赖、运行游戏或GPU验证。当前主检出 master，调查 HEAD `bf92f4585d8c23a0d26ef7bed26719d90b0168bd`；主线可继续推进，work 进入时须重新核实来源与依赖。

## 输入指纹

| source | SHA256（本次读取内容，非仅 HEAD） |
|---|---|
| `docs/specs/overview.md` | `c23beeb31d1b44f3aa9ace38b9aae10482bf2595152d1c6f78a2e767f0de90a4` |
| `docs/specs/auto-lang/ui/overview.md` | `dcd54eb7140195c24857e38ac297ee9058c0045d5b7363db8c10f39832949c48` |
| `docs/specs/repository-boundaries/project.md` | `5d83f8e5767e9784b0aefaef3b5b3c2c40ec28a3e0c47459f8164ee5e652f764` |
| `crates/auto-lang/Cargo.toml` | `32c0f3c7b562aa9c4fe4dfe0622383104d34ca124cbeff37ce290d4f89c741b6` |
| `Cargo.lock` | `956278dc80c2ea866266799ed276ec91a186d95418a15fc6920964f9b2301e4c` |
| `crates/auto-lang/src/ui/desktop_protocol/rqhost.rs` | `12ab9ba18ade247132a6265c2e79728acb984198af8e9bccbe9869e457e204fd` |
| `crates/auto-lang/src/ui/mpv/present.rs` | `ddb6883879fe128b9002bbb410359b063dad96e82a26c2f8d0f7c89713301031` |
| `crates/auto-lang/src/ui/mpv/widget.rs` | `3e91fd26a302b73513123fd74de83372aadff9ed4a1fadde29073c85aaaee57c` |
| `docs/design/autoui/rq-remote-renderer.md` | `151aaaa3bd496dc40f790f5d266e578acaecbbaf4e6fad746fec2ed60c30f500` |
| `examples/godot/README.md` | `578a8fcaf0b8c7ca4f869cc6459157f96a8aea49a31bdc7afca1847cc083b67a` |
| `docs/design/strategy/autographics-forge-v06.md` | `456768b16b4f86d5c90974ef1122736a14b8ae643bf1ef3edc7f00577f982bba` |
| `docs/design/45-autographics-wgpu-engine-and-game-prototypes.md` | `3e06851963c08aef5562b15ccb1ade111e57bdd0a5356a746992ed07a0987263` |

官方来源已通过 GitHub 只读接口核实：

- Bevy `efe61cc5b9872f3f9e1ee7553325612c094ccc97`；Breakout 路径 `examples/showcase/breakout.rs`，blob `b9bfb8b4b2cd19aa15b460f1323ad5c2af0e5954`。当前官方源码的底墙反弹，未定义掉球失败/生命；音效资源 `sounds/breakout_collision.ogg`，完整资源消费清单仍由 T-01 盘点。
- Godot demos `b761b4cd5718a7118f2c95eb55f181d0a5f004f1`；`3d/squash_the_creeps/README.md` blob `8006f2783acf3ad8baceb63a80269b36b0c16138`，`Main.tscn` blob `105c56d67284326f5efdb5caf426c0f063f9f255`。README 的音频版权说明不等于播放证据；完整脚本/动画/资源消费仍在 T-01 核实。

## 取号与约束核查

- 未找到同目标活动 Plan；活跃/归档最大编号 828，中央计数 829。排他创建 `docs/plans/.new-plan-allocation.lock`，锁内复核 max/计数，调用现有 `scripts/new-plan.sh` 成功取 829，计数变 830，并复核只存在一个 829 文件；finally 释放/删除本次锁。
- 首次沙箱内 Git Bash 启动失败（进程 -1073741502），未生成计划/修改计数；自动审批允许后在沙箱外重试脚本成功。未替换取号算法、未申请修改实现/外仓。
- Design45 按域级文档登记，并把下一设计章号更新为46；只定目标与边界，不改现行 Spec。既有其他 WIP 保留。

## 合同审查结论

- 第一批样本确定为 Breakout 与 Squash；公开 Iced 共享设备是第三个集成探针。来源锁定、profile与原例规则明确，没有把候选全部开工。
- T-02/T-03 形成 G-base；T-05/T-08 可提早实验。2D先闭环和3D并行没有互相等待的环；同树单写、共用核心owner，不自动创建agent。
- 所有AC/SD都有work任务和验证入口；原型Rust driver与I1–I3后继真实Auto/生产UI/RQHost明确分层，没有在merge阶段代做实现。
- 对同设备、局部更新、实例/资源代际、失败/取消、玩法与声音、释放及测量有正反例和观察量；未来入口由任务创建，本轮无功能pass。
- 未知库/资源/预算策略由T-01/T-02有界调查；影响样本/支持面/验收则回new，不授权自行删项。
- 新模块SD均provisional，只有实现复审/merge后写current Spec。原型可独立复审；合入/生产接线等待迁移收据，不把814-A或公开Iced探针当迁移完成。

结果：`stage: new`、`outcome: pass`（仅规划合同预检）、`next: work`（用户确认r1并授权执行后）；Plan保持drafting，0/11任务完成。当前自查不是独立final review。

实跑预检：sections 0–10 齐全，11 个唯一任务/10 个验收/3 个规范增量，drafting 与 current_step=0 一致；829 唯一、计数830；五份本次入口文档的35个本地链接/锚点均有效；未创建实验workspace；触及的既有文件格式检查通过。没有运行 Cargo、docs_gen、样本或性能测试。
