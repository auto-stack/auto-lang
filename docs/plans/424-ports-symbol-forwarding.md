# Plan 424: ports 符号转发——component/composable 经端口再导出

> **状态**: ✅ 执行完成(2026-08-23,worktree auto-musk;T1-T4 全落地,回归全绿——见 §6 执行记录)
> **前置**: Plan 408(已归档,component fn 合成机制);auto-musk PLAN-037(已合并,ports/ports 门控机制为本次基座)。
> **仓库**: **auto-lang**(`crates/auto-lang/src/ui_gen/vue.rs` 生成侧 + `crates/auto-man/src/vue.rs` ext 链);auto-musk 为迁移验证方。
> **目标**: ports `.at` 端口可转发 **component 与 composable 符号**——调用方经端口引用图标/逃生舱组件/组合式函数,不再在调用点直连 npm/`.vue`/`.ts`。消灭 auto-musk PLAN-037 待澄清 1(composables 域)与 icons 域的 KNOWN-DEBT,web 耦合面 100% 收拢进 `ports/*.web.at`。

---

## 6. 执行记录(2026-08-23)

- **T1 生成侧**(auto-lang 9d6b999a):`generate_fn_module_full` 对
  Component/Composable 条目发 ES re-export;`.vue`/`platform:` 源是 default
  export → `export { default as X }`,npm/.ts 源命名转发。调用方门控零
  parser 改动(named import 指向 `@/ext/<port>`,tag 注册/composable 自动
  调用不变),单测 ×2 锁定。
- **T2 canary k4-ports-forwarding**(GREEN):端口三 kind 混写,调用方经
  稳定名 `symbols.at` 引用(adapter 选择 `.web.at`);auto build + vue-tsc
  端到端绿。capability-tests README 登记。default-export 语义(.vue/
  platform: 源)为 §4 风险表之外的实发现,T2 阶段修正(`default as` 别名)。
- **T3 musk 迁移**(比 §2 草案多一格):fn-kind 同步支持转发(import 供
  wrapper 体内引用 + re-export 纯转发两行共存;`export-from` 不引入局部
  绑定,合法 ES)+ auto-man 放宽「零 fn 的 use 模块报错」为纯转发端口合法。
  musk 落地 **四端口**:icons(lucide ×38)/ renderer(markstream +
  platform:markdown + deck.vue)/ composables(useT + useI18n + gate_router
  + settings 伴生 fn)/ upload(raw_upload 的 fn+ref 常量混居——fn
  re-export 原样转发,PLAN-037 时「拆分得不偿失」的存量障碍消除)。
  34 处调用面改引 `*.at`;**调用面 use.web 非 .at 目标零命中,白名单
  8 域全清**(剩余非 .at 引用仅存在于 ports/*.web.at 内部,即设计目标形态)。
- **复审补强(合并 master 后)**:fn-kind 的 import 行改为**仅发射 wrapper
  体实际引用的符号**(先转译 wrapper 体再决定 import)——纯转发端口(零
  wrapper)不再产生 unused import,产物对 `noUnusedLocals: true` 的工程同
  样成立(此前依赖 noUnusedLocals: false 基线)。
- **T4 收口**:auto-lang 3087 单测 + auto-man 6 单测 + k2/k3/k4 canary +
  musk 三测(auto build / cargo test / vitest 2 存量失败基线)全绿。
  附带清理环境残留 `D:\nonexistent`(测试污染空目录)致 test_exists 误红。

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
