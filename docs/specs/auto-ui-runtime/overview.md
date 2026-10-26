# auto-ui-runtime overview

UI运行、renderer与宿主域由auto-ui-runtime单源承载，与语言SDK及UI编译域分离。
职责、接口和测试载体规则以[项目规范](project.md)为准；实施与复审来源见[计划索引](plans.md)。

PLAN-841：VM/Iced/runtime 使用 UI owned 声明与受检路径，导入失败在 VM 构造前传播；Core 不再承载 UI 兼容臂。 见 [project.md](project.md)。
