# Demo介绍与实图素材契约

> PLAN-723 · 2026-10-01 · VitePress静态介绍

## 稳定集合与来源

沿application-demos.md的28项候选建立稳定id与slug，覆盖auto-lang示例、auto-os系统应用、auto-term与auto-os-config。数量描述这批网站介绍材料，不等于桌面注册项、所有系统app或同等产品完成度。不得因拍摄困难悄悄删项、合并019/030视频、将022冒充auto-kanban产品，或将AutoTerm冒充AutoShell。

目录内容：website/.vitepress/theme/data/demos.json；公开图片：website/public/apps/demos/<slug>.png；双语概览：website/{zh/,}apps.md嵌入DemoCatalog.vue。脚本generate-demo-pages.mjs生成58个旧路由兼容跳转，--check要求目标与JSON一致。六分类只筛选本地状态，SSR初始显示全部28项。

## 真图资格与内容边界

每项至少一张真实应用渲染图，记录所用VM/Vue/native形态。不修改图像内容，不用概念图/假UI/接口mock响应代替实拍；加载中、空白容器、断图或阻塞服务错误不算合格主图。内置数据是实际应用运行的数据，可展示但必须标明：天气非实时、聊天非微信网络/模型连通、数据库示例含事务模拟、独立syslog三条演示记录非宿主故障。

媒体使用真实文件和实际扫描/流/缩略图服务；拍摄可采用公开或专用测试资源。照片库拍摄使用三张已公开AutoOS截图文件，播放器真实读取WebM测试片段，音乐扫描真实WAV测试音。文件管理器浏览隔离测试目录；监视器采样拍摄机器，绝不暗示访问者机器信息。PNG发布字节应与原始捕获/复用文件一致。

既有真图可复用，保留运行形态、证据版本和日期。不能确定拍摄时间时记captureDate:null及真实reusedAt/来源依据，不能把导入日当拍摄日。AutoTerm留存Rust原生验证图记录2026-09-22；当前本地VM空画面拒收并登记诊断。再拍摄独立于本次静态介绍验收，不能将旧图认作当前双端全部能力通过。

## 可追踪记录

docs/reports/p723-capture-catalog.json冻结稳定ID、来源仓/路径/采样HEAD、具体源文件hash、截图原路径/发布路径、SHA-256、尺寸、运行形态、拍摄/复用日期、数据性质、真实交互动作与视觉检查。二进制身份用hash记录；未证明构建来源commit时明确不确定，不能仅用采样HEAD替代构建证据。临时日志、原始试拍/接触表在ignored .p723-runtime，不作为清理后唯一证据。

## 网站行为与验证

目录桌面3列/平板2列/手机1列；有名称的分类按钮aria-pressed，结果计数aria-live；图片按钮用EvidenceImage，键盘Enter开启，Esc/关闭回焦，查看原图是图片链接。概览单H1，分类/条目层级正确、locale互链、正确源码入口；就地details含操作/条件/状态/截图版本；原详情路由跳转概览稳定锚点并展开。v0.5不运行应用/桌面后台，不嵌iframe；v0.5.1在线体验与UI Playground按后续设计独立交付。

验收检查28个唯一ID/slug、原图字节hash/尺寸、双语内容、概览28项完整SSR/图片资源、locale、58兼容跳转、直接锚点与筛选后深链、就地展开、六类筛选/放大回焦、五宽度×深浅×中英（含全部条目展开）无溢出，以及原产品指南与OS/v05展示回归。Category A不运行Cargo tests/docs_gen；实拍注明实际端，不等于28项跨端一致性证明。
