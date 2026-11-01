# UI/Man 结构收口跨仓边界（PLAN-014）

> 状态：active（2026-11-01 交付；auto-ui master d66627a / auto-man master 1eeac78）。
> 来源计划：auto-ui `docs/plans/archived/014-ui-man-structure-closure.md`。

## 交付与消费边界

- **auto-ui**：compiler `ui_gen/vue/`（11家族）、runtime `aura_view_builder/`
  （12家族）与 `iced/renderer/`（7家族）内部结构收口；公开路径/公开面
  逐名恢复，行为零改。三面规范：
  [structure-closure](../../../../auto-ui/docs/specs/ui/structure-closure.md)、
  [structure-test-map](../../../../auto-ui/docs/specs/ui/structure-test-map.md)。
- **auto-man**：CLI `crates/auto/src` 拆分 cli_spec/diag/cmd_*（解析/诊断/编排
  四分）；主入口保纯分派。规范：
  [CLI structure-closure](../../../../auto-man/docs/specs/cli/structure-closure.md)。
- **语言仓**：零 auto-ui/auto-man 代码边（890/900 交付保持）；本文件与本节
  project.md 指针为唯一回写。
- **消费固定owner**：Man CLI 经 `tool_ui_ops`（I-TOOL）+ `auto_tool_client`
  消费 `auto-ui-tool` 进程；daemon detach 唯一走 `auto_tool_execution`；
  禁止第二工具发现/owned进程设施。

## 接受与指纹语义

- 接受 890/900/901/018 交付组合为其正式边界；901 的 generator 指纹失效
  语义随 ui_gen 布局变化如实生效（缓存身份变化→失效重建，不伪造旧指纹）。
- Man 对 auto-lang master 的 901 死 feature 漂移（with-file-history 别名）
  已随 PLAN-014 T-03 修复（ supersede fix-cli-stale-feat 的不完整版：
  auto-core 侧别名同时摘除）。

## UI-007 接续（部分冻结）

014 零主题/官方系列/整棵 examples 归位（examples/ui 于 master 仍为空）；
主题/示例归 007。接续候选：①Aura tracked三元组（path/id_map/probe）收敛为
单一 ctx 结构（29签名/6子文件+convert_element调用点）；②CLI 诊断出口统一
收编（30处内联 ai-mode 闭包）；③gen 产物涂改自愈（工具合同增强）。
