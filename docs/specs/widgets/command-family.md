# command 家族（command / command-input / command-empty / command-list / command-group / command-item / command-separator / command-shortcut）

来源计划：PLAN-692 W-2（2026-09-23，用户裁定：Combobox 对照 shadcn-vue 官方重实现）。

## 契约

- **vue 臂**：八元素经 `assets/shadcn-ui/command`（shadcn-vue 官方 Listbox 实现，reka-ui Listbox + 自研过滤）接入 shadcn 组件路径，import `@/components/ui/command`。行为面：输入实时过滤（含大小写不敏感 contains）、空态仅在"有搜索词且零匹配"时渲染、候选项 hover/键盘高亮（data-[highlighted]）+ click/Enter 选中（`@select`，选中即清搜索词）、分组 heading 且全隐组自动隐藏、复杂候选项经 children 槽（图标/`command_shortcut`）。
- **registry 机制（通用）**：schema 带 `vue:` 映射但未进手写 `register_defaults` 的元素，由 `apply_schema_vue_mappings` 补建 spec（overlay 只更新既有 spec 的盲区）；spec 必须同时登记 kebab 别名（DSL 标签 `command-input` 为 kebab、registry 精确键为下划线 canonical）。
- **VM 臂**：`iced: none` 维持——`size`/command 家族在 iced 侧无实现，schema 兼容零破坏；接线另立。
- **已知边界**：popover 内使用时选中后不自动关闭（需 open 状态绑定，未接线）；reka sizes 初始测量在本仓环境常命中 18px thumb 下限（预存观察项，A/B 证实与实现无关）。

## 验证

- 单测：`p692_schema_only_command_family_gets_vue_specs`（registry 八元素支持面 + 既有家族不干扰）。
- 单测：`p692_command_family_shadcn_mapping`（生成映射：Command/CommandInput/CommandEmpty/CommandList/CommandGroup/CommandItem 全落位）。
- 实机：widgets-gallery `#/combobox`（Simple 过滤/空态/点选回填 + Complex Items 分组/快捷键）与 `#/command`。
