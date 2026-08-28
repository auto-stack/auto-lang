# autoui-skill

> **Status**: active
> 路径：`crates/autoui-skill`（非 Rust crate，AI 技能包）  | 技术栈：Markdown 契约 + 模板

面向 AI agent 的 **AutoUI 项目生成技能包**（SKILL.md 名为 `autoui`）：定义生成/修改
AutoUI 项目的 generator contracts（C1–C9）、patterns、reference 与 templates。
与仓库根 `.agents/skills/autoui-verifier`（双端一致性验证技能）配套使用；
不参与 cargo 构建。

## 目标与范围

- 约束 AI 生成 AutoUI 代码的形态（契约化，而非自由发挥），使产物可被 `auto gen`/
  `auto build` 确定性处理。
- 不做：验证执行（autoui-verifier 技能负责）。

## 模块清单

| 模块 | 职责 | 状态 |
|---|---|---|
| SKILL.md | 技能入口：C1–C9 generator contracts | active |
| patterns / reference / templates | 生成模式库与参考实现 | active |
