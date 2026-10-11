# v0.5 六个月文案校正：L0 复核

2026-10-11 用户明确要求“5 个月应该改成 6 个月”。本次标题改为“六个月，三大步” / “Six Months, Three Major Steps”，时间范围同步为 2026 年 4 月至 10 月；对应既有测试期望与当前 UI 规范同步。

review: pass。基面 5ab336c96a73191e33a59b0cf1b70f4ca936818f；已提交源码 af021ed7a6bcf2f8f907e9520cf80bcc099a62e8；构建的 website tree 7534870417758cf789229b31ea41e9e03b57aa71。独立复核步骤重新检查实际三文件差异及既有四个场景断言：双语标题、正文日期、固定章节/目录顺序、历程先于实图、原生 hash 点击和手机浮层不推正文。所有断言保留，只有预期标题更正。构建成功 201.08s，390/1440×EN/ZH 四场景通过 8.8s；git diff --check 无空白错误。无 Rust 改动，Category A 不跑 Cargo/docs_gen；无新增遗漏、延期或 workaround。复核与实现同会话，以原始日志、提交、规范补丁和实际页面截图重建结论，不声称另派 agent。

证据目录 v05-six-months-correction/ 保存原始构建/检查 gzip、SHA、中文桌面章节实图和单行规范冻结差异。r3 的既有截图/冻结报告保留历史身份；本次是其标题规范的获准后继校正。待投影描述的 ui-presentation 输出 hash 及 additional_title_correction_review 同步本记录，实际账本未修改，先前 P758-D1 收尾状态不变。
