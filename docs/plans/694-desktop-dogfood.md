---
plan_id: PLAN-694
status: executing               # drafting → executing → execution_done → reviewed → archived
feature_name: desktop-dogfood
author: [zcode]
created_at: 2026-09-23
updated_at: 2026-09-23

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [SD-01 桌面孵化链 ↔ exe CLI 方言对齐契约（spawn 参数 = render_cli 消费面）, SD-02 虚拟桌面加载 a2r exe 的端到端契约（发现/拉起/宿主/回收）]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [docs/design/autoui/rq-remote-renderer.md, docs/design/autoui/desktop-protocol-v1.md]
current_step: 1
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

- [ ] AC-01 桌面 launcher 拉起 a2r exe（desktop 模式）实机录证：窗进桌面
      + DisplayList v2 渲染 + 点击/键入交互闭环。
- [ ] AC-02 方言对齐单测绿（参数拼装双断言）+ 实机观测行
      （`[render] render-mode: desktop (CLI)`）。
- [ ] AC-03 死亡回收：taskkill exe → 桌面窗清理 + 可再启动（录证）。
- [ ] AC-04 ≥2 个 rust-workspace exe 可从桌面启动（发现面覆盖核验）。
- [ ] AC-05 D6 残面复核结论在档（通过=销号；不通过=残面登记不阻塞）。
- [ ] AC-06 既有门禁零新增红。

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
- [ ] T-03 端到端实机：launcher UI 拉起 → 交互 → 回收录证（AC-01/03）+
      第二 exe（AC-04）+ 发现面对齐注记。
- [ ] T-04 D6 残面一键复核（同场走查）。验证：AC-05 结论。
- [ ] T-05 门禁回归（desktop_protocol/session 作用域 + 裸 cargo t 基线）。
- [ ] T-06 复审收口：P693-D2/P693-D1 销号划线、SD-01/02 落表、设计档
      §8 补端到端实证行、693 review 联动注记（其 execution_done 待 review
      ——本计划 G2 证据可作为其 T-04 后置项的收尾材料）。

## 复审记录

- draft 交付（2026-09-23）：stage=new，PLAN-694 rev1。outcome=pass。
  next=work。

## 待澄清事项

1. T-01 a/b 案裁定（倾向 a=宿主侧增发）——若 cmd_autodesk gate 参数校验
   冲突则 b（render_cli 认 --autodesk-broker 别名）。执行期定，无阻塞。
2. apps.manifest（auto-os）与 rust-workspace 发现名的对齐口径——T-03 配对
   项注记（auto-os 侧改动走其仓流程，本仓不越界）。
3. 693 review 与本计划的顺序（693 execution_done 待 review）——建议 693
   先 review 收口（其 T-01~T-03 证据已足），本计划 T-06 引用其结论。
