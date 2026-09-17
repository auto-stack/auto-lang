# SPEC — 027-file-manager（Plan 440 立；PLAN-016 重写）

AutoOS 桌面文件管理器（Finder / Explorer 双栏形态）。桌面事实轨 = VM
（`auto run -r vm` / 桌面 in-process 装载）；vue 轨（`auto run`）为前端调试轨。

> PLAN-016（2026-09）现代化：mock 平行数组退役 → 真实文件系统；emoji 图标 →
> lucide（按扩展名字面量分支）；固定坐标 popover → alert-dialog + 锚定
> popover + toast()；zinc 硬编码 → 语义 token + dark_mode（桌面 SetTheme 回写
> 链即时换肤）。本节取代 Plan 440 原描述。
>
> PLAN-023（2026-09）增量：网格图片缩略图（auto.image.thumb → Plan 547 媒体
> 管线）+ 地址栏可伸缩坍缩（§1.5/§2.5）。

---

## 1. 数据层（真实文件系统，T-05/T-06）

- 主目录：`Env.get("USERPROFILE")` → 回落 `HOME`；快捷访问 = 主目录 +
  Desktop/Documents/Downloads/Pictures/Music（`file.exists` + `file.is_dir`
  门控显隐；回收站为非目标，不设）。
- 列表物化：`fs.read_dir`（JSON 文件名数组，`for-in + ""+name` 字符串物化
  —— kanban 先例路径）→ 逐条目直连 native 三件套 `file.is_dir` /
  `file.size` / `fs.mtime`（PLAN-016 新 native，epoch 秒，失败 = -1）。
  **禁止 metadata JSON 字段级读取**——JsonValue 可选字段在 VM 轨产出 None
  级联 TypeError（evidence/016/d3-vue-fs.md 实证）。
- 容量：单目录 cap 500 条（防宿主进程卡顿），超出 `item_count_str` 标注
  "已截断"。
- 排序：目录恒先；marks 选择排序（025 sys_store 同款）；name/date 字符串
  比较（date 为 "YYYY-MM-DD HH:MM" 字典序 = 时间序），size 为 int 键。
- 隐藏项：dot-prefix（`hidden_name`）；Windows 隐藏属性 stdlib 不可达。
- 错误态：canonical/is_dir 门控 + toast 反馈；`fs.read_dir` 失败中止
  handler（状态变更全部置于列目录之后，失败保留原视图）。

## 1.5 地址栏与面包屑（PLAN-023）

- 胶囊占满：面包屑容器 `flex-1 min-w-0`，平时占满导航钮簇与右侧操作区
  （搜索框起）之间全部可用宽；`overflow-hidden` 兜底裁。**顶栏禁用
  `justify-between`**——iced Row SpaceBetween 会把 Fill 子件降级为内容宽
  （实证胶囊恒 ~495px），伸缩一律由 `flex-1` + 对侧 `shrink-0` 承担。
- 深路径坍缩：全链段数 > 5 且未展开 → 首 1 段 + `...` + 末 2 段（视图三面
  crumbs_head / crumb_gap / crumbs_tail——规避循环内条件节点纵向堆叠债，
  R4-2 实证形态）；`...` 点击 = `CrumbsExpand` 就地展开全链（不导航），
  任何导航（NavTo 入口）重置回坍缩态。全链恒存 `crumbs`。
- 段名截断：crumb 按钮 `max-w-[10rem] truncate`——vue 轨真省略号；VM 轨
  裁切无 "…" 字形（renderer truncate = 单行 + clip，框架既有口径）。

## 2. 主题与图标（T-01/T-02）

- 全视图语义 token（bg-background/bg-card/border-border/text-foreground/
  text-muted-foreground/bg-accent/bg-primary/bg-destructive 族）。
- `var dark_mode bool`：宿主 SetTheme 执行臂对声明该变量的 app 回写
  （renderer execute_set_theme），vue 轨生成器据此绑根 dark class。
- 图标：`icon (name:)` 元素 + FileIcon 组件（components/file_icon.at，按
  扩展名字面量分支——vue 轨 icon 动态名 Circle 占位规避，TreeIcon 范式）。

## 2.5 网格缩略图（PLAN-023）

- `auto.image.thumb(path, size) -> str`（stdlib 新原语）：图片文件排队
  方形 rendition，返回媒体 URI（"" = 不支持/失败）；解码走 Plan 547 媒体
  管线 worker 池 **Thumbnail 优先档**（最低优先、`MediaPin::None` 可驱逐），
  主线程零解码。
- 接线：物化循环内 `is_image_ext(ext)`（jpg/jpeg/png/webp，与管线解码器
  白名单同款——gif/bmp/svg/ico 管线不收，照旧 FileIcon）且 `kept < 120`
  （THUMB_CAP）时排队，URI 存行字段 `thumb_src`。
- 渲染：grid 卡 `thumb_src` 非空 → `image_surface (fit: "cover")` 入
  h-20 圆角容器；空 → FileIcon 原样。URI 未就绪本帧渲染为空，随 250ms
  Tick 渐进浮现；解码后 per-asset Handle 缓存稳定不闪（P547）。
- vue 轨：`image.*` 走 ts_adapter VM-only 白名单 `__vmOnly` 降级（Plan 444
  形态），thumb_src 恒空 → FileIcon 回落（与既有 fs.* 桩同口径）。
- 驱逐卫生：thumb 原语 queue 后立即 release 配平引用——条目 30s 宽限后
  可驱逐，decoded LRU 保 URI 渲染直至预算压力（重导航自愈）。
- Windows Shell 缩略图（IShellItemImageFactory/ thumbcache 复用）为后续
  可选优化臂，不在本计划（PLAN-023 §0 裁决记录）。

## 3. 弹层（T-03/T-04）

- 新建/重命名/删除确认 = alert-dialog（025-sys-monitor 形态，state 驱动
  open）。
- 操作反馈 = toast()/toast.success()/toast.error()（Plan 412 __toast 管线，
  renderer 窗口级堆叠悬浮层）。
- 右键菜单 = 逐行锚定 popover（shell dock 菜单范式：open 按 `ctx_id ==
  item.id` 匹配，placement bottom-end/bottom-start；oncontextmenu.prevent
  触发）。

## 4. 文件操作（T-06；D-4 口径）

新建文件夹 `file.create_dir`；新建文件 `file.write_text(p,"")`；重命名
`fs.rename`（同目录）；复制 `fs.copy_recursive`（`copy` 为 .at 关键字不可
点调）；剪切 = `fs.rename` 跨目录；删除：文件 `file.delete` / 空目录
`file.remove_dir`，非空目录拒绝（toast；递归删除待 stdlib 提案）。
写操作前 `file.exists` 重名门控；所有路径来自真实解析（canonical 后）。

## 5. 桌面互操作（T-07/T-08/T-09；协议 v1.7）

- `__desktop_cmd` 总线（shell 同款状态面）写 `open_with	<app-id>	<path>`；
  v1.7 起 pump 对全部注册表窗联合排空（原仅特权四窗）。
- 关联解析：fs_util.app_for_ext（静态默认表，与 041/031 pac `opens:` 声明
  同源）；无关联 toast.error（"打开方式"选择器待注册表 opens 投影面）。
- 宿主执行臂：未知 app / opens 未声明扩展名拒绝（toast）；未运行 launch 后
  write_state `auto_open_path`、已运行聚焦 + 写入；目标 App Tick 消费
  （041 ConsumeOpen、031 SettleTick 臂）。

## 6. 测试

- tests/desktop_mcp.py（VM 模式）：真实 FS 断言套件 + tests/testdata 副本
  （破坏性操作只对副本；地址栏 submit 跳转）。
- open_with 端到端 = 桌面宿主级（acceptance bus 注入），证据
  auto-os docs/plans/evidence/016/t07-open-with-e2e.png。
- 已知残留：MCP 服务器线程偶发静默失联（框架级，mock 时代同机制）；
  Tick 延迟引导避开 iced boot 临界区竞态（原 Init 内同步 fs 调用偶发挂起）。
