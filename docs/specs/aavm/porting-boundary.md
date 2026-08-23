# AAVM v2 移植边界清单（plan-431 Phase A）

- 基线锚点：**master `b3bd64f5`**（429-C1 临时基线；v0.5 tag 落地后重锚定，
  重跑 `docs/plans/reports/429-c1-baseline.md` 的命令与本目录清单脚本）
- 数据文件：本目录 `data/*.csv`（函数级清单由脚本生成，规则见 §6）

## 1. parser.rs（17,708 行，436 个函数）

| 类别 | 函数数 | 行数（近似区间合计） | 处置 |
|---|---|---|---|
| core | 367 | ~14,653 | 移植（语句/表达式/类型/闭包/use/模式/Pratt） |
| ui | 63 | ~2,775 | 剔除（widget/store/scene/msg/route/on_events/tag/grid/cover/style） |
| task | 6 | ~258 | 剔除 |

明细：`data/parser_fns.csv`（列：fn_name/indent/start/end/lines/kind）。

**精度声明**：分类按函数名关键词启发式生成。已知误报方向：
- `tag_*` 辅助函数可能被误标 ui（如 `tag_starts_enum_decl` 实为 core 前瞻辅助）；
- `parse_task_msg_pattern` 被标 ui 实为 task。
**432 执行时以本表为初始假设，逐函数移植时人工确认并回写 CSV**（kind 列后加
`!` 后缀表示已人工复核）。UI 段整块跳过；边界处的公共辅助函数
（capitalize_first / damerau_levenshtein_1 等）归 core。

## 2. codegen.rs（13,195 行，161 个函数）

| 类别 | 函数数 | 行数 | 处置 |
|---|---|---|---|
| core | 150 | ~12,626 | 移植 |
| ui/task | 11 | ~552 | 剔除 |

明细：`data/codegen_fns.csv`。**注意**：UI/config-accum 逻辑大量藏在 core 命名
的大函数内部（emit_* 巨型 match），启发式覆盖率低于 parser——**移植 emit 函数时
需人工剔除内部的 UI 分支**，剔除处以 `// BOUNDARY-OUT: <原分支>` 注释标记。

## 3. engine.rs（8,354 行，71 个函数）

| 类别 | 函数数 | 行数 | 处置 |
|---|---|---|---|
| core | 62 | ~7,642 | 移植（栈机 + 调度最小核） |
| ui/task | 9 | ~692 | 剔除（debugger/trace/UI console/异步 HTTP native） |

明细：`data/engine_fns.csv`。同 codegen：UI console/HTTP native 段藏在大函数内，
需人工标记剔除。

## 4. opcode 处置表（194 条，编号与 Rust 一致）

| 处置 | 数量 | 说明 |
|---|---|---|
| 移植 | 183 | engine 有 dispatch 分支的核心执行路径 |
| 仅声明 | 6 | SWAP/CONST_U8/SHL/SHR/JMP_L/PRINT——枚举占位，engine 未实现（SHL/SHR 即 429-B3 盘点的位移运算符缺口），v2 保持声明不动 |
| 剔除(并发/actor) | 5 | SPAWN/SPAWN_GO/CREATE_FUTURE/AWAIT_FUTURE/POLL_FUTURE——432 按需恢复 |

明细：`data/opcode_table.csv`。"engine 有 dispatch"按 `OpCode::{name}` 在基线
engine.rs 中出现与否机械判定，无 UI 专属 opcode（UI 走 native/catalog 而非 opcode）。

## 5. native_catalog.rs（2,319 行，521 条）

| 处置 | 数量 | 判据 |
|---|---|---|
| 核心 | 455 | 名称不在 UI/异步域 |
| 剔除(UI/异步) | 66 | auto.ui.*/auto.view.*/auto.store.*/auto.task.*/auto.http.*/auto.ws.*/auto.process.spawn*/async 系 |

明细：`data/catalog_table.csv`。v2 的 `natives.at` 按核心子集移植（X-macro 模式
保留）；429-B1 的 shim 方法面盘点（Vec/String ~60 方法）叠加其上作为 P0。

## 6. 清单再生成方法

```bash
# 基线快照导出
for f in parser vm/codegen vm/engine vm/opcode vm/native_catalog; do
  git show b3bd64f5:crates/auto-lang/src/$f.rs > /tmp/$f.rs
done
# 函数级提取(下一函数起点作区间边界,规避字符字面量 '{' 干扰)
python docs/specs/aavm/data/extract_fns.py /tmp/parser.rs > docs/specs/aavm/data/parser_fns.csv
# opcode / catalog 同法(脚本存档于 plan-431 提交的 target/a431/*.py,tag 重锚定时复用)
```

## 7.（A4）Rust 侧拆分建议清单（喂给 432 债务记录，不提前重构）

1. parser.rs 17.7k 行单文件——core/ui 混杂，理想拆分 `parser/{core,ui}.rs`;
2. codegen/engine 的巨型 emit/run 函数内嵌 UI 段——理想按 dispatch 表外提;
3. `Duration`/`Instant` 等手写臂已由 plan-430 生成段接管,引擎侧残留 u128 族
   访问器待目录化;
4. native_catalog 521 条单数组——核心/UI 分文件;
5. opcode `仅声明` 6 条中 SHL/SHR 是实际语法缺口（429-B3）,实现后本表需更新。
