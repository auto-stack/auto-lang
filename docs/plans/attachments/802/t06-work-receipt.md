# PLAN-802 T-06 工作收据（送审冻结基线）

代码提交：`4029dd705` + R1 修复 `ad1427e33`（branch `plan-802-dev`，base
`1dd123361`，worktree `D:/autostack/.wt/lang-802/auto-lang`，git status
clean）。auto-down 兄弟 `c39ac7f`（detached，只读消费）。本收据为 work 阶段
证据汇总；merge-owned 门禁（裸 `cargo t`/`cargo tv`/触面档/tf 批量回归）按
2026-10-14 时序裁定**保持 pending**，由最终 review 通过后的 merge 阶段在冻结
提交上执行。

## R1 修复轮（F802-R1-01 闭合，2026-10-17）

V06 实机验证已补齐：探针应用 + 驱动（[v06-live/](v06-live/)，pac +
src/front/app.at + v06_driver.py，MCP 标准 urllib 客户端 + Win32 SendInput
真滚轮）在真实 iced 窗口（`auto run -r vm`，本机 Windows 会话）8/8 PASS
exit 0（[证据 JSON](v06-live/v06-evidence.json.txt)、[auto 日志](v06-live/v06-auto.log.txt)、
[末帧截图](v06-live/v06-final-window.png)）：

| 步骤 | 结果 |
|---|---|
| baseline-top | oy=0.0（701 顶部中性值） |
| programmatic-150 | oy=150.0 **命令值直达**（trace 无中间值）；2.25s 含探针采样量化（press+state 每轮 ~0.6s；701 ≤2s 契约对象为用户滚动收敛，非程序化） |
| clamp-to-max | 越界 999999 → oy=7541.0（= content 8020 − viewport 479.200 精确上界），复采稳定 |
| user-wheel-readback | **真滚轮**（SendInput，ALT 手法前台化+点击激活）5 格：oy 7541→7241（5×60px），读端点回填 **0.713s ≤ 2.0s**（701 收敛契约，含心跳） |
| unbound-counter-control | 未绑定键入队（note=unbound-fired）后真实编辑器 oy 不变 |
| scroll-state-reader-observe | ScrollStateReader pane editor-scroll-ed 可观测（viewport/content/offset 三元） |

**实机验证定位的预存缺口（已修，`ad1427e33`）**：`render_dynamic_view` 的
code_editor 臂（VM 轨主路径）自 PLAN-629 寄宿化以来未把 editor-scroll-{key}
绑进 scroll controller 注册表（29480 desktop 泛型臂 701 起已绑）——
`code_editor_scroll_to` 的 intent 在 VM 轨 pane 缺失被 drain 静默丢弃，写端点
在 VM 实机窗口恒 no-op。**非 802 迁移引入**（旧 shim 直 enqueue 同一注册表
同队列；802 provider 化不改变入队目标）。修复 = 镜像 29480 的一行
`bind_controller(scroll_id, scroll_id)`（handle=widget id 单源）。修复后
701 供②b 端点合同在 VM 轨闭环。回归：plan802 + code_editor_natives +
plan794 + plan728 21/21 PASS。

探针资产注记（复现要点）：Init 先于首帧渲染（编辑器注册表空，set_text
no-editor raise——缺席臂逐字面值在实机再次验证）；内容装载走 value prop
数据流（`.doc` 模型 → build 期 renderer 侧 set_text，注册表保证在位）；
驱动 stdout 必须落文件（PIPE 不读灌满后阻塞 auto 进程首帧）；窗口前台化
需 ALT 键手法（直 SetForegroundWindow 被前台锁静默拒绝）。

## 一、22 native ↔ 19 provider 方法映射（T-01 矩阵实施核对）

见 [t01-frozen-matrix.md](t01-frozen-matrix.md)。实施逐项落地：cursor 三读
共用 `cursor()` 一次 DTO、scroll 双读共用 `scroll_offset()` 一次 DTO——与
迁移前 `code_editor_cursor`/`controller_snapshot` 单调用形态同构。负 offset
（9906）/1基→0基（2932）/负参钳 0（9910）均留 shim 侧（i32→usize 边界），
provider 收标准 usize。catalog/codegen/intrinsics 零改动（ID/canonical 名/
短名全部不变——fixture V02 ID 钉扎断言 22 项全对）。

## 二、验证记录（work 阶段定向面；命令与选择数）

| 命令 | 结果 |
|---|---|
| `cargo check -p auto-lang`（default=ui-iced） | PASS（411 告警=预存基线；本改动文件零新增告警） |
| `cargo check -p auto-lang --no-default-features` | PASS（no-ui 编译前提） |
| `cargo test --manifest-path test/fixtures/modularity/code-editor-provider-consumer/Cargo.toml` | **7/7 PASS**（V01 真VM全管线×2 / V02 缺席矩阵+身份+if_absent×3 / V03 delta FIFO+ownership×2） |
| `cargo t plan802` | **7/7 PASS**（core 剪断 / 装配身份 / 真注册表往返 / 文件 IO 字节保真 / provider swap 寿命 / 滚动投影链 / OS 剪贴板往返） |
| `cargo t code_editor_natives` | 3/3 PASS（既有 e2e 经新 provider 路径不变） |
| `cargo t plan701 plan670 plan794 plan785` | 25/25 PASS（含 785/794 源码扫描守卫） |
| `cargo t plan728` | 5/5 PASS（分页 editor load/edit/save 真注册表） |
| 合并终跑 `cargo t plan802 code_editor_natives plan794 plan785 plan728` | 27/27 PASS |

## 三、V06 证据（R1 修复轮后全量闭合）

~~条件缺口~~ **已闭合**：live iced 窗口消费链已实机验证（见顶部 R1 修复轮
表格——程序化命令值直达/clamp 精确上界/真滚轮 ≤2s 收敛/未绑定反控/reader
观测，8/8 PASS）。保留的组件级证据（真 controller 注册表投影链 + 真 arboard
剪贴板往返）不变，作为 headless 补充面。

## 四、V07 边界/剩余边/净删

- **777 口径**（text_scan.core_to_ui 同款 grep）：vm/native.rs `crate::ui`
  引用 **52→23**；vm/ 全目录 55→26。
- **本族生产边清零**：22 shim 零 `crate::ui` 直引（plan802_core_cut 五形态
  精确扫描守卫永久锚定）。
- **剩余边清单**（非目标，如实登记）：diff 族 9915–9917 四处（`code_editor::
  diff::envelope`，PLAN-703 面）+ native_dnd 一处（PLAN terminal/dnd 族）+
  测试模块引用（code_editor_natures e2e 布景 `ce::` 六处、794 scroll 测试
  九处——测试面非生产边）+ 注释两处。term/media/dialog/ffi 族未迁（与
  SD-02"其他族仍未迁"一致）。
- **a2r/merged 直接 UI 臂未误删**：ui_gen/rust.rs `auto_lang::ui::code_editor`
  引用 25 处原样；`git diff` 确认 ui_gen/** 零改动。
- **792 触面（闭包不变 metadata 证明）**：根 Cargo.toml 仅 exclude 登记
  （无 members/deps 变化）；crates/auto-lang/Cargo.toml 零改动；无新外部
  依赖（代码仅 std）；ui_gen/**（bundle 生成器）零改动→同输入生成物字节
  不变；792 probe（uiminimal）不消费 code_editor natives；default 与
  no-default 两特征组合编译均过。结论：792 交付闭包成员/manifest/生成器
  未变，按编排合同"闭包不变须 metadata 证明"路径处理，fresh/locked/重定位
  driver 重跑归批量回归/merge 档。

## 五、施工期发现（非本计划范围，已登记 KNOWN-DEBT）

**VM 单作用域局部变量索引 ≥16 加载失败**（预存限制）：第 17 个及以后的
`let` 局部变量经 LOAD_LOCAL 读回为垃圾值（纯 int、零 native 可复现：
17 局部 `print(v16)`→`0`）。V01 fixture 写族程序因此拆两段（每段 ≤13
局部）。与 802 迁移无关（worktree 基线复现、不涉 provider/shim 路径）。
KNOWN-DEBT 编号 P802-D1。

## 六、SD 增量素材（提案，merge 阶段发布）

按计划 §5 表：SD-01（vm/overview：editor 22 项 owned provider、缺席矩阵/
装配/寿命）、SD-02（ui/overview：adapter 单一 registry 与宿主装配，其他族
仍未迁）、SD-03（code-editor-session-endpoints：provider 后 VM/a2r 同源，
单位/IO/收敛不变）。本收据二/三/四节即 SD 证据素材。
