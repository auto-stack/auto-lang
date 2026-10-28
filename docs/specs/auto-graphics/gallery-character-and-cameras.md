# AutoGraphics Gallery 公共 3D 角色与两维运行相机（GY-B）

Status: delivered（PLAN-880）。本文件权威记录 ag-examples 中 G07（3D 角色，
固定方向基）与 G08（运行相机 2D/3D）两逻辑项的公共消费知识：按项锁、四
子入口、无窗口验收适配与包边界。旧六项（GY-A）见
[gallery-basic-six](gallery-basic-six.md)；引擎能力权威事实在
[game-character3d](game-character3d.md) 与
[runtime-game-cameras](runtime-game-cameras.md)，本文不扩大引擎 profile。

## 目录形态（PLAN-880 后）

- ag-examples `c36c412`（master，PLAN-880 分支 `codex/plan-880-dev` ff 合入）：
  目录共 8 逻辑项/14 必需子入口。新增 g07/layout-a、g07/layout-b、g08/2d、
  g08/3d（全 ready）；能力 C07（3D character）/C08（runtime cameras）；
  g07 子入口 dimension=3d。
- 内容分母：旧六项/十入口原 pin 原验收记录不动（根 `engine-lock.json`
  逐字节不变）；`validate --all` 强制 8/14 集合相等。

## 按项引擎锁（engine-locks/plan875.json）

- `example.json` 的 `engine.lock` 按示例指向仓根相对锁路径：旧六项指根
  `engine-lock.json`（861，runner `498ce814…`），g07/g08 指
  `engine-locks/plan875.json`。
- plan875 锁身份：AG `1cc68aba0`（875 最终交付，含 874；autographics-dev
  历史可达）、SDK pin `7b987ca8`/681 文件/17 本地包（875 保留副本
  `prepare_sdk --verify-only` PASS）、两 workspace 锁 `47176cc0`/`bc7287b5`
  （与 861 锁同值）、platform 栈 32MiB 主/16MiB worker、runner
  `autographics-authoring` SHA-256 `6c461c35…`、identity manifest
  `plan880-consumer-identity.json`（1385 文件逐文件来源收据，sha256
  `6f100c88…`；随引擎分发走，不入内容仓）。
- 锁路径必须受控 root 内相对常规文件（绝对/逃逸/链接/未知 schema/坏身份
  拒绝）。launch/validate/package/catalog/acceptance 全按本项锁判定引擎
  身份：861 runner 启动 g07/g08、875 runner 冒充旧六项均拒绝。
- 构建配方 = 消费包内 `cargo build --locked --offline --bin
  autographics-authoring`（包根 `.cargo/config.toml` 携带 32MiB PE 主栈
  `/STACK:33554432`，blob `992c9e1e`；缺它主线程 `--inspect` 溢出——
  PLAN-880 T-05 实测修正）。第三方 registry 缓存是唯一外部前置。

## 内容与源码消费（字节恒等）

| 子入口 | scene | behavior（api） | 对照源 |
|---|---|---|---|
| g07/layout-a | character3d/layout_a.agscene | main.at（character3d） | main-fast.at（speed 4→5） |
| g07/layout-b | character3d/layout_b.agscene | main.at（共享同字节） | main-fast.at |
| g08/2d | runtime-camera/two/layout_a.agscene | main.at（camera2d） | main-alt.at（rect/view 换配置） |
| g08/3d | runtime-camera/three/layout_a.agscene | main.at（camera3d） | main-alt.at（dist6/pitch30/ortho16） |

全部 `autographics.scene/2`、零内嵌 behavior、零资产；12 个消费文件与
git@1cc68aba0 树内样本逐字节相等（合同冻结 sha256）。玩法参数归用户源
（speed4/jump8/gravity20/clearance0.002、对角 0.7071 归一；2D rect
x[0,30] y[-4,12] viewport 12x8→夹界 [6,24]、letterbox 570x380/双侧
115px；3D orbit dist8/pitch45/persp、zoom 1.1/notch∈[4,12]、drag
0.25°/px、pitch 封顶 75°）。正常启动 `gallery.py launch` 只传公共 CLI
（`--open/--behavior/--behavior-api`）；`--frames/--report` 无输入不证明
玩法；不调用 OS 自动化。

## 无窗口验收（checks/gyb + tools/verify_gyb.py）

- 验收适配 = 提交的 oracle 源 `checks/gyb/gallery_runtime.rs` +
  物化驱动 `tools/verify_gyb.py`：以显式 `--engine-root`（固定消费包）
  物化独立 cargo workspace（依赖仅 driver+agp+glam+serde_json，
  `--locked --offline`、`--test-threads=1`、64MiB 线程护栏），对实际
  examples 文件走公开 Storage/SourceSnapshot/Host/Play → Host 发布 →
  `SurfaceRuntime::capture` 无窗口 GPU PNG。不改引擎、不触 private、
  不发 OS 事件。
- 32 oracle 分母（全绿，新鲜重跑可由 `--check-only` 复验）：G07 导入/
  真跑贴地/冻结输入不动/speed4→5 敏感/对角归一/x 墙 5.498 接触包络/
  跳边沿一次（含 layout-b 同源坠坑分歧）；G08/2d tick0 夹界(6,1)/
  follow∈[6,24]/同 tick 共发布/letterbox 115×380+alt 146.67/暂停不动/
  PNG 800×380；G08/3d orbit 源内读/独立 glam VP 恒等（逐 tick + 旧位
  检出）/yaw90 世界转 +x/像素随 orbit 变/暂停缩放丢弃/zoom 封顶/
  Stop 文档复原/重启原位/alt ortho 配置+VP/世界轴替换负控可区分。
- 严格 acceptance：非根锁示例逐子入口绑定消费源 sha256
  （`previews/<slug>/acceptance.json`，runner 绑定 `6c461c35…`）；
  源漂移即 validate 失败（Ready 失效）。旧六项沿用原 schema 不伪补。

## 包闭环

- 八个单项 ZIP：g07/g08 包只含本项子入口+README+本项锁+过滤目录+工具+
  acceptance，**不含根 861 锁/旧六项**，解包后独立 `validate --all` 通过；
  8/8 离线 A→B 搬迁（validate+reopen 恒等）+ 坏包负控（资产篡改引擎拒；
  g07/g08 行为源单字节翻转 → 严格绑定 validate 红 → 修复还原绿）。
- 引擎单独固定包（plan-artifacts）：消费包源树+`.cargo`+identity
  manifest+预构建 runner（sha 锁定）；重定位 A→B inspect 双绿；离线重建
  与预构建 exe 不同哈希（只有预构建件匹配锁 pin）。

## 实测边界（不声称）

- 仅 windows-native-x64；新 runner/新包/新路径的 OS 真键鼠路由未重测
  （OS 自动化暂停，2026-10-28 裁定）；874/875 历史 OS 证据仅作未改
  引擎/router 的能力来源。浏览器在线/移动/远端发布非目标。
- G07 静态占位外形（不认领完整 G3-01）；G08 仅单角色单相机有限服务；
  Enter 只动作不宣称重开游戏。AG 全量套件为里程碑档（非合并门）。

Provenance: [PLAN-880](../../plans/archive/880-autographics-gallery-character-and-cameras.md)
（归档），冻结合同
[attachments/880/work/gallery-contract.md](../../plans/attachments/880/work/gallery-contract.md)。
源：ag-examples `c36c412`（基 6a223d9）；引擎消费 git@1cc68aba0（875 交付）；
runner `6c461c35…`；SDK `7b987ca8`（875 保留副本）。G09..12/统一发行基线
（GY-C）由后继计划消费。
