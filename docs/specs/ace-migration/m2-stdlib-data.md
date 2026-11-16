# ACE M2 标准库数据域（PLAN-975）

来源：[PLAN-975](../../plans/975-ace-m2-stdlib-data.md)（r10 合同）。本文件沉淀**已验证**的知识；
未交面保持具名缺口，不以规划代现状。整包终态归档时（T-06）补齐数据域全量面。

## B1 原语切片：`char.to(Str)` 物化（T-B1-D 沉淀，2026-11-16）

**语义**。`c.to(Str)` 把一个 char 标量物化为**全新 owned 单标量 Str**（UTF-8 1..=4 字节）。
这是 decode/parse 子串物化核心决定（用户裁定 B1，2026-11-16）的落地原语：slice view→owned
促升（clone/to/concat）先前已存在，本原语补齐唯一缺口（动态码点）。

**类型与所有权**。接收者必须是 char 值；computed scalar 无可借 buffer，故 `owned-only`
规则拒绝把物化结果再借出为视图存储。结果是独立 owned buffer：跨函数返回、与源变更隔离
均验证；分配走各端预算（AV arena `owned_alloc_precheck`+字节 fuel；A2R `try_reserve`+
`rt_alloc`；A2PY charge；AC reserve+malloc 门）。

**错误契约**。非法标量（代理区 D800..DFFF、> U+10FFFF、负值）在**typed gate**（
`char.from_codepoint`）即拒绝——AV `TrapKind::CharInvalid`（`text.invalid-scalar`）、
A2PY `trap:InvalidScalar`、A2C `A2X_TRAP_INVALID_SCALAR`；到达物化的 char 值恒为合法标量。
NUL（U+0000）是普通 1 字节标量：可物化、可读回、**非终止符**（"a"+NUL+"b" 保 3 字节）。

**六路径支持矩阵**（同源程序、独立 oracle：`'A'`1B/`U+4E2D`3B/`U+1F600`4B → lens 134、
拼接等于 `"A中😀"` → 1347；NUL 组合 → 31）：

| 路径 | 落地点 | 证据 |
|---|---|---|
| AV（解释器） | HIR `TextFromChar` → MIR `StrFromChar`(0x4C) → 执行器 arena 物化 | json975_char_to_str_primitive / b1_slice |
| native/mn | native lower `ac_rt_own_from_char`（text-runtime 真实现）＋fingerprint `fromchar` 渲染 | 四端测试 mn 腿 |
| AV/C | 同 text-runtime C 面 | 同上 |
| A2R（rustc exe） | a2x-rust `a2x_str_from_char`（lower+lower_coll） | 四端 1347 / FAULT_ALLOC 反例 |
| A2PY（OS Python） | `a2x_str_from_char`（chr(ic).encode） | char975 两用例 |
| AC（cl exe） | a2x-c `a2x_str_from_char`（手写 UTF-8 编码器，rt_runtime/coll_runtime） | 四端 1347 |

a2x-core 两道 gate（semantic_view 计算面/指纹序列化 `from_char(...)`）先期拒绝该 expr 的
缝已闭合。**绑定证明（突变负控，全 RED 后回退）**：删 a2x-rust lower 臂 → 命名
`a2x.unsupported-expr` 拒绝；错 runtime 字节 → oracle 1110≠1347；删 native fingerprint 臂 →
E0004 non-exhaustive 编译红（穷举 match 防御）。fingerprint 文本变更触发 cache-miss 重建（非
staleness）——设计如此。

**owner 反例**。A2R exe 以 `A2X_RT_FAULT_ALLOC=1` 运行：首个 owned 分配恰为物化 →
`trap:Resource`（与 945 clone-fault 同约定）——物化是真实分配，非视图包装。

## 数据域已验面速览（详见 Plan 975 §9 收据）

- `stdlib/ace/std/json.at`：encode 族＋decode/parse 族（点路径含**数组段**
  `"services.0.name"`＝legacy `JsonValue.get_at` 面映射；越界/非数字段＝missing(0)、
  截断/结构死端＝malformed(-1)）——console 内联 lane。
- `stdlib/ace/std/redis.at`：RESP2 over platform.net/1；`redis_get` Str 物化（UTF-8
  标量逐个再物化＝多字节 byte-exact 穿透；nil/error/畸形→""）。
- `stdlib/ace/std/sqlite.at`：967 内容钉版 SQLite 3.46.0 provider 的 Auto 包装
  （`:memory:` 腿零 973 依赖；文件腿 973 分账）。

## 具名缺口（不冒已支持）

- **泛型 typed decode（`decode[T]` 结构感知形）**：需 966 comptime 线；record-typed 组合
  decode 受 tracked 模块跨 lane 拼写缝阻塞——2026-11-16 用户裁定归 **973 io 装配线**统一
  裁定；r10 V-LOAD-MODULE（canonical MODULE 面）具名阻塞 **1064**（drafting）。
- sqlite 文件持久化/迁移面＝973；Linux 实跑＝1063；schema 迁移策略＝986。
