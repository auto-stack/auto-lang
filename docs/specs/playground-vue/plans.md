# playground-vue — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 581 | playground-notes-foundation | ✅（reviewed→archived） | archive/ | 组件三层化——新增 SnippetRunner（拼图层：autorun/内联折叠输出/单 ▶ 动作位/无后端提示）与 PlaygroundCard（卡片层：toolbar transpile·debug·live·share 四开关、exampleSelector 默认关=裁定内唯一行为变化、noteId 预留，真嵌套 Runner〔#output 插槽+defineExpose 运行面+runHandler/双栏嵌入〕）；AutoPlayground 731→31 行 @deprecated 薄包装；types.ts 契约（SnippetRunnerProps/PlaygroundCardProps/NoteMeta）；defineProps 须内联类型（vitepress compiler-sfc 限制 P581-D2）；债务 P581-D1..D4 |
