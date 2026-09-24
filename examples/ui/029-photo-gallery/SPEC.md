# SPEC — 029-photo-gallery（Plan 628 → PLAN-043 Part 1 后端化）

> Purpose: 图库——现代移动/平板风照片浏览器。顶部沉浸式分段导航栏 + 搜索/排序/
> 网格密度工具栏 + 无边框照片流网格 + 悬浮胶囊大图查看器（上一张/下一张/收藏）。
>
> **数据源（PLAN-043 Part 1，2026-09-23 起）**：后端 `photo_service` 能力臂
> 实时扫描——pac `photo_root:` 声明根目录（现 `C:\Users\zhaop\Pictures`），
> 生成后端原生供给三路端点：
> - `GET /api/photos/scan`——递归索引（jpg/jpeg/png/webp/gif/bmp），条目含
>   token id（blake3，绝对路径不出后端）/标题/相册（rel_dir 首段，根文件归
>   "photos"）/尺寸（头部解析）/大小/mtime 日期/sort_key，url 字段发绝对地址；
> - `GET /api/photos/thumb/:id?w=N`——按需渲染缩略图（解码 → EXIF 朝向规范化
>   → 等比缩放 → JPEG），磁盘缓存 `%TEMP%/autoos-photo-thumbs/`（mtime+宽度
>   参与缓存键，改图自动失效）；
> - `GET /api/photos/full/:id`——原图字节（Content-Type 按扩展名）。
>
> 前端 Init 拉 scan 建网格；缩略图与原图全走 HTTP——浏览器（CORS-any）与
> VM native（image widget 只认 http/data/builtin）双端通吃，相对/本地路径
> 一律不用。目录增删后重启 app 即跟随。
>
> Plan 628 的离线烘焙链（prepare_gallery.py 扫一遍目录 → 缩略图 +
> gallery_data.json → generate_at.py 硬编码进 app.at）**已删除**——目录
> 变化图库不跟随是产品级缺陷（PLAN-043 Part 1 根因）。
>
> 主题：AutoOS Dark/indigo 默认；root 声明 `dark_mode` bool /
> `accent_color` str 契约变量（变量名即双端契约），工具栏可运行时切换。
>
> 收藏：id 落 app Storage 键 `photo-gallery.favs`（CSV），ToggleFav 翻标 +
> 重建 CSV + 重写盘，重开保留。

## 形态

单文件单组件（025/027/028 形态）：全部状态内聚 `src/front/app.at` 的 App
根 widget，无 routes/store 子组件/模块级 fn。网格 ↔ 查看器两视图由
`var mode str`（"grid" | "view"）全页条件切换，不走路由。取消旧版 macOS
三段式生硬侧边栏，采用现代移动/平板级 Edge-to-edge 沉浸式相册布局。

后端半身：最小 `src/back/api.at`（`GET /api/gallery/status` 控制面——后
端进程因它而存在，photo 三路由由生成器无条件发射，020-music-player
同款形态）。pac 声明 `api: "rust"` / `back_port: 4429` / `photo_root:`。

## 数据形状

`photos` 主列表元素（Init 由 scan 响应构建）：

```
{ id: str        // blake3 token（收藏/查看器定位键）
  title: str     // 文件名去扩展名
  album: str     // "photos"（根文件）| 子目录名（如 "Screenshots"）
  date: str      // "YYYY-MM-DD"（mtime）
  meta: str      // "宽×高 · 大小 · 日期"（尺寸解析失败时省宽高段）
  fav: bool      // 收藏标（fav_ids 初始化时命中即真）
  thumb: str     // 绝对缩略图 URL
  full: str      // 绝对原图 URL
  sort_key: int  // mtime epoch 秒（排序键）
}
```

## 行为契约

- **Init**：`Storage.get("photo-gallery.favs")` 恢复收藏 →
  `Http.get_json("/api/photos/scan")` 建 `photos` → `ApplyFilter()`。
  scan 失败（后端未起）→ 空列表 + `load_error` 文案（空态三态之一）；
  `root_missing: true` → 空态「照片源目录不存在」。
- **动态相册分组**：`ApplyFilter` 每次按 `photos` 首现序重建
  `album_tabs = [全部, <各 album>, 收藏]`（计数同重建）。"photos" 标签
  📷 图片、"Screenshots" 标签 📱 截图，其余 album 用目录原名。
- **过滤**：相册键相等 || all || favorites(fav 标)；搜索子串命中
  `title.lower()`。
- **排序**：`sort_key`（mtime）选择序，desc 最新在前 / asc 最早在前。
- **查看器**：`OpenPhoto(id)` 在 `view_ids`（当前过滤排序序）定位，前后
  循环环绕；`cur_*` 字段从 `photos` 反查填充。
- **收藏**：`ToggleFav(id)` 翻 `photos[i].fav` → 重建 `fav_ids` +
  CSV → `Storage.set` → `ApplyFilter`；查看器态同步 `fav_label`。

## 已知边界

- HEIC/RAW 不入索引（image crate 无解码器），listed-but-broken 比缺席更糟。
- 相册标签固定为 全部/各目录/收藏 分段——两级以上嵌套目录只按首段归组。
- scan 索引为后端进程内一次性（OnceLock 语义）——目录变化后重启 app
  即跟随（不做目录监视推送）。
