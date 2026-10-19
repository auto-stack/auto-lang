# 书籍示例执行描述

> PLAN-813：TAPL 书页与 Playground Notes 共享的 Auto 示例执行合同。

## 作者数据与生成

- 单一作者清单是书籍仓的 `book/tapl/examples.json`；网站构建将它复制到生成书页数据与 `website/public/tapl-examples.json`。生成目录不作为第二作者源。
- 每条记录以 `id` 唯一定位执行入口，`pair_id` 关联中英译本；`page`、`line`、`display` 与 `source_sha256` 将记录绑定到作者页中的具体代码块和实际内容。目录审计拒绝孤立、重复、缺失或哈希不匹配的绑定。
- `source_file` 可指向完整的可执行 Auto 文件；`entry` 指定入口；`files` 保存同一项目的其他相对路径和源文本；`input` 与 `timeout_secs` 为请求级运行参数。网站编辑 entry 时以编辑内容覆盖对应文件，其余文件仍来自描述。

## 模式与预期

- `run` 调用 `/api/run`，当前预期种类包括精确 `stdout` 和 `stdout_regex`。
- `test` 调用 `/api/test`，检查确实执行的断言及逐项结果。
- `display-only` 不显示 Run，必须提供具体 `display_only_reason`；典型例子是 TOML 包清单，而非 Auto 源码。
- 执行模式不是目标语言转译承诺。书页上的 Rust/Python/C/TypeScript 展示块只有在明确标注为转译产物时才代表生成结果；手写的语义等价实现必须说明其状态及当前转译器边界。TAPL 第 8 章保留可运行改写，并在中英文页面解释与较完整目标语言表达形式的差异。

## 网站和审计

- AutoFence 和 CodeView 消费同一执行描述；编辑、输入、多文件、超时、后端错误、期望结果和测试报告都应在书页入口可见。请求使用同源 Playground API，不把读者浏览器指向作者机器的 localhost。
- `scripts/check-tapl-examples.py` 对整个目录校验唯一绑定、作者源哈希、预期结果与实际 API 运行；负控制必须证明错误 stdout 期望会被拒绝。`website/tests/tapl-run.spec.ts` 覆盖 Run、输入、多文件、Test 和 display-only 行为。
- 2026-10-19 基线为 401 条：377 run、14 test、10 display-only；执行审计 401 通过，负控制拒绝错误预期。书稿、Playground/VM 依赖或测试配置变化后，须重新执行受影响的目录审计/运行并记录对应 revision。
