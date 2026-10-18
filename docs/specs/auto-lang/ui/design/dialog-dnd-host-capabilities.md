# 对话框与拖出宿主能力（PLAN-804）——双独立 provider 槽 / 缺席矩阵 / 完成事件链 / 父窗策略

> 沉淀来源：PLAN-804（2026-10-18，dialog_open/save 2927/2928 +
> dnd_start 2938 迁宿主 provider；779/785/790/794/802 装配模式的
> dialog/dnd 批次）。VM 侧总览见 vm/overview「dialog/dnd native
> 宿主能力（PLAN-804）」节；UI 装配现状见 ui/overview §现状（PLAN-804）。

## 1. 双独立能力槽（值域冻结）

- **`DialogHostProvider`**（2927/2928）：`fn open(&self, filter: &str)
  -> String` / `fn save(&self, default_name: &str) -> String`。跨界仅
  owned `String`——出参路径串（`to_string_lossy` 形，`""`=取消/宿主不可用，
  非 raise）；入参 raw `filter`/`default_name` 是 shim 侧 `pop_string_arg`
  已物化 owned String 的临时 `&str`（785 set_text 同款，不泄漏 VM 字符串
  索引/借用缓冲）。provider 接**原始** filter/default_name——逗号/分号/
  空格/tab 分割、去前导点、空项丢弃的公共过滤语义只在单一 OS 实现
  （`host_adapters::dialog`）维护，core 不复制第二套 parser。
- **`DndHostProvider`**（2938）：`fn start(&self, payload_json: String)
  -> bool`。JSON 是 dnd 既有 wire 输入——text/files/virtual_files、
  base64 与裸反斜杠重试规则由唯一 UI 载荷 parser
  （`ui::native_dnd::parse_payload_json`）解释，core 不复制。返回 Bool =
  旧 start_drag 会话合同：**true=执行过会话（落下或取消均 true）**，效果
  另由 `on_dnd_finished` 事件报告——不得把 cancel 映射 false / 把 true 当
  drop 成功。空载荷/坏 JSON/已有会话重入 → false。
- 两槽**独立装配**：一种能力在位不得掩盖另一种缺席（dialog 在位 +
  dnd 缺席 = 各自语义分别生效）。

## 2. 缺席矩阵（T-01 冻结，= 迁移前 feature-off 臂逐字面值）

| native | 无 provider | 在位取消/不可用 |
|---|---|---|
| dialog_open/save | 弹实参**前** raise `dialog_open/save: the \`ui-dialog\` feature is disabled` VMError（错误中止下栈面与旧臂一致） | 空串 |
| dnd_start | 弹实参后返 false（G5 值降级，非 raise） | 同左（会话级失败 false） |

## 3. 装配规则（与 779/785/790/794/802 逐条同构）

显式进程默认恒胜兼容 if-absent（同锁原子）、构造期进程默认快照
（`AutoVM::new`）、实例级 `set_dialog_host_provider`/
`set_dnd_host_provider` 覆盖；`NativeInterface` Clone 保槽、VM merge 保
目标实例、注入不动 `binding_revision`/proof。调用 provider 前**释放槽锁**
（accessor clone Arc 即返）——真实模态阻塞可重入，provider 回调可再访问/
设置槽；`Send + Sync` 不授权 COM 跨线程。

## 4. 实现落点与门控

- **dialog**：单一 OS 实现 `host_adapters::dialog`（IFileDialog；
  `ParentedOsDialogProvider` + unparented open/save），挂 `ui-dialog`
  **单门控**——OS 实现不受 ui renderer 门控制，`ui-dialog`-only（无 ui）
  构建经 `install_ui_host_capabilities`（门=`any(ui, ui-dialog)`）继续安装，
  恢复迁移前高层对话框路径。旧 `ui::dialog` 门面保留为薄委托（消费方
  零改）。
- **dnd**：`ui/dnd_host_provider.rs`（windows ∧ native-dnd 门控，模块本身
  在 ui 门内=有效三分），委托既有 `ui::native_dnd::win32::start_drag`。
  **DoDragDrop 在调用线程（输入线程）内联阻塞**——不得改成 spawn/async
  提前返 true（历史「专用 STA 线程」叙述已纠正为输入线程内联真实现）。

## 5. 完成事件链与代际锚定

完成效果经生产订阅链交付：`dnd_start`（UI 线程内联模态 DoDragDrop）→
`session::dnd_finished_subscription`（生产 16ms 泵，仅 Desktop 模式挂载）→
生产 update 臂 `DesktopEvent::DndFinished` → `dnd_finished_target`
（**按发起者身份的代际锚定**，由 dispatch 环置位——与完成时焦点在哪
个 App 无关；焦点他窗偏差抗性由
`dnd_finished_delivery_anchors_at_initiator` 锚永久钉扎）→
`inject_native_event` → 发起方 App 的 `on_dnd_finished`。非广播——仅
一个目标收到（真宿主链实录：干扰方 App marker 零送达）。

## 6. 父窗策略（VM parented / a2r unparented 差异保留）

VM 默认路径 parented：宿主窗 = `EnumWindows` 最大可见本进程顶层窗 +
`OnceLock` 进程缓存（进程寿命——共享父窗线程不得随测试线程退出销毁）；
a2r 旧门面路径 unparented（owner=0，存活期采样断言由 plan804 V04
正反控钉扎）。迁移前双份 rfd 构建合一（VM 直建与 `ui::dialog` 各一份 →
唯一 `host_adapters::dialog`）；不在本轮修 OnceLock 缓存、拖入或 OLE
实现。

## 7. 验收面注记（R4 冻结）

真实宿主链证据形态：`run_dynamic_desktop`（子进程真 main）→ 真实鼠标
点击发起方按钮 → handler 内 `dnd_start` → 生产订阅/完成臂 → 发起方
handler 落盘 `finish=copy`；启动停滞/交付失败由 `ChildGuard`（spawn 即
归属，Drop 全出口 kill+wait）回收宿主子进程，退出键态零残留。
examples/plan804_real_host 与 STALL_MS/NO_DRAG 反控钩子是**验收设施**，
不新增生产 API 或异步拖出语义。
