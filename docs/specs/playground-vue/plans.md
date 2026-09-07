# playground-vue — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 581 | playground-notes-foundation | ✅（reviewed→archived） | archive/ | 组件三层化——新增 SnippetRunner（拼图层：autorun/内联折叠输出/单 ▶ 动作位/无后端提示）与 PlaygroundCard（卡片层：toolbar transpile·debug·live·share 四开关、exampleSelector 默认关=裁定内唯一行为变化、noteId 预留，真嵌套 Runner〔#output 插槽+defineExpose 运行面+runHandler/双栏嵌入〕）；AutoPlayground 731→31 行 @deprecated 薄包装；types.ts 契约（SnippetRunnerProps/PlaygroundCardProps/NoteMeta）；defineProps 须内联类型（vitepress compiler-sfc 限制 P581-D2）；债务 P581-D1..D4 |
| 582 | playground-notes-explorer | ✅（reviewed→archived） | archive/ | Notes Explorer 笔记站——useNotes（manifest 加载/索引/搜索）+ NotesSidebar（品牌行/title prop/ScrollArea 浮动滚动条/三级归类 Demo·书籍·测试·Parity/书籍序 tapl→…→Modern C/章节目录层 chNN stem 聚合+剥前缀）+ NotesExplorer（深链 #/notes/<id> replaceState/↑↓ Ctrl+Enter/来源 chip→GitHub）+ ExpectedOutputPanel 三态行级对照（expectedKind stdout\|result 分派——P581-D3 清偿）+ PlaygroundCard 扩展（expectedOutput tab/files 文件 tab/ideMode）；无后端降级卡+retryBackend；用户裁定后端壳单模式化（直嵌 IDE/品牌入侧栏/元信息入标题栏/紧凑化+排序）与 parity 收录；18 条用户裁定全录见计划「用户裁定变更全录」表 |
