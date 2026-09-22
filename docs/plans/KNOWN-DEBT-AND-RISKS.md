### P683（2026-09-22，Plan 683 rq-remote-renderer 方案 2 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P683-D1 | low | remote wire v2 | **mesh/canvas 原语 v2 词汇面外**（G2 词汇表未列；tiny_skia `layer.primitives` 位图降级承接未实施）——试点三例无 canvas 面，首个 canvas 应用上 remote 时需 Path 序列化或位图降级（T-01 勘定登记） | headless.rs lower_layers_v2 观测臂；设计档 rq-remote-renderer §4 |
| P683-D2 | low | remote 降格 | **Transform 旋转/剪切矩阵元不可达**——iced `Transformation` 未公开矩阵元（仅 scale_factor/translation），v2 降格只产轴对齐 [s,0,0,s,tx,ty]；旋转内容（canvas 旋转动画等）remote 面暂缺 | headless.rs lower_text_v2 近似臂观测 |
| P683-D3 | low | remote 图像 | **svg 图像（tiny_skia Image::Vector）降格省略**——v1/v2 词汇 src 均为位图通道；svg 组件经 lucide 栅格化外的直引形态需 src 词汇扩展或宿主侧 svg 栅格化 | headless.rs lower_image_v2 Vector 臂观测 |
| P683-D4 | low | daemon 重放 | **shadow blur 无 canvas 原语**——v2 wire 携带 ShadowSpec 全参，daemon 重放为偏移半透明垫层（无 blur 扩散；alpha 按有无 blur 稀释）——观感近似，004 卡面 shadow-sm 实测可接受 | broker_surface.rs paint_shadow_scrim |
| P683-D5 | low | remote 环境注记 | **daemon 窗最小化环境怪**——本机他方窗口争焦会把 rqhost daemon 窗最小化（resized 0x0 观测行；SW_RESTORE 即复，渲染管线无恙）；走查/巡检脚本需带恢复臂。与 master 在案 taskbar 环境红族同环境 | T-04 走查记录（plan 683） |
| P683-D6 | low | remote IME | **Windows 中文 IME 组合态实机验证未执行**（review r1 F-2 登记）——wire 面已接 input_method::Event 三态映射（Preedit/Commit/Closed）+ASCII 键入端到端绿（单测+实机），但中文 IME 组合/候选窗定位（preedit cursor 矩形）无实机用例；残缺则按计划待澄清③混合方案（text_input 类交互保留 daemon 侧原生组件）兜底 | headless.rs input_to_iced_events IME 三臂；plan 683 待澄清③ |
### P684（2026-09-22，Plan 684 gallery 示例修复批 work 执行登记）

| id | 级别 | 领域 | 内容 | 锚点 |
|---|---|---|---|---|
| P684-D1 | low | 027 back API | **vue 轨写操作无后端端点**——rev7 只做了读面（fs_list/fs_home/fs_drives）；重命名/粘贴/新建/删除在 vue 轨统一守卫提示「写操作仅在 VM 轨可用」。后端加写端点（fs_rename/fs_mkdir/fs_delete/fs_copy 契约+impl）即可让 gallery 内嵌全功能，形态沿用 thin 契约纪律 | examples/ui/027-file-manager/src/back/api.at；src/front/app.at 写守卫四处 |
| P684-D2 | low | 027 媒体面 | **vue 轨图片缩略图缺失**——image.thumb 为 VM 轨媒体管线专属（256px rendition 渐进浮现），vue 轨条目 thumb_src 恒 ""，网格视图回落 FileIcon。补齐需后端缩略图端点或 media 路由跨轨化 | examples/ui/027-file-manager/src/front/app.at NavTo vue 分支 thumb_src:"" |
