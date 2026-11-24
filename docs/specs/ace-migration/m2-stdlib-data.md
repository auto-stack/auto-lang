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

## M2-B：动态 `JsonValue`、模式驱动 `encode<T>` 与纯 Auto CSV（PLAN-1124）

`json_auto.at` 与冻结的 PLAN-975 原语模块 `stdlib/ace/std/json.at` 组合（后者字节不变），为普通 Auto AV 提供只读访问的动态 JSON 树。两种入口的公开名相同：`packages` 面经 `use std.json: ...` / `use std.csv: ...` / `use std.encoding.csv: ...` 导入；单文件面按下述规则自动嫁接。声称自己是 `std` 的包（`package std`）被 `ace.stdlib-reserved` 拒绝。

**单文件嫁接规则**：程序引用了模块的公开名、且没有自行声明该名时才嫁接。引用指把它用作值、调用目标、类型或 spec，并且在该处没有被局部绑定遮蔽；注释和字符串字面量里的名字不算引用，局部变量、参数、嵌套函数、闭包参数等局部绑定也不算。嫁接体的自由函数统一位于保留命名空间（`__std_json_` / `__std_csv_`），公开名只是转发函数，所以程序自己的 `fn parse`、`fn minify`、`fn json_cp` 始终是程序自己的，库的内部调用也不会被重绑到它们上；程序仍使用库的其他入口时，被遮蔽的公开入口不再附加。程序声明了库的记录名 `JsonValue` 时，嫁接的记录改名为 `__std_json_JsonValue`（库入口返回的就是这个类型），程序自己的 `JsonValue` 不受影响。程序名与保留名（`__std_json_*`、`__std_csv_*`、`__json_*`、`__csv_*`）相同时得到普通的重复定义诊断（`source.duplicate`），位置是程序自己的声明。`JsonEncodable` 这个名字按 spec 识别、不改名：程序里声明 `as JsonEncodable` 的记录由合成器编码，程序另行声明一个同名的记录或枚举不改变这一点。

### 1. 动态 JSON 树与格式化

- **入口**：`JsonValue`（声明遵从 `JsonEncodable`）、`parse(s str) ?JsonValue`（任何失败返回 `None`）、`parse_strict(s str) JsonValue`（失败抛可捕获的 `JsonError: ...`）、`is_valid(s str) bool`、`minify(s str) Str`、`prettify(s str) Str`。
- **对象语义**：键保持插入顺序；重复键 **后成员胜出**，位置保留首次出现处。
- **访问器**：`.kind()` / `.type_of()`（`null`、`bool`、`number`、`string`、`array`、`object`）、`.is_null()` … `.is_object()`、`.len()`、`.has_key(key)`、`.get(key) ?JsonValue`、`.get_at(i) ?JsonValue`、`.at(i) JsonValue`、`.keys() List<Str>`、`.as_array() List<JsonValue>`、`.get_str` / `.get_int` / `.get_f64` / `.get_bool`、`.as_bool` / `.as_int` / `.as_number` / `.as_f64` / `.as_string` / `.as_str`。缺键与越界的 `.get` / `.get_at` 返回 `None`，与值为 `null` 的 `Some` 区分；类型不符抛 `TypeError`；`.get_*` 缺键与 `.at` 越界抛 `JsonError`。`as_array()` 的元素按语言的列表视图规则使用（如 `remove` 后再访问）。
- **数值解析**：文本按 RFC 8259 语法严格校验后 **正确舍入** 为 `f64`（平位取偶，与宿主平台一致），最多考察 800 位有效数字，其余非零位参与舍入。超出有限范围的值抛 `JsonError: number overflow`；不足最小次正规数之半的值读为零，保留符号。纯整数字面量落在 `i32` 范围内时保留精确整型（`.as_int() -> int`）；对其他数值调用 `as_int` 抛 `TypeError`。
- **序列化文本**：精确整数输出整数字面量（含 `-2147483648`）；其他数值输出 **最短往返十进制**，读回得到同一个 `f64`：十进制指数 `k`（首位的权）在 `-6 <= k <= 20` 时用位置记法，否则用 `d.ddde<k>`（如 `1e100`、`1e-7`）。最短候选有多个时取最接近精确值者，等距时取末位为偶数者（与 ECMAScript `Number.prototype.toString` 一致）。文本里的负零——字面量 `-0`、`-0.0`、`-0e5`，以及舍入到零的负数如 `-1e-400`——输出 `-0`，因为同一个 double 只能读回为 `-0`；运行期求得的零符号不可观测，输出 `0`。`f32` 字段经 `f64` 输出，位数可能多于 `f32` 自身的最短形式。
- **`minify` / `prettify` 是文本级格式化**：先完整校验输入，再原样保留每个记号（数字文本不变）并重排空白，因此不丢任何数值。缩进格式固定：每个成员独占一行、每层缩进两个空格、`"key": value`、空容器写作 `[]` / `{}`、末尾无换行。
- **预算**（单位是 **字节**，按 UTF-8 计）：每个接收文本的入口（`parse`、`parse_strict`、`is_valid`、`minify`、`prettify`）都先检查输入是否超过 `1_048_576` 字节；节点超过 `65_536` 个、嵌套超过 `128` 层抛 `JsonError`，捕获后不留下分配。输入达到 131_071 字符时（再短的文档即使每个值只占一个字符，也装不下 65,536 个分隔符），在构建任何节点之前先数分隔符：合法文档里每个逗号和冒号恰好引入一个值。这些上限高于默认执行限额（`ExecLimits::default()`，100 万步）下能完成的解析规模，对提高限额的宿主生效。节点预算的两侧都在提高限额的用例里断言：65,537 个值被拒绝；恰好 65,536 个值时同样的文本进入解析器（接受侧的文本首字符即非法，因为完整解析 65,536 个值需要数千万步）。

### 2. 模式驱动泛型 JSON 编码（`spec JsonEncodable` / `json_encode_synth.rs`）

- **契约与推导**：`spec JsonEncodable { fn json_encode() Str }` 搭配泛型入口 `encode<T has JsonEncodable>(value T) Str` 与 `encode_pretty<T has JsonEncodable>(value T) Str`。未声明 `fn json_encode() Str` 的 `type R as JsonEncodable` 由编译器按 `TypeItem.fields` 的声明序合成编码，跨模块记录与字段重命名自动反映。
- **覆写在每个位置生效**：记录自己声明的 `fn json_encode() Str` 在顶层、嵌套字段和 `List` 元素三种位置都被调用。`List` 元素位置有一个前提：`List` 元素是视图而不是完整的值，不能整体交给方法，所以带覆写的记录只有在字段都是复制标量（整数、浮点、布尔，及其嵌套）时才能作 `List` 元素。记录拥有自有数据（文本、列表、枚举或选项、函数，直接或经嵌套记录）时，带覆写的记录不能作 `List` 元素，编译期以 `source.json-encode-unsupported` 在该 `List` 字段处具名拒绝，与覆写体读什么无关。不带覆写的记录作 `List` 元素时逐字段经视图读取，不受此限（其中含借用 `str` 字段的记录仍受语言既有的 owned-field 视图规则限制：该字段的视图只能由 `.len()` 消费，否则 `verify.view-escape`）。
- **字段域与负控**：支持 `Str` / `str`、`int` / `i32` / `i8` / `u8` / `uint` / `u32` / `i64` / `u64`、`f32` / `f64`、`bool`、`?T`（`null` 或内值）、`List<T>`、嵌套的 `JsonEncodable` 记录与空记录。未声明遵从 `JsonEncodable` 的记录被泛型边界拒绝（`source.instances-bound`）；不支持的字段类型（如 `char`）、嵌套的未遵从记录、递归环、`List<List<...>>`、`List<?T>` 在编译期被拒绝（`source.json-encode-unsupported`）。
- **`encode_pretty(v)`** 是 `encode(v)` 的缩进排版，两者只差空白。

### 3. 纯 Auto CSV（`stdlib/ace/std/csv.at`）

- **入口**：`csv_parse(text) Str`、`csv_parse_with_delimiter(text, delim) Str`、`csv_encode(data) Str`、`csv_encode_with_delimiter(data, delim) Str`（保留 PLAN-1124 之前 `stdlib/auto/encoding/csv.at` 的四个名字与参数形状，返回类型为 `Str`），以及直接操作 `List<List<Str>>` 的 `csv_parse_rows` / `csv_encode_rows`。前两组的文本形态是“字符串二维数组”的 JSON。
- **语法**：解析接受引号内的分隔符、换行与 `""` 转义，`\r\n` 与 `\n` 行尾（单独的 `\r` 也视为行尾），尾随分隔符产生空字段；空串返回 `[]`（零行）。编码用 `\n` 分隔记录，只在字段含分隔符、引号或换行时加引号。退化行（空行、只含空字段的行）不保证往返。
- **分隔符与错误**：分隔符须是单个 Unicode 标量且不是引号或换行（`;`、`\t` 与非 ASCII 单标量都可以），否则抛 `CsvError`。未闭合引号、非引号字段中的裸引号、闭引号后的杂字符抛可捕获的 `CsvError`，捕获后不留下分配。
- **预算**（单位是 **字节**，按 UTF-8 计）：`csv_parse_rows` 的文本（`csv_parse` / `csv_parse_with_delimiter` 经由它）与 `csv_encode` / `csv_encode_with_delimiter` 的 JSON 文本都先检查输入是否超过 `1_048_576` 字节。解析方向在输入达到 65_536 字符后先数分隔符（引号外的分隔符与换行；换行可以是 LF、CRLF 或单独的 CR，各算一次；末尾的换行不算），单元格超过 `65_536` 个即在构建任何单元格前抛 `CsvError`。`csv_encode_rows` 在构建输出前先数单元格，超过 `65_536` 个抛 `CsvError`。`csv_encode` 把 JSON 文本转成行时逐格检查单元格预算，对超限的大输入是先建后拒。单元格预算的两侧在提高限额的用例里断言，口径同 JSON 节点预算；拒绝侧按每一种行尾与稠密输入断言“构建前拒绝”。

## 具名边界与跨 Plan 承接（不冒已支持）

- **外部归属边界**：SQLite 宿主磁盘文件持久化/崩溃恢复归 **`PLAN-973`/`PLAN-1064`**；`packages` 跨模块 `p967_*` provider 执行装配归 **`PLAN-1043`**；schema 迁移策略归 **`PLAN-986`**；Linux 实跑 `L-01..L-05` 归 **`PLAN-1063`**（`deferred-required / NOT_RUN`，不在 Windows 跑）；`KNOWN-DEBT-AND-RISKS.md` 记录跨仓待办，旧 `stdlib/auto/encoding/csv.at` 注释占位不参与 AV 加载。
