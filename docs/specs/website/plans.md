# website — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 581 | playground-notes-foundation | ✅（reviewed→archived） | archive/ | Notes manifest 管线——scripts/build-playground-notes.mjs 四源确定性采集（vm-golden 460 全带期望输出〔.expected.out 优先/.expected.result 终值回退〕/aavm 158/books 裸 ```auto 围栏 634〔EN 源〕/demo 28 含 4 项目；parity 后置 582）→ public/playground-data/notes.json（gitignore、0.82MB 单文件、byte-identical 幂等）；prepare-content 末段接线三态生成 + --check 计数断言；website build 预存红 core.md 裸 `<prefix>`（P581-D1 上游）；债务 P581-D1..D4 |
| 582 | playground-notes-explorer | ✅（reviewed→archived） | archive/ | /playground（EN/ZH）换 Notes Explorer（52+5 组可浏览，无后端降级卡可浏览态完整）；旧 /playground/ SPA 退役 meta-refresh 重定向；prepare-content 裸尖括号通用转义器（P581-D1 预存红清偿，build 红转绿）；AutoFence 书页围栏 ▶ Run（autorun/收起还原/import 锁+笔记站链接，markdown fence 钩子）；playground-notes e2e 三断言 + playwright.config（webServer preview）；parity 语料入 manifest（51 条五家族）；债务 P582-D1..D4 |
