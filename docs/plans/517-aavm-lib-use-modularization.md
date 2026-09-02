---
plan_id: PLAN-517
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: aavm-lib-use-modularization
author: [zhaopuming]
created_at: 2026-09-02
updated_at: 2026-09-02

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-017]     # 自举：用 Auto 写 Auto 编译器（aavm）

affects: [aavm]
current_step: 0
total_steps: 14
---

# [PLAN-517] aavm lib use 模块化：七文件依赖 DAG 重组 + AA2R use 发射 + CLI 入口

## 变更摘要

D5 翻案（用户裁定）：511 W0 的"聚合定案"非终局——lib 应当用上自身的中阶
`use` 能力做模块化重组（自举本味：lib 成为 aavm `use` 能力的第一个真实
多模块程序）。本计划依据已落盘调研
[lib-modularization-map.md](../specs/aavm/design/lib-modularization-map.md)
（`scripts/aavm_lib_xref.py` 实测）执行三件事：

- **W1 AA2R use 发射**（塔式硬前置，514 W2 未覆盖的 g16 空缺）：a2r.at
  `ar_run` 增 Use 分支，与主 a2r live 逐字符 + rustc 零错。
- **W2 lib use 模块化**：破唯一环边（`p_peek_text` 迁 parser.at）→
  七文件按依赖 DAG 加互引 use + pub 导出 → 拼接式消费者双轨剥离兼容。
- **W3 CLI 入口**：生成 `auto/aavm.at`，`auto run auto/aavm.at <目标.at>`
  即得单文件启动入口（此前不存在的缺口）。

**与 Plan 514 的关系（顺序已裁定：514 先行做完，517 随后开工）**：use
模块化与方法化是正交轴，先后皆可执行；用户裁定 514（W3 方法化重启 +
W4/W5 收口）先收口，本计划在**方法化后的 lib** 上执行。三个连带：
① pub 导出面会因方法随类型归属而缩表（方法不再是模块级符号）——映射图
需在 514 完成态重跑定稿；② `p_peek_text` 环边可能已被方法化自然消除
（若转为 P 类型方法即随 parser.at 归位）——步骤 4 改核验优先；
③ a2r.at 行号锚点为 514-W3 前快照，开工时重定位。

全程 TDD：既有闸门为保护网（红=行为变化=回退），新能力语料红先行。

## 目标

1. AA2R 能发射 use 语句：corpus_a2r 新件 g16_use_stmt 与主 a2r live 逐
   字符一致，双侧产物 rustc 零错。
2. lib 七文件 use 模块化：`aavm_lib_xref.py` 零反向边；pub 面与映射图
   一致；双轨剥离后拼接路径与改造前**逐字节等价**。
3. `auto/aavm.at` CLI 入口可用：`auto run auto/aavm.at <目标.at>` 编译
   并执行目标程序（无参时内置冒烟）。
4. 全程既有闸门（含 514 W2 新纳入常规门禁的 compile 腿）+ 矩阵 +
   99_unit 不破绿。

### 非目标（Out of Scope）

- lib 方法化（γ4）——归 514 W3 重启（P514-W3 清单），不在本计划。
- 99_unit 测试件自身 use 化（auto test session 不播种源目录，D5 发现②
  仍真；维持聚合双轨）。
- `use.rs`/`use.py`/`use.c` 异构导入：宿主语义范围，不动。
- 定向/通配导入语法的语言级变更：只用既有形态
  （`use auto.lib.x: a, b` / `: *`）。
- 跨模块类型共享语义增强（pub type 导入的 aavm 目标语言侧深度）：仅用
  宿主既有行为。

## 架构方案

塔式爬升（先发射后使用），三波各带独立判据：

```text
W1 AA2R use 发射          W2 lib use 模块化              W3 CLI 入口
───────────────         ─────────────────────         ──────────
g16_use_stmt 先红   →    p_peek_text 迁移破环      →   auto/aavm.at 生成
ar_run Use 分支     →    七文件 use+pub(逐文件)    →   auto run 实测冒烟
主 a2r 对齐考古     →    双轨剥离(harness/生成器/   →   入口留档
rustc 实编译        →      parity ②⑤腿)            →   折叠点
```

**关键设计约束**：

- **双轨剥离**（行为不变的构造性保证）：拼接式消费者
  （M1–M5 harness 的 AUTO_LIB_FILES_V2 拼接、`gen-aavm2-unit.py`、
  parity ②⑤腿转译输入）统一剥除 `use auto.lib.*` 行——剥离后拼接产物
  与今日逐字节等价（符号仍同居一程序）；真 use 解析只走模块路径
  （`auto run` 入口 / `auto build`）。
- **use 发射产物形态以主 a2r 为规范**（W0 考古定案：Rust `use` 限定名 /
  crate 内路径 / 忽略+全限定——三种候选，以主 a2r 对 use 的现行转译为
  准镜像）。
- **每文件一提交**：use+pub 改造逐文件进行，闸门绿才进下一个。
- **矩阵每折叠点必跑**（514 W2 修复后 46/46 基线，不得回破）。

## 需求分析与背景调查

（取材：lib-modularization-map.md（本计划事实基础）、Plan 514 执行期
提交（515370ae4 等）与 P514-W3 债、a2r.at Missing 清单）

### 基线（2026-09-02，master 61ae33782 + 92644f8ac）

| 门禁 | 结果 | 来源 |
|---|---|---|
| tv 标准门禁（compile 腿已去 ignore 入常规） | 全绿（37/37 corpus 含） | 514 W2 折叠证据（515370ae4） |
| 五方矩阵 | 46/46 全绿（P511-5 已清偿） | 同上 |
| `cargo tf` | 3350 绿 | 同上 |
| 99_unit | 13 绿 | 同上 |
| Plan 514 状态 | executing，W3 方法化挂起（P514-W3-1 书写约定已验证可行 / W3-2 主 a2r patch 存档差 1 处） | KNOWN-DEBT P514 节 |

### 调研结论（详见 lib-modularization-map.md）

- 依赖 DAG：`token ← lexer ← parser ← {typeinfo ← codegen ← engine}` +
  `a2r → {token, lexer, parser, typeinfo}`；engine 纯解释器层（仅依赖
  codegen）；a2r 不依赖 codegen/engine。
- 唯一环边：`p_peek_text`（codegen.at:355 定义 / parser.at:1580 引用，
  拼接模式链接期解析掩盖）→ 迁 parser.at 破环。
- 跨文件重名定义零；pub 导出面：token 3 / lexer 2 / parser 38（含迁入
  的 p_peek_text）/ typeinfo 1 / codegen 5 / engine+a2r 入口 pub
  （`ev_run`/`ev_run_files`/`ar_run`）。
- AA2R use 发射缺失在案（a2r.at:26 Missing 清单）；514 W2 交付 g01–g15
  方法发射，g16 空缺。

### 风险与对策

| 风险 | 对策 |
|---|---|
| 主 a2r 对 lib 内 use 的转译形态未实证（矩阵②腿） | W0 考古先行（探针：lib 形态 use 经主 a2r 转译 rustc）；有洞先修主 a2r（447 先例） |
| use 发射产物与 pac `auto build` 拼装不匹配 | W1 以"产物可被 auto build 直接使用"为验收一部分 |
| 模块初始化序改变行为（宿主模块顶层执行序） | lib 文件顶层无副作用语句（纯声明），W0 核实；双轨剥离构造性等价兜底 |
| 与 514 撞车（同文件双 worktree） | 已裁定顺序：514 先收口，517 开工前置 = 514 status: reviewed/archived |
| session 可达性：`auto run` 入口 use auto.lib.* 解析 | 17_modules 样式（auto/ stdlib 根）已实证；W0 探针复验 |

## 详细设计

### W1 AA2R use 发射（a2r.at，自由函数风格书写）

1. **语料先行（红）**：corpus_a2r 增 `g16_use_stmt`（lib 形态：定向
   `use mod: a, b` + 通配 `use mod: *` 双件或单件双形态）——落盘即红
   （ar_run 遇 Use 报 unsupported）。
2. **W0 考古定案**：主 a2r 对 use 语句的现行转译形态（trans/rust.rs
   use 符号提取位 :17531 族）——Rust use 限定名 / crate 路径 / 忽略+
   全限定三选一，以实测为准；lib 形态 use 探针经主 a2r → rustc 零错。
3. `ar_run`（a2r.at:3014）顶层分派增 `Use` 分支：解析 use 路径与导入
   清单，发射镜像主 a2r 形态；`ar_prescan` 对 use 语句的跳过/登记与
   主 a2r 对齐。
4. rustc 实编译 + `auto build` 可用性验证（pac 路径拼装）。

### W2 lib use 模块化（依据映射图）

1. **破环**：`p_peek_text` 自 codegen.at:355 迁 parser.at（p_peek 同族
   归位，纯函数零风险）；`python scripts/aavm_lib_xref.py` 确认零反向边。
2. **use+pub 改造**（逐文件一提交，依赖序）：
   - token.at：零 use；pub `TokenKind`/`keyword_kind`/`kind_name`。
   - lexer.at：`use auto.lib.token: TokenKind, keyword_kind, kind_name`；
     pub `Token`/`tokenize`。
   - parser.at：use token/lexer；pub 38 面（映射图 §2 全表）。
   - typeinfo.at：use token/lexer/parser；pub `t_is_type_prop`。
   - codegen.at：use token/lexer/parser/typeinfo；pub 5 面
     （`CG`/`OpCode`/`cg_compile`/`cg_compile_files`/`op_name`）。
   - engine.at：`use auto.lib.codegen: CG, OpCode, cg_compile,
     cg_compile_files, op_name`；pub `ev_run`/`ev_run_files`。
   - a2r.at：use token/lexer/parser/typeinfo；pub `ar_run`。
   定向 vs 通配缺省定向（待澄清②）；dump 族入口
   （`lex_dump`/`parse_dump`/`typecheck_dump`/`codegen_dump`）一并 pub。
   **pub 清单以步骤 1 在 514 完成态重跑的映射图为准**——方法化后方法
   随类型归属，模块级符号面缩表（下表为方法化前基线，仅示意文件职责）。
3. **双轨剥离兼容**：`gen-aavm2-unit.py` 增剥离逻辑 + `--check` 同步
   校验；M1–M5 harness（`AUTO_LIB_FILES_V2` 拼接位）与 parity ②⑤腿
   转译输入同步剥离。验证含**拼接产物与改造前逐字节等价抽验**。

### W3 CLI 入口

1. 生成 `auto/aavm.at`（缺省真模块版：`use auto.lib.engine: ev_run,
   ev_run_files` + `fn main`：`auto.process.args` 取路径 →
   `ev_run_files(path)` → print；无参跑内置冒烟语料）。入口形态缺省
   真模块版，`auto run` session 可达性 W0 探针复验（17_modules 同款
   auto/ 根样式）；不可达则回退聚合剥离版（待澄清③）。
2. 实测留档：`auto run auto/aavm.at crates/auto-lang/test/vm/aavm2/
   corpus_m4/b07_fib.at` 输出 55；无参冒烟输出。

### 规格与登记（贯穿）

lib-modularization-map.md 执行期状态注记（D5 翻案定案记录）；
divergences.md（use 发射形态注记）；project.md / auto/lib/README
（模块化后结构与入口用法）/ KNOWN-DEBT 视情。

## 测试设计（TDD：保护网 + 红先行）

> **同步规约对齐注记（2026-09-02 规约/格式裁定后补）**：①g16 落当前
> corpus_a2r 平铺 live 对拍形态——三件套金样/四路统一 runner/bless 基建
> 归队列①批量迁移建立，本计划不单建（避免 scope 膨胀）；②use 语料的
> a2r 运行闸（compile corpus 覆盖 corpus_use）为 511 既有欠账、归队列①
> 清偿，本计划不双计——但 **lib 自身 use 化的运行闸在本计划范围内**
> （双轨剥离 + 矩阵②⑤腿即是对转译版 lib 的运行验证）。

### 保护网（全程不得破绿）

514 W2 后的完整闸面：tv 标准门禁（含 compile 腿 37/37）/ 99_idiom_probe
16 件 / 001-002 金样 / repro_242 / 五方矩阵 46/46 / vm-files-ci 三层 /
99_unit 13 件。W2 每文件改造后全绿（红=行为变化=回退该文件）。

### 红先行

| 红灯 | 载体 | 转绿点 |
|---|---|---|
| corpus_a2r g16_use_stmt（AA2R 遇 use unsupported） | W1 步骤 1 落盘 | W1 实现 |
| `aavm_lib_xref.py` 反向边（p_peek_text 环） | 调研已红 | W2 步骤 4 |
| `auto run auto/aavm.at`（入口不存在） | 基线即在红 | W3 |
| lib 形态 use 探针（主 a2r 转译/rustc，若洞则登记） | W0 落盘 | W0 定案或转主 a2r 修复 |

### 命令

- 门禁：`cargo test -p auto-lang --lib --features test-vm-files --
  test_aavm2 --include-ignored --test-threads=1`
- Auto 侧：`./target/debug/auto.exe test -d crates/auto-lang/test/vm/aavm2/99_unit`
- 矩阵：`cd parity && cargo run -- --root . --auto-binary ../target/debug/auto.exe aavm`
- 模块化校验：`python scripts/aavm_lib_xref.py`；
  `python scripts/gen-aavm2-unit.py --check`
- 入口：`auto run auto/aavm.at <目标.at>`

## 验收标准

1. corpus_a2r g16 与主 a2r live 逐字符一致 + 双侧产物 rustc 零错 +
   `auto build` 可用。
2. `aavm_lib_xref.py` 零反向边；七文件 use+pub 与映射图一致；拼接产物
   改造前后逐字节等价（抽验留档）。
3. `auto run auto/aavm.at b07_fib.at` → 55 实测留档；无参冒烟可用。
4. 全程闸门（含 compile 腿）+ 矩阵 + 99_unit 零破绿；`cargo tf` 绿。
5. D5 翻案定案注记入 lib-modularization-map.md；文档回写完成。
6. 无静默丢弃（延后项显式登记）。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

> 约定：W0 在 master 直接做；W1 起在 worktree `.worktrees/plan-517-dev`
> 内；折叠点（步骤 3/9/12）矩阵+CI 绿后合入 master。

### W0 考古与探针（master）

1. [ ] 基线复测刷新（514 完成态：门禁/矩阵/tf/99_unit 数字重跑留档）+
   **映射图在方法化后 lib 上重跑定稿**（`aavm_lib_xref.py` 需升级支持
   type 体方法提取——方法名随类型归属解析依赖边；pub 表按"方法随类型"
   原则缩表）；主 a2r use 转译形态考古定案（lib 形态 use 探针经主 a2r →
   rustc 零错；发射形态记录）；`auto run` 对 `use auto.lib.*` 的 session
   可达性探针（17_modules 同款）；lib 文件顶层无副作用语句核实（方法化
   后复核）。验证：探针件 + 形态记录入 lib-modularization-map.md 执行注记。
2. [ ] corpus_a2r `g16_use_stmt` 语料先行落盘（红）。验证：AA2R 侧红证。

### W1 AA2R use 发射（worktree）

3. [ ] `auto/lib/a2r.at`：ar_run Use 分支 + 预扫对齐（发射形态按步骤 1
   定案）。验证：g16 live 逐字符绿 + rustc 零错 + `auto build` 可用 +
   折叠点①合入。

### W2 lib use 模块化（worktree 续）

4. [ ] 破环核验：先重跑 `python scripts/aavm_lib_xref.py`——若 514
    方法化已将 `p_peek_text` 转为 P 类型方法（随 parser.at 归位）则环边
    自然消失仅留档；仍为 codegen.at 模块级符号则执行迁移。
    验证：零反向边 + tv-aavm2 绿。
5. [ ] token.at + lexer.at use+pub。验证：tv-aavm2 + 99_unit 绿。
6. [ ] parser.at + typeinfo.at use+pub。验证：同上。
7. [ ] codegen.at + engine.at + a2r.at use+pub。验证：同上。
8. [ ] 双轨剥离兼容：gen-aavm2-unit.py（+--check）/ M1–M5 harness /
    parity ②⑤腿。验证：`--check` 过 + 全闸门绿 + 拼接产物逐字节等价抽验。
9. [ ] 折叠点②：矩阵 46/46 + CI 绿合入
    （`feat(aavm): Plan 517 lib use 模块化——七文件 DAG+pub+双轨 (Plan 517)`）。

### W3 CLI 入口（worktree 续）

10. [ ] 生成 `auto/aavm.at`（真模块版或按待澄清③回退聚合版）。
    验证：`auto run auto/aavm.at .../corpus_m4/b07_fib.at` → 55。
11. [ ] 无参冒烟 + 入口用法文档位。验证：冒烟输出留档。
12. [ ] 折叠点③：全闸门 + 矩阵 + 入口冒烟全绿合入。

### 收尾

13. [ ] 文档回写：lib-modularization-map.md 执行期注记（D5 翻案定案）、
    divergences.md（use 发射形态）、project.md、auto/lib/README
    （模块化结构+入口用法）、KNOWN-DEBT 视情。
14. [ ] 复审（/auto-plan:review）→ `cargo tf` → status: reviewed。

## 复审记录

## 待澄清事项

1. ~~与 Plan 514 W3 重启的先后~~ **已结案（2026-09-02 用户裁定）**：514
   先行做完（含 W3 方法化重启与 W4/W5 收口），517 随后开工；开工前置 =
   514 status: reviewed/archived。方法化翻转脚本不再与本计划交叉。
2. **定向 vs 通配**（阻塞步骤 5–7）：缺省定向清单（38 符号冗长但显式，
   一对一风格延续）；parser 大面可在执行期按可读性翻转为
   `use auto.lib.parser: *`。
3. **`auto/aavm.at` 入口形态**（阻塞步骤 10）：缺省真模块 use 版
   （自举本味）；`auto run` session 可达性探针（步骤 1）不过则回退
   聚合剥离版（生成器产物）。
