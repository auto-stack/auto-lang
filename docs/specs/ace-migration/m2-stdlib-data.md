# ACE M2 标准库数据域（PLAN-975 / PLAN-1124）

来源：[PLAN-975](../../plans/archive/975-ace-m2-stdlib-data.md)（r10 合同，M2-A 终态交付）、[PLAN-1124](../../plans/1124-av-stdlib-json-and-csv.md)（r1 合同，M2-B 动态 JSON、`JsonEncodable` 泛型编码与纯 Auto CSV）。本文件沉淀**已验证**的 M2 数据域规范与实现边界。

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

## 数据域已验面（M2-A 全量交付，2026-11-16）

### 1. 纯 Auto JSON 编解码与结构/泛型解码（`stdlib/ace/std/json.at`）

- **Encode 族（`pub fn` 共 13 导出）**：`json_escape` / `json_int_str` / `json_digit` / `json_str` / `json_bool` / `json_null` / `json_int` / `json_array_open` / `json_array_close` / `json_object_open` / `json_object_close` / `json_kv` / `json_structurally_balanced`（字符串感知括号检查）。
- **Decode & 路径 Walker 族（`pub fn` 共 6 导出）**：
  - `json_is_valid(s str) bool`：全递归 JSON 语法验证，拒绝前导零、`1.5.5`、尾随垃圾、未转义控制字符 `< 32`、非法转义（`\q`、短/非十六进制 `\u`）。
  - `json_kind_at(s str, path str) int`：支持嵌套对象与点路径数组段（`"services.0.name"`、`"grid.1.0"`），返回 `-1`（malformed）/ `0`（missing）/ `1`（null）/ `2`（bool）/ `3`（int）/ `4`（str）/ `5`（array）/ `6`（object）/ `7`（other number）；越界/非数字段返回 `0`，截断/死端返回 `-1`。
  - `json_parse_str_ok` / `json_parse_str`（`B1` `char.to(Str)` 物化 + UTF-16 `\uD800..\uDBFF`+`\uDC00..\uDFFF` 代理对解码；孤立代理干净返回 `""` 不触发 `CharInvalid` 陷阱）/ `json_parse_int`（`i32::MIN` `-2147483648` 无溢出解析）/ `json_parse_bool`。
- **五端实跑等价（`json975_packages_coverage_av_mn_a2r` / `_a2c` / `_a2py`）**：同一跨模块源（`app::json` + `app::main`）在 **AV**、**native/mn**（`MnDriver`）、**A2R**（`rustc`）、**A2C**（`cl`，含 `a2x-c/src/lower_rt.rs` 内容视图参数转发修复）与 **A2PY**（OS Python）五端均得 `42` / `value:42`。
- **Generic `decode<T>` 与 966 CT 元数据（`json975_typed_decode_and_ct_metadata`）**：`spec JsonDecodable` + `type ServiceModel/CounterModel as JsonDecodable` + `fn decode<T has JsonDecodable>`（2 实例经 `AV_DATA_PROFILE` 得 `102`，`source.instances-bound` / 缺成员负控拒）+ `966` `ct_field_count`/`ct_has_field`（`31` 与缺 `cte.meta.fields/v1` 负控拒）+ `app::service` `pub type Service` 跨模块编解码（`8106`）。

### 2. 纯 Auto RESP2 Redis 客户端（`stdlib/ace/std/redis.at`）

- 经 `platform.net/1` TCP 与严格服务端线级交互：`redis_connect`/`redis_close`/`redis_cmd*`/`redis_bulk_*`/`redis_reply_*`/`redis_set`/`redis_get_len`/`redis_get`/`redis_incr`/`redis_del`/`redis_exists`/`redis_ping`。
- `redis_get` 按 UTF-8 规范逐字节校验首字节、续字节（`128..191`）、过长编码、代理区（`55296..57343`）与 `> 0x10FFFF` 后以 `char.to(Str)` 物化；nil/error/畸形 UTF-8（`badutf8`/`overlong`/`surr`）释放缓冲区并返回 `""` 不陷阱。

### 3. SQLite Wrapper + Pinned Provider（`stdlib/ace/std/sqlite.at`）

- Canonical `pub fn` 作者源（`sqlite_open`/`sqlite_open_memory`/`sqlite_exec`/`sqlite_query_int`/`sqlite_close`）单源同时服务 `text-record`（经 `52d2720dd` 对 `r-pub`/`public-fn` 准入，2 项 foreign 用例零修改绿）与 `packages-modules`（`5` 精确 `pub` 导出、`pub`-stripped 镜面 RED）。
- 对 `967` 内容钉版 SQLite 3.46.0 provider 执行 `:memory:` CRUD、显式 `BEGIN; INSERT; ROLLBACK;` 与 SQL 失败（`ForeignStatus`）后 `ROLLBACK`（`SUM=9`, `COUNT=2`, `callee_count=19`）、exec-before-open/double-close `ForeignStatus` 陷阱与无 `use.c` 命名编译拒绝（缺 pin 改为强 `.expect(...)` 失败，不静默跳过）。

### 4. `V-LOAD-MODULE` 边界（消费 `IF-1064-MODULE`）

- `json975_v_load_module_closure` 与 `db975_sqlite_at_packages_module_face` 验证跨模块 `json_cp` `source.module-private` 拒绝、`json.at`/`redis.at`/`sqlite.at` `pub`-stripped 镜面 RED、伪造 `std::json` `ace.stdlib-reserved` 拒绝、`PackageStore::prepare`/`restore` 篡改拒与私有函数体修改移动 `input_hash`。

## M2-B 增量交付：动态 `JsonValue`、模式驱动 `encode<T>` 与纯 Auto CSV（PLAN-1124，2026-11-23）

### 1. 动态 JSON 树与格式化（`stdlib/ace/std/json_auto.at`）

在不改写冻结的 PLAN-975 原语模块 `stdlib/ace/std/json.at` 字节的前提下，`json_auto.at` 与之组合为普通 Auto AV（单文件与 `packages` 跨模块）提供完整的动态 JSON 树：

- **扁平 LCRS 树与入口**：`JsonValue`（实现 `JsonEncodable`）、`parse(s str) ?JsonValue`（失败返回 `None`）、`parse_strict(s str) JsonValue`（严格抛 `JsonError: ...`）、`is_valid(s str) bool`、`minify(s str) Str`、`prettify(s str) Str`。
- **预算与数值精度**：显式冻结上限 `1 MiB`（`1_048_576` B）输入、`65536` 节点、`128` 层嵌套（越界抛 `JsonError` 且局部 arena 零泄漏）；纯整数字面量在 `i32` 范围内保留精确整型槽位，超出或含小数/指数解析为 `f64`；序列化时遇到非有限浮点抛出 `JsonError: non-finite float`。
- **去重与访问器**：对象 `keys()` 保持插入顺序，重复键采用 **后成员胜出（last-member-wins）** 原地覆写值且不重复键名；提供 `.kind()` / `.type_of()`、`.is_null()`..`.is_object()`、`.len()`、`.has_key(key)`、`.get(key) ?JsonValue`、`.get_at(i) ?JsonValue`、`.at(i) JsonValue`、`.get_str`/`.get_int`/`.get_f64`/`.get_bool`、`.as_bool`/`.as_int`/`.as_number`/`.as_f64`/`.as_string`/`.as_str`/`.as_array`（类型错抛 `TypeError`，缺键/越界抛 `JsonError`）。

### 2. 模式驱动泛型 JSON 编码（`spec JsonEncodable` / `json_encode_synth.rs`）

- **契约与推导**：`spec JsonEncodable { fn json_encode() Str }` 搭配泛型入口 `encode<T has JsonEncodable>(v T) Str`、`encode_pretty<T has JsonEncodable>(v T) Str`。未显式声明 `fn json_encode() Str` 的 `type R as JsonEncodable` 由编译器从 `TypeItem.fields` 声明序自动合成分段编码（含跨模块 `pub type` 导出与字段重命名自动反映）；用户显式 `fn json_encode() Str` 优先保留。
- **字段域与负控**：支持 `Str`/`str`、全宽整数（`i8`/`u8`/`i32`/`u32`/`i64`/`u64`）、`f32`/`f64`、`bool`、`?T`（`null`/内值）、`List<T>`（含 `List<Record>` 的元素视图字段投影）、嵌套 `JsonEncodable` 记录与空记录 `{}`。未实现 `JsonEncodable` 的记录被泛型边界拒绝（`source.instances-bound`）；不支持字段（`char`、嵌套未合规记录、循环引用、嵌套 `List<List<...>>`/`List<?T>`）在编译期被拒绝并诊断 `source.json-encode-unsupported`；伪造 `package std` 声明被 `ace.stdlib-reserved` 拒绝。

### 3. 纯 Auto RFC-4180 CSV（`stdlib/ace/std/csv.at`）

- **入口与兼容别名**：纯 Auto 实现（无 `use.c`/FFI），自动或经 `use std.csv` / `use std.encoding.csv` 加载；导出 `csv_parse`/`csv_parse_with_delimiter`（返回 `List<List<Str>>` 的 JSON 二维数组文本）、`csv_encode`/`csv_encode_with_delimiter`、`csv_parse_rows`/`csv_encode_rows`（直接操作 `List<List<Str>>`）。
- **语法与错误边界**：严格遵循 RFC-4180（引号字段内逗号、`""` 转义、`""` 跨行换行、`\r\n` 与 `\n` 行尾、尾随空字段）；自定义单字符分隔符（如 `;`、`"\t"`）拒绝空串、引号、换行及多字节分隔符（`CsvError`）；未闭合引号与字段内裸引号抛出可捕获 `CsvError` 且 arena 零泄漏。

## 具名边界与跨 Plan 承接（不冒已支持）

- **外部归属边界**：SQLite 宿主磁盘文件持久化/崩溃恢复归 **`PLAN-973`/`PLAN-1064`**；`packages` 跨模块 `p967_*` provider 执行装配归 **`PLAN-1043`**；schema 迁移策略归 **`PLAN-986`**；Linux 实跑 `L-01..L-05` 归 **`PLAN-1063`**（`deferred-required / NOT_RUN`，不在 Windows 跑）；`KNOWN-DEBT-AND-RISKS.md` 记录跨仓待办，旧 `stdlib/auto/encoding/csv.at` 注释占位不参与 AV 加载。
