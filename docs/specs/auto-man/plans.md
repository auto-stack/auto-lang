# auto-man — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 437 | 024-charts-official-library | ✅（archived） | archive/ | vue 全项目生成接入 components/ 组件包目录（组件 SFC 落 src/components/<Widget>.vue，与页文件相对 `from "../components"` 的 pages/ 候选解析）；collect_ext_import_files 排除 Package 引用（避免 escapes-root 误杀）；KNOWN：内置脚手架管线对脚手架内部 import 的传递依赖盲视（chart 族需 @vueuse/core/reka-ui/cva/button 手工补，见 plan-437 债务 D6，归属 PLAN-457） |
| 442 | cross-platform-closure（auto-man 侧） | ✅（reviewed→archived） | archive/ | vue 依赖按使用发射（P0-1：VueDependencyUsage 标记检测〔App.vue+全组件 SFC 语料,ui/button 标记带结尾引号防 button-group 误配〕驱动 OPTIONAL_DEPS 按组发射；CodeEditor.vue 壳 usage 感知同步；sync 路径 package_json_deps_drifted 双向漂移检测；npm_deps 去重）+ CodeEditor 模板 setSearchEffect→setSearchQuery 修复（P0-2：该 API 在 @codemirror/search@6 不存在，fresh checkout pnpm build 必炸根因）；musk 侧复核 fresh build 依赖零命中+deps-guard TRANSITIONAL 清零；债务 P442-1..5;P442-1..6 台账 |
