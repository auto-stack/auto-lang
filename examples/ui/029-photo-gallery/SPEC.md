# SPEC — 029-photo-gallery（Plan 537）

> Purpose: 图库——macOS 相册风图片浏览器。侧边栏相册导航 + 搜索/排序/
> 网格密度工具栏 + 响应式缩略图网格 + 大图查看器（上一张/下一张/收藏）。
> **Frontend-only，无后端；图片源为 picsum.photos 固定 seed 网络 URL。**
> 主题：AutoOS Dark/indigo 默认；root 声明 `dark_mode` bool /
> `accent_color` str 契约变量（变量名即双端契约，006 先例），工具栏可
> 运行时切换（moon/sun icon + 五色点 coral/ocean/sage/amber/indigo）。
>
> 本文件按 T5 与 `src/front/app.at` 实际行为逐条对照定稿（2026-09-04）。

## 形态

单文件单组件（025/027/028 形态）：全部状态内聚 `src/front/app.at` 的 App
根 widget，无 routes/store 子组件/模块级 fn。网格 ↔ 查看器两视图由
`var mode str`（"grid" | "view"）全页条件切换（028 overlay 门控同款），
不走路由。

## 数据形状

### 种子平行列表（唯一真源，handler 按下标读，027/028 实证形态）

| 列表 | 类型 | 内容 |
|---|---|---|
| `p_ids` | int ×24 | 1..24 |
| `p_seeds` | str ×24 | "gal-01".."gal-24" |
| `p_titles` | str ×24 | 中文标题（晨雾山谷/夜色天桥/落日余晖/…，按相册主题命名） |
| `p_tls` | str ×24 | 预小写标题（搜索域；中文与原串相同，028 预小写平行列表同思路） |
| `p_albums` | str ×24 | nature/city/sky/abstract 各 6（轮转排布） |
| `p_dates` | str ×24 | "YYYY-MM-DD"（2026-05..08，两两互异） |
| `p_keys` | int ×24 | YYYYMMDD 整数排序键（**排序不用 str 比较**——028 规避浮点/宽度差的整数判定同思路） |
| `p_favs` | bool ×24 | 预置收藏 id 3 / 8 / 15 / 21（共 4 张） |

### handler 构建的列表

- `photos`：Init 构建的主 struct 列表（id/title/album/seed/date/fav/thumb/
  full；thumb/full 为已拼好的完整 URL，模板零拼接）。
- `view_list`：ApplyFilter 产出的过滤+排序视图列表（struct 形状同 photos；
  预置一项使 TS 推导字段类型，027 files_view 同款）。
- `view_ids`：与 view_list 平行的 int id 列表（OpenPhoto 定位 + 循环导航；
  标量列表下标读保真——028 注入形态约束同思路）。

### 状态全集

- 视图/交互：`mode`、`album`（all|favorites|nature|city|sky|abstract，默认 all）、
  `search_q`、`sort_dir`（desc 默认=最新在前 / asc）、`sort_label`（"最新 ↓"/
  "最早 ↑"）、`density`（"2"/"3"/"4" 默认 3）、`grid_class`
  （"grid grid-cols-N gap-3"，**经 `class:` prop 绑定**——见勘误②）。
- 计数/文案（handler 预拼，模板零方法调用——028 T1 盘点）：`view_label`
  （"{n} 张照片 · {相册标签}"，搜索中为 "{n} 张照片 · 搜索 "{q}" · {标签}"）、
  `cnt_all/cnt_favorites/cnt_nature/cnt_city/cnt_sky/cnt_abstract`（侧边栏
  六计数，全种子系统计，收藏数随 ToggleFav 联动）、`has_view`（空态门控，
  027 has_items 同款）。
- 查看器（元信息 handler 预取，模板不做字段链查找）：`cur_id/cur_title/
  cur_meta/cur_full/prev_id/next_id/fav_label`。
- 主题契约：`dark_mode bool = true`、`accent_color str = "indigo"`；
  `accents` 五色点列表（name + 色点类，`for a in .accents` 渲染，
  `${a.dot}` 插值进 class——028 实证）。

## 过滤 / 排序规则（ApplyFilter，被 Init/SelectAlbum/SetSearch/ToggleSort/ToggleFav 复用）

1. **相册过滤**：`all` → 全量；`favorites` → fav==true；其余 → album 相等。
2. **搜索过滤**：`q.lower()` 为空串或 `p_tls[i].contains(q.lower())`
   （子串、大小写不敏感；仅标题域，不含相册/日期）。
3. **排序**：selection sort + used 标记（028 实证习语），键 `p_keys` 整数
   比较：asc 取最小 / desc 取最大；日期互异故天然稳定。
4. **产出**：view_list + view_ids + has_view + 六计数 + view_label。

## 查看器行为

- `OpenPhoto(id)`：mode="view"；view_ids 内定位 → prev/next **循环取模**
  （首张的上一张=末张，末张的下一张=首张；**单张时 prev=next=自身**，
  即循环语义天然自恰；未做禁用态——`disabled:` 绑定无双端先例，风险规避）
  → cur_title/cur_full/cur_meta（"{相册中文标签} · {date} · 1600×1200 ·
  {seed}"）/fav_label 全部 handler 预取。
- `PrevPhoto/NextPhoto` = `OpenPhoto(prev_id/next_id)` 薄封装。
- `ToggleFav(id)`：翻转 `p_favs[i]`（状态列表元素赋值，027 实证）→
  ApplyFilter 重算（收藏计数/列表联动）→ 若查看器正显示该图则刷新
  fav_label（"❤ 已收藏"/"♡ 收藏"）。
- 布局全部常规 flex（顶栏/大图区 `bg-black/80` + contain/底栏），**不用
  absolute/fixed**（VM 定位支持面窄，019 的 absolute 角标是 vue-only）。

## 卡片点击与收藏按钮（待澄清①的落地形态）

AURA 无 stopPropagation 原语。落地：**开图点击区与收藏按钮是兄弟节点**——
卡片 col 内：图片区 button（开图）+ 信息行（标题 button 开图 + ♡/❤
button 收藏）。互不嵌套，无冒泡问题。

## 图片源与降级

- 缩略图 `https://picsum.photos/seed/gal-{NN}/400/300`；大图
  `https://picsum.photos/seed/gal-{NN}/1600/1200`。同 seed 同图、确定性、
  无 API key（004 网络头像 URL 先例）。
- VM：reqwest 阻塞下载 + Handle 缓存（renderer.rs），首开稍慢；离线时
  首字母色块兜底。Vue：浏览器原生 `<img :src>`；离线显示 alt 文本。
  **功能断言不依赖图片字节加载成功。**

## 双端差异注记

1. **`hover:` 类仅 Vue 生效**（VM 忽略）：卡片 hover 边框、侧边栏/按钮
   hover 均为增强；选中/收藏等关键状态都有静态样式表达（025 行 hover
   降级同思路）。
2. **icon (name:) 名单受 VM lucide 闭集约束**（renderer.rs `lucide_svg`
   84 项）：本例只用 `sun`/`moon`/`chevron-left`/`chevron-right`。
   计划原拟的 images/heart/mountain/building-2/cloud-sun/sparkles 在
   lucide-vue-next 存在但 **不在 VM 表**——相册图标改用 emoji 文本
   （🖼️/♡/🏔️/🏙️/☁️/✨，027 先例），返回按钮用文本 "←"。
   `pac.at icon: "image"`（VM 表内名字）。
3. **button 组件默认变体类**（h-10 / bg-primary / hover:bg-primary/90 /
   text-primary-foreground）会与业务类叠加：本例凡透明底/自定义高按钮
   显式 `bg-transparent`/`hover:bg-transparent`/`text-foreground`/
   `h-auto|h-6` 压制（cn/tailwind-merge 后者优先）。
4. **动态网格密度用语义 grid 元素三静态臂**：VM 侧 `grid` 元素的
   `cols:` 状态绑定不解析（回落 1 列）、`class:` 状态绑定不消费、CSS
   `grid-cols-N` 类只对语义 grid 的 class 仲裁生效——故密度 2/3/4 以
   `if .density == N { grid { cols: N … } }` 三臂静态展开（028 静态
   `cols: 4` 同构造），双端一致（T7 实测 2/4 列截图在案）。
   Vue 曾用 `class: .grid_class`（`:class` 绑定）工作，为双端统一改
   语义 grid；`density` 相应由 str 改 **int**（SetDensity(int)）。
5. **fit 语义**：缩略 `cover`（h-40 容器裁切）、查看器 `contain`
   （完整显示）——schema 单一定义，两端语义一致（首个 fit 双端应用）。

## 执行勘误（相对计划文本，语义不变）

1. 状态名 `view` → **`view_list`**：`view` 是语言关键字（view 块），
   `.view = …` 解析报错（Expected term, got DotView）。
2. 密度网格演进为**语义 grid 元素三静态臂**（见差异 4）；`grid_class`
   状态删除、`SetDensity(str)` → `SetDensity(int)`。
3. 相册图标 emoji 化 + icon 名单收缩（差异 2）。
4. 新增 `view_ids`/`has_view`/`accents` 三个计划未点名状态：分别为
   OpenPhoto 定位（规避 handler 读注入 Obj 数组的失效面）、空态门控
   （模板不可调 .len()）、五色点渲染参数化；均为已实证形态。

## 运行

- Vue 独立：`auto run`（front_port 4029）
- VM 独立：`auto run -r vm`
- 桌面宿主：`cargo run -p auto-lang --features ui-iced --example
  ui_desktop -- --fullscreen --apps-dir examples/ui`（零登记收录）
