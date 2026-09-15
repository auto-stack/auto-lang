# a2r-std

> **Status**: active
> 路径：`crates/a2r-std`  | 技术栈：Rust（serde_json / ureq / rusqlite / redis）

a2r（Auto→Rust）转译产物的运行时标准库：让转译出的 Rust 代码有与 Auto 语义对齐的 std 实现。

## 目标与范围

- 提供转译代码依赖的运行时类型与函数：list、hashmap、string_builder、str、json、http、fs、env、math、time、sqlite、redis。
- 行为对齐 AutoVM 后端的标准库语义（parity 检查的对照对象之一）。
- 不做：不实现转译器本身（在 auto-lang）；只覆盖转译产物实际用到的 std 子集。

## 模块清单

| 模块 | 职责 | 状态 |
|---|---|---|
| list / hashmap / string_builder | 集合与字符串构建 | active |
| str | 字符串函数 | active |
| json | JSON 读写（serde_json） | active |
| http | HTTP 客户端（ureq） | active |
| fs / env / math / time | 文件系统、环境、数学、时间 | active |
| sqlite | 嵌入式 SQL 数据库（rusqlite 0.30 bundled，Plan 415-B1） | active |
| redis | Redis 客户端（redis 0.27 纯 Rust 同步 API，Plan 415-B2） | active |

## sqlite 模块约定（Plan 415-B1）

- Auto 面三文件配对：`stdlib/auto/sqlite.at`（接口层，编译期类型检查用）+
  `sqlite.rs.at`（#[rs] 示意层，手抄源）+ `crates/a2r-std/src/sqlite.rs`（真实现）。
- 类型名 `SqliteDb`（刻意非 `Db`，避免转译器全局名映射劫持用户同名结构体）；
  RAII 关闭，无显式 close。
- 哨兵错误约定（与 fs/http 一致）：不 panic——`open` 失败回退内存库并记
  `last_error()`；`exec` 失败返 -1（批脚本/行返回语句经 `execute_batch` 回退
  报 0）；`query` 失败返空表；所有单元格字符串化（NULL→""、BLOB→"<N bytes>"）。
- VM 侧无 native 实现：VM 程序不得调用（接口注释已声明）；cookbook stub
  真实化留待 VM 侧落地。
- 3 段式 `auto.sqlite.*` 调用受 Plan 223 预存死臂限制（`auto.env.get` 同病，
  KNOWN-DEBT 415-B1 条目），Auto 侧统一用 2 段式 `sqlite.open(...)`。

## redis 模块约定（Plan 415-B2）

- 同 sqlite 三文件配对形态；类型名 `RedisClient`；RAII 关闭、无自动重连。
- `open` 失败保留无连接态（后续操作全部返回失败哨兵并记 `last_error()`）；
  连接操作取 `&mut self`——Auto 侧绑定必须用 `var`（发射为 `let mut`）。
- 活体契约测试挂 `AUTO_TEST_REDIS_URL` 守卫（CI 不依赖运行中的 Redis）；
  无服务器失败哨兵测试无条件执行。
- **发射器碰撞名守卫模式**：模块方法名与既有 match 臂碰撞（如 `get`/`set`
  撞 List 索引化/Map::insert 改写）时，类型守卫必须**嵌入既有臂体顶部**，
  不得新建前置臂（match 首个命中臂胜出，前置臂会遮蔽既有逻辑——
  `12_specs/008_arc_dyn_spec` 回归实证）。无碰撞名（如 ping/del/exists/
  exec/query）可仿 SqliteDb 用独立守卫臂。

## 手抄副本签名比对（KNOWN-DEBT 396 半收偿）

`crates/auto-lang/src/tests/a2r_std_signature_parity.rs`：逐对比较
`stdlib/auto/*.rs.at` 与 `crates/a2r-std/src/*.rs` 的 `pub fn` 名称集与元数
（json 分派改名表 / list 型块隐式 self 归一化；遗留漂移入显式允许清单）。
类型级比对（int↔i32/i64 类映射不可机械反转）与生成路线仍属 396 原始债面。
