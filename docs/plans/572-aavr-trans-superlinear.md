---
plan_id: PLAN-572
status: execution_done        # drafting → executing → execution_done → reviewed → archived
feature_name: aavr-trans-superlinear
author: [zhaopuming]
created_at: 2026-09-06
updated_at: 2026-09-06(T5b 收口,execution_done)

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-017]      # 自举(P532 步骤 9 的 exe² 代际对拍依赖本修复)

affects: [aavm]                # auto/lib/a2r.at 为主;消费面 ⑤腿 harness
current_step: 6
total_steps: 6
---

# [PLAN-572] AA2R 转译超线性根修(P532 步骤 9 阻塞解除)

## 变更摘要

**问题**:AA2R(`auto/lib/a2r.at`,Auto 写的 Auto→Rust 转译器)在转译含
`auto/lib/codegen.at` 的源时呈挂死级超线性。⑤腿语料(corpus 小源)
58/58 从不含此形态故从未暴露;P532 步骤 9(原生自编译代际对拍)的
exe¹ `--trans` 吃 lib 拼合源(462KB)3h+ 不完,exe² 无法构建,自举
代际验证被阻塞(见 P532 待澄清⑥,2026-09-06 登记)。

**实证阶梯**(2026-09-06,P532 会话调查留档):

| 输入 | 规模 | exe¹ --trans 耗时 |
|---|---|---|
| token.at 单文件 | 9KB | 0.0s |
| parser.at 单文件 | 83KB | 1.0s |
| a2r.at 单文件(自转译) | 152KB | 4.3s |
| engine.at 单文件 | ~110KB | 0.2s |
| **codegen.at 单文件** | **219KB** | **>7min 不完** |
| lib 七文件拼合源 | 462KB | >2.5h(两代 exe 各一次,被回收) |

**最小化阶梯**:type CG 重建(fields+static new+方法群逐个递增)
——**27 方法 0.1s / 28 方法(加入 `emit_store`)→ >7min 悬崖**
(~300 倍/方法,指数族特征)。控制组:CG 后段 16 方法(type X 包装)
组合秒级;单方法(16 个逐一)全秒级;三角(fields+new+emit_store)
秒级——**方法群跨段组合触发**(前 27 含作用域/var_ty 族+loop_jump,
第 28 加 emit_store 引爆),非单方法/单形态问题。

## 目标

1. **根因定位到单点**:在 ar_prescan_fn / ar_scan_mutations /
   ar_fixpoint_mutates / ar_scan_self_calls / ar_scan_method_writes
   族的组合复杂度中指认具体算法与触发交互(可复现的最小触发集)。
2. **根修**:codegen.at 单文件转译回到同量级线性(参照 a2r.at
   152KB=4.3s;codegen.at 219KB 应 <10s 量级)。
3. **下游解锁**:exe¹ --trans 拼合源(462KB)有限时间完成(量级
   留档),P532 步骤 9(exe² 构建+二代代际对拍)恢复执行并收口。
4. 保护网零回归:⑤腿 58/58+use_corpus+goldens+M4/M5(改动作用域
   对应的 `cargo taa` 档)全绿。

### 非目标(Out of Scope)

- AA2R 转译产物的语义变化(超线性是性能问题,产物应对拍不变——
  小源 golden 前后一致即证);aavm 其他性能面;P532 步骤 9 本体的
  代际判定(修复后回到 P532 执行,不在本计划内)。

## 架构方案

调查型计划(根因未知,步骤 1-2 为定位,3 为修复,4 为回归):

```text
T1 复现器固化         T2 最小触发集二分        T3 根修
─────────────       ─────────────────       ─────────────
scratch 阶梯脚本  →   27 方法群逐一减一    →   定位点修复
(N=1..28 矩阵)       找最小引爆组合          (算法/缓存/终止性)
                                             ↓
                    T4 闸门回归               T5 P532 步骤 9 解锁
                    ─────────────           ─────────────────
                    taa 作用域+⑤腿           exe² 构建+二代对拍
                    +小源 golden 不变         (回到 P532 执行)
```

**已知嫌疑面**(P532 会话排除记录):
- ✗ 非巨型 is 匹配(50 臂 is 函数秒级);
- ✗ 非大 enum(enum OpCode 60 变体单独秒级);
- ✗ 非字符串拼接/self 引用(均构造过对照,秒级);
- ✓ 指向 **prescan 族跨方法组合**:`ar_fixpoint_mutates` 的不动点
  迭代(每轮全方法重扫)+ `ar_scan_mutations` 的用户方法 mutates
  查表(method_find/vty)嵌套;或 `ar_prescan_fn` 对每 fn 的
  scan_mutations 调用链在该方法群形态下退化。

## 需求分析与背景调查
（从 docs/specs/overview.md 与相关 module spec 取材）

- **上游**:P532 残留③静态差分清零(2026-09-06,SEMANTICALLY
  IDENTICAL)后,aavm 编译面已与宿主全等;本问题在**转译面**(
  AA2R),两者独立。
- **消费面**:⑤腿 `build_aavm_rust_bin`(宿主 merged 转译构建
  exe¹)**不受影响**(宿主 Rust 转译器无此问题,秒级);受影响的
  是 exe¹ 内 AA2R 的运行期转译(--trans 模式)——即"exe¹ 编译
  aavm.at+lib"这条自编译链。
- **复现资产**(P532 留档,scratch 形态):
  - exe¹:`%TEMP%/aavm2-bin-*/target/release/aavm2_bin.exe`(⑤腿
    内容寻址缓存,重建:`cargo test -p auto-lang --lib --features
    test-aavm test_aavm2_compile_corpus`,注:Plan 568 后⑤腿测试
    已入 test-aavm 档);
  - 阶梯构造:python 切 `auto/lib/codegen.at` L82-819(type CG
    完整区),重建 `type CG { fields+static new+前 N 方法 }` 源,
    N=27/28 为悬崖边界;
  - 拼合源:`%TEMP%/p532_gen2_concat.at`(剥 use 七文件+aavm.at
    依赖序,462KB);
  - P532 步骤 9 管道脚本:`scripts/aavm_native_gen_check.sh`
    (--skip-gen2 判定段可独立复跑)。

## 详细设计

（T2 定位后回填;此处为预案）

- **T2 最小触发集**:27 方法群逐一减一(27 个候选 × 转译一次),
  找出与 emit_store 共同引爆的最小集合;再对最小集做 token 形态
  归因(调用图/字段写/方法名重合)。
- **T3 修复形态候选**(按定位结果择一):
  - fixpoint 迭代上限/收敛加速(mutates 位单调,应 ≤M 轮;若实现
    有复位 bug 则修终止性);
  - scan 的查表缓存(method_find/vty 每调用线性扫 → 一次性索引);
  - prescan 对同一 token 区间的重复扫描合并。
- **产物不变性锚**:任一小源(如 corpus b44 方法族)修复前后
  `--trans` 输出逐字节一致(转译产物与性能无关的证明)。

## 测试设计(TDD)

- **红先行**:T1 阶梯脚本固化为回归资产;codegen.at 单文件转译
  超时(如 >60s 判红)即红——修复后转绿。
- **闸门**:⑤腿 compile_corpus 58/58+use_corpus+goldens;改
  a2r.at → 按作用域映射 `cargo taa aavm2_a2r aavm_at_mode`
  (AGENTS.md AAVM/AA2R Test Tier);review 前裸 `cargo taa` 兜底。
- **产物锚**:小源转译 golden 前后一致断言。

## 验收标准

1. 根因单点指认(算法+触发交互),留档于本计划复审记录。
2. codegen.at 单文件(219KB)`--trans` <60s;lib 拼合源(462KB)
   有限时间完成(实测值留档;判据:小时内可完成即可,量级注记)。
3. 小源转译产物逐字节不变(golden 锚);⑤腿+作用域 taa 全绿。
4. **P532 步骤 9 解锁实证**:exe² 构建成功+二代代际对拍可执行
   (结果回 P532 记录,判定本身归 P532)。
5. 无静默丢弃;债项登记 KNOWN-DEBT(如有)。

## 执行步骤
（原子任务;调查在 master 纯探针可做,修复 in worktree
`D:/autostack/.wt/lang-572/auto-lang`;折叠点(T4 后)矩阵+CI 绿后合入）

1. [✅ 已完成] T1 复现器固化:scratch 阶梯脚本(N=1..28 矩阵+计时)入库
   `scratch/p572/`;codegen.at/拼合源两形态基准耗时留档。
   ——证据:`scratch/p572/ladder.py`+`baseline_predoc.txt`(commit 237d8443b);
   N=1..27 全绿 0.01-0.13s 平滑、N=28(emit_store)90s 超时红、
   codegen.at(176KB)/拼合源(504KB)均 90s 超时红——悬崖复现与 P532 实测一致。
2. [✅ 已完成] T2 最小触发集二分:27 方法群逐一减一,锁定最小引爆组合;
   token 形态归因(调用图/字段写/方法名);根因单点指认留档。
   ——证据:scratch/p572/t2_findings.md(commit bf34d4873)。最小引爆集
   ={emit, emit_store} 二方法;触发面=A(mutates=1,任意 self 写)+
   B(自身零写)调用 self.A(B 自带 self 写即 OK——g 变体);根因=
   **a2r.at L1084 ar_fixpoint_mutates 链式写回落值拷贝临时**(D25 范式
   违例,同坑 ar_prescan_ext L1288 注释在案,全文件唯一)→ mutates 位
   永不持久 → `while grew` 死循环(实为无限循环,非有限超线性)。
3. [✅ 已完成] T3 根修:定位点修复(in worktree);红(超时)→绿(<60s);
   小源 golden 产物不变锚。
   ——证据:commit 237839ffe(a2r.at L1082-1090 显式写回)。红→绿:
   N=28 阶梯 90s 超时→**0.15s**;codegen.at 176KB→**7.38s**(<10s 量级
   达标);拼合源 504KB→**61.29s**(P532 实测 3h+ 被杀,验收"小时内"✓)。
   golden 锚:旧 exe¹(106d6ff)vs 新 exe¹(ce20d8ee)corpus_a2r
   18/18 + corpus_m4 58/58 `--trans` 输出逐字节一致(corpus_use 多文件
   形态归 T4 测试档覆盖)。
4. [✅ 已完成] T4 闸门回归:⑤腿 58/58+use_corpus+goldens+`cargo taa
   aavm2_a2r aavm_at_mode`;折叠点合入。
   ——证据:scratch/p572/t4_gate.md(commit 40fd69a5d)。⑤腿
   compile_corpus 58/58 ✓;compile_use_corpus ✓;aavm2_a2r 2/2
   (goldens_check+main_dump;is_corpus 预存红剔除此时尚未判明,后由
   全量对拍覆盖);golden 76/76 逐字节。裸 taa 基点(f2ae1cb29)vs
   修复态:失败集 13/13 **逐名一致**(12 Windows 栈溢出环境族+
   charts_gallery 预存)——零回归;tf 两态 3460/3461 逐同款。
   折叠:master 8a21073c4(merge --no-ff),worktree 已回灌。
5b. [✅ 已完成] T5b exe² 5 缺陷续修(2026-09-06 用户裁定:本计划续修):AA2R
   发射缺口 2(E0308 实参 &str 强转缺@ev_run_files(argv[1])、E0382
   mc clone 注入缺@engine.at 切片)+ gen2 harness 3(prelude `mod IO`
   点号形态×2、双 main)→ exe² 构建绿→二代对拍执行(结果回 P532
   步骤 9);a2r.at 再改→作用域闸门+golden 锚复跑。
   ——证据:commit 7b4ca6052(fold master 211a12f21)。实际修 5+1 缺陷:
   a2r.at 四臂(process.args List<str> 契约/IO::read_line/push 非 Copy
   ident 克隆/fstr 字面大括号直通——第 6 缺陷为固定点判据暴露的自举
   转义逐代翻倍)+ 管道三处(concat 去 aavm.at 镜像⑤腿 lib-only/
   harness 补 --files/exe¹ 选取按 mtime 确定化)。红→绿:t5b_repro
   3/3;**P532 步骤 9 两判据全过:一代 8/8+exe² 构建绿+二代 8/8+
   转译固定点 PASS(自举闭合)**;golden corpus 76/76 逐字节不变
   (每刀双验);裸 taa 3612/3625 与基点 13 失败逐名一致、tf
   3460/3461(charts_gallery 预存同款)——零回归。留档:
   scratch/p572/gen2/(t5_unlock.md 追记+native_gen_table.md+
   errors_short.txt)。
5. [✅ 已完成] T5 P532 步骤 9 解锁:exe¹ --trans 拼合源完成→exe² 构建→
   二代对拍执行(结果回 P532 步骤 9 记录;判定收口归 P532)。
   ——解锁面(572 修复直效):拼合源转译 **61.29s**(470KB,修复前
   3h+ 挂死);一代判定表 8/8;管道推进至 exe² 段;回执已写 P532
   步骤 9(T5b 终版回执=判据全过,待 P532 翻牌归档)。证据:
   scratch/p572/gen2/t5_unlock.md。

## 复审记录

## 待澄清事项

1. **量级口径**(T5):拼合源转译"有限时间"的可接受上限(分钟/
   小时级)——影响是否需要额外性能优化(非本计划范围,超线性质
   级改善后按实测裁定)。
   ——〔2026-09-06 实测回填:61.29s,分钟级,远优于"小时内"判据;
   额外性能优化不触发。本条可结。〕
2. **Windows 栈溢出环境族**(T4 发现,基点即红,非 572 回归):裸
   `cargo taa` 12 测试 STATUS_STACK_OVERFLOW(001_smoke/m1/m2/m3/
   m4/m5 语料族+p532_lib_static_diff)——nextest 与 libtest 双路径、
   主检出/基点/修复态三处同签名;[env] RUST_MIN_STACK=16MB 只抬
   libtest 线程栈,nextest 进程主线程不受控;CI(Linux)守护。
   另 `test_charts_gallery_compiles`(tf 唯一红,564-Q6 邻接族)。
   两态失败集 13/13 逐一致(scratch/p572/t4_gate.md)。处置:需
   维护者归因(环境栈上限 vs 语料增长),复审时登 KNOWN-DEBT。
3. **exe² 5 缺陷处置归属**(T5 阻塞,2026-09-06 登记):拼合源转译
   首次可达后 exe² 构建露 5 错——2 个 AA2R 发射缺口(E0308 实参
   &str 强转缺@ev_run_files(argv[1])、E0382 mc clone 注入缺@
   engine.at 切片)+3 个 gen2 harness 项(prelude `mod IO` 与
   `IO.read_line()` 点号发射形态×2、aavm.at main 与 harness 追加
   main 重复)。**〔2026-09-06 用户裁定:本计划续修→T5b(步骤 5b);
   已清偿〕**见 scratch/p572/gen2/t5_unlock.md。
4. **aavm_at_mode_b34_struct 宿主侧红**(T5b 期间发现):`--ignored`
   档实测 b07 PASS/b34 FAIL(host transpiler 自建 aavm_at 跑 b34
   结构语料不过;该测试为 `#[ignore]`昂贵档,不在任何标准门禁内;
   572 未触宿主 transpiler,与本计划改动无关的表象成立,但基点
   未对拍——复审时裁定归置(host 侧 struct 字段表,P523-2② 域)。
