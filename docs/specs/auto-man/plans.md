# auto-man — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 437 | 024-charts-official-library | ✅（archived） | archive/ | vue 全项目生成接入 components/ 组件包目录（组件 SFC 落 src/components/<Widget>.vue，与页文件相对 `from "../components"` 的 pages/ 候选解析）；collect_ext_import_files 排除 Package 引用（避免 escapes-root 误杀）；KNOWN：内置脚手架管线对脚手架内部 import 的传递依赖盲视（chart 族需 @vueuse/core/reka-ui/cva/button 手工补，见 plan-437 债务 D6，归属 PLAN-457） |
