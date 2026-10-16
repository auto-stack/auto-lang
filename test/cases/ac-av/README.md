# PLAN-798 AutoDown sources/oracles for the ac-av probe (T-01)

这些 `.case.ad` 用例从 T-01 起进入仓库，作为本 profile 的正式 AutoDown 源与
oracle。**execution pending**：`ac-av-core-run` recipe 与 worker 绑定属 T-07，
须等 D-796（796 合入）与 D-788（788 合入）后由已合入的统一 probe/run 接缝
正式选择/执行（计划 §5.6/并行合同）。在依赖满足前，这些用例由 Rust 定向测试
（`experimental/ac-av-probe/probe/tests/**`）以同一输入/同一 oracle 实际执行；
不得以本文件的存在宣称正式 runner 通过。

## suite 说明

- recipe（拟，T-07 冻结）：`ac-av-core-run` —— 仅消费声明式 Atom 参数或已有
  source 引用；worker 不读取 expected 决定结果，父 runner 校验 actual 来源/
  工具身份并比较 oracle。
- oracle 形态：`exit`（AV Success 值 / 诊断态）与 `trace`（mark 事件序列），
  均为独立预期，非两后端互证。

```test-case
case {
  schema: "autolang.case/1"
  id: "av.core.vertical.two-fn"
  tags: ["p798", "ac-av", "vertical"]
  links: ["PLAN-798"]
  source: { file: "av/core/vertical_two_fn.at", syntax: "auto", path: "main.at" }
  entry: { mode: "program" }
  checks: [
    { id: "av", adapter: "ac-av-core-run", action: "run",
      expect: { completion: "success", exit: { code: 0 } } }
  ]
  status: { skip: { reason: "execution pending T-07 (D-796/D-788); Rust-directed coverage active", issue: "PLAN-798-T07", owner: "PLAN-798" } }
}
```

```test-case
case {
  schema: "autolang.case/1"
  id: "av.core.semantics.shortcircuit-latent-trap"
  tags: ["p798", "ac-av", "semantics", "short-circuit"]
  links: ["PLAN-798"]
  source: { file: "av/core/shortcircuit_latent_trap.at", syntax: "auto", path: "main.at" }
  entry: { mode: "program" }
  checks: [
    { id: "av", adapter: "ac-av-core-run", action: "run",
      expect: { completion: "success", exit: { code: 0 } } }
  ]
  status: { skip: { reason: "execution pending T-07 (D-796/D-788); Rust-directed coverage active", issue: "PLAN-798-T07", owner: "PLAN-798" } }
}
```

```test-case
case {
  schema: "autolang.case/1"
  id: "av.core.semantics.trap-rem-overflow"
  tags: ["p798", "ac-av", "semantics", "trap"]
  links: ["PLAN-798"]
  source: { file: "av/core/trap_rem_overflow.at", syntax: "auto", path: "main.at" }
  entry: { mode: "program" }
  checks: [
    { id: "av", adapter: "ac-av-core-run", action: "run",
      expect: { completion: { stage: "run", message: { contains: "av.trap.rem-overflow" } } } }
  ]
  status: { skip: { reason: "execution pending T-07 (D-796/D-788); Rust-directed coverage active", issue: "PLAN-798-T07", owner: "PLAN-798" } }
}
```

```test-case
case {
  schema: "autolang.case/1"
  id: "av.core.semantics.trace-order"
  tags: ["p798", "ac-av", "semantics", "trace"]
  links: ["PLAN-798"]
  source: { file: "av/core/trace_order.at", syntax: "auto", path: "main.at" }
  entry: { mode: "program" }
  checks: [
    { id: "av", adapter: "ac-av-core-run", action: "run",
      expect: { completion: "success", exit: { code: 1 }, trace: { events: "a,b,a", compare: "list" } } }
  ]
  status: { skip: { reason: "execution pending T-07 (D-796/D-788); Rust-directed coverage active", issue: "PLAN-798-T07", owner: "PLAN-798" } }
}
```

```test-case
case {
  schema: "autolang.case/1"
  id: "av.core.resources.front-door-nesting"
  tags: ["p798", "ac-av", "resources", "negative"]
  links: ["PLAN-798"]
  source: { file: "av/resources/nesting_over_budget.at", syntax: "auto", path: "main.at" }
  entry: { mode: "program" }
  checks: [
    { id: "av", adapter: "ac-av-core-run", action: "run",
      expect: { completion: { stage: "admission", message: { contains: "av.resource.frontend-nesting" } } } }
  ]
  status: { skip: { reason: "execution pending T-07 (D-796/D-788); Rust-directed coverage active", issue: "PLAN-798-T07", owner: "PLAN-798" } }
}
```
