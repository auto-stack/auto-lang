# vue-gen-gap fixture（PLAN-671 T-09）

vue 生成器缺口批的最小通用验证工程——**零消费方业务名**（通用性红线
AC-02 的产品面），一个工程覆盖全部七类 + 附a/附b 特性：

| 特性 | 落点（src/front/） | 验证面 |
|---|---|---|
| ① 平名内建声明层 | gap_store.at `console_log` + `Process.exit` + `Env.get` | 生成 `src/natives.d.ts`（注册表∩裸用）+ `src/lib/natives.ts` 抛错桩 + main.ts 装载；对象形态（Process/Env）走 ts_adapter `__vmOnly` 白名单改写 **+ Phase 2 防御性 `declare const`（单源五名表∩成员访问用面）+ globalThis Proxy 抛错桩**（兜底未改写裸引用位） |
| ② store 自调 | gap_store.at `.Reset` 内 `store.Bump()` | 生成 composable 内改写为裸调 `Bump()` |
| ③ bp use-fn 于 store 文件 | gap_store.at `use bps...tree_util: toggle_id` + `.Bump` 内消费 | store 文件尾内联 `function toggle_id` |
| ④ 负标量初值 | gap_store.at `var neg_idx int = -1` | `ref<number>(-1)`（668 R-23 复验） |
| ⑤ v-for 内双参 contextmenu | app.at `code_editor { oncontextmenu: .OnCtx }` + `.OnCtx(x, y)` | 签名 `(x: any, y: any)` + `@contextmenu="OnCtx($event.x, $event.y)"`（Plan 421 载荷优先于循环参） |
| ⑥ button text variant | app.at `button (variant: "text")` | 捆绑 cva 联合含 `text: ""`（chromeless parity） |
| ⑦ gen-only 工程完整 | （机制面） | `src/auto-sources.ts` 真值 + `src/vite-env.d.ts`（038 Phase B T8 复验） |
| 附a 多段插值 | app.at `text "${.store.line}:${.store.col}"` | `{{ store.line }}:{{ store.col }}` 界符完整 |
| 附b menubar 族 props | app.at menubar-item(title/icon/shortcut/enabled) + menubar-checkbox-item(checked) | strict（无 --lenient）零 S001/S002 |

## 运行

```bash
cd tests/vue-gen-gap/specs/gap
auto build --gen-only -r vue        # 无 --lenient；EXIT=0
cd gen/front/vue
pnpm install && pnpm build          # vue-tsc + vite 双绿
```

依赖：`dep bps` 指仓根 `blueprints/`（tree_util 为 canonical bp）。生成物
（`gen/`、`deps/`）已 gitignore。bps 依赖语料自带的 S001 INFO（reference
语料的既有 schema 漂移）为容忍面，不属本 fixture 验证口径。
