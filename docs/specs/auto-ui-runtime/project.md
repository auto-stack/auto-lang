# auto-ui-runtime

> **Status**: active（PLAN-814 B/T-04 交付；A 期 @ f6e07acee 为骨架）
> 路径：`crates/auto-ui-runtime` | 技术栈：Rust
>
> PLAN-814 T-04：UI **运行/renderer/宿主域**独立成包——原 auto-lang
> `src/ui/` 全树（iced renderer、desktop_protocol、code_editor、
> terminal、mpv、native_dock/scroll/dynamic/session/shell/mcp_server、
> interpreter、headless 等 ~20.4 万行）+ host_adapters + back_proxy +
> `assets/*.at` 五件 + UI 动态构建管线（原 lib.rs Plan 015 glue 区=
> `dynamic_build.rs`）单源迁入；旧路径经上层再导出全保持。

## 职责与边界

- 依赖：auto-lang-core（SDK）+ auto-ui-compiler（UI 编译域）+
  auto-val/auto-text-core/auto-terminal-core/auto-media-core/a2r-std
  轻核；**不反依赖**旧 auto-lang 兼容包，不被 SDK/compiler 反依赖
  （DAG：Compat→{SDK,UIC,UIR}、UIC→SDK、UIR→{SDK,UIC}；777 域=ui）。
- 旧 auto-lang 对本包的依赖边 **default-features=false**（T-08 捕获
  语义）：no-UI 消费者经该边不得拖入 ui-iced；本包自身 default=
  ["ui-iced"]（独立消费=旧默认重后端语义保持）。
- 777 `ui_to_language` 文本扫描以本包+compiler 为源根，基线 **0**=
  零命中不变量围栏（禁 `auto_lang::` 引用；SDK 引用形态
  `auto_lang_core::` 不在扫描内）。
- **并集归属注记（PLAN-814 T-11 merge 组合，2026-10-21）**：
  `app_registry`/`i18n_lookup` 两模块因 PLAN-815 T-04 先落
  auto-ui-compiler（其 auto-ui-tools 消费 `auto_ui_compiler::
  app_registry`），且 DAG 禁 compiler→runtime，**单源定于
  auto-ui-compiler**；本包 `ui/mod.rs` 经 `pub use
  auto_ui_compiler::{app_registry,i18n_lookup}` 再导出——
  `auto_ui_runtime::ui::app_registry` 与旧 `auto_lang::ui::
  app_registry` 路径全保持，零双源。

## 公开能力与 feature 面

- `ui` 门下 UI 核心抽象（Component/View/VTree 族）；`ui-interpreter`
  （event_router/hot_reload/interpreter）；`ui-iced`（iced 后端+
  renderer）；`ui-headless`；`image-pipeline`；`code-editor`（双
  owner：SDK a2r diff 转发+本包 editor registry/provider）；
  `highlight-treesitter`；`ui-clipboard`/`ui-dialog`/`native-dock`/
  `native-clipboard`/`native-dnd`；`iced-layout-tests`；
  `mpv-native/gpu/widget/spike`；`autodown`（跨仓 autodown-core，
  optional）；`test-native-dock`。旧 auto-lang 同名 feature 全部
  转发本包（组合面逐项保持）。**R9-F03 声明面组合全绿**：
  no-default/bare-`ui`/`ui-headless`/`ui-dialog`/`code-editor`/
  default 六形态 0 错（iced 专属面经 cfg 隔离——style_iced_adapter/
  anchor_slot/sched_diag::trace/vm_bridge/i18n 钩子随实门；
  PAYLOAD_SEP/encode_payload/ScrollStateEntry 纯项上移 mcp_types
  单源+iced 侧再导出；log 依赖显式钉 std）。
- 动态构建管线公开面（crate 根）：`build_dynamic_component`/
  `has_ui_keywords`/`run_file_dynamic_ui{,_with_scenario}`/
  `ui_module_parse_failures`（cfg 门=原 auto-lang 根项形态）。
- 外部消费门：`test/fixtures/modularity/ui-runtime-consumer/`
  （canonical 双直依赖=生成工程镜像形态，5/5+fresh+locked，闭包零
  旧包）。

## 装配语义（单点+r3 登记权）

- `install_ui_host_capabilities()`（pub，any(ui,ui-dialog) 门）=
  provider 全族装配单点：frame/clipboard×2/scroll/terminal/
  code-editor/dnd/dialog + autodown 引擎 + i18n lookup 钩子。
  no-op 边界精确为 `not(any(ui, ui-dialog))`——bare ui/headless/
  dialog-only 形态仍装配其 feature 内能力（R10 审定口径澄清，
  非无宿主化）。
- **不登记 UI 文法**：auto-ui-dialect 依赖与 `UiGrammarSource`/
  `UiCompatInstaller` submit 权属旧 auto-lang 兼容包与
  auto-ui-compiler（r3 §5.3）；本包零 inventory submit——纯 SDK
  缺席/显式拒绝语义不受本包链接影响（T-08 缺席臂回归即违反本条）。

## 单源状态与生命周期

- 进程级：native_widget GLOBAL 工厂、native_widget registry、mpv
  RUNTIMES（UI 线程亲和）、mem_guard、UI console——随迁单源，不复制。
- thread_local：theme 活动主题真源（style::theme）、`STORE_EXTRA_
  FILES`/`USE_DIAGS` 带外通道（SDK 单源，本包经 SDK 面消费）。
- FontSystem 多处 OnceLock 为预存局部双源（继承现状，不新增——
  PLAN-814 合同"不改现状不新增"）。
- 浮层几何提取（aura_view_builder）：直接消费 compiler StyleClass
  IR 的纯扫描（headless 同轨），**position 族后继覆盖仲裁**与
  IcedStyle::from_style 同语义（`absolute relative` 终态=relative
  不 hoist；R10-F01 修复，overlay 探针五组合 mismatches=0）。
- 退出/寿命：terminal/media/editor 生命周期、provider 缺席=冻结降级
  臂、store drain、use 诊断跨包单源（V-04 族 91/91 回归面）。

## 资产与测试归属

- `assets/{shell,desktop,switcher,notification_center,dashboard}.at`
  单源本包（include_str 相对深度保持；compiler 测试经
  `../auto-ui-runtime` 单源读）；Inter 字体/壁纸随 iced 资产。
- 语料 `test/ui/*` 单源留 crates/auto-lang/test/ui（多消费者），
  `test/parity/native` 同形；61+6 个 UI 域测试在 `src/ui_tests/`
  （挂载门=原 tests.rs 逐项）；plan446 批一在 dynamic_build。
- 测试选择面：t/tf/tt/tb/taa/ta/t3 七档含 `-p auto-ui-runtime`
  （九档逐 ID 核销见 t07-closeout）。

## canonical import 与生成工程合同

- Rust 生成物 canonical import：UI 面 `auto_ui_runtime::ui::*`、
  语言/VM 面 `auto_lang_core::*`（策略单源=compiler
  `ui_gen::import_policy`，新旧映射表在档；负控 pin：生成物零
  `auto_lang::`）。
- 生成工程 manifest 双直依赖：`auto-ui-runtime`（含 ui-iced）+
  `auto-lang-core`；e2e 真编译门=compiler 两 harness（2/2）。

## 发行与兼容

- 交付闭包 DELIVERY_CRATES 含本包；792 投影
  `PROJECTION_AUTO_UI_RUNTIME`（跨仓 autodown 边剥除 owner 随迁）。
- 旧兼容：`auto_lang::ui::*`/`auto_lang::back_proxy`/
  `auto_lang::host_adapters` 全路径再导出（V-02 pin 锁面）；旧
  feature 组合经转发保持；**旧样式路径** `ui::style::IcedStyle`/
  `ui::style::iced_adapter::*`（R9-F02 恢复——style 门面模块内
  compiler DSL glob+iced 适配器旧路径，同一类型单源=
  style_iced_adapter，reviewer pin 编译过）。

## M815-B:宿主退出请求与会话生命周期公开面(PLAN-815 SD-07)

新增 `request_host_exit`/`host_exit_requested`(代计数器)/
`host_exit_window_begin|end`(幂等窗口)公开面——CLI 直跑形态零行为变化
(无人置位);会话只响应窗口内到达的新代请求(初始化期请求不丢、跨会话
陈旧请求不误杀,run_session 启动清零)。实际可重入边界=每 VM 会话一个
隔离子进程(`--internal-vm-exec`,iced/winit 每进程单事件循环约束的
进程级解法;宿主经 RunIo 每请求精确 env 传会话上下文,行 `q` 经
request_host_exit 干净收口)。托管形态取消/EOF 终止整 owned 树并
有界收尸。（PLAN-815 M815-B，T-09 R2 pass @ 生产冻结位 0f2858e35；终稿正文与证据：attachments/815/handoff.md §规范增量）

## PLAN-840：同步测试的完成态

无 iced 帧循环的同步载体，在读取 Init 派生几何或交互终态前，须通过真实
DynamicComponent 泵续跑 Init/CPU/I/O，并重建视图登记下层 demand，直到工作
终态；单次 drive_scheduler_to_quiescence 不等于完成。共享 test-only 驱动逐轮
检查有界截止时间，pending 超时必须失败；所有原几何/交互断言保持。多片 Init
早退及真实几何完成的对照与 pending 截止时间反控永久保留。

该约束只修正测试载体，未修改生产 dynamic.rs 或 VM 调度。D-762-1 的生产
helper 债及其单独授权约束保持，不冒称本计划清偿了生产 helper。

RQHost 末窗测试中的待定采纳必须来自真实服务端发布：客户端 adopt 返回
不代表其 peer 已入 pending，须按有界条件等待并持有实际客户端端点。
不得手工把客户端端点再塞入服务端 pending 冒充采纳；否则真实 peer 晚到可
污染清空后的关闭断言。原双窗/待定/末窗/从未开窗的停机断言保持，生产
rq_update 的退出逻辑不变。

## PLAN-831 仓属与来源（SD-05，as-built）

本 crate 物理属 auto-ui 仓（迁移基线=语言仓 e199caf8a 内容）。字体/.at 资产与测试随 owner 迁移；同步测试完成态约束（PLAN-840 节）继续适用（840 修复已随 T-11 并集移植入本仓副本）。VM/Iced 运行面：run_ui 真实运行验收=子应用服务级观测（Launched(url) 端点探活 + cancel 后同端点确证关闭，连接拒绝唯一计关闭；超时/未知探测不可判定期不充当任一结论）。
