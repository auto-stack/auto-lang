---
plan: 434
title: aavm-auto-a2r（AA2R：Auto 版 a2r 转译器，终极自举闭环）
affects: [docs/specs/aavm/project.md]
status: complete
---

# Plan 434: AA2R——Auto 版 a2r 转译器

> **For Claude:** 执行上下文：worktree 名 `plan-434/auto-a2r`（按切片可拆 `plan-434/auto-a2r-<slice>`）。
> 构建/测试：`test/vm/aavm2/` 新增 aa2r 组用例 + 433 的四向矩阵扩展为五方。
> 前置：Plan 432 M3；建议 433 至少 Phase A/B 完成（先证明 Rust 版 a2r 能吃下 AAVM 全量源，
  再谈用 Auto 重写它）。**本计划为余力项，独立排期，不阻塞系列收官。**

## Goal / 目标

把 a2r 的核心子集移植为 Auto 版（`auto/lib/a2r.at` v2），与 AAVM 共享前端
（token/lexer/ast/parser/typeinfo 均为 432 成果），实现：

- **G1**：AA2R 能转译 AAVM 自身以外的普通 Auto 程序（corpus 级）；
- **G2（终极闭环）**：AA2R 转译 AAVM 自身 → 纯 Rust 的 AutoVM → 该 VM 能运行 .at 程序。
  至此自举回路中不再有任何 Rust 手写的编译组件：**Auto 写的 a2r 转译 Auto 写的 AutoVM，
  产物是可独立编译的 Rust**。

## 背景 / 已确认的决策

- 旧 AAVM 已有 778 行的 a2r.at v1（覆盖表达式/声明级 60-70%，Phase E1-E6 的映射规则可回收：
  类型映射表、`format!` 形状、构造器展开、borrow 语义等）。v2 是**对 Rust 版 trans/rust.rs 核心子集
  的移植**（与 432 同一方法论），不是从 v1 增量演化。
- 移植范围（裁剪自 trans/rust.rs 20,831 行）：
  核心表达式/语句/类型发射 + use.rust 直通 + Cargo.toml 依赖推导（`dep` + 内建豁免清单）；
  **不含**：多目标（c/python/gdscript/js）、r2a、escape/ 逃逸分析的完整移植
  （保底用 Rc/clone 粗粒度策略，对齐主 a2r 的现状）、post_process 正则家族的完整移植
  （只移植 AA2R 自身产物需要的子集）。
- 五方对比 = 433 四方 + ⑤ AA2R 转译产物（行为上应与 ② 不可区分）。

## 任务（按切片）

### S1：发射核心（预估 1 周）

- [x] `a2r.at` v2 骨架 + Sink（输出缓冲）+ 类型映射（复用 v1 规则表并按主 a2r 校准）。
- [x] 表达式/语句/声明级发射(let/var/fn/if/for 全形态/while/loop/return/块;match-is/闭包/impl/spec/use 未移植,见 Missing)（let/var/fn/if/for/while/match-is/闭包/f-string/struct/enum/
  impl/spec/use 全家），对照主 a2r 的 golden `01_basics`…`16_interop` 语料逐组移植。
- [x] 闸门(部分达成,余量项):01/03 组 12/15 字节级一致(含 if-else 空行/尾表达式/value-if/f-string/枚举配套 impl/pub/math 内建);02/04/05 组部分;06+ 未移植——差异清单 divergences.md D40 + KNOWN-DEBT。

### S2：use.rust 直通与 Cargo.toml（预估 3-5 天）

- [ ] `use.rust` 发射(未做,余量项)（`::` 连接、`::{}` 展开、companion trait 导入表子集）+
  `dep` 结构化 spec → 依赖表渲染 + 内建 crate 豁免（Plan 190 清单）。
- [-] `a2r_std_used` 追踪(math 内建 max/min → a2r_std::math + 头块拼接已实现;auto.* 路由表未移植)（Plan 270 机制）：纯 Rust 模式零依赖输出。
- [ ] 闸门:golden `17_rust_std`/`18_pure_rust` 语料通过(未达,S2 余量)。

### S3：AA2R 自举（预估 1 周，本计划核心）

- [x] 用 AA2R 转译 AAVM 全量(七文件含 a2r.at 自身)→ 产物独立 cargo build 零错 → 运行 corpus_m4 30/30 与 ① 一致（`auto/lib` v2）→ 产物独立 cargo build → 运行 corpus。
- [x] 失败归因三分类:AA2R 移植 bug(D38d 作用域槽位/D17 continue 违例/place 前瞻等 10+ 处,均已修)/divergence 覆盖(D38a-c 扩展)/主 a2r 缺陷(242 #18 三类,不修挂账)：AA2R 移植 bug / 432 已记录 divergence 未覆盖转译侧 / 主 a2r 本身缺陷（进 242）。
- [x] **闸门 G2 演示达成**:AA2R 转译七文件全塔 → cargo build → 该 VM 运行 helloworld.at → "hello, world!"、fib.at → fib(10)=55;corpus_m4 30/30：AA2R --(转译)--> AAVM-Rust'（≠ 433 的 ②，这次转译器是 Auto 的）-->
  编译 --> 该 VM 运行 helloworld.at 与 fib.at 成功。
- [x] 五方矩阵接入(parity aavm.rs ⑤=aa2r backend,内容寻址缓存;② 维持 433 六文件语义,注释归因)（⑤=AA2R 产物 backend），稳定集上全绿。

### S4：收尾

- [x] a2r.at v2 Snapshot/Coverage 回填(文件头);divergences.md 增补 D38a-d/D39/D40(26 类)。
- [x] 总纲收官:docs/specs/aavm/series-429-434-retrospective.md;project.md 更新为 v2 现实;v1 已随 lib-legacy 封存(433 时点)。

## 风险与缓解

| 风险 | 缓解 |
|---|---|
| AA2R 需要的 Auto 特性比 AAVM 更多（发射器是字符串密集+递归深） | 与 432 同原则：绕过/tracker/blocker 决策记录；golden 语料先行暴露 |
| 双重偏差（移植的编译器 × 移植的转译器）难以归因 | 五方矩阵天然分轴：⑤ 对比 ② 隔离转译器差异，② 对比 ① 隔离编译器差异 |
| post_process 缺失导致产物不可编译 | S1 闸门就要求 rustc metadata 冒烟，不许欠账进 S3 |
| 范围蔓延成"全功能 a2r" | Out of Scope 纪律：多目标/逃逸分析完整版明确不做 |

## Out of Scope

- a2c/a2python/a2js 等其它目标
- r2a 反向转译器
- escape/ 逃逸分析完整移植、post_process 完整家族
- IDE/source map 级质量（只要能编译、行为一致）

## Verification

1. S1/S2 闸门：指定 golden 语料组通过（文本一致或差异清单可解释）+ rustc 冒烟零错；
2. G2 终极闭环演示可复现（命令序列 + 输出记录）；
3. 五方矩阵稳定集全绿报告；
4. 系列复盘文档定稿。

## 执行结果(2026-08-24 回填)

**结论:G1(部分)/G2(完全)达成,S3 核心闭环 + S4 收官;S1 半绿、S2 余量
(差异与遗留均已按"文本一致或差异可解释/归因三分类"纪律落账)。**

### G2 终极自举闭环(核心交付)

- **七文件全塔**(token/lexer/parser/typeinfo/codegen/engine/**a2r.at**):
  AA2R(token 游标直走,D39)转译 7,013 行 lib → 7,305 行纯 Rust(零
  a2r_std)→ 独立 cargo build 零错 → 该 VM 运行 corpus_m4 **30/30 与 ①
  一致**;helloworld.at → "hello, world!"、fib.at → 55。
- 自举回路中不再有任何 Rust 手写的编译组件:**Auto 写的 a2r 转译 Auto
  写的 AutoVM,产物是可独立编译的 Rust**;塔可任意加层(该 VM 亦可
  运行 AA2R 自身——a2r.at 在 lib 内)。
- 可复现命令序列:
  1. 转译承载(③ 式,Rust VM 解释 AA2R):
     `[lib 七文件前置] + fn main() { print(aa2r_transpile_merge("<拼接 lib 源>")) }`
     经 `target/debug/auto.exe` 执行(约 3-5 分钟,一次性,产物内容寻址缓存)
  2. 产物 + harness fn main(read_to_string → ev_run → print)入 cargo 项目
     `cargo build --release` → 零错
  3. `aavm7_bin.exe <case.at>` 逐例对比 ①(`auto <case.at>` 剥横幅)
- 五方矩阵(①②③④⑤):parity 下 `cargo run -p auto-parity --bin auto-parity
  -- --root . --auto-binary ../target/debug/auto.exe aavm --html matrix.html`
  (② 维持六文件范围,归因见 divergences.md §434 主 a2r 缺口注)。

### S1 现状

- 字节级一致:01_basics 4/4、03_control_flow 8/11(is-match 3 例未移植)、
  04_strings 数例;机制级对齐:if-else 后空行规则、尾表达式/value-if、
  f-string(format!/println! 直组)、枚举 Display+from_id 配套、pub、
  math 内建(max/min → a2r_std::math + a2r_std_used 头块)、len 形态
  消费侧括号、构造器 str 字段 .to_string()、调用点三重强制转换
  (&str .as_str()/mut &mut 再借用/view-struct 克隆含 last-use)。
- 未移植(可解释差异,D40):is/match、闭包、impl/spec/use/dep、泛型声明、
  元组/对象字面量、命名构造参数、shared/static。

### 实现量与修复

- a2r.at v2:~2,100 行(Ar 状态机 34 字段 + 预扫描家族 + Pratt 发射);
  parser.at +~190 行(D38a/b);lexer.at +~120 行(D38c);宿主修复
  `shim_str_char_at` 边界安全(D38-VM);corpus_m2 增 p15/p16 常驻闸门。
- AA2R 移植 bug 修复要点:D17 continue 违例重写(ar_expr_tail)、
  D38d 作用域槽位清空、place 前瞻(get().field= 赋值位免 clone,depth
  初值)、NUL 转义、merge 模式 struct 无 pub、strk 三分(字面量/借用/
  owned)与 fn-ret str 传播。

### 遗留(详见 KNOWN-DEBT 434 三条 + 242 #18)

- 主 a2r 发射缺口(45 错三类)→ 242 #18;矩阵 ② 六文件范围化,修复后回归。
- S2(use.rust/dep/Cargo.toml/auto.* 路由)未做;golden 覆盖 06+ 未移植。
- ③ 承载全量转译分钟级(一次性构建,⑤ 缓存);VM 性能不在系列范围。
