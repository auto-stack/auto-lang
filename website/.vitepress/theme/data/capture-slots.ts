// PLAN-756: pending captures, described in docs/reports/website-v05-capture-guide.md.
export const CAPTURE_SLOTS: Record<string, { title: [string, string]; scene: [string, string] }> = {
  'SHOT-01': { title: ['AutoEdit workspace', 'AutoEdit 工作台'], scene: ['Project tree, open code tabs and an active editing area.', '项目文件树、代码标签页与活动编辑区。'] },
  'SHOT-02': { title: ['AutoEdit search', 'AutoEdit 搜索'], scene: ['A reproducible project search with matching file and text visible.', '同一示例项目的搜索词、结果与对应文件。'] },
  'SHOT-03': { title: ['AutoEdit comparison', 'AutoEdit 差异比较'], scene: ['Two versions of one file, with a readable change and comparison controls.', '同一文件的两个版本、清晰的差异块与比较操作。'] },
  'SHOT-04': { title: ['AutoMusk project workspace', 'AutoMusk 项目工作台'], scene: ['A sample project, a real conversation and its observed tool activity.', '示例项目、真实对话与实际工具活动。'] },
  'SHOT-05': { title: ['AutoMusk Plan and review', 'AutoMusk 计划与复审'], scene: ['Requirements, executed steps and their recorded verification.', '同一任务的需求、执行步骤与验证记录。'] },
  'SHOT-06': { title: ['Musk Kanban and Canvas preview', 'Musk Kanban 与 Canvas 预览'], scene: ['Capture only after the release scope and this workflow are verified.', '待发布集合与该操作流程验收确认后，展示真实任务和预览。'] },
  'SHOT-07': { title: ['JadeEdit document workspace', 'JadeEdit 文档工作台'], scene: ['A coherent knowledge base, document tabs and an edited page.', '同一知识库的目录、文档标签与编辑页面。'] },
  'SHOT-08': { title: ['JadeEdit knowledge links', 'JadeEdit 知识关系'], scene: ['Related pages, a backlink and a real search result.', '相关页面、反链与实际检索结果。'] },
  'SHOT-09': { title: ['JadeEdit draft recovery', 'JadeEdit 草稿恢复'], scene: ['An unsaved draft restored as a separate copy, without replacing the original.', '未保存草稿恢复为独立副本，保留原文档。'] },
  'SHOT-10': { title: ['Playground source and result', 'Playground 源码与结果'], scene: ['A supported snippet and its actual backend result, with the connection state visible.', '受支持示例与实际后端结果，同时显示连接状态。'] },
  'SHOT-11': { title: ['AutoUI Web view', 'AutoUI Web 端'], scene: ['The same kanban fixture and source revision as the desktop capture.', '与桌面图相同的看板数据、源码版本与状态。'] },
  'SHOT-12': { title: ['AutoUI desktop view', 'AutoUI 桌面端'], scene: ['The matching kanban fixture rendered through the native VM path.', '使用原生 VM 路径渲染同一看板数据与状态。'] },
}
