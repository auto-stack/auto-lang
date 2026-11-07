# auto-vm / vm-codegen-hazard-catalog（PLAN-995）

状态：delivered（随 PLAN-995 交付）· 日期：2026-11-08
来源：READER-009 round23 观察目录的实证复核（双 pin 对照：master b90c17dd1 与 009 t01 冻结工具链）。
消费方：一切在 VM 上执行大文本/动态集合密集 .at 代码的宿主（auto-reader、auto-ui 服务）。

## hazard 目录（六类定谳）

| 类 | 触发形态（round23/995 探针） | 定谳 | 永久 oracle |
|---|---|---|---|
| H1 动态集合二遍迭代 | 同一 `json.parse` 文档 books 两遍迭代/迭代后读 prefs/groups（千条量级） | **双 pin 不复现**（master+009 冻结件全绿：n1=n2=1000、prefs 迭代后完好）——ws-perf01 时代遗留结论 | `vm::tests_json_hazards::h1_*`（千条二遍+嵌套二遍） |
| H2 大文本 `==` 误判 | 216KB 级 distinct 串在 if/表达式/交换/inline-trim 各位置比较 | **证伪/归因修正**：误否族根因=探针源码缺 `]` 闭合（bisect 实证缺 1 字节、fd=216478）；VM 比较各位置均正确 | `h2_big_string_eq_positions_and_inline_trim`（四位置+负控） |
| H3 大串累积跨模块布局 | 同形重建函数在 min/extra-fn 两种模块布局 | **证伪**：两布局输出逐字节同（sha 一致）；真假分歧由 H2 同一源缺陷解释 | `h3_accumulation_layout_independence` |
| H4 模块 var 大串存取 | 216KB 模块 var 赋值→跨请求比较 | **基线安全**（双 pin） | `h4_module_var_large_string_store_and_compare` |
| H5 `json.stringify` 少参 ABI | `.at` 1/2 参调用（shim 固定弹 3 参） | **已修复**（PLAN-995：`.at` 小写拼写补位，1/2/3 参输出一致） | `h5_stringify_arity_abi` |
| H6 `Str.substr/slice` 跨块丢字 | start 字节落多字节字符中段时步进读 | **已修复**（PLAN-995：start-clamp 改回退式，步进覆盖无损） | `h4_*`（4096 步进重建==原文）+ 100 项既有 string 测试 |

## 规则（合同）

1. **比较语义**：VM 字符串 `==`/`!=` 在任意表达式位置（if 条件/let 表达式/交换操作数/含内嵌方法调用）语义一致，与文本规模无关（≤216KB 实证；更大规模无封界，按本合同推定不设阈值）。
2. **迭代语义**：`json.parse` 物化文档的任意集合可重复迭代、迭代次序不影响其它集合/字段读取。
3. **模块布局无关性**：函数行为不因同模块内其它函数的存在/顺序而变。
4. **`Str.substr(start, len)` / `Str.slice(start, end)`**：字节索引 + 双向回退式字符边界 clamp——结果为 `s[boundary↓(start)..boundary↓(end)]`；分块步进读重建无损（H6 修复后合同）。`Str.len` 为**字节**长度（PLAN-872 钉死），`.length`/`GET_ELEM` 为字符语义——单位混用是调用方责任，本合同不改变。
5. **`json.stringify(v[, replacer[, space]])`**：JS 变参语义——缺参按 undefined（compact 输出）；`.at` 与 JS 轨拼写等价（H5 修复后合同）。

## 检测与豁免流程

- 新增 hazard 候选：service 级 .at 探针（plan-artifacts scope）→ 双 pin 对照复现 → 受控矩阵拆变量（比较位置×拼段形状×规模）→ 定谳（修复/证伪/封界）。
- **修复类**入上表并加永久 Rust oracle（`crates/auto-lang-core/src/vm/tests_json_hazards.rs`）；**封界类**须在本文档登记禁用模式+检测报告形态（当前无封界类）。
- 复核既有 hazard 主张时**必须检查探针自身源码**（H2/H3 教训：缺闭合 `]` 的探针产生了两个伪 hazard 并误导归因——双变量实验不得同时改变拼段闭合与比较位置）。

## retire 记录

- H1（二遍迭代空段）：round23 目录主张在 round24 pin 与 master 均不复现，retire。
- H2/H3（inline-trim 误判/跨模块布局敏感）：归因修正为探针源缺陷，retire。
