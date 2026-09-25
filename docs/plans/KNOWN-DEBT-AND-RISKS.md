### P683（2026-09-22，Plan 683 rq-remote-renderer 方案 2 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P683-D1 | low | remote wire v2 | **mesh/canvas 原语 v2 词汇面外**（G2 词汇表未列；tiny_skia `layer.primitives` 位图降级承接未实施）——试点三例无 canvas 面，首个 canvas 应用上 remote 时需 Path 序列化或位图降级（T-01 勘定登记） | headless.rs lower_layers_v2 观测臂；设计档 rq-remote-renderer §4 |
| P683-D2 | low | remote 降格 | **Transform 旋转/剪切矩阵元不可达**——iced `Transformation` 未公开矩阵元（仅 scale_factor/translation），v2 降格只产轴对齐 [s,0,0,s,tx,ty]；旋转内容（canvas 旋转动画等）remote 面暂缺 | headless.rs lower_text_v2 近似臂观测 |
| P683-D3 | low | remote 图像 | **svg 图像（tiny_skia Image::Vector）降格省略**——v1/v2 词汇 src 均为位图通道；svg 组件经 lucide 栅格化外的直引形态需 src 词汇扩展或宿主侧 svg 栅格化 | headless.rs lower_image_v2 Vector 臂观测 |
| P683-D4 | low | daemon 重放 | **shadow blur 无 canvas 原语**——v2 wire 携带 ShadowSpec 全参，daemon 重放为偏移半透明垫层（无 blur 扩散；alpha 按有无 blur 稀释）——观感近似，004 卡面 shadow-sm 实测可接受 | broker_surface.rs paint_shadow_scrim |
| P683-D5 | low | remote 环境注记 | **已销号（PLAN-690 T-07 自愈臂落地）**——daemon `WindowResized` 0x0 守卫：零尺寸（最小化路径）不向 app 转发、基线尺寸不动（恢复期假空白根除），真实尺寸恢复 resize 自然同步；管道环 p690 测试③断言绿。原走查恢复臂诉求由守卫替代 | PLAN-690 commit（rqhost.rs 守卫 + p690 测试） |
| P683-D6 | low | remote IME | **大部销号（PLAN-690 G1）**——中文 IME 组合/候选定位实机已验：组合串+候选窗在 daemon 窗内按 App 焦点框定位可见（录证 `docs/plans/reports/p690-ime-walkthrough/`），enable 下行/定位/上屏通道全链在档。**残面**：物理 IME「组合→上屏」连续实机段（commit 串经子类桥入值）在多会话争焦窗下未录成（前台锁拒合成注入；FG-FAIL 留痕），桥→上行→入值段由管道环 p690 测试覆盖（同路径构造性等价），留一键复核（安静窗内 003 remote 走查脚本即验）。**694 T-04 对账注记（PLAN-697 AC-04）**：一键复核在 694 走查期再次因环境竞夺未录成（⛔ 阻塞在案，非状态翻转）；694 复审裁定管道环=协议级等价，残面降级为可选复核——行状态维持"大部销号+残面在案"不变 | PLAN-690 win_ime.rs 子类桥 + 走查录证；plan 690 待澄清②；694 T-04/复审对账（PLAN-697 刷新） |
### P684（2026-09-22，Plan 684 gallery 示例修复批 work 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P684-D1 | low | 027 back API | **vue 轨写操作无后端端点**——rev7 只做了读面（fs_list/fs_home/fs_drives）；重命名/粘贴/新建/删除在 vue 轨统一守卫提示「写操作仅在 VM 轨可用」。后端加写端点（fs_rename/fs_mkdir/fs_delete/fs_copy 契约+impl）即可让 gallery 内嵌全功能，形态沿用 thin 契约纪律 | examples/ui/027-file-manager/src/back/api.at；src/front/app.at 写守卫四处 |
| P684-D2 | low | 027 媒体面 | **vue 轨图片缩略图缺失**——image.thumb 为 VM 轨媒体管线专属（256px rendition 渐进浮现），vue 轨条目 thumb_src 恒 ""，网格视图回落 FileIcon。补齐需后端缩略图端点或 media 路由跨轨化 | examples/ui/027-file-manager/src/front/app.at NavTo vue 分支 thumb_src:"" |
### P693（2026-09-23，Plan 693 native-exe-tri-mode work 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P693-D1 | low | a2r 生成物存量 | **rust-workspace 既有 member 的生成 Cargo.toml 仍含 `ui-gpui = ["auto-lang/ui-gpui"]` 行**（PLAN-691 移除 feature 前生成）——存量 member 重复 resolve 即败（fresh 生成已被本计划生成器根修）；各 member 下次 `auto build -r rust` 重生成自然收敛，禁手工清（生成物 never-track） | rust_ui.rs generate_cargo_toml（已修）；主检出 examples/rust-workspace/011-calculator 等存量 |
| ~~P693-D2~~ | medium | desktop 接入跨仓配对 | **已销号（PLAN-698 T-04，2026-09-23）**——形态勘定=子进程（库形态被 winit Windows 单事件循环结构性否决，RecreationAttempt 实锤）；`spawn_desktop_daemon` 孵化 `auto rqhost --pipe autodesk-rqhost-desktop-<pid>` + AUTO_DESKTOP_ENDPOINT env 注册表发布（命名裁定定稿）；003 converter.exe 全环录证（adopt→开窗→v2 帧→EOF 回收→末窗自退，reports/p698-batch/T04-desktop-pairing.md）。SD-04 落 ui/overview.md；WM 深度集成仍归 auto-os 侧后续 | renderer.rs run_session Desktop 臂；rqhost.rs spawn_desktop_daemon；desktop.ps1 |
| P693-D3 | low | a2r 测试漂移 | **`merged_api_client_crud_fallback_for_uncovered_endpoints` 预存红勘定新增**（不在既有已知红清单）——测试期望旧 CRUD 兜底形（`static API_DATA` + `-> Vec<Value>`），实发为 PLAN-681 route A `api_impl` 伴随模块形；base rust_ui.rs 外科对照同败（与本计划 diff 零交集）。修 = 测试期望随 route A 形更新，归 PLAN-681 线或独立 L0 | rust_ui.rs:4813 测试 vs generate_merged_api_client route A 臂 |

### PLAN-698（2026-09-23，Plan 698 desktop-substrate-hardening-batch work 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P698-D1 | low | submenu 浮动式残余 | **像素级确认被 D5 screenshot 通道阻塞**——任意 menubar 开态 autoui_screenshot 10s 超时（基线零改动同现=通道级预存，与浮动/内联形态无关，判位留痕 T03 工件）；浮动面板截图验证待 D5 清偿后补一轮。交互细节：子面板开态点击=项动作+菜单关闭并存（shadcn 语义可接受但不精细）；hover 换子菜单归 P695-D1 键盘/hover 导航批 | popover.rs Panel::overlay；reports/p698-batch/T03-submenu-floating.md |
| P698-D2 | low | desktop 端点生命周期/接线 | **daemon 子进程与宿主无级联**（Windows Job 对象挂接缺——宿主强杀后 daemon 按末窗语义存活，零窗常驻）；注册表 launch 面尚未消费 AUTO_DESKTOP_ENDPOINT（env 发布就绪，launcher 自动带 `--desktop-endpoint` 拉起 a2r exe 归 auto-os shell 线）；虚拟窗内嵌/键面深集成归 P694-D2 | renderer.rs run_session Desktop 臂；desktop.ps1 |

## PLAN-692 债务候选（2026-09-23，W-2/W-1 走查批）

1. ~~**ui-cache 键缺生成器版本（infra，高优先）**~~：**已销号（PLAN-698 T-01，2026-09-23）**——生成器指纹进缓存键（build.rs hash src/ui_gen/** → AUTO_UI_GEN_FINGERPRINT；UICache.generator_fingerprint 不匹配整缓存失效重签，VERSION 1→2 迁移弃旧）；692 W-1 场景实机回归绿（改生成器→自动失效重生成，免手工清）。入库评估=生成数据注记落 SD-01（runtime/overview.md），入库策略不阻塞。（锚点：ui_cache.rs；build.rs）
2. **popover 选中后不自动关闭**：Combobox 选中候选项后 popover 保持打开（shadcn 官方为选中即关）。需 popover open 状态绑定 + CommandItem @select 联动关闭。
3. **VM(iced) 臂 command 家族未实现**：八 command 元素 `iced: none` 维持——VM 臂渲染降级。接线（含 `size` prop 的 iced scrollable 宽度）另立。
4. **reka sizes 初始测量 18px 下限（观察项）**：本仓环境下 ScrollArea thumb 长度常命中 reka getThumbSize 的 18px 下限（A/B 证实与实现无关；拖拽数学内部自洽，仅视觉比例偏小）。
5. **脚手架 h1-h6 基础 margin 内联隐患**：基础样式给 h1-h6 的 mt/mb 对"内联进 flex 行"的标题同类泄漏（PLAN-692 W-3 实证一处）；其余位置待观察，出现即补 m-0。

### P694（2026-09-23，Plan 694 desktop-dogfood work 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| ~~P694-D1~~ | medium | 桌面宿主稳定性 | **已销号（PLAN-697，2026-09-23 外因结案）**——勘定归因档案 `docs/plans/reports/p697-stability/attribution.md`：机器级外因（内核 bugcheck 三日三蓝屏 0x9F/0x1A/0xA + GPU TDR 族；走查窗口 System Event 6008 = 09-23 10:59:12 非正常关机实锤），蓝屏瞬时吞进程 = 无 panic 无错误行无 WER 应用崩溃，与观测签名逐项吻合。嫌疑面 back-proxy lazy-start/media 供给/inproc panic 边界经 13 轮复现矩阵（基线窗口化×5+全屏×2+禁A×2+禁B×2+他App×2 全绿零死亡，嫌疑面点火实证）+ WER 零宿主崩溃记录三重排除。观测加固落地：ui_desktop 宿主 `[ui-desktop] start/alive/exit` 足迹桩 + panic 钩子（死亡判位表在档案；CLI 路径 X9 形制已在）；对照缝 AUTO_NO_BACK_PROXY/AUTO_NO_NATIVE_MEDIA（勘定器械，生产零影响）。残面=机器稳定性本身（GPU 驱动/内核，非本仓代码面） | 归因档案 p697-stability/；examples/ui_desktop.rs 足迹桩；back_provision.rs 对照缝 |
| P694-D2 | low | 桌面 shell 输入面（VM 轨） | **launcher 全量列表导航在 VM 轨不可用**——搜索框键入（物理字符派发为 `key_<char>` VM 事件，input widget 不收 CharTyped）、滚轮滚动、方向键（Named 键不路由至 bind 表）三路实测均不通（028-launcher overlay）；行击直点正常。本计划以 recent 存储夹具绕行完成发现面录证；shell 侧输入路由修复归 auto-os/shell 线（本计划非目标） | 028-launcher overlay；session.rs 键盘路由（key_<char> 派发臂）；walkthrough 走查留痕 docs/plans/reports/p694-dogfood/ |

### P695（2026-09-23，Plan 695 menubar-widgets-batch work 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P695-D1 | low | menubar 键盘面 | **方向键/Enter 键盘导航未实现**（T-08 时间盒裁定）——需 iced 焦点基建（逐项 focus 节点 + 键盘订阅 + 索引态进 MENUBAR_OPEN 注册表扩展），Esc 关闭已走 Popover 既有捕获（AC-04 最小集满足）。实施时对齐 shadcn：↑↓ 移项 / → 进 submenu / ← 回父级 / Enter 激活 | convert_menubar_component；menubar-snapshot.md PLAN-695 扩注 |
| ~~P695-D2~~ | medium | VM submenu 浮动式 | **已销号（PLAN-698 T-03，2026-09-23 翻案 T-11）**——真因勘定=iced 0.14 官方嵌套协议（overlay::Nested 递归 Overlay::overlay 钩子）未实现，嵌套 Popover overlay 永不注册；Panel::overlay 收集钩子落地（绝对坐标零平移）+ 视图臂浮动恢复（RightTop），内联降为 AUTO_MENU_SUBMENU_INLINE=1 降级路径。契约单测+快照+全程零死亡佐证；残余（像素截图确认被 D5 通道阻塞+交互细节）转 P698-D1 | popover.rs Panel::overlay；aura_view_builder 浮动臂；reports/p698-batch/T03-submenu-floating.md |
| P695-D3 | low | autoui-verifier × menubar | **MCP 合成指针事件 × hover-switch 干扰**——合成 press 的指针位移穿过被包裹 trigger 触发非预期 toggle，菜单开合状态在 MCP 驱动下呈非确定（真实鼠标语义不受影响）；autoui-verifier 对 menubar 的状态断言须容忍切换抖动或改坐标无关派发（menubar-snapshot.md 扩注已记） | aura_view_builder MouseArea 包裹臂；T-11 走查实录 |
| P695-D4 | low | gallery golden | **已销号（fold 门内完成，25cb53fa7）**——基线重基线落定（理由随提交：auto-os master 演进 + menubar.at 语料定稿），复跑对账绿（1108s exit 0） | crates/auto-lang/tests/fixtures/gallery_vue_golden.txt |
| P695-D5 | low | auto-edit 截图通道 | **auto-edit VM 臂截图与 1s 状态栏 Tick 争用（预存特征）**——无菜单态 screenshot 亦恒超时，AC-06 对照截图不可得；可读性由 token 断言链（bg-popover light=白/popover-foreground=墨）+ 快照结构验证 + probe_menu.py 承载。修复归 041/render 时序线 | tests/probe_menu.py（exit 0 两轮）；T-13 走查实录 |
| P695-D6 | low | gallery Vue 臂活体 | **widgets-gallery Vue 臂活体走查环境受阻**——worktree 无 front workspace/node_modules，`auto run`（vue）宿主窗不自动起 vite（3024 恒 000）。unblock：宿主窗交互启动 web 或主检出跑 692 同款全量再生成流程。SFC 发射面已由 p695 vue 单测 + gallery_properties schema 符合门 + fold 前 gallery_golden 确定性覆盖 | T-11 执行记录；.auto/ui-cache.json（合并后须再生） |

### P696（2026-09-23，Plan 696 HTTP runtime hardening 复审遗漏/延期扫描）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| ~~P696-D1~~ | medium | VM pubsub / publisher SSE | **已销号（PLAN-698 T-02，2026-09-23）**——bus.subscribe 转真运行面（案①进程内 iterator 注册表：EVENT_BUS broadcast::channel(256) 镜像生成 events.rs + 转发线程 + AsyncHttpStream 臂，696 回收语义沿用）；POST publisher 臂 publish_post_broadcast（Typing/New{Type} 对齐 api_gen，has_sse 门控经 record_api_return_type 新侧信道）；017 VM 臂全环录证（SSE 帧与 Axum 形态逐字节一致 + 带外 POST typing UI 渲染 typing 指示）。§8.1 落表+seam 注记划线 | stdlib.rs shim_bus_subscribe/EVENT_BUS；http_server.rs publish_post_broadcast；reports/p698-batch/T02-vm-bus-subscribe.md |
| P696-D2 | low | VM HTTP 生命周期 | **VM `#[api]` listener 尚无 graceful shutdown API**——本文 §7.3 只记录目标行为；本计划落实了 body/SSE producer 与断连连接任务的回收，不包含 Ctrl+C/SIGTERM 停止 accept、排空活跃连接的服务级接口。后续实现时需定义关闭信号、listener 停止和活跃连接排空契约。 | `docs/specs/stdlib/design/http-server.md` §7.3/§8.1；`crates/auto-lang/src/vm/ffi/http_server.rs` |

### P701（2026-09-25，Plan 701 供料包复审遗漏/延期扫描）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P701-D1 | medium | VM i64 算术面 | **.at 层 TAG_I64 操作数的算术解码错位**——`time.now_ms()/1000` 实测归 0（DIV 臂按 i32 解码 TAG_I64 nv；SUB 等同疑）；now_ms 全宽值在 nv 栈/unified print/桥面（nv_to_pub_value 已补臂）正确，唯 .at 内算术不可用。下游 bench「.at 内差值形」的前置清偿件（平台级，另件）；工作期探针 #[ignore] 留档 | tests/plan701_supply_probes.rs `vm_time_now_ms_agrees_with_now_sec_at_level`；specs/auto-lang/vm/design/time-natives.md 桥面节 |
| P701-D2 | low | trans JS 轨 | **JS 轨 print 模块内遮蔽（预存运行期边界）**——javascript.rs 维持裸 `console.log`（JS golden 字面断言，改道已回退 c9901f762）；store `console` 字段模块内在 JS 轨运行期同形遮蔽，无 TS2339 编译面。SD-03 已改口注记；如需清偿随 JS golden 重基线一并 | crates/auto-lang/src/trans/javascript.rs:427；specs/auto-lang/ui/design/vue-use-fn-emission.md strict-TS 节 |
