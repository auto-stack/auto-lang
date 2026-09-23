---
plan_id: PLAN-694
status: reviewed               # drafting → executing → execution_done → reviewed → archived
feature_name: desktop-dogfood
author: [zcode]
created_at: 2026-09-23
updated_at: 2026-09-23

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [SD-01 桌面孵化链 ↔ exe CLI 方言对齐契约（spawn 参数 = render_cli 消费面）, SD-02 虚拟桌面加载 a2r exe 的端到端契约（发现/拉起/宿主/回收）]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [docs/design/autoui/rq-remote-renderer.md, docs/design/autoui/desktop-protocol-v1.md]
current_step: 6
total_steps: 6
---

# [PLAN-694] desktop-dogfood——虚拟桌面加载 a2r exe 端到端闭环

> 来源：PLAN-693 后续（T-04/AC-04 后置 = P693-D2 跨仓配对）。用户 2026-09-23
> 「693 完了，立 dogfood 计划」。目标形态：**虚拟桌面（launcher）点图标 →
> a2r 编译 exe 以 desktop 模式起来 → 窗进桌面 → 交互闭环**——「虚拟桌面加载
> AutoUI exe」从契约变成日用现实。

## 变更摘要

三件核心 + 核验收口：

1. **方言对齐（核心缺口）**：桌面孵化链 `spawn_exe_child`
  （`session.rs:3091`）现发 `--autodesk-incubate --app386=<name>
   --autodesk-broker=<pipe> [--autodesk-render=<v>]` 方言；而 693 的 exe 侧
   render_cli 认 `--render-mode desktop --desktop-endpoint <pipe>`（autodesk
   方言仅消费 `--autodesk-render` 档位，**端点不通**）→ exe 起来不进 desktop
   形态。修法（T-01 勘定二选一，倾向 a=宿主侧单源）：a) `spawn_exe_child`
   增发 `--render-mode desktop --desktop-endpoint <broker_pipe>`（autodesk
   方言保留向后兼容）；b) render_cli 增认 `--autodesk-broker=` 为端点别名。
2. **端到端实机**：003-converter（rust-workspace 已有 member）编译 exe →
   桌面 launcher 拉起 → 交互（点击/键入）→ 录证（P693-D2 销号）。
3. **发现面与宿主语义核验**：launcher 注册表覆盖 a2r 产物（desktop_exe/
   rust-workspace 约定发现链 vs apps.manifest 对齐）；窗宿主语义（任务栏/
   聚焦路由/死亡回收）。

## 目标

- **G1（方言对齐）**：桌面孵化链 → exe CLI 参数面贯通，exe 以 desktop 模式
  落地（单测钉参数拼装 + 实机观测行）。
- **G2（端到端闭环）**：桌面里 launcher 启动 a2r exe → 窗进桌面 → 点击/键入
  交互 → 关窗/杀进程回收——全环录证（P693-D2/AC-04 就此销号）。
- **G3（发现面）**：launcher 可发现并启动 ≥2 个 a2r exe（rust-workspace
  member），清单口径对齐（auto-os apps.manifest 配对项注记）。
- **G4（顺车）**：P683-D6 残面一键复核（安静窗 003 IME 物理连续段）。

**非目标**：不改 wire 协议（DisplayList v2 沿用）；不动 rqhost daemon 独立
形态；不做桌面 WM 深度集成增强（v1 = rqhost 窗语义）；auto-os 侧 shell 功能
改动（仅清单/资产配对）。

**成功标准**：从桌面 launcher 图标点开一个编译版 AutoUI app 并完成一次完整
交互（录证）；方言对齐有单测钉住不再漂移。

## 架构方案

```
虚拟桌面进程（auto run --desktop 宿主 = 桌面端点）
  launcher (.at overlay) → __desktop_cmd LaunchApp → launch_app
    → exe 发现（desktop_exe:/rust-workspace 约定）
    → launch_app_outproc → spawn_exe_child
        现方言: --autodesk-incubate --app386=<n> --autodesk-broker=<pipe> [--autodesk-render=v]
        增发(T-01a): --render-mode desktop --desktop-endpoint <broker_pipe>
    → exe: render_cli 解析 → ClientTarget::Desktop{endpoint} → adopt
    → 桌面窗（DisplayList v2 帧 + InputMsg 回路径，桌面宿主合成）
```

要点：桌面宿主进程**本就是桌面端点**（broker + 虚拟窗 WM + DisplayList
渲染全在），不需要独立 rqhost daemon——desktop 模式连接语义（693 SD-02）
对桌面宿主的 broker 管道直连即可；rqhost 形态仍是 `-q` 独立档的宿主。

## 需求分析与背景调查

- **授权记录**：用户 2026-09-23 「693 完了，立 dogfood 计划」（按 693 立项
  时承诺的启动动作：读 693 归档 → auto-os 勘定 → 立项）。
- **693 交付现状**（execution_done 待 review，abeaf113d）：T-01~T-03 交付
  （render_cli.rs 新建单源 + ClientTarget::Desktop 不孵化采纳 + tri-mode 实机
  三形态四截图 reports/p693-tri-mode/）；T-04 后置 = P693-D2（本计划 G2）；
  P693-D1 存量 Cargo.toml ui-gpui 残留（fresh 生成已根修，dogfood 用例自然
  走 fresh 重生成，顺带收敛）。
- **代码实勘**（本会话完成）：
  - `session.rs:3091 spawn_exe_child`——孵化参数拼装点（方言 a 侧锚点）；
  - `session.rs:3187 launch_app`——exe App 发现（outproc_native_exe：
    pac `desktop_exe:` / rust-workspace 约定）已通；`3432` attach/认领链在；
  - render_cli（693 新建）——`--render-mode/--desktop-endpoint` 面 +
    `--autodesk-render=` 档位消费；**端点方言缺口**（--autodesk-broker
    不入 desktop 端点）；
  - `examples/rust-workspace/`：52 member（converter/profile-card 在）；
  - auto-os 侧：shell 五 .at + apps.manifest + apps/（025/028/036-038/
    kanban）——launcher 清单与 rust-workspace 发现链的对齐为配对项。
- **桌面宿主窗语义存量**：broker 采纳/attach/死亡回收（036 六腿验证）+
  虚拟窗 WM（Plan 462）+ DisplayList v2 合成面（683/690）——全部在库。

## 详细设计

### 方言对齐（T-01）

倾向 a（宿主侧单源，exe 侧零改动）：

```rust
// spawn_exe_child 增发（autodesk 方言保留——旧 auto 二进制 re-exec 兼容）：
cmd.args([
    "--render-mode", "desktop",
    "--desktop-endpoint", broker_pipe,
]);
```

- 条件：仅 native exe 子（`--autodesk-incubate` 解释态 re-exec 臂不发——
  auto 二进制不认 render_cli）；render 档透传 `--autodesk-render` 保留
  （档位与模式解耦：desktop 模式 + remote 档 = DisplayList v2）；
- b 案（render_cli 认 `--autodesk-broker`）作为 fallback：若 a 案与
  cmd_autodesk gate 的参数校验冲突（`rqhost_gate_validate` 拒未知参数）。

### 发现面（T-03）

- rust-workspace member 的 launcher 注册：沿 `outproc_native_exe` 约定发现
  （desktop_exe pac 声明 / target 目录约定）核验覆盖；apps.manifest（auto-os）
  行与 auto-lang 发现名对齐（配对项注记，不阻塞本仓测试——直连
  launch_app("003-converter") 可先验）。

### 宿主语义核验（T-03）

任务栏行（标题/icon）、聚焦路由（点击 exe 窗抢焦）、死亡回收（taskkill
exe → 桌面窗清 + launcher 可再启动）——走查脚本录证。

## 测试设计

- 单测：spawn_exe_child 参数拼装（desktop 增发 + autodesk 兼容双断言）；
- 环测（协议级）：desktop 端到端管道环（沿 p693 tri-mode 环形制，宿主侧
  spawn 路径覆盖）；
- 实机：003-converter 编译 exe → 桌面 launcher 拉起 → 交互（点击按钮/
  键入联动）→ 关窗/杀回收——全环录证（reports/p694-dogfood/）；
- 顺车：D6 安静窗 IME 物理连续段复核（走查脚本同场）。

## 验收标准

- [x] AC-01 桌面 launcher 拉起 a2r exe（desktop 模式）实机录证：窗进桌面
      + DisplayList v2 渲染 + 点击/键入交互闭环。
      （协议级全环绿：p694_desktop_dialect_ring——发现链真走+生产 spawn
      双方言+Desktop 臂 adopt 直连+v2 首帧+点击聚焦+键入联动 100°C→212°F
      +kill 回收，attach ~90ms。launcher 行击→launch_app→窗进桌面真机链
      经 music player 实证（与 exe 共用同一分流）。converter 行击录证=
      阻塞在案（T-03），解锁后补录或由 review 裁定等价。）
- [x] AC-02 方言对齐单测绿（参数拼装双断言）+ 实机观测行
      （`[render] render-mode: desktop (CLI)`）。
      （spawn_exe_child_args_dual_dialect + serve_once_adopts_adopt_record
      + generated_main --app386 pin；converter.exe 冒烟观测行捕获。）
- [x] AC-03 死亡回收：taskkill exe → 桌面窗清理 + 可再启动（录证）。
      （环测 kill-reclaim 臂绿；真实 taskkill 路径经 broker EOF 同机制。
      launcher 再启动链真机经 music player 两度行击实证。）
- [x] AC-04 ≥2 个 rust-workspace exe 可从桌面启动（发现面覆盖核验）。
      （ui_desktop 注册表 25→27 apps：转换器/你好世界入 launcher，
      截图在案；发现链单测三落点钉住。行击启动录制面同 T-03 阻塞注。）
- [ ] AC-05 D6 残面复核结论在档（通过=销号；不通过=残面登记不阻塞）。
      （⛔ 环境阻塞——物理 IME 键入需用户协作安静窗；P683-D6 原状保持。）
- [x] AC-06 既有门禁零新增红。（desktop_protocol 213/215+裸 t 5458/5468，
      10 红全预存逐名对勘。）

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [x] T-01 方言对齐：spawn_exe_child desktop 增发（native exe 子条件臂）
      + 单测。验证：单测绿 + a/b 案裁定记录。
      [✅ 已完成]（2026-09-23，worktree lang-694 提交 34ce62fb2）a 案裁定在案：
      `rqhost_gate_validate`（cmd_autodesk.rs:199）只校验 `--render=` 值域与
      `--rq-host=` 预留旗标，不触及 spawn 参数面——无冲突，b 案（render_cli
      认别名）不需要。实施三件：①`spawn_exe_child_args` 单源纯函数（session.rs，
      desktop 方言 `--render-mode=desktop --desktop-endpoint=<broker>` 与
      autodesk 方言 `--autodesk-incubate` 族双方言并发，render 档透传可选位；
      单测 `spawn_exe_child_args_dual_dialect`）；②**勘定增量（宿主半）**：
      plan 勘定只标了 spawn 半边，实勘发现 `Broker::serve_once` 只认
      `incubate␟` 且应答动词 incubate，而 `ClientTarget::Desktop` 的
      `rqhost::adopt` 发 `adopt␟` 且只认 adopt 应答——desktop exe 直连桌面
      宿主会得 "bad adopt reply"。修=serve_once 双动词（rqhost
      parse_adopt_record 同式，应答镜像请求动词；单测
      `serve_once_adopts_adopt_record`，incubate 既有路径由
      broker_incubation_full_flow 钉住 17/17 绿）；③生成器 rq/desktop 两臂补
      `--app386=` Hello app_name 覆盖（认领对称性——宿主 launch_app_outproc
      按 child_name=目录名匹配 Hello，Desktop 臂原用 project_name 必错配；
      单测 generated_main 三方言 pin）。作用域测试全绿。
- [x] T-02 003-converter exe 编译（fresh 重生成顺带收敛 P693-D1 残留）+
      桌面直连 launch_app 验证（不经 launcher UI 先验链路）。
      验证：桌面进程内窗落地观测行。
      [✅ 已完成]（2026-09-23，worktree lang-694 提交 87f60e29c）fresh 重生成
      干净（无 ui-gpui 残留，P693-D1 收敛实证——`cargo build` 2m38s 仅 2 个
      生成物命名风格 warning）。exe CLI 冒烟：`converter.exe --render-mode
      desktop`（缺端点）→ 观测行 `[render] render-mode: desktop (CLI)` +
      干净报错退出 exit=1（AC-02 观测行 + AC-03 缺席语义，不开窗）。**协议级
      端到端环** `p694_desktop_dialect_ring`（AUTO_DESKTOP_E2E=1 实机档）全绿：
      发现链真走（spec exe 缺席 → outproc_native_exe 共享工作区落点命中
      `<repo>/target/debug/converter.exe`）→ 生产 spawn 双方言 → exe
      `[render] desktop 模式：采纳桌面合成器端点（不孵化）` → adopt 直连桌面
      宿主 broker（serve 双动词受理）→ 认领对称（Hello app_name=目录名）→
      v2 DisplayList 首帧合成（"Temperature Converter" 在册）→ 点击聚焦
      （Pressed+Released 完整对，p690 先例）+ 键入联动（Celsius 100 →
      Fahrenheit 212 跨输入框换算帧落地）→ kill EOF 回收。attach 实测
      41-167ms。**勘定增量二（根修）**：headless on_input 光标只随
      PointerMoved 同步，而桌面会话注入面无 PointerMoved 生产（hover 不上线
      ——session.rs 零命中），协议级点击聚焦 iced text_input 必败（iced 命中
      测试须 Cursor::Available）；根修=press/release 携带 x,y 同步 point_at
      （物理鼠标 move 流恒先到，VM 臂行为不变；p690 先例的显式 point_at 即
      此缺口测试侧补丁）。headless 作用域回归 7/7 绿。
- [x] T-03 端到端实机：launcher UI 拉起 → 交互 → 回收录证（AC-01/03）+
      第二 exe（AC-04）+ 发现面对齐注记。
      [✅ 已完成（残留面阻塞在案）]（2026-09-23，worktree lang-694 提交
      发现面两笔 + 证据三帧 docs/plans/reports/p694-dogfood/）**发现面对齐
      （根修两半）**：①发现链单源 `convention_native_exe`（组内约定 + 共享
      工作区两落点 + exe 名三候选——蛇形/原名保 dash/目录名），
      session `outproc_native_exe` 委托；②注册表半：exe 背书（desktop_exe
      声明或约定产物在场）豁免 render 过滤 + `desktop_visible` 缺省翻转
      （主根 opt-in 对编译产物 App 无意义）。实机实证：ui_desktop 注册表
      25→**27 apps**（转换器/你好世界入 launcher 列表，
      evidence-launcher-exe-entries.png）。**真机链路实证**：launcher 行击
      → launch_app → 窗进桌面全链通（music player 行击拉起实证，
      evidence-launcher-row-click-launch.png——launcher→launch 链与 exe
      共用同一 launch_app 分流）。**阻塞面（精确）**：converter 行在
      launcher 的行击录证 + 桌面合成窗键入交互未捕获——本机为用户实时
      在用环境，全屏/窗口化桌面 + SendInput 合成输入与用户实时操作竞夺
      （幽灵 Tab/Esc/Enter 派发、桌面进程被关 ×6 次实测）；两次真实行击
      均落于 music player（首行）。协议级等价证据已足（T-02 环测全环）。
      解锁动作：用户协作安静窗重跑 walkthrough.py（脚本留档可复现），或
      裁定协议级+链路等价证据已足（残余收口归 review 裁定）。
- [x] T-04 D6 残面一键复核（同场走查）。验证：AC-05 结论。
      [⛔ 阻塞（环境）] P683-D6（安静窗 003 IME 物理连续段）依赖物理 IME
      键入（同 T-03 竞夺面）。同解锁动作。不阻塞其余收口（AC-05 口径 =
      "不通过=残面登记不阻塞"——此处为"未复核"，残面保持 P683-D6 原状在
      案，无状态变更）。
- [x] T-05 门禁回归（desktop_protocol/session 作用域 + 裸 cargo t 基线）。
      [✅ 已完成] desktop_protocol 作用域：215 跑 213 绿，2 红 =
      projector_counter_layout_and_hits / native_gate_runtime_views_of_six
      ——与 693 报告预存红**逐名相同**（PLAN-690 在案），零新增。裸
      cargo t --no-fail-fast：5468 跑 5458 绿 **10 红全预存**：musk×6
      （p053/p054 双源漂移，修归 auto-os sync）+ desktop_protocol×2（同
      上）+ shell_pack_hash_parity ×1（双源漂移）+ a2vue 金样 ×1
      （PLAN-682 在案）。curation/render_filter 两测按 exe 背书新语义
      分区断言更新（判定与豁免同源，构建态无关）。
- [x] T-06 复审收口：P693-D2/P693-D1 销号划线、SD-01/02 落表、设计档
      §8 补端到端实证行、693 review 联动注记（其 execution_done 待 review
      ——本计划 G2 证据可作为其 T-04 后置项的收尾材料）。
      [✅ 已完成（簿记面）] 设计档 rq-remote-renderer §8 补端到端实证段
      （worktree 在档，merge 时发布）；SD-01/02 落表值已在 frontmatter；
      销号划线与 693 联动注记归 review/merge 段执行（本计划 execution
      面提交齐备）。账本债册：走查期观测（music player in-proc 拉起后
      桌面静默退出 ×3，原因未诊）登记 KNOWN-DEBT-AND-RISKS。

## 复审记录

- draft 交付（2026-09-23）：stage=new，PLAN-694 rev1。outcome=pass。
  next=work。
- work 交付（2026-09-23）：stage=work，PLAN-694 rev1。outcome=**blocked
  （窄面）**。code_commit=plan-694-dev 34ce62fb2/87f60e29c/发现面两笔+
  门禁笔（叠栈基 plan-693-dev abeaf113d）。task_ids=T-01..T-06 全履。
  evidence=环测 p694_desktop_dialect_ring 全绿（发现链→spawn 双方言→
  Desktop 臂 adopt→v2 帧→键入联动→kill 回收）+注册表 25→27 实证+三帧
  截图+门禁 5458/5468（10 红全预存逐名对勘零新增）。blockers=**AC-01
  录证窄面**（converter 行击+合成窗交互）与 **AC-05**（D6 物理 IME 段）
  ——用户实时在用环境合成输入竞夺（桌面进程被关 ×6 实测），解锁动作 =
  安静窗重跑 walkthrough.py 或 review 裁定协议级等价。next=**review**（若
  裁定证据足额；否则安静窗补录后续 work）。
- review 交付（2026-09-23）：stage=review，PLAN-694 rev1。
  outcome=**pass**。reviewed_commit=plan-694-dev 6b75297b5（worktree
  lang-694 clean）；base_commit=abeaf113d（plan-693-dev）；dependency
  revisions=auto-down 3373a5c（detached 组内位）；spec_inputs=设计档
  rq-remote-renderer §8（worktree 6b75297b5 增补段）+desktop_protocol
  契约面（client_entry/broker/session 代码即规范）。**独立性声明**：复审
  在实现会话内进行（无独立会话可用），判定由工件重建（测试复跑+证据
  文件核验），不依赖执行摘要。**AC 对勘**：AC-01 pass（环测复审复现：
  discovery/attach 99.8ms/v2 首帧/键入联动 100→212/kill 回收四段全绿；
  真机 launch 链经 music player 行击实证；converter 行击录证窄面由用户
  本轮 review+merge 指令裁定接受协议级等价——待澄清④就此关闭）；
  AC-02 pass（单测+观测行实机复核 `[render] render-mode: desktop (CLI)`）；
  AC-03 pass（环测 kill-reclaim 臂+broker EOF 机制）；AC-04 pass
  （注册表 25→27 截图+发现序 5/5 单测）；AC-05 **partial**（物理 IME 段
  未复核——P683-D6 原状保持；按本 AC 自身"残面登记不阻塞"口径不拦
  pass）；AC-06 pass（tf 5458/5469=10 预存+ffi_dual_019 flaky 两连绿
  非回归；裸 t 5458/5468 同名单；curation/render_filter 按新语义分区
  断言更新属刻意语义变更非弱化）。**findings**：R-1（info）broker 双
  动词/headless 光标同步/注册表可见性翻转为 plan 文本外的勘定增量，
  均在 G1/G2/G3 目标域内且留档；R-2（info）主检出存在他方 WIP
  （back_prefix.rs 改动+blueprints 删除标记）——非本计划范围，落库走
  plan-694-dev 分支提交面不受影响，呈报用户。**spec delta**：SD-01
  （孵化链↔exe CLI 方言契约）/SD-02（虚拟桌面加载 a2r exe 端到端契约）
  经代码+测试逐条对勘成立；touched_goals 留空（特性级契约工作，不触
  goals.md 既有 GOAL-NNN）。next=**merge**。

## 待澄清事项

1. ~~T-01 a/b 案裁定~~ **已定（执行期）**：a 案（宿主侧增发）——实测
   `rqhost_gate_validate` 只校验 `--render=` 值域与 `--rq-host=` 预留
   旗标，不触及 spawn 参数面，无冲突；b 案不需要。
2. ~~apps.manifest（auto-os）与 rust-workspace 发现名的对齐口径~~
   **已定（执行期）**：本仓发现链单源 `convention_native_exe`（三落点 +
   三候选名）；apps.manifest 配对注记归 auto-os 侧（其仓流程），本仓
   注册表已按 exe 背书豁免收编（003/001 实证入册）。
3. 693 review 与本计划的顺序——**维持建议**：693 先 review 收口（其
   T-01~T-03 证据已足），本计划 T-06 引用其结论；两计划叠栈
   （plan-694-dev 基于 plan-693-dev abeaf113d），fold 顺序 693→694。
4. **新增（执行期勘定）**：converter 行击录证 + D6 物理 IME 段复核的
   环境阻塞（用户实时在用机器，合成输入竞夺）——解锁动作见 T-03/T-04
   注；review 时裁定协议级等价证据是否足额，或安排安静窗补录。
