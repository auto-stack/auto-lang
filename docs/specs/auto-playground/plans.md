# auto-playground — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 582 | playground-notes-explorer | ✅（reviewed→archived） | archive/ | /api/examples 读 notes.json 单一事实源（serde 映射 schema 兼容，三路回退目录扫描；entry files[0] 回退保 api=manifest 不变量）；frontend 壳单模式化——左侧品牌+NotesSidebar 导航直选载入、右侧直嵌 AutoPlaygroundFull（noteMeta 入标题栏/ExampleSelector 删/Load Replay 隐藏/排序 Run→Trans→Debug→Share）；files-only 物化运行（prepare_files_temp_dir 嵌套写盘+根 entry main.at 供 source_dirs 解析——parity 多文件笔记可 Run，base64·Decode 10 ok 实证）；backend 宿主 e2e 19 条（含 debug/replay/bytecode-meta 族）|
