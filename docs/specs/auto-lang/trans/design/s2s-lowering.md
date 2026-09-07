# s2s 改写契约（脚本糖 → 正常模式桥）

> 来源：[PLAN-560](../../../../plans/archive/560-script-mode-w2-sugar-batch.md)
> （2026-09-05，脚本模式 W2 语法糖批；设计源
> `docs/design/strategy/script-mode-interop.md` §1/§3）。对应代码：
> `trans/auto_s2s.rs`（管线+规则表）、`trans/emit.rs`（AST 发射器）、
> `trans/py_known.rs`（静态分析）、`trans/s2s_rules.rs`（A/B/C 族规则）。

## 范围

AutoScript（`.as`）糖源 → 正常模式桥源的 source-to-source 改写：
`.as` 执行 = `lower_source` 产物走正常编译管线（plan-560 T03 激活）；
`auto trans --path x auto` 与 `--dump-lowered` 共用产物。语义全在桥上
（§1 单一真相源）——糖有 bug 看改写产物即知。

## 帧形（P555-D2 裁定，plan-560 T02 落地）

- **AST 帧 + 链式规则**：parse → `LoweringRule { id: A1..F4, transform }`
  有序单遍全过（后继规则看见前驱改写）→ `emit_code` 发射。
- W1 token 帧退役（无法表达语句展开与表达式重排）。
- **发射器纪律**：`Op`/`Type` 的 Display 是 S-expr 调试形态——源发射
  必须走符号表直渲；Str 必须再转义（内嵌引号产物合法化）；块不写尾随
  换行（语句层统一补，幂等性钉死 `lower(lower(x)) == lower(x)`）；
  Store 类型标注省略（推断回填——显式异型标注为罕见面）。
- **词法惯例教训**：lexer arm 收到的 `c` **尚未消费**（with_equal 等
  自行 next）——双星探测须用迭代器快照越过 c 看第二位（首版误用
  peek() 恒见 c 自身，单星全误翻 Power，单测钉）。

## py_known 静态分析（规则/窥孔共用）

- py-producing 形态：`py_*`/`obj_*` 桥调用、`import_module`、
  **导入项调用**（`var t = arange(6)`——item 表查询）、导入项/裸模块
  Ident 直传、py-known 的 Dot/Index 链。
- 保守单遍：分支不敏感、闭包不内视——错报"已知"代价=改写走 py 桥
  （运行期响亮错），错报"未知"代价=糖保留（语义不变），双向安全。

## 规则表（A/B/C 族；D/E 通道见桥侧）

| 规则 | 糖 | 产物 | 门控 |
|---|---|---|---|
| A1/A2 | `t.sum(a)` / `t.sum(dim: 0)` | `py_call(t, "sum", ...)`（kwargs 由 codegen 落 py_call_kw） | **仅 py-known 接收者**——盲改会让 obj_call Auto 臂拒绝 `s.len()`（Auto 方法分派保持糖态走原生通道） |
| B1/B9 | `w.shape` / `torch.nn.Linear` | `py_getattr`（dotted 递归） | py-known |
| B2/B3/B4 | `m.weight = w` / `x[i]` / `x[i]=v` | `py_setattr`(467) / `py_getitem` / `py_setitem` | py-known |
| B5 | `x[a..b]`/`x[..b]`/`x[a..]`/`x[a..=b]`/`x[a..b..c]` | `py_getitem(x, py_slice(...))`：Nil 端→null、`..=`→end+1、**步长=嵌套 Range(a, Range(b,c)) 位形** | py-known；B3 让位切片位形 |
| B6 | `len(x)` | `obj_len`（双通道组合子） | 无条件（len 非保留字） |
| A5 | py_* 调用的闭包实参 | `py_callable(closure)` 自动包裹 | py_callable/py_with 豁免（防重入/内联形态要裸闭包） |
| D7 | `print(x)` | `py_str(x)` | 仅 py-known **裸名**（嵌套形态走 print shim 运行期臂防二遍重入） |
| C3/C4 | `if t:` / `and or not` py-known 位 | `py_truthy` 包裹 | py-known（Auto 真值零打扰） |
| C5/C6/C7 | `a @ b` / `a ** b` / `a is b` | **解析层直产** `py_matmul`/`py_pow`/`py_is`（非 s2s 规则——免新增 Op 变量全仓波及） | 解析期（@T 类型位不受影响） |

## 管线集成与门控

- `execute_autovm_with_path`：`.as` 扩展名（`#[rust]` 行首预检压回保留
  原源）→ `lower_source` → 正常编译。`#[rust]` 预检为行首启发式
  （完整判定在 session 解析段）。
- **550 门控硬化 E5501**（T14，用户裁定 a+b 收口）：`.at` 文件含三信号
  （use.py/null/nil 字面量）→ 诊断错误。**文件上下文限定**：内联/eval
  无 path 不门（musk null 语义测试族走 run_with_capture 通道——VM
  语义测试不应被文件门拦）；tv 文件测试走 `run_with_capture_and_path`
  （.at 受门/.as 豁免）；豁免矩阵 `.as`/`#[script]`/`#[rust]` 压回。

## 测试载具

- `tests_s2s` 6 例：identity 幂等 / 规则注入即生效 / 非法源拒绝 /
  A/B 族 source-to-source / B5-B6-A5-D7 / **五套件语料 round-trip**
  （parse→emit→re-parse→emit 逐文件幂等，19 套件 .as 载体）。
- 探针 `test/s2s-lowering/p01-p14`；三方门禁=parity runner 双扩展名 glob。

## Plan 567 增量契约

- **E1 隐式传播规则**（builtin_rules 链尾注册）：`MAY_RENAMES` =
  py_call→py_call_may(453)/py_getattr→476/py_getitem→477；kwargs 形态
  保持 Pair 由 codegen `is_py_call_kw_form` 路由 478。改写产物追加
  `.?`（ERROR_PROPAGATE）；幂等纪律：`Expr::ErrorPropagate` 臂 no-op。
- **with-as 块形态**（parser with_stmt 产）：`Stmt::Expr(Expr::Block[
  var __w = e; var x = py_enter(__w); try{b} catch(__we){py_exit;py_raise}
  finally{py_exit}])`——a2py `match_with_as_block` 回译 `with e as x:`；
  emit 以 `if true` 包装（裸 `{` 语句命中 E0007/convert_last_block 双歧义，
  恒真 If 免疫且幂等）；convert_last_block 收窄纯 pair 块。
- **tv .as 语料**：vm_file_tests 扫描 `.at` 优先回落 `.as`，is_script 标记
  先过 lower_source（99_script_err 四例，python feature 门控）。

## 已知边界

P560-D1/D2/D3/D6 已清偿（567）；P560-D4/D5 面随 567 收口（门控语料/
kwargs×may）；py_known 保守面维持（P560-D5 in案）；`.?(d)` 表达式位链式
消费缺陷（存量，453 同形复现——567 待澄清⑥/P567-R 族债外独立登记）。
