---
plan_id: PLAN-523
status: archived              # drafting → executing → execution_done → reviewed → archived
feature_name: aavm-a2r-midlang-coverage
author: [zhaopuming]
created_at: 2026-09-03
updated_at: 2026-09-03

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/aavm/design/divergences.md: D-AA2R-struct 清偿(523 条目)+D-a2r-mode-entry/D-let-shadow-global/D-str-index 新定案登记"
  - "docs/specs/aavm/project.md: 队列①核销注记+能力同步规约载体状态回填(fourpath/goldens/确定性实证)"
new_spec_components:
  - "crates/auto-lang/src/tests/aavm2_a2r.rs: 四路统一 runner(test_aavm2_fourpath_runner,path1-4+译文回链+主 a2r 三方同锚)+三件套金样校验/再生(test_aavm2_goldens_check,A2R_BLESS)+rustc 发射闸(test_aavm2_a2r_corpus_rustc)+per-case dir 语料 walker"
  - "crates/auto-lang/test/vm/aavm2/corpus_a2r/g19–g25: 中阶七族三件套语料+金样(per-case dir 格式首落)"
  - "scripts/aavm4_check.py: 四路/金样 Python 薄壳(--check/--fourpath/--bless)"
  - "scripts/aavm_build_smoke.sh: auto build a2r 产品链冒烟(转译→cargo build→运行对拍)"
  - "crates/auto-lang/src/trans/rust.rs: 主 a2r H1–H5/H7 洞清偿+fix_vec_tuple 字面量字符串跳过修复(既有洞)"
  - "auto/lib/a2r.at: AA2R 中阶发射镜像(构造字面量/字段读写/返回推断/str 下标/一元负/全局五件套)"
  - "auto/lib/codegen.at+engine.at: H6 T3 通道字节码镜像+nat#103(m4/m5 双存量红根除)"
touched_goals: [GOAL-017]     # 自举（目标 2 a2r 模式中阶覆盖 + 同步规约基建落地）

affects: [aavm]
current_step: 17
total_steps: 17
---

# [PLAN-523] aavm a2r 模式中阶覆盖收口（目标 2 + 同步规约基建）

## 变更摘要

清偿 511 待澄清①欠账并落地同步规约基建（project.md 能力同步规约/验证
矩阵/单用例全闭环/用例三件套四项规约的工程载体）。七项：

- **AA2R 发射面补全**：struct 声明/构造/字段读写（b34–b37 对应 g19+
  发射件）、for-in 数组/字符串下标/一元负/全局变量（b38–b43 对应件）
  ——与主 a2r live 逐字符 + rustc 零错；
- **摘除 compile corpus 跳过前缀**（`a2r_skip_prefixes` b34–b43），
  a2r 运行闸全量覆盖中阶语料（红项根因修复）；
- **`auto build`（pac）产品冒烟常态化**；
- **517 CLI 入口 a2r 模式验证**（aavm.at 经主 a2r 转译后可构建运行）；
- 清偿 D-AA2R-struct divergence 与 511 待澄清①遗留口径；
- **四路统一 runner**：单用例四途径（AutoVM+aavm / AutoVM+aa2r /
  a2r+aavm / a2r+aa2r）一致判定 + **译文回链**系统化（rustc 编译并
  运行译文、输出对拍——现仅探针冒烟覆盖的缺口）；
- **用例三件套金样格式**（`case.at`+`case.expected.out`+
  `case.expected.rs`，cookbook 模板）+ **bless 再生工作流** + 主 a2r
  发射确定性前置检查。

**与 524/525 的关系**：推荐执行序 523 → 524（宿主小修，改动面零交叠，
可互换/并行）→ 525（OOP 批，**开工前置 = 本计划 archived**——其验收
口径依赖本计划建成的四路 runner 与三件套基建）。

## 目标

1. AA2R 能发射 511 中阶全部语法（struct 族 + for-in 数组/字符串下标/
   一元负/全局变量；use 已由 517 W1 交付 g18），corpus_a2r 新件与主
   a2r live 逐字符一致 + 双侧产物 rustc 零错。
2. `test_aavm2_compile_corpus` **无跳过前缀**全绿（a2r 运行闸覆盖
   b34–b43 与 corpus_use 多文件用例）。
3. `auto build`（pac）产品路径冒烟常态化（脚本/CI 位 + 本地命令）。
4. `auto/aavm.at` 经主 a2r 转译构建后可运行（CLI 入口 a2r 模式）。
5. 四路统一 runner 落地：逐用例判定表（path1/3 执行输出锚 ②、path2/4
   译文锚 ③、译文回链执行对拍、主 a2r 同锚 ③——oracle 回归入闸）；
   转译二进制内容寻址缓存。
6. 三件套金样格式落地：首批语料迁移到 per-case dir 三件套 + bless
   工作流 + `--check` 同步校验；主 a2r 发射确定性检查（HashMap 迭代
   序类非确定位先规范化，M4 槽释放组先例）。

### 非目标（Out of Scope）

- OOP 批语法（方法/泛型/闭包/May/生成器）——Plan 525；
- CLI 参数透传/process.args 修复/parity 新鲜度——Plan 524；
- 全量存量语料的金样迁移（首批=中阶新件+代表抽验集，全量迁移视成本
  在 W3 裁定，见待澄清①）；
- 矩阵⑤腿塔顶性能稳定化（观察项，归 525 搭车测量）。

## 架构方案

```text
W0 确定性+考古          W1 AA2R 发射补全        W2 运行闸摘前缀      W3 四路+三件套基建      W4 产品+收尾
──────────────        ─────────────────       ─────────────       ────────────────       ───────────
主 a2r 确定性检查  →   g19+ 语料三件套先红  →   摘 skip prefixes →  四路统一 runner    →   auto build 冒烟
发射面逐语法考古  →   ar_use 同款逐族实现  →   红项根因修复     →   译文回链系统化     →   aavm.at a2r 模式
金样格式模板定案  →   rustc 双侧零错        →   corpus_use 入闸  →   三件套迁移+bless   →   divergence 清偿
                                                                  主 a2r 同锚金样       文档+复审
```

**关键设计约束**：

- **宿主为规范**：AA2R 发射文本 = 主 a2r 现行输出（live 逐字符）；主 a2r
  自身有洞先修主 a2r（447 先例）。
- **三面闸即验收**：每语法族落地 = VM 闸（已在 511 全绿）+ 发射闸（本计划
  W1）+ 运行闸（本计划 W2）三面同绿，禁无登记暂缓（同步规约）。
- **金样锚定 oracle**：主 a2r 译文同锚三件套 ③（live 对拍盲区闭合）；
  bless 由参考实现+主 a2r 生成、diff 走评审。
- **四路 runner 与日常分闸分层**：日常 CI 维持分闸快跑；runner 用于新
  能力验收与折叠点全量判定（规约定案）。

## 需求分析与背景调查

（取材：project.md 同步规约/验证矩阵/单用例全闭环/三件套四节、
`crates/auto-lang/src/tests/vm_file_tests.rs` compile corpus 跳过位、
`test/vm/cookbook/`（三件套现成模板）、Plan 511/517 归档、D-AA2R-struct
divergence 登记）

### 基线（2026-09-02，master d54ec540d）

517 终态全绿面：tv `--include-ignored` 19/0/0 / 99_unit 13/13 /
矩阵 46/46 / tf 3371/3372（唯一红 schema_drift_fence 归属并行 UI 线）。
compile corpus 现状：`a2r_skip_prefixes = ["b34".."b43"]` 十前缀跳过
（511 待澄清①缺省处置的欠账本体）。

### 发射面缺口清单（AA2R 现状 → 需补）

| 语法族 | corpus 侧 | AA2R 现状 | 预估改动位 |
|---|---|---|---|
| struct 声明（type X { 字段 }） | b34–b37 | `ar_prescan_type` 仅字段表（不发射 impl/构造）；构造字面量 `X { a: e }` 无发射 | ar_prescan_type/ar_emit_type2 + 构造字面量臂（ar_call_tail 结构构造已有——`ty_find` 本文件表命中路径,需扩跨 use 导入类型?本计划语料单文件,无跨文件） |
| 字段读 `p.a` | b35 | 点访问 `.` 发射已有——struct 实例字段读应直发（考古主 a2r 形态） | 点访问臂微调 |
| 字段写 `p.a = e` | b35 | 赋值 place 路径 | ar_store/ar_assign 位 |
| for-in 数组/表达式 | b38 | for range 已有；迭代器协议形态待考古 | ar_for 位 |
| 字符串下标 | b39 | 下标读 str 分支 | 下标臂 |
| 一元负 | b40 | `neg` 前缀 | ar_expr 前缀位 |
| 全局变量 | b42/b43 | load.global/store.global 发射 | ar_store/点访问 global 集 |

（W0 考古逐族定案发射形态——上表为预估,以主 a2r 实测输出为准。）

### 风险与对策

| 风险 | 对策 |
|---|---|
| 主 a2r 对中阶语法发射有洞（struct 构造序/迭代器协议） | W0 探针逐族经主 a2r → rustc 实测；有洞先修主 a2r |
| 主 a2r 发射非确定（HashMap 序） | W0 确定性检查先行；非确定位规范化（M4 槽释放组先例）后再落金样 |
| 转译版 aavm 跑 corpus_use 需 File shim 真实触达 | 511 shim 已 std 直通（read_to_string）；W2 实测多文件路径 |
| 金样 bless 流程增加日常摩擦 | 分层：日常 live 对拍快内环不变,金样为验收外环（规约定案） |
| ⑤腿塔顶高负载贴线（P517-1 观察） | 折叠点跑矩阵前置=重建 auto.exe（P517-2 纪律）;负载敏感复跑 |

## 详细设计

### W0 确定性检查与考古（master 纯文档+探针）

1. 主 a2r 发射确定性：同输入连跑 N 次逐字节比对（b34–b43 + g 系样）；
   非确定位登记并规范化方案定案。
2. 逐族发射考古：b34–b43 每件经主 a2r transpile → 产物存档 → rustc
   组合编译运行（W1 镜像目标文本）；主 a2r 洞登记顺修。
3. 三件套格式模板定案（cookbook 模板裁剪：去掉 reference.rs 运行夹具
   位,保留扩展口）；bless 脚本设计（生成/diff/`--bless` 再生）。

### W1 AA2R 发射补全（worktree，红先行）

- 语料：corpus_a2r 增 g19_struct_decl / g20_struct_ctor / g21_field_rw /
  g22_for_in_arr / g23_str_index / g24_neg / g25_globals（件名 W0 定），
  **直接落 per-case dir 三件套**（新件一律金样格式——格式裁定）；AA2R
  侧红证。
- 实现：a2r.at 按族补发射（ar_prescan_type 构造/字段、ar_call_tail
  构造臂、点访问/赋值、for 迭代、下标、neg、global 集）；每族 live
  逐字符 + rustc 双侧绿。
- use 族已绿（g18,517）不重做。

### W2 运行闸摘前缀（worktree）

- 摘 `a2r_skip_prefixes` 十前缀（保留注释位留痕）；compile corpus
  全量跑；红项逐根因修复（预判多数直接绿——lib 转译后自带中阶编译
  逻辑,511 W3 File shim 已真实现）。
- corpus_use 多文件用例入 compile corpus 运行闸（转译版 ev_run_files
  路径首次实测）。

### W3 四路统一 runner + 三件套基建（worktree）

- `scripts/aavm4_check.py`（或 Rust 测试件,W0 定载体）：输入 per-case
  dir → path1（VM 内 ev_run）/path3（主 a2r 转译版 aavm 二进制）输出
  锚 ②；path2（VM 内 ar_run）/path4（AA2R self-bin）译文锚 ③；**译文
  回链** rustc 编译并运行对拍；主 a2r 译文同锚 ③；逐用例判定表输出。
  转译二进制内容寻址缓存（复用 compile corpus 机制）。
- 三件套迁移：中阶新件（W1）+ 代表抽验集（b07/b13/b32 等,缺省 5-8 件）
  迁 per-case dir 金样;`--check` 同步;主 a2r 同锚。存量全量迁移视成本
  裁定（待澄清①）。

### W4 产品与收尾（worktree→master）

- `auto build`（pac）冒烟：脚本 `scripts/aavm_build_smoke.sh`（或并入
  runner）——pac 转译→cargo build→跑 b07/b34 抽验,命令留档 CI 位。
- 517 CLI 入口 a2r 模式：aavm.at 经主 a2r 转译构建的产物跑行数协议
  用例（转译版 IO.read_line/parse_int 可用性实测,洞登记）。
- D-AA2R-struct divergence 清偿 + 511 待澄清①口径核销注记。
- 文档回写 + 复审 + tf。

## 测试设计（TDD：保护网 + 红先行 + 金样锚定）

### 保护网

517 终态全绿面（19/0/0 + 矩阵 46/46 + 99_unit + vm-files-ci）全程
不破绿。

### 红先行

| 红灯 | 载体 | 转绿点 |
|---|---|---|
| corpus_a2r g19–g25（AA2R 遇族语法 unsupported/错发射） | W1 三件套落盘 | W1 逐族实现 |
| compile corpus 摘前缀后 b34+/corpus_use 红（若有） | W2 摘除动作本身 | W2 根因修复 |
| 四路 runner 判定表（工具不存在） | W3 落地即首次全量输出 | W3 |
| 主 a2r 确定性（若非确定位存在） | W0 探针 | W0 规范化 |

### 命令

- 门禁/矩阵/99_unit 同 517（含 P517-2 纪律：矩阵前置重建 auto.exe、
  worktree 绝对路径）。
- runner：`python scripts/aavm4_check.py <case-dir> [--all] [--bless]`。
- 冒烟：`bash scripts/aavm_build_smoke.sh`。

## 验收标准

1. corpus_a2r g19–g25 与主 a2r live 逐字符一致 + 双侧产物 rustc 零错
   （三件套 ③ 锚定含主 a2r）。
2. `test_aavm2_compile_corpus` 无跳过前缀全绿（b34–b43 + corpus_use）。
3. 四路 runner 逐用例判定表：中阶全量语料四途径 + 回链全 PASS；主 a2r
   译文同锚金样。
4. `auto build` 冒烟脚本常态化位 + 本地全绿留档；aavm.at a2r 模式
   实测留档（或洞登记）。
5. 三件套格式+`--check`+bless 落地；迁移集（新件+抽验集）金样全绿。
6. D-AA2R-struct 清偿；511 待澄清①核销注记；全程保护网不破绿；
   `cargo tf` 绿（基线红除外,归属注明）。
7. 无静默丢弃（延后项显式登记）。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

> 约定：W0 在 master 直接做；W1 起在 worktree `.worktrees/plan-523-dev`；
> 折叠点（步骤 4/8/12/16）矩阵+CI 绿后合入 master。

### W0 确定性与考古（master）

1. [✅ 已完成] 主 a2r 确定性 27/27 绿（b34–b43+g01–g18 单文件 5 连跑 + merge 三连跑逐字节一致，无非确定位）+ 逐族考古（9 件产物存档 rustc 实跑：3 绿 6 红 → 主 a2r 五洞 H1–H5 登记）+ 载体定案（三件套/bless/runner=Rust 测试件+Python 薄壳）。文档：`scratch/p523/w0/ARCHAEOLOGY.md`；探针：`scratch/p523/w0/{determinism_probe.sh, det/, arch/, probe/, merge/}`。
2. [✅ 已完成] g19–g25 七件三件套语料落盘（per-case dir，`.at` 先行；金样 W1 洞修复后 bless）。红证 7/7 MISMATCH 实跑留档（诊断 worktree 参考宿主管线）：g19–21 构造字面量 PARSE-ERROR / g22 for-in `&` 形态+返回推断 / g23 下标 / g24 一元负括号 / g25 全局零发射——清单见 ARCHAEOLOGY.md §7。

### W1 AA2R 发射补全（worktree）

3. [ ] struct 族发射（声明/构造/字段读写）：ar_prescan_type 扩 + 构造
    臂 + 点访问/赋值位。验证：g19–g21 live 逐字符 + rustc。
4. [ ] 折叠点①：中阶补缺族（for-in 数组/字符串下标/一元负/全局变量）
    发射。验证：g22–g25 live + rustc + 全量 tv 绿 + 矩阵 → 合入。

### W2 运行闸摘前缀（worktree 续）

5. [✅ 已完成] 摘除 `a2r_skip_prefixes` 十前缀，compile corpus 全量跑。
    验证：**直接全绿 46/46**（H1–H7 修复连带覆盖，零红项——预判应验）。
6. [✅ 已完成] 红项根因修复。验证：无红项（全量直接绿；m5/m4 双存量
    红已在 W1-4 H6 提前根除）。
7. [✅ 已完成] corpus_use 多文件入运行闸：bin harness `--files` →
    ev_run_files 模式（524 位置参数形态同款）+ 缓存键补 harness 文本
    （旧键漏 harness 致演进走旧 exe）+ 并发互斥锁。验证：转译版
    ev_run_files 首测即绿 6/6。
8. [✅ 已完成] 折叠点②：全量 tv 3557/3557 + 99_unit 13/13 + 矩阵
    46/46 → 合入（worktree 提交 0adc35429）。

### W3 四路 runner + 三件套（worktree 续）

9. [✅ 已完成] 四路统一 runner 实现（Rust 测试件 #[ignore] + bin --trans
    模式 + 译文回链 rustc 编译运行对拍 + 主 a2r 三方同锚 + 判定表 +
    staging/原子改名内容寻址缓存）。验证：判定表 14/14 全 PASS——首曝
    并根除两枚既有洞（fix_vec_tuple 字面量内 "vec![" 伪触发级联误插
    .to_string()；H4 语义修正码点十进制字符串形态 + let-遮蔽-全局写块
    形 b43 怪序）。留档 scratch/p523/fourpath_table.txt。
10. [✅ 已完成] 三件套金样 14 件落盘（g19–g25 per-case dir + corpus_m4
    抽验集 7 件平铺旁挂——布局裁定：存量 .at 不动零 walker 影响）+
    `--check`。验证：goldens_check 14/14 绿（③ 主 a2r 同锚 + ② 参考
    输出锚）。
11. [✅ 已完成] bless 工作流（A2R_BLESS=1 再生 + scripts/aavm4_check.py
    薄壳 --check/--fourpath/--bless + diff 走 git 评审口径）。验证：
    篡改 g24 金样 → 红 → bless → 绿 → 复原逐字节一致，演示留档
    scratch/p523/bless_demo.txt。
12. [✅ 已完成] 折叠点③：判定表留档 → 门禁全绿（tv 3558/3558 + 矩阵
    46/46 + 99_unit 13/13 + fourpath 14/14 + goldens 14/14）→ 合入。

### W4 产品与收尾

13. [✅ 已完成] `auto build` 冒烟脚本常态化：`scripts/aavm_build_smoke.sh`
    （a2r 产品链转译→cargo build→运行对拍）。验证：b07=55/b34=10-20
    本地全绿留档；pac 最小工程 target 生成缺口登记 P523-1（坑位即 CI 位）。
14. [✅ 已完成] aavm.at a2r 模式实测：平铺 merge 转译+原生 shim 构建成功，
    b07_fib 位置参数直达 **55** 实测留档；两洞登记（argv.get Option 解包/
    转译版运行期 struct 字段表 → D-a2r-mode-entry + P523-2）。
15. [✅ 已完成] D-AA2R-struct divergence 清偿注记 + 511 待澄清①核销 +
    文档回写（divergences.md 523 四条目/project.md 队列①核销+同步规约
    载体状态/KNOWN-DEBT P523-1/2/3）。
16. [✅ 已完成] 折叠点④：矩阵 46/46 + tf 3397/3398（唯一红=在档
    schema_drift 存量，归属注明）→ 合入。
17. [ ] 复审（/auto-plan:review）→ status: reviewed → merge 沉淀归档。

## 复审记录

**复审人**：zhaopuming（/auto-plan:review，2026-09-03）；worktree
`.worktrees/plan-523-dev`（e1d9264ad）实跑复核，verify-don't-trust 全条重证。

### 验收标准逐条判定

| # | 标准 | 判定 | 证据 |
|---|---|---|---|
| 1 | g19–g25 live 逐字符 + 双侧 rustc 零错 | **PASS** | `test_aavm2_a2r_is_corpus` 25/25（复审实跑）；`test_aavm2_a2r_corpus_rustc --ignored` 绿 |
| 2 | compile corpus 无跳过前缀全绿 | **PASS** | 前缀仅存于注释留痕（vm_file_tests.rs:926）；`test_aavm2_compile_corpus` 46/46 + `compile_use_corpus` 6/6（复审实跑） |
| 3 | 四路 runner 判定表全 PASS + 主 a2r 同锚 | **PASS** | `fourpath_runner --ignored` 14/14（复审实跑重出判定表） |
| 4 | auto build 冒烟常态化 + aavm.at a2r 模式实测 | **PASS（附注）** | `aavm_build_smoke.sh` b07=55/b34=10-20 复审实跑；a2r 模式 exe 复审重放 b07=**55**。附注：pac 最小工程 rust target 生成缺口（P523-1，脚本以 a2r 核心链顶位——计划文本"或并入 runner"预留口径） |
| 5 | 三件套+--check+bless 落地 + 迁移集金样全绿 | **PASS** | `goldens_check` 14/14（复审实跑）；bless 演示闭环留档（篡改→红→bless→绿→复原一致） |
| 6 | divergence 清偿/核销/保护网/tf | **PASS** | divergences.md 523 条目+project.md 队列①核销在档；复审全量门：**tv 3558/3558（缓存三清三跑）+ tf 3397/3398（唯一红=schema_drift 在档存量，归属注明）+ 矩阵 46/46 + 99_unit 13/13**；tt 1358/1360（2=既登记 P523-3 既有缺陷对，较 28 件基线**改善 26 件**——W3/W4 主 a2r 修复顺带治愈多数过期金样） |
| 7 | 无静默丢弃 | **PASS** | 全部延后项显式登记（P523-1/2/3 + D-a2r-mode-entry）；无 TODO/TEMP 残件（复审 grep 实查） |

### 复审发现与修复（gate 中修正）

- **R1（已修）**：复审 tv 并发首建暴露 merge 剥离目录共享固定路径竞态
  （`aavm2-merge-lib-stripped` 两 corpus 腿并发互删——199KB/400KB 残缺
  merge 产物 + 115 错连锁实证）。修复：剥离目录按进程隔离
  （e1d9264ad）；缓存三清三跑 tv 3558/3558 验证。此竞态为 517 W2 引入
  的既有债，本计划 W2 增设第二个 corpus 腿后首次可触发。
- **R2（注记）**：步骤 15 括号中的"README"回写——specs/README.md 无
  aavm 条目位（不存在对应回写面），project.md/divergences.md 已覆盖
  语义；spec 台账入账归 /auto-plan:merge 的 specs.json+spec-index 流程。

### 遗漏/延后/workaround 猎查

- 遗漏：未发现（plan 任务 ↔ diff 逐项对上；W0 留档 ARCHAEOLOGY.md 已
  入库 scratch/p523）。
- 延后：P523-1（pac 最小工程 target 缺口）/P523-2（a2r 模式两洞）/
  P523-3（tt 档 2 件既有缺陷+存量）——均显式登记且计划文本预留口径
  （步骤 14 "或洞登记"/W4 "或并入 runner"），非静默缩水。
- Workaround：a2r 模式实测的 argv.get 文本垫片（产品位、随 D-a2r-mode-entry
  登记，归 525/宿主小修偿还）；bin 并发首建 staging+原子改名（设计性
  方案而非临时垫）。

### 结论

**七条全 PASS，无阻断债 → status: reviewed**，就绪 /auto-plan:merge。

## 待澄清事项（全部裁定闭环，2026-09-03）

1. **三件套迁移范围**——**已裁定（W3-10）**：新件 g19–g25 per-case dir +
   抽验集 7 件（b07/b13/b32/b33/b34/b36/b42）corpus_m4 平铺旁挂（存量
   .at 不动，两 walker 零影响——布局裁定）；全量存量迁移不做，随新件
   增量落格。
2. **runner 载体**——**已裁定（W0 §6，按缺省）**：Rust 测试件
   （#[ignore] 验收档）+ Python 薄壳 `scripts/aavm4_check.py`
   （--check/--fourpath/--bless）。
3. **aavm.at a2r 模式口径**——**已裁定（W4-14）**：524 先行落地 → 用
   位置参数形态（b07 实测 55）。
4. **m5 语料腿 master 既有红（跨计划回归，非本计划引入）**：二分实跑
   定位 = PLAN-057 T3（`e01eeba0b`，参考 codegen for-in Call 源通道
   泛化）暴露 511 W2 的 v2 engine nat#112 镜像桩（`engine.at:688`
   自证注释）——宿主修对后 v2 仍零迭代，b38 分歧。处置——**已按缺省
   并入本计划并于 W1-4 提前清偿**（`cg_for` 裸调用源改 T3 通道字节码
   级镜像 + engine nat#103=auto.list.len；m5 行为 46/46 + m4 字节码
   46/46 双绿）。详见 `scratch/p523/ARCHAEOLOGY.md` §8.1。
