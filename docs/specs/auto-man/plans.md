# auto-man — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 437 | 024-charts-official-library | ✅（archived） | archive/ | vue 全项目生成接入 components/ 组件包目录（组件 SFC 落 src/components/<Widget>.vue，与页文件相对 `from "../components"` 的 pages/ 候选解析）；collect_ext_import_files 排除 Package 引用（避免 escapes-root 误杀）；KNOWN：内置脚手架管线对脚手架内部 import 的传递依赖盲视（chart 族需 @vueuse/core/reka-ui/cva/button 手工补，见 plan-437 债务 D6，归属 PLAN-457） |
| 442 | cross-platform-closure（auto-man 侧） | ✅（reviewed→archived） | archive/ | vue 依赖按使用发射（P0-1：VueDependencyUsage 标记检测〔App.vue+全组件 SFC 语料,ui/button 标记带结尾引号防 button-group 误配〕驱动 OPTIONAL_DEPS 按组发射；CodeEditor.vue 壳 usage 感知同步；sync 路径 package_json_deps_drifted 双向漂移检测；npm_deps 去重）+ CodeEditor 模板 setSearchEffect→setSearchQuery 修复（P0-2：该 API 在 @codemirror/search@6 不存在，fresh checkout pnpm build 必炸根因）；musk 侧复核 fresh build 依赖零命中+deps-guard TRANSITIONAL 清零；债务 P442-1..5;P442-1..6 台账 |
| 552 | desktop-app-curation（auto-man 侧） | ✅（reviewed→archived） | archive/ | 画廊生成器随探针清退：02-components 分类臂摘除 042 前缀；画廊收录面 43→35（generate_gallery_host 仍扫 examples/ui，探针目录迁出后自动不再收录，画廊不再出现裸 id 条目）；P552-1..6 台账 |
| 553 | pixel-paint（auto-man 侧） | ✅（reviewed→archived） | archive/ | 画廊 generate_gallery_host 分类 if 链 03-apps 臂 +031 前缀（031-paint 收录画廊但不上双列错类——024-040 空洞回填的画廊分类随行件）；详档见 auto-lang ui/plans.md 553 行 |
| 559 | vue-dualend-embed-debts（auto-man 侧） | ✅（reviewed→archived） | archive/ | desktop-host api-client 守卫放开（粘合可安装时：gen 生成粘合或项目 src/back/api.ts 择先，run 内先到先得+每次覆写）+desktop_extra_app_roots 兄弟探测（../auto-os-config/auto，id=os-config 对齐 vm extra_roots_from）+api_gen install_project_api_glue（契约抽取零端点时项目 TS 粘合装入 gen lib/api.ts+dist）+Taskbar ⚙️ 按钮资产与宿主 launchSettings 聚焦-或-启动；详档见 auto-lang ui/plans.md 559 行 |
