// PLAN-779: VM 宿主能力装配点（pilot = frame 族 9918/9919）。
//
// Design 37 §4 四层拆分中的"装配"层落点：core（vm）只持有能力接口与
// per-VM 存储，不引用任何 UI/宿主实现；宿主（UI adapter、外部 consumer）
// 经显式 factory/provider 注册供值——不依赖 linker 自动拉 inventory 的
// 偶然性。进程默认 + 实例覆盖两级：
// - 进程默认：宿主启动时 `set_process_default_frame_provider` 显式注册；
//   `AutoVM::new` 在构造期**快照**当前默认进本实例——之后进程默认替换
//   不影响已建实例，两 VM 实例可持不同 provider（不串线）。
// - 实例覆盖：`AutoVM::set_frame_provider` / `NativeInterface::
//   set_frame_provider`，任何时点对本实例替换。
//
// 缺席语义（与 pilot 前一致）：无 provider = 未捕获，读侧 0。provider
// 注入不改变绑定身份（shim 函数/ID/contract 均不动），因此**不** bump
// binding_revision、不撤销 reference proof——register/merge 的既有失效
// 规则原样保留（计划 AC-02/V-03）。
//
// provider `Send + Sync` 只约束 provider 自身可跨线程共享；不代表 !Send
// 的 VM 实例可跨线程迁移（计划 §3）。

use std::sync::Arc;
use std::sync::RwLock;

/// 帧时间戳观测能力（pilot 唯一族；毫秒、单调时源由实现方保证）。
///
/// 值域约定与 VM 内建 9918/9919 出口一致：i64 内部态，shim 侧
/// saturating 收窄 i32（`min(i32::MAX)`）；未捕获/未发生 = 0。
pub trait FrameProvider: Send + Sync {
    /// 帧开始时间戳（宿主定义的帧起点语义）。
    fn begin_ms(&self) -> i64;
    /// 呈现完成时间戳（宿主定义的 present 语义）。
    fn present_ms(&self) -> i64;
}

/// 进程级默认 provider（宿主显式注册；None = 未注册 → VM 缺席 0）。
///
/// 替换语义：后注册者胜（宿主启动/测试装配用）；已建 VM 实例的快照
/// 不受影响。
static PROCESS_DEFAULT_FRAME_PROVIDER: RwLock<Option<Arc<dyn FrameProvider>>> = RwLock::new(None);

/// 注册进程默认 frame provider（显式权威 API：替换现存值，后注册者胜）。
/// 宿主启动/测试装配用；已建 VM 实例的快照不受影响。
pub fn set_process_default_frame_provider(provider: Arc<dyn FrameProvider>) {
    let mut slot = PROCESS_DEFAULT_FRAME_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_FRAME_PROVIDER poisoned");
    *slot = Some(provider);
}

/// 兼容默认装配（F779-R2-01）：**仅当槽位为空时**安装。检查与安装在同一
/// 写锁临界区内完成——并发的显式 `set_` 与本操作互斥，不存在"检查后
/// 被覆盖/覆盖他人"的窗口。供 UI 等内建宿主的隐式入口使用：兼容默认
/// 只补缺，不覆盖调用方显式选择（显式 `set_` 恒胜；需要强制替换走显式
/// API）。
pub fn set_process_default_frame_provider_if_absent(provider: Arc<dyn FrameProvider>) {
    let mut slot = PROCESS_DEFAULT_FRAME_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_FRAME_PROVIDER poisoned");
    if slot.is_none() {
        *slot = Some(provider);
    }
}

/// 当前进程默认（`AutoVM::new` 构造期快照消费）。
pub fn process_default_frame_provider() -> Option<Arc<dyn FrameProvider>> {
    PROCESS_DEFAULT_FRAME_PROVIDER
        .read()
        .expect("PROCESS_DEFAULT_FRAME_PROVIDER poisoned")
        .clone()
}

/// 清除进程默认（回到缺席语义：新建 VM 未捕获 0）。测试装配/宿主拆卸用。
pub fn clear_process_default_frame_provider() {
    let mut slot = PROCESS_DEFAULT_FRAME_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_FRAME_PROVIDER poisoned");
    *slot = None;
}

// ── PLAN-785: 文本剪贴板能力（auto.clipboard.text 2925 / set_text 2926）──
//
// 779 装配模式的第一个真实迁移批次：core 只持本接口与 per-VM 存储；
// OS 实现留在 UI 侧（clipboard_provider adapter 委托既有 arboard 桥），
// no-ui 宿主可注入自定义 provider。与 frame 族的差别：文本能力进出均
// 为**值**——出参 owned `String`（None=宿主不可用→shim 落空串，非
// raise）、入参 `&str`（shim 侧已 `pop_string_arg` 物化为 owned Rust
// String，接口不泄漏 VM 字符串索引/借用缓冲）。
//
// 缺席语义（V03/R-C）：无 provider = 与迁移前 no-ui 构建同形 VMError
//（shim 侧保留原 "the `ui-clipboard` feature is disabled" 文本，函数名
// 即 2925/2926 诊断定位面）；有 provider 但 OS 不可用 = empty/false。
// 任何状态不得伪装成功写入。

/// 文本剪贴板能力接口（ owning Rust 文本/布尔值域，无 UI 类型）。
pub trait ClipboardTextProvider: Send + Sync {
    /// 读剪贴板文本（None = 宿主/OS 不可用；空剪贴板返回 `Some("")`，
    /// 与 arboard `get_text` 的空串形一致——由实现方保证）。
    fn text(&self) -> Option<String>;
    /// 写剪贴板文本；false = 写入失败（不可用/被占用）。
    fn set_text(&self, text: &str) -> bool;
}

/// 进程级默认 clipboard text provider（显式注册；None = 未注册 → shim
/// 缺席错误）。替换/快照/if-absent 语义与 frame 族逐条相同。
static PROCESS_DEFAULT_CLIPBOARD_TEXT_PROVIDER: RwLock<Option<Arc<dyn ClipboardTextProvider>>> =
    RwLock::new(None);

/// 注册进程默认（显式权威 API：替换现存值，后注册者胜）。
pub fn set_process_default_clipboard_text_provider(provider: Arc<dyn ClipboardTextProvider>) {
    let mut slot = PROCESS_DEFAULT_CLIPBOARD_TEXT_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_CLIPBOARD_TEXT_PROVIDER poisoned");
    *slot = Some(provider);
}

/// 兼容默认装配（仅空槽补缺，同锁原子——语义同 frame 族 F779-R2-01）。
/// 供 UI 等内建宿主隐式入口：不覆盖调用方显式默认。
pub fn set_process_default_clipboard_text_provider_if_absent(
    provider: Arc<dyn ClipboardTextProvider>,
) {
    let mut slot = PROCESS_DEFAULT_CLIPBOARD_TEXT_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_CLIPBOARD_TEXT_PROVIDER poisoned");
    if slot.is_none() {
        *slot = Some(provider);
    }
}

/// 当前进程默认（`AutoVM::new` 构造期快照消费）。
pub fn process_default_clipboard_text_provider() -> Option<Arc<dyn ClipboardTextProvider>> {
    PROCESS_DEFAULT_CLIPBOARD_TEXT_PROVIDER
        .read()
        .expect("PROCESS_DEFAULT_CLIPBOARD_TEXT_PROVIDER poisoned")
        .clone()
}

/// 清除进程默认（回到缺席语义）。测试装配/宿主拆卸用。
pub fn clear_process_default_clipboard_text_provider() {
    let mut slot = PROCESS_DEFAULT_CLIPBOARD_TEXT_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_CLIPBOARD_TEXT_PROVIDER poisoned");
    *slot = None;
}

// ── PLAN-790: 文件/图像剪贴板能力（files_get 2934 / files_set 2935 /
//    image_get 2936 / image_set 2937）───────────────────────────────────
//
// 785 文本批次的 files/image 姊妹篇：core 只持本接口与 per-VM 存储；
// OS/PNG/DIB 实现留在 UI 侧（clipboard_media_provider adapter 委托既有
// clipboard_native Win32 桥），no-ui 宿主可注入自定义实现。
// 与文本族的**关键差别**：迁移前无 OS 面（非 Windows / 未开
// native-clipboard）走 G3 值降级（空表/false/null），非 VMError——
// provider 缺席/失败一律保持同形值降级，接口不得用 Result 吞错改异常。
//
// 值域纪律（计划 §5）：provider 只使用标准库 owned 数据——路径列表
// `Vec<PathBuf>`、图像三元组 `ClipboardImage{path,width,height}`；不暴露
// VM 堆句柄、字符串池索引、UI 类型或 Win32 HANDLE。临时 PNG 生命周期
// 仍由实现方（既有 OS 策略）管理：落 temp 不即删，返回路径在消费期可读。

/// image_get 产物元数据（G2 Record 值域镜像：owned path + 像素尺寸；
/// ui 侧 TempImage 的 core 无 UI 形态）。
pub struct ClipboardImage {
    /// 临时 PNG 文件路径（实现方保证消费期可读）。
    pub path: std::path::PathBuf,
    pub width: u32,
    pub height: u32,
}

/// 文件/图像剪贴板能力接口（owning 标准库值域，无 UI 类型）。
///
/// 四方法与 VM 内建 2934..2937 一一对应；全部**值语义**：
/// - 读侧缺席/不可用 = `Vec::new()` / `None`（shim 落空表/null）；
/// - 写侧失败（含空路径列表）= `false`；
/// - 任何实现不得伪装成功写入。
pub trait ClipboardMediaProvider: Send + Sync {
    /// 读文件列表（CF_HDROP 值域；空 = 缺席/不可用）。
    fn files_get(&self) -> Vec<std::path::PathBuf>;
    /// 写文件列表；false = 失败（空输入/被占用/无权限）。
    fn files_set(&self, paths: &[std::path::PathBuf]) -> bool;
    /// 读图像为临时 PNG 元数据；None = 无图像格式/不可用。
    fn image_get(&self) -> Option<ClipboardImage>;
    /// 写 PNG 文件到剪贴板；false = 失败（不可读/超限/Win32 败）。
    fn image_set(&self, png_path: &std::path::Path) -> bool;
}

/// 进程级默认 clipboard media provider（显式注册；None = 未注册 → shim
/// G3 值降级）。替换/快照/if-absent 语义与 frame/text 族逐条相同。
static PROCESS_DEFAULT_CLIPBOARD_MEDIA_PROVIDER: RwLock<Option<Arc<dyn ClipboardMediaProvider>>> =
    RwLock::new(None);

/// 注册进程默认（显式权威 API：替换现存值，后注册者胜）。
pub fn set_process_default_clipboard_media_provider(provider: Arc<dyn ClipboardMediaProvider>) {
    let mut slot = PROCESS_DEFAULT_CLIPBOARD_MEDIA_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_CLIPBOARD_MEDIA_PROVIDER poisoned");
    *slot = Some(provider);
}

/// 兼容默认装配（仅空槽补缺，同锁原子——语义同 frame/text 族）。
/// 供 UI 等内建宿主隐式入口：不覆盖调用方显式默认。
pub fn set_process_default_clipboard_media_provider_if_absent(
    provider: Arc<dyn ClipboardMediaProvider>,
) {
    let mut slot = PROCESS_DEFAULT_CLIPBOARD_MEDIA_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_CLIPBOARD_MEDIA_PROVIDER poisoned");
    if slot.is_none() {
        *slot = Some(provider);
    }
}

/// 当前进程默认（`AutoVM::new` 构造期快照消费）。
pub fn process_default_clipboard_media_provider() -> Option<Arc<dyn ClipboardMediaProvider>> {
    PROCESS_DEFAULT_CLIPBOARD_MEDIA_PROVIDER
        .read()
        .expect("PROCESS_DEFAULT_CLIPBOARD_MEDIA_PROVIDER poisoned")
        .clone()
}

/// 清除进程默认（回到 G3 值降级缺席语义）。测试装配/宿主拆卸用。
pub fn clear_process_default_clipboard_media_provider() {
    let mut slot = PROCESS_DEFAULT_CLIPBOARD_MEDIA_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_CLIPBOARD_MEDIA_PROVIDER poisoned");
    *slot = None;
}

// ── PLAN-794: scroll controller 与子锚点宿主能力（9900..9905 + 9922/9923）──
//
// 779/785/790 装配模式的 scroll 族批次：core shim 不再直引 UI 的 scroll
// 模块（controller 注册表/intent 队列/锚点状态机），经本接口由宿主供值——
// UI 宿主注册委托既有 registry 的 adapter（renderer 排空/测量/锚点状态机
// 仍是单一真源，core 不复制第二个 registry）；no-ui 宿主可注入自定义
// 实现（V01 外部 consumer）。值域纪律（计划 §2/§5）：接口只使用 owned
// 标准库值——`String` 句柄/错误 reason、f64 测量、core 自有轴/intent/状态
// DTO；不暴露 UI `Axis`/`ScrollIntent` 类型、VM heap id、字符串池索引或
// renderer 引用。
//
// 缺席语义（T-01 冻结矩阵，= 迁移前 no-ui 降级臂逐字面值）：controller
// → Int 0；to_start/to_end/by/to → Bool false；state → Float 0.0（均非
// raise）；to_child → "error:unsupported"；anchor_state → "unknown"。
// 句柄语义：任意非空 opaque string（不强制 "@scrollctl:" 前缀——.at 侧
// 裸字符串柄等价）；9901..9904 空串句柄跳过入队仍按 provider 在位性返
// true/false。

/// 单个滚动轴（core 自有形态；UI 宿主 adapter 转换为 UI 侧 Axis 类型）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScrollAxisDto {
    X,
    Y,
}

/// 程序化滚动意图（core 自有形态；natives 恒 Programmatic 来源，故不带
/// source 字段——UI adapter 落 `ScrollSource::Programmatic`）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollIntentDto {
    ToStart { axis: ScrollAxisDto },
    ToEnd { axis: ScrollAxisDto },
    ScrollBy { axis: ScrollAxisDto, delta: f64 },
    ScrollTo { axis: ScrollAxisDto, offset: f64 },
}

/// `auto.scroll.state` 的测量快照投影（8 具名字段 record 的值源；未绑定/
/// 未测量 = 全零——由实现方保证，与 controller 注册表 default 投影一致）。
/// progress 由实现方按其几何单源投影——core 不复制滚动算法。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ScrollSnapshotDto {
    pub offset_x: f64,
    pub offset_y: f64,
    pub viewport_w: f64,
    pub viewport_h: f64,
    pub content_w: f64,
    pub content_h: f64,
    pub progress_x: f64,
    pub progress_y: f64,
}

/// PLAN-781 锚点请求状态（core 自有形态；`Invalid(reason)` 的 reason
/// `not_found`/`not_bound`/`unsupported`/`unmeasured` 由实现方透传）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScrollAnchorStateDto {
    Pending,
    Applied,
    Invalid(String),
}

/// scroll 宿主能力接口（owning 值域，无 UI 类型；五方法对应八 native：
/// controller 句柄分配 / intent 入队（to 双轴坐标形 = 两次调用，X 先 Y 后，
/// 由 shim 编排）/ 快照读出 / 锚点请求入队 / 锚点状态查询）。
///
/// 实现契约：enqueue_anchor_request 返回 Err(reason) 即回执
/// "error:{reason}"（bad_handle/empty_key/unsupported 由实现方校验——UI
/// adapter 委托既有 controller::enqueue_anchor_request 原语义）；状态机
/// 完整继承 781（queued≠完成、managed 拒绝、换绑代际、一次性消费——见
/// controller.rs，renderer 侧解析不迁）。方法在 VM 锁外调用，实现方自管
/// 内部同步。
pub trait ScrollHostProvider: Send + Sync {
    /// 分配下一个 controller 句柄（不透明非空串）。
    fn next_controller_handle(&self) -> String;
    /// 程序化 intent 入队（renderer/宿主消费期排空解析）。
    fn enqueue_intent(&self, handle: &str, intent: ScrollIntentDto);
    /// 最近测量快照读出（未绑定 → 全零默认）。
    fn controller_snapshot(&self, handle: &str) -> ScrollSnapshotDto;
    /// 锚点定位请求入队；Err(reason) → "error:{reason}" 回执。
    fn enqueue_anchor_request(&self, handle: &str, key: &str) -> Result<(), &'static str>;
    /// 锚点请求状态查询；None = 无记录（"unknown"）。
    fn anchor_request_state(&self, handle: &str, key: &str) -> Option<ScrollAnchorStateDto>;
}

/// 进程级默认 scroll host provider（显式注册；None = 未注册 → shim 走
/// T-01 冻结的缺席降级臂）。替换/快照/if-absent 语义与 frame/text/media
/// 族逐条相同。
static PROCESS_DEFAULT_SCROLL_HOST_PROVIDER: RwLock<Option<Arc<dyn ScrollHostProvider>>> =
    RwLock::new(None);

/// 注册进程默认（显式权威 API：替换现存值，后注册者胜）。
pub fn set_process_default_scroll_host_provider(provider: Arc<dyn ScrollHostProvider>) {
    let mut slot = PROCESS_DEFAULT_SCROLL_HOST_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_SCROLL_HOST_PROVIDER poisoned");
    *slot = Some(provider);
}

/// 兼容默认装配（仅空槽补缺，同锁原子——语义同 frame/text/media 族）。
/// 供 UI 等内建宿主隐式入口：不覆盖调用方显式默认。
pub fn set_process_default_scroll_host_provider_if_absent(provider: Arc<dyn ScrollHostProvider>) {
    let mut slot = PROCESS_DEFAULT_SCROLL_HOST_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_SCROLL_HOST_PROVIDER poisoned");
    if slot.is_none() {
        *slot = Some(provider);
    }
}

/// 当前进程默认（`AutoVM::new` 构造期快照消费）。
pub fn process_default_scroll_host_provider() -> Option<Arc<dyn ScrollHostProvider>> {
    PROCESS_DEFAULT_SCROLL_HOST_PROVIDER
        .read()
        .expect("PROCESS_DEFAULT_SCROLL_HOST_PROVIDER poisoned")
        .clone()
}

/// 清除进程默认（回到 T-01 冻结的缺席降级语义）。测试装配/宿主拆卸用。
pub fn clear_process_default_scroll_host_provider() {
    let mut slot = PROCESS_DEFAULT_SCROLL_HOST_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_SCROLL_HOST_PROVIDER poisoned");
    *slot = None;
}

// ── PLAN-802: code editor 宿主能力（22 native 全族）─────────────────────
//
// 779/785/790/794 装配模式的 code_editor 批次（2910..2915、2919..2924、
// 2932/2933、2939、9906、9908..9913）：core shim 不再直引 UI 侧
// code_editor 注册表（及 editor 寄宿 scroll controller / OS 剪贴板
// 桥），经本接口由宿主供值——UI 宿主注册委托既有 registry 的 adapter
// （editor 注册表仍是单一真源，core 不复制第二份 rope/undo/fold 状态）；
// no-ui 宿主可注入自定义实现（V01 外部 consumer fixture）。
//
// 值域纪律（计划 §2/§5 + T-01 冻结矩阵）：接口只使用 owned 标准库值——
// `String` 键/文本/delta JSON、usize/i64/f64 标量、core 自有 DTO；不暴露
// `CodeEditorCore`/rope/`ScrollIntent`/`ClipboardOp`/renderer 引用或 VM 堆
// 句柄。19 具名方法覆盖 22 native（cursor×3 / scroll_offset×2 共享一次
// 读取，与迁移前 `code_editor_cursor`/`controller_snapshot` 单调用形态
// 同构）；禁止字符串 method 总分发。
//
// 单位/语义锚（T-01 冻结，673/701 不变）：
// - 光标：0 基行 + char 列；selection_len = bytes；fold 入口 1 基行（shim
//   侧 max(1)-1 转 0 基后入接口）；edit offset = byte offset（char 边界
//   校验在实现侧，非边界返 false）。
// - delta：destructive read——每次调用恰消费一次队列，空队列返当前水位
//   的 `"deltas":[]` JSON；`text` 不清队列。
// - save：rope→文件直写（无全文 VM 字符串往返）；BOM/EOL 包装归 front。
// - scroll_offset：未绑定/未测量 = 0.0（注册表 default 投影同形）。
// - cut/copy/paste：`Option<bool>`——None = 宿主有编辑器面但无 OS 剪贴板
//   面（= 迁移前 code-editor on + ui-clipboard off 组合的 raise 臂），
//   Some(b) = 操作执行结果；任何实现不得伪装成功写入。
//
// 缺席语义（T-01 冻结矩阵，= 迁移前 feature-off 分支逐字面值）：无
// provider 时 21 项 raise `…: the \`code-editor\` feature is disabled`
//（cut/copy/paste 为组合特征文案）；9913 scroll_to 唯一例外——弹参后
// push Bool false（值缺席臂，非 raise）。

/// `code_editor_cursor_line/col/selection_len`（2911..2913）的共享读取
/// 形态（core 自有；UI adapter 由 `code_editor_cursor` 的
/// `Option<(usize, usize, usize)>` 直投影）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeEditorCursorDto {
    /// 光标行（**0 基**，与读侧 673/701 口径一致）。
    pub line: usize,
    /// 光标列（**char 列**，非字节）。
    pub char_col: usize,
    /// 选区长度（**bytes**，非 char 数）。
    pub selection_bytes: usize,
}

/// `code_editor_scroll_offset_x/y`（9911/9912）的共享读取形态。未绑定/
/// 未测量 = (0.0, 0.0)——由实现方保证（UI adapter = controller 注册表
/// default 投影同形）。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CodeEditorScrollOffsetDto {
    pub x: f64,
    pub y: f64,
}

/// code editor 宿主能力接口（owning 值域，无 UI 类型；19 方法 ↔ 22 native，
/// 映射表见计划 T-01 冻结矩阵）。
///
/// 实现契约：方法在 VM 锁外调用，实现方自管内部同步；不跨回调持锁/借用
/// （UI 注册表回调重入安全由既有 editor 实现保证，adapter 不新增锁序）。
pub trait CodeEditorHostProvider: Send + Sync {
    /// 2910：全文本读出（None = 无此键编辑器 → shim raise no-editor）。
    fn text(&self, key: &str) -> Option<String>;
    /// 2939：统一 delta 队列 destructive read（None = 无此键编辑器）。
    fn delta(&self, key: &str) -> Option<String>;
    /// 9906：区间结构化写（插入/删除/替换按区间形状）。false = 无此键/
    /// 反区间/越界/非 UTF-8 char 边界；成功进人机统一 delta 队列。
    /// 负 offset 钳位在 shim 侧（i32→usize 边界），不到本方法。
    fn edit(&self, key: &str, start: usize, end: usize, replacement: &str) -> bool;
    /// 2911..2913 共享：光标三元组读出（None = 无此键 → shim 落 0）。
    fn cursor(&self, key: &str) -> Option<CodeEditorCursorDto>;
    /// 2914：跳到下一个正则匹配（wrap）。
    fn find(&self, key: &str) -> bool;
    /// 2915：程序化全文写（diff-guarded）；false = 无此键（shim raise）。
    fn set_text(&self, key: &str, text: &str) -> bool;
    /// 9908：批量装载（返回总 bytes；None = 无此键/IO 失败/非 UTF-8 →
    /// shim 落 -1 **值**，非 raise）。
    fn load_file(&self, key: &str, path: &str) -> Option<i64>;
    /// 9909：rope→文件直写；false = 无此键/IO 失败（**值**，非 raise）。
    fn save(&self, key: &str, path: &str) -> bool;
    /// 9910：程序化光标放置（0 基行/char 列，钳位入文档，弃选区）；
    /// false = 无此键。负参钳位在 shim 侧。
    fn set_cursor(&self, key: &str, line: usize, char_col: usize) -> bool;
    /// 9911/9912 共享：editor 寄宿 scroller 偏移读出（未绑定 = (0.0, 0.0)）。
    fn scroll_offset(&self, key: &str) -> CodeEditorScrollOffsetDto;
    /// 9913：程序化滚动（双轴绝对偏移）。UI 宿主 = intent 入队恒 true
    /// （未绑定句柄由 drain 静默丢弃）；no-ui 宿主自定义。
    fn scroll_to(&self, key: &str, x: f64, y: f64) -> bool;
    /// 2932：折叠切换（line_0 = 0 基行；1 基→0 基钳位在 shim 侧）。
    /// true = 折叠后处于折叠态。纯视图状态，无 undo 条目。
    fn fold_toggle(&self, key: &str, line_0: usize) -> bool;
    /// 2933：当前被折叠隐藏的行数。
    fn fold_hidden_count(&self, key: &str) -> usize;
    /// 2919：撤销一步；false = 无此键。
    fn undo(&self, key: &str) -> bool;
    /// 2920：重做一步；false = 无此键。
    fn redo(&self, key: &str) -> bool;
    /// 2921：全选；false = 无此键。
    fn select_all(&self, key: &str) -> bool;
    /// 2922：选区→OS 剪贴板后删除。None = 宿主无编辑器剪贴板面
    /// （shim raise 组合特征文案）。
    fn cut(&self, key: &str) -> Option<bool>;
    /// 2923：选区→OS 剪贴板（不删除）。None 同上。
    fn copy(&self, key: &str) -> Option<bool>;
    /// 2924：OS 剪贴板→光标处插入。None 同上。
    fn paste(&self, key: &str) -> Option<bool>;
}

/// 进程级默认 code editor host provider（显式注册；None = 未注册 → shim
/// 走 T-01 冻结的缺席臂）。替换/快照/if-absent 语义与 frame/text/media/
/// scroll 族逐条相同。
static PROCESS_DEFAULT_CODE_EDITOR_HOST_PROVIDER: RwLock<Option<Arc<dyn CodeEditorHostProvider>>> =
    RwLock::new(None);

/// 注册进程默认（显式权威 API：替换现存值，后注册者胜）。
pub fn set_process_default_code_editor_host_provider(provider: Arc<dyn CodeEditorHostProvider>) {
    let mut slot = PROCESS_DEFAULT_CODE_EDITOR_HOST_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_CODE_EDITOR_HOST_PROVIDER poisoned");
    *slot = Some(provider);
}

/// 兼容默认装配（仅空槽补缺，同锁原子——语义同 frame/text/media/scroll 族）。
/// 供 UI 等内建宿主隐式入口：不覆盖调用方显式默认。
pub fn set_process_default_code_editor_host_provider_if_absent(
    provider: Arc<dyn CodeEditorHostProvider>,
) {
    let mut slot = PROCESS_DEFAULT_CODE_EDITOR_HOST_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_CODE_EDITOR_HOST_PROVIDER poisoned");
    if slot.is_none() {
        *slot = Some(provider);
    }
}

// ── PLAN-804: 对话框宿主能力（dialog_open 2927 / dialog_save 2928）─────
//
// 779/785/790/794 装配模式的 dialog 批次：core shim 不再直引 rfd/
// raw-window-handle/Win32（dialog_parent 迁往 host_adapters），经本接口
// 由宿主供值；UI/OS 实现放 crate 上层 `host_adapters::dialog`（ui-dialog
// 单门控，独立于 ui renderer 门——ui-dialog-only 构建恢复旧高层路径），
// no-ui 宿主可注入自定义实现。
//
// 值域纪律（计划 §2/§5）：跨界仅 owned `String`——出参路径串（""=取消/
// 宿主不可用，非 raise）；入参 raw filter/default_name（`&str` 借用 shim
// 侧 `pop_string_arg` 已物化的 owned String，785 set_text 同款，不泄漏 VM
// 字符串索引/借用缓冲）。provider 接**原始** filter/default_name——逗号/
// 分号/空格/tab 分割、去前导点、空项丢弃的公共过滤语义只在单一 OS 实现
// （host_adapters::dialog）维护，core 不复制第二套 parser。
//
// 缺席语义（T-01 冻结矩阵 = 迁移前 feature-off 臂逐字面值）：无 provider
// → `VMError "dialog_open/save: the \`ui-dialog\` feature is disabled"`
// （shim 在弹实参前 raise——错误中止下栈面与旧臂一致）；在位 provider
// 取消/不可用返回空串。模态重入：NI 槽锁在调用前释放（accessor clone Arc
// 即返），真实模态阻塞可重入，provider 回调可再访问/设置槽（V03）。

/// OS 文件对话框能力接口（owning String 值域，无 UI/Win32 类型）。
pub trait DialogHostProvider: Send + Sync {
    /// 打开文件选择对话框；返回所选路径（`to_string_lossy` 形），
    /// `""` = 取消/宿主不可用（实现方保证）。
    fn open(&self, filter: &str) -> String;
    /// 保存目标选择对话框；返回目标路径，`""` = 取消/不可用。
    fn save(&self, default_name: &str) -> String;
}

/// 进程级默认 dialog host provider（显式注册；None = 未注册 → shim
/// 缺席错误）。替换/快照/if-absent 语义与 frame/text/media/scroll 族
/// 逐条相同。
static PROCESS_DEFAULT_DIALOG_HOST_PROVIDER: RwLock<Option<Arc<dyn DialogHostProvider>>> =
    RwLock::new(None);

/// 注册进程默认（显式权威 API：替换现存值，后注册者胜）。
pub fn set_process_default_dialog_host_provider(provider: Arc<dyn DialogHostProvider>) {
    let mut slot = PROCESS_DEFAULT_DIALOG_HOST_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_DIALOG_HOST_PROVIDER poisoned");
    *slot = Some(provider);
}

/// 兼容默认装配（仅空槽补缺，同锁原子——语义同 frame/text/media/scroll
/// 族）。供 ui-dialog 构建的内建宿主隐式入口（含 ui-dialog-only 无 ui
/// 形态）：不覆盖调用方显式默认。
pub fn set_process_default_dialog_host_provider_if_absent(provider: Arc<dyn DialogHostProvider>) {
    let mut slot = PROCESS_DEFAULT_DIALOG_HOST_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_DIALOG_HOST_PROVIDER poisoned");
    if slot.is_none() {
        *slot = Some(provider);
    }
}

/// 当前进程默认（`AutoVM::new` 构造期快照消费）。
pub fn process_default_code_editor_host_provider() -> Option<Arc<dyn CodeEditorHostProvider>> {
    PROCESS_DEFAULT_CODE_EDITOR_HOST_PROVIDER
        .read()
        .expect("PROCESS_DEFAULT_CODE_EDITOR_HOST_PROVIDER poisoned")
        .clone()
}

/// 清除进程默认（回到 T-01 冻结缺席臂语义）。测试装配/宿主拆卸用。
pub fn clear_process_default_code_editor_host_provider() {
    let mut slot = PROCESS_DEFAULT_CODE_EDITOR_HOST_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_CODE_EDITOR_HOST_PROVIDER poisoned");
    *slot = None;
}

/// 当前进程默认（`AutoVM::new` 构造期快照消费）。
pub fn process_default_dialog_host_provider() -> Option<Arc<dyn DialogHostProvider>> {
    PROCESS_DEFAULT_DIALOG_HOST_PROVIDER
        .read()
        .expect("PROCESS_DEFAULT_DIALOG_HOST_PROVIDER poisoned")
        .clone()
}

/// 清除进程默认（回到缺席错误语义）。测试装配/宿主拆卸用。
pub fn clear_process_default_dialog_host_provider() {
    let mut slot = PROCESS_DEFAULT_DIALOG_HOST_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_DIALOG_HOST_PROVIDER poisoned");
    *slot = None;
}

// ── PLAN-804: 拖出宿主能力（dnd_start 2938）────────────────────────────
//
// 同批次第二槽（与 dialog 槽独立装配——一种能力在位不得掩盖另一种缺席，
// 计划 §2）：core shim 不再直引 UI 的 native_dnd 模块，经本接口由宿主供值；
// OS/COM 实现留 UI 侧 adapter（windows ∧ native-dnd ∧ ui 三门控安装，
// 委托既有 start_drag），no-ui 宿主可注入自定义实现。
//
// 值域纪律：JSON 仍是 dnd 现有 wire 输入——provider 接 owned JSON 串，
// text/files/virtual_files、base64 与裸反斜杠重试规则由唯一 UI 载荷
// parser（ui 侧 native_dnd 模块的 parse_payload_json）解释，core 不复制第二套
// parser。返回 Bool = 旧 start_drag 会话合同（T-01 冻结）：true=执行过
// 会话（落下**或取消**），效果另由 on_dnd_finished 事件报告——不得把
// cancel 映射 false / 把 true 当 drop 成功。实际 DoDragDrop 在调用线程
// （输入线程）内联阻塞；不得改成 spawn/async 提前返 true。
//
// 缺席语义（T-01 冻结矩阵 = 迁移前 off 臂同形）：无 provider → 弹实参
// 后返 false（G5 值降级，非 raise）。

/// 拖出（OLE drag-out）能力接口（owned JSON 入 / Bool 出，无 UI 类型）。
pub trait DndHostProvider: Send + Sync {
    /// 发起系统拖拽：受理并完成会话后 true=执行过会话（落下或取消均
    /// true）；空载荷/坏 JSON/已有会话重入 → false。
    fn start(&self, payload_json: String) -> bool;
}

/// 进程级默认 dnd host provider（显式注册；None = 未注册 → shim 返
/// false）。替换/快照/if-absent 语义与 dialog 族逐条相同。
static PROCESS_DEFAULT_DND_HOST_PROVIDER: RwLock<Option<Arc<dyn DndHostProvider>>> =
    RwLock::new(None);

/// 注册进程默认（显式权威 API：替换现存值，后注册者胜）。
pub fn set_process_default_dnd_host_provider(provider: Arc<dyn DndHostProvider>) {
    let mut slot = PROCESS_DEFAULT_DND_HOST_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_DND_HOST_PROVIDER poisoned");
    *slot = Some(provider);
}

/// 兼容默认装配（仅空槽补缺，同锁原子——语义同 dialog 族）。供 UI 宿主
/// 隐式入口（windows+native-dnd+ui 形态）：不覆盖调用方显式默认。
pub fn set_process_default_dnd_host_provider_if_absent(provider: Arc<dyn DndHostProvider>) {
    let mut slot = PROCESS_DEFAULT_DND_HOST_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_DND_HOST_PROVIDER poisoned");
    if slot.is_none() {
        *slot = Some(provider);
    }
}

/// 当前进程默认（`AutoVM::new` 构造期快照消费）。
pub fn process_default_dnd_host_provider() -> Option<Arc<dyn DndHostProvider>> {
    PROCESS_DEFAULT_DND_HOST_PROVIDER
        .read()
        .expect("PROCESS_DEFAULT_DND_HOST_PROVIDER poisoned")
        .clone()
}

/// 清除进程默认（回到 false 降级语义）。测试装配/宿主拆卸用。
pub fn clear_process_default_dnd_host_provider() {
    let mut slot = PROCESS_DEFAULT_DND_HOST_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_DND_HOST_PROVIDER poisoned");
    *slot = None;
}

// ── PLAN-807: 编辑器快照宿主能力（diff_snapshots 9916 供值槽）─────────
//
// 779/785/790/794/802/804 装配模式的文本批次快照槽：core shim
// `auto.diff_snapshots(key_a, key_b)` 不再直引 UI 的 editor 注册表，经本
// 接口由宿主供值——UI 宿主注册委托 `code_editor_with(key, doc_snapshot)`
// 的 adapter（editor 注册表仍是单一真源，core 不复制 rope 状态），no-ui
// 宿主可注入自定义实现。
//
// 值域纪律：出参 = `auto_text_core::RopeSnapshot`——文本 core 的同源
// owned Arc 快照（COW 冻结视图，快照传输不经 VM 全文字符串中转）；
// **文本 core 类型，非 UI 类型**，满足 802 值域纪律。若读取期间编辑器
// 被删除/替换，已取得的快照仍有效（COW 冻结——实现方无需额外防护）。
//
// 缺席语义（T-01 冻结）：无 provider = 无法找到相应 editor → shim 落
// 既有 err envelope（"编辑器不存在: <key_a>"，值非 raise）；有 provider
// 但 key 无编辑器 = 同 err 形（判错优先 key_a 先，在 shim 侧）。feature
// off 臂（cfg not code-editor）不取参即 raise——与迁移前逐字一致。

/// 编辑器快照能力接口（同源 owned 快照值域，无 UI 类型）。
pub trait EditorSnapshotProvider: Send + Sync {
    /// 取键名对应编辑器的文档快照；None = 无此键编辑器。
    fn snapshot(&self, key: &str) -> Option<auto_text_core::RopeSnapshot>;
}

/// 进程级默认 editor snapshot provider（显式注册；None = 未注册 → shim
/// 走"无法找到相应 editor"err envelope）。替换/快照/if-absent 语义与
/// frame/…/dnd 族逐条相同。
static PROCESS_DEFAULT_EDITOR_SNAPSHOT_PROVIDER: RwLock<Option<Arc<dyn EditorSnapshotProvider>>> =
    RwLock::new(None);

/// 注册进程默认（显式权威 API：替换现存值，后注册者胜）。
pub fn set_process_default_editor_snapshot_provider(provider: Arc<dyn EditorSnapshotProvider>) {
    let mut slot = PROCESS_DEFAULT_EDITOR_SNAPSHOT_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_EDITOR_SNAPSHOT_PROVIDER poisoned");
    *slot = Some(provider);
}

/// 兼容默认装配（仅空槽补缺，同锁原子——语义同既有七族）。供 UI 等内建
/// 宿主隐式入口：不覆盖调用方显式默认。
pub fn set_process_default_editor_snapshot_provider_if_absent(
    provider: Arc<dyn EditorSnapshotProvider>,
) {
    let mut slot = PROCESS_DEFAULT_EDITOR_SNAPSHOT_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_EDITOR_SNAPSHOT_PROVIDER poisoned");
    if slot.is_none() {
        *slot = Some(provider);
    }
}

// ── PLAN-808: 终端宿主能力（34 native 全族：2943..2949、2957..2959、
//    2976..2982、2983..2987、2988..2993、2998..3003）────────────────────
//
// `pilot`（779）以来第六个真实迁移批次，但形态不同：terminal 是**有状态
// 域**——端口 + 引擎服务一起迁移到 `auto-terminal-core`，VM 只编解码。
// 端口与 owned 值域（`TerminalColor/TerminalCell/TerminalSnapshot`）定义
// 在 core crate（无 auto-lang/UI 依赖）；本模块只做进程默认槽与 re-export。
//
// 缺席语义（T-01 冻结，与其它 provider 族**不同**）：VM 无槽时 shim 回落
// `auto_terminal_core::default_provider()`（进程共享 engine service），
// 保留迁移前"无 UI 也能 spawn/rows/backlog"的行为；UI 宿主安装 adapter 后
// 才有输入泵/几何/menu/scroll/window/palette 面。无 provider ≠ 禁用引擎。
//
// 句柄归属：默认共享 service 是进程单例（旧静态句柄表的等价物）；隔离
// 注入可用 `EngineService::new()` 包装成独立 provider，句柄只在所属 service
// 解析，跨 service 返回旧缺席值，不会误释放他方指针。

pub use auto_terminal_core::TerminalHostProvider;

/// 进程级默认 terminal provider（显式注册；None = 回落共享 engine service）。
static PROCESS_DEFAULT_TERMINAL_HOST_PROVIDER: RwLock<Option<Arc<dyn TerminalHostProvider>>> =
    RwLock::new(None);

/// 注册进程默认（显式权威 API：替换现存值，后注册者胜）。
pub fn set_process_default_terminal_host_provider(provider: Arc<dyn TerminalHostProvider>) {
    let mut slot = PROCESS_DEFAULT_TERMINAL_HOST_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_TERMINAL_HOST_PROVIDER poisoned");
    *slot = Some(provider);
}

/// 兼容默认装配（仅空槽补缺，同锁原子——语义同 frame/text/media/scroll 族）。
/// 供 UI 内建宿主隐式入口：不覆盖调用方显式默认。
pub fn set_process_default_terminal_host_provider_if_absent(
    provider: Arc<dyn TerminalHostProvider>,
) {
    let mut slot = PROCESS_DEFAULT_TERMINAL_HOST_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_TERMINAL_HOST_PROVIDER poisoned");
    if slot.is_none() {
        *slot = Some(provider);
    }
}

/// 当前进程默认（`AutoVM::new` 构造期快照消费）。
pub fn process_default_editor_snapshot_provider() -> Option<Arc<dyn EditorSnapshotProvider>> {
    PROCESS_DEFAULT_EDITOR_SNAPSHOT_PROVIDER
        .read()
        .expect("PROCESS_DEFAULT_EDITOR_SNAPSHOT_PROVIDER poisoned")
        .clone()
}

/// 清除进程默认（回到"无法找到相应 editor"err envelope 语义）。测试
/// 装配/宿主拆卸用。
pub fn clear_process_default_editor_snapshot_provider() {
    let mut slot = PROCESS_DEFAULT_EDITOR_SNAPSHOT_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_EDITOR_SNAPSHOT_PROVIDER poisoned");
    *slot = None;
}

/// 当前进程默认（`AutoVM::new` 构造期快照消费）。
pub fn process_default_terminal_host_provider() -> Option<Arc<dyn TerminalHostProvider>> {
    PROCESS_DEFAULT_TERMINAL_HOST_PROVIDER
        .read()
        .expect("PROCESS_DEFAULT_TERMINAL_HOST_PROVIDER poisoned")
        .clone()
}

/// 清除进程默认（回到共享 engine service 回落语义）。测试装配/宿主拆卸用。
pub fn clear_process_default_terminal_host_provider() {
    let mut slot = PROCESS_DEFAULT_TERMINAL_HOST_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_TERMINAL_HOST_PROVIDER poisoned");
    *slot = None;
}

/// 测试隔离（crate 级共享，cfg(test)）——持有者 = **清空/替换进程默认
/// 槽的测试**（clear/set/restore 序列需跨操作原子，防彼此互踩）。普通
/// 程序测试**零配合豁免**：其唯一槽写是 install 的 if-absent 补缺，对
/// 一切非"缺席=None"断言无害（F-785-R1-01→R2-01 裁定：按具名测试枚举
/// 加锁不完备——R2 反例 = plan320 普通测试经 run 族隐式装配毒化缺席
/// 窗口；缺席断言不得依赖共享进程的全局槽）。
///
/// 已试否的生产侧写域（六写 API 入互斥域）方案：run 族入口的 execute/
/// install 实际运行在 `run_autovm_capture` spawn 的独立 VM 线程上
/// （lib.rs:694 起）——持域测试线程 join 该线程、该线程 install 取域，
/// 构成持域者→join→取域环（看门狗栈实录），方案废弃；缺席断言改由
/// `AutoVM::new_without_host_snapshot`（构造式，engine.rs cfg(test)）
/// 与外部 fixture 独立进程（no-ui 无安装路径）承担。
// PLAN-814: 上层 provider 测试共用（跨 crate 测试互斥面）
pub fn test_default_slot_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake {
        begin: i64,
        present: i64,
    }

    impl FrameProvider for Fake {
        fn begin_ms(&self) -> i64 {
            self.begin
        }
        fn present_ms(&self) -> i64 {
            self.present
        }
    }

    /// 缺席语义：未注册 → None（shim 读侧落 0 的装配层前提）。
    #[test]
    fn absent_default_is_none() {
        // 仅断言 API 形状与 None 语义；set/clear 往返在 fixture（进程外）
        // 与 AutoVM 级测试覆盖，避免本单测污染同进程其他测试的默认槽。
        // F-785-R1-01：写槽测试须持共享隔离锁（与其他槽写/装配/缺席
        // 断言测试同域串行）。
        let _slot = test_default_slot_lock();
        let snapshot = process_default_frame_provider();
        set_process_default_frame_provider(Arc::new(Fake {
            begin: 1,
            present: 2,
        }));
        assert!(process_default_frame_provider().is_some());
        // 还原（保 None 或原值——本测试进程内默认槽无既有注册者）。
        match snapshot {
            Some(p) => set_process_default_frame_provider(p),
            None => {
                let mut slot = PROCESS_DEFAULT_FRAME_PROVIDER
                    .write()
                    .expect("PROCESS_DEFAULT_FRAME_PROVIDER poisoned");
                *slot = None;
            }
        }
    }
}
