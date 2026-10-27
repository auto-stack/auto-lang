# auto-ui-compiler

> **Status**: active（M814-A 发布 @ master f6e07acee；PLAN-814 r3 合同）
> 路径：`crates/auto-ui-compiler` | 技术栈：Rust
>
> PLAN-814 T-03 交付：UI **编译/生成域**独立成包——原 auto-lang 的 UI
> 分析（aura 提取/schema）、design token 校验、action_config、lucide
> 全量表、a2ui 互操作、route 分析、widget registry、blueprint（bp）与
> **vue/jet/ark/rust 四生成器** ≈11.3 万行单源迁入；旧路径
> （`ui::style`/`aura`/`a2ui`/`ui_gen` 等）经上层再导出全保持。

## 职责与边界

- 依赖纯语言 SDK（auto-lang-core）+ auto-ui-dialect（文法单源，本包
  直接清单边）；**不反依赖**旧 auto-lang 兼容包/auto-ui-runtime。
  777 域=ui。
- 生成器源指纹：本 crate `build.rs` 扫 `src/ui_gen`（Plan 692 W-1 失效
  语义单源），经 `generator_fingerprint()` 公开（None=foreign build），
  SDK UICache 经注入消费。
- 资产随 owner：`test/a2ark`、`test/a2vue`、语料 fixtures；交付闭包
  DELIVERY_CRATES 19 包含本包（SD-08，792 driver 25/25）。
- 外部消费门：`test/fixtures/modularity/ui-package-consumer/`（真实
  Vue SFC 生成出口+指纹+style+lucide，4/4；闭包=SDK+显式文法组合零
  旧包边）。

## 显式文法装配（r3 合同 §5.3）

`assemble_ui_grammar()`（公开，幂等）：三个生成入口
（`transpile_aura`/`transpile_vue_aura`/`generate_component_from_file`）
首选显式组合；另经 `UiGrammarSource` submit 登记兜底（同上层兼容包
边界）。`ui`/`ui-headless` 空 feature：T-03 迁移代码 cfg 门归位
（pac.at ui_config 装载实臂/生成器测试门），旧包 default 链
ui-iced→ui→本包转发恢复恒开语义。

## feature 面

`ui`（UI 门面装配族）、`ui-headless`（headless 样式适配器）——均为
空门（实现依赖恒在），由旧 auto-lang feature 转发。
