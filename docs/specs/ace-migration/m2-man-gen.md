# ace-migration: m2-man-gen — AutoMan 配置/任务模板与生成器迁移（PLAN-982）

> Status: **B-CONFIG-I（配置文档求值引擎迁到 ACE）已交付**；A 切片（route C 门面）仍是
> 历史对照。**完整产品迁移**（TEMPLATE/RUST/C/WEBTARGET/PY/REVERSE/EXPORT/DEVSDK、安装/
> 升级/退役）**未交付**（M2-B）。
> 来源：PLAN-982 §5（T-01 台账 E1..E8）、§7（AC-CFG-*）、§9（work/review/merge 收据，
> 含 B-CONFIG-R 独立复审两轮）。
> 实现：auto-lang `experimental/ace/crates/{core,av-core,lowering}`、`crates/auto-lang-core`
> （`ace_config`/`ace_entry`/`config`）、`crates/auto-lang-ace-host`（配置桥）；auto-man
> `crates/auto-man-runtime/src/config_engine.rs` 与四个 pac 读点。
> 本文件只写**已验现值行为与具名边界**；未列入的能力不视为已支持。

## 1. 配置求值：引擎与选择（B-CONFIG-I）

配置文档（`pac.at`）的求值有**两个具名引擎**，选择规则与观测都在
`auto_man_runtime::config_engine`：

| 面 | 现值行为 | 来源 |
|---|---|---|
| 显式覆盖 | 进程内 `set_choice_for_test`（测试） → 环境变量 `AUTO_CONFIG_ENGINE=av\|ace` / `legacy\|vm` | `config_engine::choice` |
| 默认 | 桥已装 ⇒ **ACE**；无桥构建 ⇒ legacy，且 stderr **一次性可观测提示**（不静默） | 同上 |
| 未知取值 | 忽略（回落到"桥是否已装"），绝不静默改选 | 同上 |
| 显式 ACE 但无桥 | **走 ACE 并具名失败**（`ace.host/executor-unregistered`），不回退 | 同上（CLI 实测 exit 1） |
| 观测 | `observation()` 给**真实派发计数**（`ace_dispatches`/`legacy_dispatches`）+ 当前 `engine`；候选回执写 `config.engine`/`config.legacy_dispatch` | `config_engine`, `ace_candidate::write_receipt` |
| 读点 | `automan.rs`（探针读 + 正式读）、`pac.rs`（dep pac 读）、`ace_candidate.rs`（候选配置腿）、`auto config` 子命令 | 四读点 + `main.rs` |

**ACE 侧求值管线**（`auto-lang-ace-host::config_document::eval_request`）：
`compile_config_document`（文档行→单顺序入口的受检 `auto.config.*` 调用）→
`lower_checked_owned(entry=d___cfg_document)` → `verify_mir` → `encode_abc` /
`verify_abc_bytes` → `open_session` → AV 执行 → `close_session` 取 **owned 快照** →
投影成中立 `ConfigDocument` → `ConfigIdentity::of(engine="av")`。预算逐轴映射
（值节点/字节进会话，steps/wall 进 AV；wall 另覆盖解析→导出全段），**所有路径**关会话，
失败不回退 legacy。

**语言/宿主面已验**：`#` 行注释、JSON 形整份文档（字符串键/逗号/尾逗号）、块形对象值、
模板 `${…}`（字符串位置插值；节点头占位按 legacy 留字面）、`var` 重声明=赋值、分支/
节点体块作用域、注入 args 作**可变全局**并**胜出**同名的文档默认 pair（被覆盖的 RHS
**不执行**）、字面数组 `for` 展开（运行期列表仍具名 pending）、**≤1976 层嵌套整链投影**（2000 层在 MIR 校验由冻结的 16 MiB `VerifyMemory` 预算具名拒绝，不崩）。

## 2. 已验接口（可依赖）

| 面 | 接口/事实 | 来源 |
|---|---|---|
| 显式选择（执行腿） | `AUTO_ACE_CANDIDATE=1\|true\|yes`；默认完全走既有路径 | `ace_candidate::selected()` |
| 显式选择（配置腿） | `AUTO_CONFIG_ENGINE=av\|ace / legacy\|vm`（见 §1）；`ace_executor_installed()` 是可观测的环境事实 | `config_engine`, `ace_entry` |
| 构建闸门 | Cargo 特性 `ace-candidate`（**不进** `default`/`core`）；桥依赖 `auto-lang-ace-host` 只在 `cfg(windows)` 声明 | `crates/auto/Cargo.toml` |
| SDK 合同 | `AceExecutor::eval_config`（默认实现＝具名 `ace.config/executor-unsupported`）、`eval_config_with_ace`、`AutoConfig::from_{code,file}_with_engine(.., AceAv\|LegacyVm)` | `auto_lang_core::{ace_entry, ace_config, config}` |
| 中立模型 | `ConfigDocument`/`ConfigNode`/`ConfigValue`（含 `Node`）、`ConfigIdentity`（source hash + **结构化 context 值**摘要 + engine）、`ConfigCost`（steps/nodes/bytes/depth/`callee_count`） | `ace_config` |
| 具名终态 | `Diagnostic{stage,code,message,span}`、`Denied{code}`（`ace.config/inject-unsupported|inject-range`）、`ResourceDenied{kind}`（`nodes`/`bytes`/`wall`/…）、`Cancelled`、`HostFailure{code}`（`ace.host/executor-unregistered`/`ace.config/*`） | 桥 |
| 回执（测试钩子） | `AUTO_ACE_PROOF_RECEIPT=<path>` JSON：成功回执带 `schema`、`sdk`、`same_sdk`、`config{owner,path,present,values,deps,targets,engine,legacy_dispatch}`、`execution{…}`、`script`；**拒绝回执**是 `config{owner,present,error}` + `refusal{code,message}`（没有 values/deps/targets/engine） | `ace_candidate::write_receipt` |
| 平台 | 桥 Windows-only；执行腿在非 Windows **具名拒绝**（`ace.host/unsupported-platform`）；配置腿在非 Windows 默认落 legacy（可观测），只有**显式** `AUTO_CONFIG_ENGINE=av` 才具名拒绝（`ace.host/executor-unregistered`） | `ace_candidate`, `config_engine` |

## 3. 永久验证面

**语言侧**（`D:/autostack/.wt/lang-982/auto-lang`）：
- `experimental/ace/crates/core/tests/cases/plan982_config_document.rs` — 前门/profile/
  三形 dep/嵌套/坏邻例/单入口物化/`#`/JSON/块对象/模板 desugar/`for` 展开/空 root（11 例）。
- `experimental/ace/crates/lowering/tests/cases/plan982_config_document_e2e.rs` — 文档→单入口
  MIR→ABC 准入→宿主会话执行→owned 快照（4 例）。
- `crates/auto-lang-ace-host/tests/p982_config_engine.rs` — 真桥**六段**（成功 / SDK 消费面 / 诊断 / 值预算 / context 注入 / 无桥），含 `ConfigContext.args` 注入与根属性播种。
- `crates/auto-lang-ace-host/tests/p982_vcfg_cases.rs` — V-CFG 语言侧：SYNTAX/FLOW/CONTEXT/
  VALUE/FAIL/LIMIT/IDENTITY + 深度双档（8 例；1000 层整链服务为永久断言，2000 层只要求不崩）。
- `crates/auto-lang-ace-host/tests/p982_pac_corpus_probe.rs` — 真实 pac 语料与 legacy 逐字
  比对（`diff.is_empty()` + `ok>=140` 断言，当前 145 OK / 0 分歧）。

**产品侧**（`D:/autostack/.wt/lang-982/auto-man`）：
- `crates/auto/tests/plan982_ace_candidate_config.rs` — A 切片门面见证（5 例，含
  `config.engine`/`legacy_dispatch`、坏配置/坏求值/**运行期 trap** 三轴零写）。
- `crates/auto/tests/plan982_ace_config_b.rs` — V-CFG-PRODUCT/RETIRE（2 例：engine=av 且
  零 legacy 派发；显式 legacy 可见可计数、未知取值不降级）。
- `cargo tc`（auto-man 固定组，7 包 lib）＝258 passed / 4 skipped（交付时）。

**语料口径（round 113/114）**：真实语料 301 份（含 `.wt` 镜像），探针对**模板族按产品口径注入 `name`**后：**296 与 legacy 逐字一致、2 份 ACE 接受而 legacy 拒绝（`clib` 模板的两份拷贝）——**这是"更宽松"而非"更强"**：ACE 接受 `lib ${name} {}` 却不做**节点位插值**，生成物因此**读不回**（`plan982_ace_clib_template`：值位替换 / 节点位保留 `${name}` / 回读具名失败）
**模板节点位插值（2026-11-18 用户裁定 ①B）**：默认**不插值**（`lib ${name} {}` 的 id/实参保留字面 `${name}`，与 legacy 逐字一致，`plan982_ace_clib_template` 第一例钉住）。设 `AUTO_ACE_TEMPLATE_NODE_INTERP=1|true|yes|ace` 后节点位改走 `__fstr(expr)`：根**标量属性**已登记为局部，因此 `name: "probe"` 后 `lib ${name} {}` **真的插值**，`clib` 模板在开关下生成 `lib clibproj {_arg0: "clibproj"}` 且**可读回**（同文件第二例）。**边界（round 127 起已解除一半）**：**嵌套体内的标量属性也已登记为局部**（用全文档去重集 `aliased`，因为节点体是内联进同一语句列表、没有独立块作用域）；注入名不重复别名（`emit_injected` 已发射 `let`），嵌套同名键仍各自保留（`p982_vcfg_context_injection_axes` 钉住）。
**宽标量（`f64`/`u64`，2026-11-18 用户指示 ④ 交付）**：默认**不接受**浮点/宽整数字面量（`ratio: 1.5` ⇒ 具名 `source.unsupported`，与 legacy 表面一致）。设 `AUTO_ACE_NUMBER_SURFACE=1|true|yes|ace` 后：`ratio: 1.5` ⇒ `ConfigValue::F64(1.5f64.to_bits())`（**位精确**，`ConfigValue::F64` 存 bits），`big: 18446744073709551615` ⇒ `ConfigValue::U64(u64::MAX)`。实现口径：CST 投影器把"**恰好一个** affix-num 叶"的值升成 `ConfigPairValue::Scalar`；物化侧用 `cfg_put_f64_text`/`cfg_put_u64_text`（**十进制文本**传值，i32 返回通道不变，因而不需要给共享 AST 加 f64/u64 字面量）；节点与对象两种目标都支持（节点走 `set_prop` 并保留"注入赢 root seed 键"）。边界：只认 `整数.整数`（无指数/无符号）与装得进 u64 的十进制；`0x10`/`1i32`/`1.5.6` 仍被拒。用例：`p982_config_wide_scalars`（默认拒绝 / 门控接受 两例）。
### AV 服务链（983 前置，2026-11-18 起）

`auto service --server av` 是**真实 HTTP 服务**：Rust 负责 HTTP，**ACE AV 执行 handler**（用户裁定形状 A）。契约：

**CLI / 环境**
- `--server av`（别名 `ace`）= AV 轨；`vm`／`rust` 不受影响。
- `AUTO_AV_ENTRY`（默认 `d_main`）固定入口；`AUTO_AV_ROUTES=derive` 改按 `(method, path)` 派生 def：
  `GET /` ⇒ `d_main`，`GET /hello` ⇒ `d_hello`，`GET /a/b` ⇒ `d_a_b`，`POST /hello` ⇒ `d_hello_post`（非 GET 追加方法后缀）。
- `AUTO_AV_WALL_MS` = 单请求墙钟预算；`AUTO_SERVICE_MAX_REQUESTS`（测试阀）＝服务满 N 个请求后自然关停。
- 闸门：`AUTO_SERVICE_MAX_BODY`（默认 1 MiB ⇒ 413）、`AUTO_SERVICE_MAX_HEADER`（8 KiB ⇒ 431）、
  `AUTO_SERVICE_MAX_REQUEST_LINE`（8 KiB ⇒ 414）、`AUTO_SERVICE_MAX_CONNECTIONS`（64 ⇒ 503）。

**注入给程序的契约函数**（handler 直接调用；每次请求重新合成源码）
- 长度面：`__av_request_path_len()`、`__av_request_method_len()`、`__av_request_query_len()`、`__av_request_body_len()`、`__av_request_target_len()`。
- 查询面（单值/多值/解码）：`__av_query_<k>()`（首个值）、`__av_query_<k>_present()`、`__av_query_<k>_len()`、`__av_query_<k>_<i>()`；`%XX`／`+` 已解码，键经 `[A-Za-z0-9_]` 归一。
- 体面（最小扫描，扁平对象里的整数）：`__av_body_<k>()`、`__av_body_<k>_present()`。

**状态码映射**：成功 200（体＝返回值 + `path=` + `steps=`）；`ace.entry/missing`／`lower.entry` ⇒ **404**；路径存在但方法无 def ⇒ **405 + allow**（`AUTO_AV_ROUTES=derive`）；诊断／宿主失败 ⇒ 500（带具名 code）；`Cancelled` ⇒ 504 `ace.service/timeout`；预算回收 ⇒ 504 `ace.service/budget-exhausted`。

**收据**（每次运行末尾一行，硬断言）：`✓ AV service receipt: served=N av_dispatches=M legacy_dispatches=L same_sdk=…`；`legacy_dispatches` 来自配置引擎的真实计数，`same_sdk=false` 时进程非零退出。**永远不回退 `vm` 轨**。

**文本入口（2026-11-18 用户裁定后落地）**：仅当**模块声明 `ac.runtime.text.v1`** 时，入口可以收**借用 `str`** 形参（`StrView`；owned `Str` 仍被 `mir.entry-sig` 拒绝）。服务侧 `AUTO_AV_TEXT=1` 时请求走受检文本入口，契约是 **`fn main(query str, body str)`**（`texts[0]`=查询串、`texts[1]`=UTF-8 体，逐请求数据、同一编译产物）；文本在本次执行的 arena 内分配，借用不逃逸，执行后 arena 必须归零；文本能力同时允许 `q.len_bytes()` 与 `q == "中文"`（按内容分流）。**边界**：`execute_entry`／`AceRequest.args` 的 argv 契约不变（通用入口仍拒文本实参）。当前是单进程、每连接一线程、`connection: close`／keep-alive 皆支持，未接 `auto_web` 服务器栈。
## 4. 具名边界（不冒充支持）

- **运行期列表 `for`**：仅**字面数组**（`for d in (["a","b"])` 或绑定字面数组的 `var`）会
  展开；元素来自注入/计算的列表仍是 `source.config-document/pending`（需 list len/at 原语）。
- **值面**：i32/bool/char/text/对象/列表/节点；无 `f64`/`u64`（`u64>i64::MAX` 守卫不可达）、
  **空列表字面量 `[]` 已支持（config-document 专用规则 `cfg-empty-array-lit`）**：投影为空列表；
  共享的 `array-lit` 仍要求至少一个元素（aggregates/scheduler 语义不变）。永久用例
  `p982_config_empty_list`（正例）与 `List([I64(1), I64(2)])`（非空）共同钉住值面。
- **注入**：bool/i32/char/text 可注入并在文档中按名可见；对象/数组注入是
  `ace.config/inject-unsupported`，超 i32 是 `ace.config/inject-range`。
- **对象字段进文档的正式用法＝按名搬运（2026-11-18 用户裁定 B）**：`obj: { x: 1 }` 之后，
  `picked: obj["x"]`、`picked: obj.x`、以及节点体的 `app("main") { heap: kernel["heap"] }` /
  `heap: kernel.heap` **都可用**，目标是拿到**同值**（`p982_config_field_spelling`、
  `p982_config_field_move` 为永久用例）。**读取为标量**（把 `obj["x"]` 直接当表达式值参与运算）
  仍缺"标量读"原语（会话只回 i32 句柄），因此**按名搬运是正式用法、读取不是**。
  注意 FStr 插值里的字段（`"${obj.x}"`）**仍被拒**（`source.type-mismatch`/`source.fstr-syntax`）。
- **对象字段读取/`cfg.get`/`cfg.merge`/`${obj.field}` 模板**：**经实测修正**——`k["f"]` 只对**体内绑定的局部**成立，**根属性名直接索引会被 bind 期拒**（`source.undefined`，"no local named …"）；FStr 里带引号字符串是 `source.fstr-syntax`；FStr 点形式是 bind 期 `source.undefined`。全部钉在 `p982_config_field_spelling`。**原记录**：
  `${obj.x}` 在 **bind 期**失败为 `stage:"bind", code:"source.undefined"`（"no local named `obj` in scope"），
  永久用例 `p982_config_dot_form`；实现时该用例会自动转为"必须取到字段值"的正例。
- **保留字作 pair key**（如 `tag:`）仍被当保留 token 拒绝。
- **投影深度**：2048 处具名 `config.projection/too-deep`（递归实现；设计允许的"无固定阈值"
  需迭代式投影，未做）。实测 ≤1976 层可服务（1980+ 由冻结的 16 MiB 校验预算具名拒绝）；永久用例：1000 层整链 + 2000 层不崩。
- **默认构建**：无 `ace-candidate` 特性 ⇒ 无桥 ⇒ pac 读取落 legacy，且选择可观测；把 ACE
  设为默认发行路径属 PLAN-986（安装/退役）。
- **二阶段 context/dep**：dep props 已作为注入 args 覆盖子包文档根属性；更完整的二阶段
  上下文/依赖解析未做。
- **shell/UI/生成器/模板产品面**（TEMPLATE/RUST/C/WEBTARGET/PY/REVERSE/EXPORT/DEVSDK、
  安装/升级/退役）：未交付，属 M2-B；本文件不声明这些能力。
