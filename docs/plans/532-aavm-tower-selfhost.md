---
plan_id: PLAN-532
status: executing              # drafting → executing → execution_done → reviewed → archived
feature_name: aavm-tower-selfhost
author: [zhaopuming]
created_at: 2026-09-03
updated_at: 2026-09-04

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-017]     # 自举（终点形态:塔顶自持回路）

affects: [aavm]
current_step: 5
total_steps: 16
---

# [PLAN-532] aavm 塔顶自举：aavm 编译含自身的 lib（自持回路闭合）

## 变更摘要

GOAL-017 的终点形态：**aavm（Auto 写）编译含自身的七文件 lib 并执行，
产物再能编译 lib——自持回路闭合**。前置已达成（525 验收 4：泛型塔顶
前置 W0 盘点清单逐项覆盖；方法/is-struct/VBool/闭包全波交付）。

两形态并进（W0 定案权重）：

- **形态 A（目标语言塔顶）**：`ev_run_files`/`cg_compile_files` 吃
  **lib 源本身**——lib（13633 行/448KB,现用语法面=中阶+高阶子集）经
  aavm 自身编译执行,W1 基线即"lib 跑 lib"：aavm 编译 lib 产出的
  编译器再编译 corpus——**N 阶自持**。
  〔**2026-09-04 用户裁定:形态A 解释栈嵌套撤销**——行业 stage 惯例
  =编译器递归+原生二进制,无解释器嵌套;验证架构改为原生代际对拍,
  详见附录 B 修订。aavm 正确性主判据=模块路径语料+M4 字节级对拍;
  自举闭合=a2r 原生 aavm exe 跑 probe/corpus+自编译代际一致〕
- **形态 B（既有⑤腿强化）**：AA2R 自译整 lib → Rust 二进制跑 corpus
  （434 G2 既有）——本计划将其从"间歇快死"（P525-5,~879KB 程序贴线）
  救稳为**常态绿**。

**W0 是全计划成败关键**：lib 源可编译性盘点（gap 清单——525 的前置
核验是"语料覆盖",lib 源本身仍可能有子集外形态:f-string 完整面/方法体
`self.` 书写约定/语句位简写边界等）+ 塔顶稳定性定性（P525-5）——
**盘点结果决定波次形状**（gap 小则直接补,大则拆"补缺前置计划"）。

**开工前置（硬）**：Plan 531 archived（P523-2② 转译版 struct 字段表
洞在塔顶验证链上,不清则链不干净）。

## 目标

1. **lib 源经 aavm 自身编译执行**：`ev_run_files`/`cg_compile_files`
   吃 lib 源（或其规范化变体,W0 裁定）——lib 编译 lib 的产物能跑
   corpus 且输出与宿主参考一致。
2. **自编译代际对拍（2026-09-04 用户裁定重定义,原"N 阶解释栈嵌套"撤销）**：
   a2r 原生 aavm exe（aavm.at+lib 转译产物,⑤腿已有）运行简单 Auto
   代码（probe/corpus）与宿主一致;其自编译代（该 exe 编译 aavm.at+lib
   →再经 a2r 成 exe）跑 corpus 输出与一代一致（至少两代;固定点留档）。
   每代均为原生二进制,零解释器嵌套。
3. **⑤腿塔顶稳定化**：P525-5 间歇快死定性/缓解,矩阵⑤腿常态绿
   （或结构化替代判据升级——W0 裁定）。
4. gap 清单逐项处置留档（aavm 扩语法 vs lib 规范改写,逐项裁定理由）。
5. 全程三件套+三面闸+四路+矩阵保护网;塔顶判据新增（自持回路三件套
   形态,W0 设计）。

### 非目标（Out of Scope）

- Auto 版其他语言转译器（塔顶后再议）;生成器 yield（P525-3 延后维持）;
  宿主语言语义变更;塔顶性能优化本体（只做稳定性定性,优化按需另立）;
  aavm 目标语言能力面扩张超出 lib 所需（盘点清单外的语法不做）。

## 架构方案

```text
W0 盘点+定性(成败关键)      W1 gap 补缺            W2 一阶自持           W3 N 阶+稳定化+收尾
──────────────────       ─────────────         ─────────────         ────────────────
lib 源可编译性盘点    →   gap 逐项处置       →   aavm 编译 lib     →   三阶自持验证
(逐文件 ev_run_files) →   (扩语法/改写裁定)  →   产物跑 corpus    →   ⑤腿稳定化
P525-5 定性/缓解      →   塔顶判据三件套     →   输出=宿主对拍     →   固定点留档+复审
(间歇快死根因)           设计                     一致                    归档
```

**关键设计约束**：

- **宿主为规范**：自持各阶输出以宿主参考（run_with_capture）为 oracle;
  N 阶输出一致 = 判据。
- **gap 处置裁定原则**：缺省 lib 规范改写（成本/风险低于 aavm 扩语法
  时——改写不改变行为,闸门全绿即证）;仅当改写破坏"一对一 Rust 对译"
  风格或改动面过大时 aavm 扩语法。逐项留档理由。
- **双轨不动**：拼接消费面（harness/parity/聚合）继续剥离路径——
  塔顶走模块路径（`auto/aavm.at` 位置参数形态,531 清障后全链可用）。
- **塔顶三件套**：`tower.at`（自持回路语料）+ expected.out（N 阶输出
  一致）+ expected.rs（AA2R 对 lib 的自译产物锚——⑤腿金样化,W0 设计
  是否可行）。

## 需求分析与背景调查

（取材：525 归档（验收 4 塔顶前置/步骤 16 核验）、P525-5、
lib-modularization-map（DAG/双轨）、aavm.at CLI 入口（524 位置参数/
531 清障）、四路 runner/三件套基建（523））

### 基线（开工时复测留档）

531 archived 后全绿面;lib 规模（13633 行/448KB 源,⑤腿拼接程序
~879KB）。

### W0 盘点方法论（预案,执行时细化）

1. **lib 源可编译性**：逐文件（依赖序）`cg_compile_files(auto/lib/...)`
   + `ev_run_files` 吃 lib 源本身——失败点即 gap;产出 gap 清单
   （语法形态/文件/行位/建议处置）。预期 gap 候选：f-string 嵌套/格式
   细节、方法体内书写约定（`self.` 前缀形态）、`is` 臂值语义边角、
   List 方法全集覆盖度、可能的主/main 形态差异。
2. **P525-5 定性**：⑤腿快死复现配方（负载窗口/连跑）+ 根因方向
   （内存/句柄/超时线）+ 缓解选项（分片构建/超时参数/程序瘦身——
   lib 规范化变体?）。
3. **塔顶判据设计**：自持三阶流程脚本化（tower runner）+ 三件套形态。

### 风险与对策

| 风险 | 对策 |
|---|---|
| gap 清单超预期大（lib 与子集差距大） | W0 即裁定拆"补缺前置计划"（511 先例形态）,本计划缩为自持本体;硬闸=盘点报告评审 |
| N 阶自持的发散（阶间输出漂移） | 逐阶对拍定位首个分歧位;宿主 oracle 三角校验 |
| ⑤腿快死根因深层（VM 内存/资源） | 结构化替代判据升级（四路+语料腿全绿为⑤腿等价证据,525 先例）+观察项维持;不阻断自持本体 |
| 塔顶程序规模增长（gap 补缺后再膨胀） | 每折叠点⑤腿时长趋势;超线则规范化变体（W0 预案） |
| lib 改写破坏行为 | 双轨+全套闸门（19+N 件）+矩阵硬闸;改写逐文件一提交 |

## 详细设计

（W0 盘点后细化——此处纲要;执行时波次内展开,镜像 525 各波映射表先例。）

- **W1 gap 补缺**：逐项处置（aavm 扩语法 or lib 改写,裁定留档）;
  每项红先行（三件套/语料）→ 绿;lib 改写走双轨闸门。
- **W2 一阶自持**：`ev_run_files("auto/lib/engine.at 主入口")`——
  aavm 编译 lib 全链;产物（指令流）执行=编译器运行;该编译器再编译
  corpus b07 → 55 对拍宿主。塔顶 runner 脚本化。
- **W3 N 阶+稳定化**：三阶链（aavm→lib¹→lib²→corpus 输出一致）;
  ⑤腿稳定化处置落地;固定点性质注记（N 阶输出稳定即自持达成）;
  tower 三件套落盘。

## 测试设计（TDD）

- **保护网**：531 archived 后全绿面（四路/tv/tt/矩阵）全程不破绿。
- **红先行**：W0 盘点报告=最大的"红清单"（gap 逐项转为语料/改写
  任务）;每项红→绿。
- **塔顶判据**：tower runner 三阶输出一致（对拍宿主）;⑤腿常态绿
  （或替代判据升级留档）;tower 三件套。
- **命令**：全套（四路/三面闸/矩阵/tf）+ tower runner
  （`scripts/aavm_tower_check.sh` 或 runner 扩展,W0 定载体）。

## 验收标准

1. **自举闭合（2026-09-04 用户裁定,原解释栈三阶塔撤销）**：a2r 原生
   aavm exe 运行简单 Auto 代码（probe/corpus 代表集）输出与宿主参考
   一致;其自编译代（exe 编译 aavm.at+lib →a2r→exe）跑 corpus 输出与
   一代一致（原生代际判定表留档）。aavm 编译器正确性主判据=模块路径
   语料全绿+M4 字节级对拍宿主（含 lib 五文件静态字节对拍——不执行,
   纯编译产物差分）。
2. gap 清单 100% 处置（逐项裁定留档:扩语法/改写/显式豁免+理由）。
3. ⑤腿：常态绿或结构化替代判据升级+P525-5 定性结论留档。
4. tower 三件套落盘;保护网零破绿;`cargo tf` 绿（基线红归属注明）。
5. GOAL-017 终点形态注记入 project.md（自举达成口径:双目标+塔顶+
   AA2R 自译——434 已备+本计划补目标语言塔顶）;无静默丢弃。

## 执行步骤
（原子任务;W0 在 master（纯文档+探针）,实现 in worktree
`.worktrees/plan-532-dev`;折叠点（5/8/11/14）矩阵+CI 绿后合入;
**开工门禁：Plan 531 archived**）

### W0 盘点+定性（master）

1. [✅ 已完成] lib 源可编译性盘点（2026-09-04;6 类 gap 全部真 lib 干净复现,
   详见附录 A;硬闸判定:6 类 ≤15 项阈值且无结构性形态 → **不拆前置计划,
   W1 原计划处置**;首错阶梯 + 影子补丁法发现:影子树 in-place 补丁对
   CG 位置构造/scanner 有回归风险 → W1 改在 worktree 真 lib 上逐项红→绿）。
2. [✅ 已完成] P525-5 定性(2026-09-04):复现配方
   `cargo test -p auto-lang --lib --features test-vm-files aavm2_m5
   -- --nocapture` 连跑×2 于空闲窗口:4/4+4/4 全绿,~112s/轮
   (scratch/p532/m5_repro.log 在案),快死未复现——与 525 记录一致
   (负载窗口敏感:折叠点全矩阵连跑+并行任务时 rc=1 空输出快速返回,
   空闲窗口健康)。根因方向:环境资源族(~879KB 拼接程序×57 语料连跑
   内存/提交峰值贴线;进程级资源分配失败,非 panic 非 assert)。缓解
   选项:空闲窗口复跑(已验证有效)/矩阵分片(实质已按腿分进程)/
   lib 规范化瘦身(塔顶超线再启)。**处置建议(W3 输入)**:结构化
   替代判据升级+观察项维持,塔顶不因⑤腿阻断。塔顶判据/三件套/runner
   载体设计定案见附录 B。

### W1 gap 补缺（worktree）

3. [✅ 已完成] 红先行语料三件落盘:corpus_m4/b58_str_methods(str 方法族,
   宿主 oracle 六行在案)+corpus_use/008_root_anchor(G0,shared_dep)+
   corpus_use/009_enum_import(G4,deptest3 复现入语料);三红确认、修后
   双闸绿(M4 反汇编逐字节+M5 行为),随各 gap 提交入 worktree。
4. [✅ 已完成 2026-09-04] 逐项处置——**19 族 gap 全清**(G0-G19:G0 resolver
   CWD 镜像/G1+G2 str natives 五方法/G3 for→while×8/G4 枚举播种/G6 fail
   位置/G7 顺次累积播种/G8 调用旗标卫生/G9 类型通道族/G10 前向引用
   pass0+泛型全串+快照清洗/G11 跨模块方法 defer/G12 fnret 侧车/G13 LHS
   get 中转×28/G14 enum 前置清扫/G15 File.read_text/G16 len 兜底+mod
   避让+审计导入/G17 链式跳类型续传/G18 vpay 载荷类型侧车/G19 ⑤腿
   replace 转译臂+转译兼容改写);每项红→绿,语料闸门 13/13 全程零回归。
5. [✅ 已完成 2026-09-04] 折叠点①——**lib 七文件全链经 aavm 编译+执行
   通过**(token/lexer/parser/typeinfo/codegen/engine/a2r;master 合并
   1642be9ea,worktree 已回同步)。门禁:tv 3559/3559+tt 3746/3746 全绿;
   tf 3397/3399(双红=存量基线 docs_gen kitchen_sink/schema_drift_fence,
   525 在案非 P532 归属);⑤腿 compile_corpus 58/58+use_corpus+goldens ✓。

### W2 一阶自持（worktree 续）

6. [▶ 进行中 2026-09-04] tower runner 实现——〔2026-09-04 裁定:runner
   载体改判原生代际对拍脚本(附录 B 修订),嵌套塔 tower{1,2,3}.at
   退役留档〕三件套+runner 已落盘
   (worktree 提交);**模块路径二阶失真定位与修复进行中**(用户裁定
   (a)+架构指令:拼接系错误实现,修后默认改模块加载):
   - **根修①已落地(engine.at Call 臂 args_stack 深度对齐写)**:原
     push-only 无弹出+Ret 按深度索引读,残留条目使深度≥2 的恢复错槽
     →cur_args 错→参数寻址 bp-cur_args+rel-1 落随机栈槽。修后:m1(tokenize=11 正常)/
     m5x/w2/w3/mainS 全绿;**宿主模块路径 5-10% 静默空输出 flake 同根
     治愈**(deptest2 12/12,修前 9-10/12);gate aavm2_m 13/13 零回归。
   - **根修②已落地(2026-09-04,commit 103c1e637,codegen.at loop_enter
     帧槽位深度对齐写)**:brk_js/cont_js/cont_tg 无条件 push 而按
     loop_depth 索引读写,首循环退出后残帧永居 brk_js[0],后续同层循环
     全部读写残帧——历史 break 洞被逐次重 patch(实证:洞 4215 被 59 次
     重 patch),末位循环(cg_compile_mod_seeded 尾循环)的 end 覆盖
     cg_expr pratt break 跳→野跳 37898/37895→随机槽→get.field on
     non-instance(即残留② tw_c1 系症状本因;dump 路径 "Module not
     found: 11.11.11" 同族)。修法镜像根修① set-else-push。门禁:aavm2
     全家 20/21 绿(m4/m5/m3/m2/m1 byte 级对拍+engine+use corpus+goldens
     全绿;1 红=compile_corpus 重跑即绿=既有 flake)。
   - **残留③已登记(下会话)**:模块路径内层编译语义错——tw_c1 修②后
     越过野跳,tw_c1b 实证内层编译 "fn main() { print(7) }" 产出
     "expected eos, got LParen @line 11"(单行源无 line 11→@tokline
     读到杂值;print 调用臂未命中,primary 提前返回;拼接路径同源同参
     编译干净=拼接/模块行为分歧实证);tw_dump1 路径 cg_use_scan 读
     use 行得 "11.11.11"(疑似 .text/.kind 字段错位)。诊断资产:
     scratch532/{tw_c1,tw_c1b,tw_dump1,tw_p*,tw_q*,tw_r*,tw_s*}.at+
     /tmp/{c1d4,c1d6,c1d7,c1d8,c1d9}.err(VM CALL/RET/PRO 全程 trace+
     FRAME bp 链 walk+LOOP 帧深度账+JMP 窗口账)+win 定位法:模块路径
     单次运行 6-10 分钟(非挂死;45-120s 超时会误杀),stderr 需
     AUTO_PRINT_STDERR 镜像(vm_print 补丁,已随诊断回退)。
   - **静默空输出 flake 未根治**:净 lib 构建 3 连跑 rc=0 空输出/rc=1
     空输出/rc=124 超时各一,时长 348-574s 漂移——fix① 治愈声明
     (deptest2 12/12)不完整,与残留③并案查。
   - **残留③清零进行中(2026-09-04 晚,方法学升级:静态字节差分取代
     运行时 trace)**——新临时闸门 `test_aavm2_p532_lib_static_diff`
     (aavm2_m4.rs,worktree 未提交):同一 probe(scratch532/libdiff/
     main.at)宿主 compile_and_link_multi(加 root source dir)vs aavm
     codegen_dump_files(拼接 harness 可信执行)归一化逐行对拍,.line
     元数据脱敏计数。**四修已落盘(commit 828125e04)**:③ cg_fstr 空
     字面段镜像宿主 lexer(每遇 $ 无条件发射,仅末尾空抑制;原跳空段
     →parts/字节序双分歧);④ cg_if else-if 语义修复(JmpZ 立即回填
     下一分支 cond 起点,仅真跳留链尾——原版 else if 中间分支永不可达
     ,0b/0x 词法错,语料无 else-if 形态故漏网);⑤ serialize 字符串转义
     镜像宿主 {:?};⑥b cg_is_arm_body 包块作用域(tokenize locals
     34→16)。差分首分歧单调前移 1498→2047→2846;门禁 m4/m5/use_
     corpus/goldens 全绿(大套件并行 flake 重跑即绿)。
   - **残留③末段(⑥c,2026-09-05 收窄)**:差分口径升级为语义等价
     (.line 整行剔除/jmp-call 目标抽象/fn.prolog-reserve 容量豁免/
     **字段与全局名经池解析**——双侧一致升级,serialize+normalized_dump
     同步,commit e622d1675 含根修⑦ is 多模式测序 swap_remove 镜像 +
     ⑧ 字段/全局名渲染)。语义口径下指令流唯一残余=**绝对槽号区域
     差**(tokenize 注释臂 aavm +4,locals 16 vs 14)——根因=两侧作用域
     推入结构层级不同(宿主 while 体/is 臂嵌套更深:d5/d6/d8 vs aavm
     d4/d5/d6),完全镜像需作用域架构专项对齐(非热修);帧多预留 2 槽
     自洽且零语义。差分测试(test_aavm2_p532_lib_static_diff)留
     worktree 未提交(红=该已知差),作用域对齐落地后转绿随闸门提交。
     诊断资产:scratch532/libdiff/、%TEMP%/p532_{rust,aavm}.txt、
     scratch/p532/diff_dumps.py(离线对齐器)。
   - **残留③末段(⑥c,2026-09-05 收窄)**:差分口径升级为语义等价
     (.line 整行剔除/jmp-call 目标抽象/fn.prolog-reserve 容量豁免/
     **字段与全局名经池解析**——双侧一致升级,serialize+normalized_dump
     同步,commit e622d1675 含根修⑦ is 多模式测序 swap_remove 镜像 +
     ⑧ 字段/全局名渲染)。语义口径下指令流唯一残余=**绝对槽号区域
     差**(tokenize 注释臂 aavm +4,locals 16 vs 14)——根因=两侧作用域
     推入结构层级不同(宿主 while 体/is 臂嵌套更深:d5/d6/d8 vs aavm
     d4/d5/d6),完全镜像需作用域架构专项对齐(非热修);帧多预留 2 槽
     自洽且零语义。差分测试(test_aavm2_p532_lib_static_diff)留
     worktree 未提交(红=该已知差),作用域对齐落地后转绿随闸门提交。
     诊断资产:scratch532/libdiff/、%TEMP%/p532_{rust,aavm}.txt、
     scratch/p532/diff_dumps.py(离线对齐器)。
   - 架构指令执行面(拼接→模块加载默认化)在残留③清零后启动
     (aavm2_lib_source 族消费面,超出本计划部分立项)。
7. [ ] 原生一代对拍：a2r 原生 aavm exe 跑 corpus（代表集）与宿主一致
   （⑤腿 58/58 已绿,补代表集判定表固化）。
8. [ ] 折叠点②：自举闭合判定表留档（验收标准 1 原生代际形态）→ 合入。

### W3 N 阶+稳定化（worktree 续）

9. [ ] 自编译代际对拍（原生,两代）：原生 aavm exe 编译 aavm.at+lib
   →a2r→exe² →exe² 跑 corpus 与一代一致;首个分歧位定位机制。
10. [ ] ⑤腿稳定化处置（按 W0 定案落地;或替代判据升级登记）——裁定后
    ⑤腿升格为主判据通道,常态绿为硬要求。
11. [ ] 折叠点③：代际判定表+⑤腿处置留档 → 合入。
12. [ ] 验收资产落盘：原生代际判定表+固定点性质注记；嵌套塔
    （tower{1,2,3}.at+runner）降级为里程碑层资产保留——@ignore/脚本
    隔离,平时零开销,大版本升级人工跑一次(附录 B 二次裁定)。
13. [ ] 文档回写：project.md GOAL-017 终点注记/队列③核销/divergences/
    KNOWN-DEBT（P525-5 处置）。
14. [ ] 折叠点④+复审（/auto-plan:review）→ tf → status: reviewed。
15. [ ] merge 沉淀归档。
16. [ ] （里程碑广播）自举达成口径汇总：双目标+塔顶+AA2R 自译全景。

## 复审记录

## 附录 B:W0-2 塔顶判据设计定案(2026-09-04)

**〔2026-09-04 用户裁定修订(生效形态)〕**——原解释栈嵌套三阶塔撤销,
理由:行业自举验收=编译器递归+原生二进制(stage 惯例),无解释器嵌套;
解释栈每次运行分钟级,不可入开发循环。修订后判据:

- **自举本体**:a2r 原生 aavm exe(⑤腿,已有)运行 probe/corpus
  代表集,输出与宿主参考一致。
- **自编译代际固定点(原生)**:exe¹=a2r(aavm.at+lib);exe²=a2r(exe¹
  编译的 aavm.at+lib);exe¹/exe² 各跑 corpus 输出一致(≥两代,固定点
  留档)。每代原生二进制,秒-分钟级。
- **编译器正确性主判据**:模块路径语料全绿+M4 字节级对拍;lib 五文件
  `codegen_dump_files` 静态字节对拍宿主(纯编译产物差分,不执行)。
- **ev_exec(Auto 写 VM)**:直测语料闸门(aavm2_m5 engine 族),不靠嵌套。
- 判定表载体:`scripts/aavm_tower_check.sh` 改写为原生代际对拍脚本。
- ⑤腿自裁定起升格为主判据通道,其稳定化(W3 步骤 10)为硬要求。
- **〔2026-09-04 二次裁定:嵌套塔降级为里程碑层,不撤销〕**tower{1,2,3}.at
  + 嵌套 runner 资产保留,平时不进任何闸门(不进 t/tf/ta,不进折叠点),
  仅**大版本升级时人工跑一次**作为"解释栈零漂移"最终验收(镜像 466
  churn 层"平时排除、特定闸门运行"分层先例)。里程碑能绿以模块路径
  残留③最终清零为前提——塔同时充当模块路径长期健康的远期驱动;
  本计划折叠验收不依赖塔。

**〔以下为原定案,留档备查〕**

**tower runner 载体**:独立脚本 `scripts/aavm_tower_check.sh`(测试设计
节预告形态;CI/本地双形态,镜像 aavm4_check.py shell-out 模式;不并入
既有脚本——判定表独立留档)。

**三阶自持链形态**(形态 A 为主判据;宿主 oracle=run_with_capture):

- **阶0(宿主基线 R0)**:宿主直跑 corpus 代表集(b01..b55 全 57 件或
  W2 裁定的代表子集)输出汇总。
- **阶1(一阶)**:语料 `tower1.at`:`fn main` 读 corpus 源文 →
  `ev_run(corpus)` → print。宿主 `auto auto/aavm.at auto/lib/…`
  路径下运行 = 宿主解释的 aavm(Auto 编译器)执行编译+运行。输出 O1。
- **阶2(二阶)**:`tower2.at`:读 tower1 源文 → 经 use 拉起的 lib 编译
  器 `ev_run(src)` —— 即"lib 编译『lib 编译器跑 corpus』"。输出 O2。
- **阶3(三阶)**:`tower3.at` 同构再进一层。输出 O3。
- **判据**:O1==O2==O3==R0(逐行;首个分歧位定位=阶号×语料名交叉
  报告,runner 判定表落档)。固定点性质注记:N 阶输出稳定即自持。

**tower 三件套形态**(待澄清#4 预案落定):

- `test/vm/aavm2/tower/tower{1,2,3}.at`(自持回路语料,分层 driver);
- `tower.expected.out`(=R0,宿主生成,bless 再生);
- 第③件(AA2R 自译 lib 产物锚,~879KB)——**裁定:hash 锚形态**
  (`tower.expected.rs.sha256` 内容寻址 + 生成说明注释;全量金样评审
  摩擦过大,879KB 不可行,W0 评估结论)。

**⑤腿判据口径**:常态绿(根治)或结构化替代判据(四路+语料腿全绿
=等价证据,517/525 两度先例);W3 按 P525-5 定性结论裁定,塔顶本体
不因⑤腿阻断。

## 附录 A:W0-1 lib 源可编译性盘点报告(2026-09-04)

**方法**:依赖序逐文件真 lib 探针(`auto auto/aavm.at <lib 文件>`)+
剥 use 行单文件编译探针 + 影子树补丁阶梯(首错→最小补丁→下一首错)
+ 静态方法面普查。复现资产:`scratch/p532/deptest3/`(G4 枚举导入)、
`scratch/p532/deptest2/`(多 use 对照组+跨模块裸名分歧)、
`scratch/p532/deptest/`(dep 错误浮出对照)、`scratch/p532/rebuild_flat.py`
(影子管线,W1 参考;其"0"输出因扫描回归作废,仅首错观察有效)。

**gap 清单(修订版 2026-09-04 二次核验后;G4 改性/G5 撤销,详见下)**:

| # | 形态 | 位/规模 | 建议处置(裁定候选) |
|---|---|---|---|
| G0 | 多段根锚定 use 解析缺失(resolver 只做引用文件目录拼接,缺宿主 CWD 直探) | `cg_resolve_into`(codegen.at:3243);挡全部 lib 文件多文件编译;repro:`auto auto/aavm.at auto/lib/lexer.at`→Module not found | aavm 补齐(镜像宿主 load_module_inner 探测序;塔顶只需 CWD+dir 两级) |
| G1 | `str.len()` 方法(str 接收者)——分派只路由 List;engine ArrLen 已双支持 | lib 源 67 处;repro:剥行单文件编 lexer.at | aavm 补:str+len→ArrLen 分派臂 |
| G2 | str 方法族 char_at/slice/replace/trim(65/101/17/3 处)——native 表仅 List 族(id 100/101/103/106/107/112+print) | lexer.at:45/225-228/421、a2r.at:497-511 等;静态确证(分派无 str 臂+natives 无 id) | aavm 补:CallNat id 110-113 + engine 执行臂(引擎自跑宿主同名内建;影子已验证可行性) |
| G3 | `for <cond> {}` 裸条件循环——**parser 支持而 codegen cg_for 只实现 for-in**(parser.at:1630 续解析臂 vs codegen.at:2866 fail) | 8 处(lexer.at 全部:`for scanning`×7+`for p < len`×1);AA2R ⑤腿已支持(434 G2 绿在案) | **lib 改写 for→while**(裁定原则:行为同一,8 处机械改写,while 双轨全支持) |
| G4 | **跨模块枚举导入**——dep 枚举类型作表达式限定符(`Color.Red(7)`)报 unknown ident;007 语料只盖 struct | repro:deptest3(宿主 7/70,aavm unknown ident: Color);挡 lexer.at 全部 `TokenKind.X` | aavm 补:枚举类型经 use 导入的表达式限定符解析(struct 走 tys 播种,枚举缺同路) |
| G6(辅) | 编译错误无位置信息(fail 不带行号) | codegen.at fail 族 | aavm 补:@tokline 注入(影子已验证;W1 起 gap 定位/塔顶调试提效) |

**已撤销(影子伪影,2026-09-04 复核)**:原 G4"多 use 丢失"、G5"dep 错误被吞"
均为影子树扫描回归的伪象——真 lib 上多 use 三变体 85==宿主、baddep 错误
正常浮出(link 的 err 收集有效,复现在案)。

**分歧注记(非阻塞,登记 divergens 候选)**:跨模块裸名调用(无 use 导入)
——宿主链接期全局解析(宽松),aavm 编译期 import_symbol 严格判定。
lib 源 use 声明完备不受影响;塔顶语料需遵守显式导入约定。

**硬闸判定**:6 类 ≤ 阈值 15 项;无结构性形态(逐项镜像宿主语义,补丁
面小、复现干净;G3 为 8 处机械改写)。**不拆前置计划**,W1 按原计划
逐项红→绿。注意事项:(a) 逐文件全链编译状态在 G0-G5 修复前不可知
(W1 逐项解锁后重盘);(b) 影子树教训——CG 位置构造、scanner 对补丁
敏感,W1 必须 worktree 真 lib + 全套闸门,不做 in-place 影子补丁。

## 待澄清事项

1. **gap 超预期拆分**（W0 步骤 1 硬闸）：盘点报告若显示 lib 与子集
   差距大（预估 gap >15 项或含结构性形态）→ 拆"塔顶前置补缺"独立
   计划,本计划余波顺延——硬闸评审（用户参与）。〔已裁定:6 类 ≤15
   不拆,附录 A;实际处置 19 族,W1 内消化〕
2. **lib 改写 vs 扩语法边界**（W1 各项）：裁定原则已立（改写优先）;
   逐项留档;争议项（破坏一对一风格/大改写面）升级用户裁定。
3. **⑤腿处置口径**（W3 步骤 10）：常态绿（根治）vs 结构化替代判据
   （四路+语料腿等价证据,525 先例）——按 W0 定性结论裁定;塔顶自持
   本体不因⑤腿阻断（替代通道已在 517/525 两度验证）。〔W0 定性结论
   已留档:结构化替代判据+观察项维持〕
4. **tower 三件套 ③ 件可行性**（AA2R 自译 lib 产物锚,~879KB 金样）：
   尺寸/评审摩擦 W0 评估;不可行则 ③ 件以 hash 锚替代（内容寻址）。〔已
   裁定:hash 锚形态,附录 B〕
5. **W2 模块路径二阶失真**（2026-09-04 登记,步骤 6 阻塞）：拼接路径
   全绿但模块路径(`auto auto/aavm.at`)执行的 lib 对特定形态产出错编译
   (丢 fn 体/enum 表达式位失联/get.field on non-instance;探针 tw_* 在
   案)。**〔2026-09-04 用户裁定〕选 (a) 定位修复根因**——并给出架构
   指令:**拼接方式是早期 AI 误解造成的错误实现**,除平台分歧适配链
   (http.at/http.vm.at/http.rust.at 形态)外,多 .at 文件应为独立模块、
   由 VM 正常模块加载,不得直接拼接;模块路径失真修复后,**默认拼接
   实现改为默认模块加载**(aavm2_lib_source 族与 M4/M5 harness 的
   拼接消费面切换——超出本计划 W2/W3 范围的部分立项跟进)。附:宿主
   模块路径 ~5-10% 非确定性静默空输出(pre-P532 实证)同此定位一并
   查清。〔2026-09-04 进展〕根修①②落地(见步骤 6);**残留③在案**:
   内层编译 print 调用臂未命中(eos@line 11 杂值)+cg_use_scan 读
   "11.11.11"(疑字段错位)+静默空输出 flake 仍在(净 lib 3 连跑
   rc=0 空/rc=1 空/rc=124 各一)。每层根因独立,同"深度索引 vs
   append-only"疑族待下一层核。
