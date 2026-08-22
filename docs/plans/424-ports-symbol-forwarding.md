# Plan 424: ports 符号转发——component/composable 经端口再导出

> **状态**: 🟢 立项待执行(draft)
> **前置**: Plan 408(已归档,component fn 合成机制);auto-musk PLAN-037(已合并,ports/ports 门控机制为本次基座)。
> **仓库**: **auto-lang**(`crates/auto-lang/src/ui_gen/vue.rs` 生成侧 + `crates/auto-man/src/vue.rs` ext 链);auto-musk 为迁移验证方。
> **目标**: ports `.at` 端口可转发 **component 与 composable 符号**——调用方经端口引用图标/逃生舱组件/组合式函数,不再在调用点直连 npm/`.vue`/`.ts`。消灭 auto-musk PLAN-037 待澄清 1(composables 域)与 icons 域的 KNOWN-DEBT,web 耦合面 100% 收拢进 `ports/*.web.at`。

---

## 0. 背景与现状(PLAN-037 终态)

PLAN-037 Phase 5 落地了 fn 端口转发(`ports/platform.web.at`:use.web 绑定 + wrapper fn),
但两类符号**无法经端口转发**,被迫留调用点(当时实测结论):

1. **component**(lucide-vue-next ×14 / markstream-vue / platform:markdown / deck.vue):
   组件需要 view 树内实例化 tag,fn 模块无 re-export 机制。
2. **composable**(useT ×6 / vue-i18n ×4 / gate_router / relay_command_runner):
   `use.web composable` 在**调用方** SFC 生成 `import + const t = useT()`(自动调用 + refs 解构);
   fn wrapper 无法复制该 setup 语义(丢反应性)。

关键洞察(立项依据):**ES re-export 天然解决两个问题**——端口生成的 TS 模块对
component/composable 条目发 `export { X } from '<specifier>'`,调用方照常
`use.web [kind] X from "ports/xxx.at"`:
- component:调用方 SFC 的 import 指向 `@/ext/.../ports/xxx`,re-export 链把真实组件递过去,
  tag 注册机制不变;
- composable:调用方的自动调用代码不变,只是 import 来源变为端口 re-export——
  `const t = useT()` 里的 useT 经 re-export 拿到真身,反应性完整保留。

## 1. MVP

1. `generate_fn_module_full`(vue.rs)对 kind=Component/Composable 条目发
   `export { a, b } from 'specifier'`(specifier 复用 `ext_import_specifier`;
   npm 原样、platform: → `@/platform/x.vue`、本地 → `@/ext/...`)。
2. 端口内混写三种 kind 合法;fn 条目走现有 wrapper 路径。
3. auto-man ext 链:传递复制(`expand_at_module_web_imports`)已覆盖本地文件;
   确认 npm 条目零复制(本就无需)。
4. 调用方门控:对 `use.web component/composable X from "<.at>"` 的解析与直接引用
   完全同路径(仅 specifier 指向端口)——预期零 parser 改动,验证即可。

## 2. auto-musk 迁移(验证面)

1. `ports/icons.web.at`:`use.web component MessageSquare, ListTodo, Scroll, BookOpen, ... from "lucide-vue-next"` 等全部图标条目;调用方 14 处改引端口。
2. `ports/renderer.web.at`:markstream-vue、platform:markdown、deck.vue。
3. `ports/composables.web.at`:useT、useI18n(refs)、gate_router。
4. 收口断言:调用面 `use.web` 非 `.at` 目标**零命中**(PLAN-037 白名单清零);
   auto build / cargo test / vitest 基线全绿。

## 3. 测试设计

- auto-lang 单测:fn 模块三种 kind 混写 → TS 产物含 import(fn)与 export(component/composable)各行;
  canary(k3 或新 k4)放一个端口转发的 component + composable,端到端 vue-tsc。
- 回归:musk 全量三测;`resolve_at_adapter` 门控交互(.web.at 端口 + re-export 组合)。

## 4. 风险

| 风险 | 等级 | 对策 |
|---|---|---|
| composable refs/call_args 经 re-export 后 caller 侧语义漂移 | 🟡 | refs 是 caller 侧机制不受影响;MVP 用 vue-i18n locale 实测 |
| 组件 tag 注册对 .at 来源的 specifier 解析差异 | 🟡 | 现有 ext_components 走同一 ext_import_specifier,预期直通;k3 预验证 |
| platform: 前缀 re-export 的模块解析(vue-tsc 需显式 .vue) | 🟢 | ext_import_specifier 已处理(@/platform/x.vue) |

## 5. 执行步骤(草案,执行时细化)

1. T1 生成侧:generate_fn_module_full 发 re-export + 单测。
2. T2 canary:k4-ports-forwarding(component + composable 转发端到端)。
3. T3 musk icons/renderer/composables 三域迁移 + 断言清零。
4. T4 收口:全量回归 + PLAN-037 待澄清 1 关闭登记。
