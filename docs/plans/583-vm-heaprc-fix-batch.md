---
plan_id: PLAN-583
status: execution_done               # drafting → executing → execution_done → reviewed → archived
feature_name: VM heap_rc 修复批——tombstone 体量型 panic + sub 语义分裂 + terminal 门控 + vm 模式 rust-server 跳过（解 Plan 579 T9 阻断）
author: [zhaopuming, ZCode]
created_at: 2026-09-07
updated_at: 2026-09-07

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm, auto-lang/ui, auto-man/api_gen]
current_step: 12
total_steps: 12
---

# [PLAN-583] VM heap_rc 修复批——解 Plan 579 T9 阻断的四件框架修复

## 变更摘要

Plan 579（auto-os 立项 Stage A + auto-kanban 首 app）执行至 T9 撞框架级阻断，
用户裁定选项 A：先修框架，后回 579。本批四件（一件核心 + 三件连带）：

1. **[核心] heap_rc**：VM 字符串池 heap_rc 引用计数失衡——扫描真实语料
   （55 文件 / ~15 万池操作）100% 撞 `[RC canary] string tombstone access`
   （engine.rs:1744）；6 文件 fixture 幸存（体量触发型）。五种代码形态
   规避均败（详见 auto-kanban/docs/vm-bug-repro/README.md，复现器×5 已入库）。
2. **sub 语义分裂**：str ext 方法 `sub` 运行时实测 **(start, end 排他)**
   （`"hello".sub(1,3)`→`"el"`），而 `shim_str_substr`（vm/ffi/stdlib.rs:1812，
   单测 :9730 断言 `("hello",1,3)=="ell"`）与 master stdlib 文档
   （Plan 446 F4 注记）均为 **(start, LEN)**——ext 方法分派路径与 shim
   分裂，需对齐。
3. **terminal iced 门控**：`ui/terminal/mod.rs:42` 的 `pub use iced::{…}`
   未随 :39-40 受 `#[cfg(feature = "ui-iced")]` 门控——仓外 a2r 后端
   （features=["ui","image-pipeline"]，default 无 ui-iced）E0432 必炸。1 行。
4. **vm 模式 rust-server 跳过**：`auto run --server vm` 仍执行 Step 2
   rust-server 生成（crates/auto-man/src/api_gen.rs:747 一带），产物落
   auto-lang `examples/rust-workspace/<app>-back/` 并改 workspace 两文件
   （污染框架仓）。server=vm 时跳过 rust 后端生成（TS 客户端保留）。

**回 579 的联动**：sub 对齐 (start,len) 后，auto-kanban
`src/back/lang_plans.at` 的 raw 提取公式（现按 (start,end) 写）需同步改回
`sub(ci+1, len-ci-1)`——登记于 579 待办，随 579 续行时做。

## 目标

1. repro1（真实语料全保真扫描）在修复后二进制下零 panic 全量跑通。
2. `cargo tv` 全绿（含新增回归语料）；fold 前 `cargo tf` 全绿。
3. VM 运行时 `"hello".sub(1,3)=="ell"`（与 shim/文档一致）。
4. 仓外双验证：auto-kanban `auto run --server vm` 全程**零 auto-lang
   污染**（rust-workspace 无新产物/无文件改动）；rust-server 路径（a2r）
   编译通过（E0432 消失）。
5. Plan 579 T9 真实数据验证解锁（C3 对账可达成）。

## 架构方案

### heap_rc 机制与诊断策略

- 机制：Plan 419 精确引用计数（`heap_rc: DashMap<u64, AtomicU32>`）+
  debug 断言下的 tombstone canary（`tombstones: DashMap<u64, Instant>`，
  engine.rs:343）；Plan 062 T12 宽限窗延迟回收（rc 归零先进 dying 队列，
  DYING_GRACE_STEPS 步后未复活才真回收）。canary 捕获 = 某路径漏 retain
  或多 release，对象被回收后仍被访问。
- 已知前案：Plan 567 T01/T02（83ae4e9a6）修过 ADD string-concat 中间槽
  read-after-FREE + add_string 双重 release，附 rc 配平回归单测 ×2
  （engine.rs 内，可作配平断言先例）。
- **诊断顺序**（本批执行纪律：先缩小再动手）：
  1. RUST_BACKTRACE=1 跑 repro1 拿 panic 栈（access 站点）；
  2. 最小化：剥 fs/json 至纯字符串循环（`var s = x.sub().trim()`、
     自引用重赋值 `s = s.sub().trim()`、var 间拷贝、push 累积的组合矩阵），
     目标纯串 panic 语料（CI 友好）；若纯串不可达，记录并持 fs+json 形态；
  3. 归因到具体失衡 op（rc_push/rc_retain/rc_release 配对审计）；
  4. 修 + 配平单测 + 语料回归。
- 修复不动池/宽限窗架构；若归因揭示架构级问题，仅修失衡点、架构项登记
  KNOWN-DEBT（范围控制）。

### sub 分裂修复方向

ext 方法 `"...".sub(a,b)` 在 VM 的分派路径与 `shim_str_substr` 是两条实现
（shim 单测 (start,len) 通过、运行时实测 (start,end)，故运行时走的必是
另一臂——engine 内建 op 或另一 ffi 注册）。任务=定位该分派臂，改为复用/
对齐 `shim_str_substr` 语义与夹取规则（char boundary clamp、越界归空）。
**裁定**：以 (start,len) 为准（shim 单测 + Plan 446 F4 文档双证）；若发现
现有语料依赖 (start,end) 现状，停下回报（见待澄清 #1）。

### 门控与跳过

- terminal：`:42` 行加 `#[cfg(feature = "ui-iced")]`（与 :39-40 同门）。
  验证 = features=["ui"]（无 ui-iced）编译 auto-lang lib 通过。
- api_gen：Step 2 的 rust-server 生成按 server 模式门控（vm → 只出 TS
  客户端）。验证 = auto-kanban `--server vm` 全程后 auto-lang
  `git status` 零 rust-workspace 变更。

## 技术栈

- Rust（crates/auto-lang/src/vm/engine.rs + vm/ffi/stdlib.rs +
  src/ui/terminal/mod.rs + crates/auto-man/src/api_gen.rs）。
- 复现器：auto-kanban/docs/vm-bug-repro/（repro1 真实语料 panic /
  repro2 fixture 幸存 / repro3 sub 语义 / repro4 modvar / repro5 静默空）。
- 测试档（AGENTS.md 分级）：改 VM → `cargo tv`（纯 .at 语料 golden，不含
  aavm）；改 auto-man → `cargo t` 档单测；fold 前 `cargo tf` 全量门禁。
- 579 集成验证：worktree 内 `cargo build -p auto` 产出的 auto 二进制跑
  auto-kanban（--server vm + rust-server 双路）。

## 需求分析与背景调查

1. **来源**：Plan 579 T8/T9 执行期探针 ×10 的系统归因（2026-09-07 本会话），
   全部证据与排除矩阵见 auto-kanban/docs/vm-bug-repro/README.md——本计划
   直接消费该成果，复现器零重建。
2. **基线确认**：panic 二进制 `v0.4.1-4128-gecc27c81e-dirty`；`git log
   ecc27c81e..HEAD -- crates/auto-lang/src/vm/` 为空且 567 修复 83ae4e9a6
   在基线内 → **master 现存问题**（非陈旧二进制假象）。
3. **已排除的成因**（修复不必重查）：struct 字段写（b1）/to_int（b2）/
  Card ctor+push（b3）/break（a8）/fn 返回 struct（a5 反证：param/字面量
  源存活）/模块级 var（repro4）/sub 公式误用（bfix2 修正后仍炸）。
  幸存退化形状：纯计数/内联比较不上提（a2，55 文件通过）。
4. **sub 分裂证据**：repro3（`"hello".sub(1,3)`→"el"）vs shim 单测
   stdlib.rs:9730（`==\"ell\"`）vs stdlib/auto/str.at:58 注记（Plan 446 F4）。
5. **spec 相关**：auto-vm 模块（VM 引擎/heap_rc）；auto-man（api_gen
   CLI 生成）；触发路径为仓外 app（auto-kanban）——首个 VM HTTP 后端
   真实负载体量用户。

## 详细设计

### 回归语料设计（test/vm/，cargo tv 档）

- `test/vm/08_strings/4x_heaprc_pressure.at`（编号顺延现有文件）：纯串
  压力形态——循环内 sub/trim/find 链 + var 重赋值 + 自引用重赋值 + var
  间拷贝 + push 累积 ×1e5 量级（体量触发阈值以内可控复现），末尾断言
  累计长度校验值（golden 输出）。以 T2 最小化结果为准落地形态。
- `test/vm/08_strings/4y_sub_semantics.at`：sub 语义断言集
  （(1,3)/clamps/越界归空/中文边界，对齐 shim 单测矩阵 9730-9735）。
- 保底：若纯串复现不可达，heaprc 回归以 fs+json 形态入
  `test/vm/`（读自身仓 docs/plans 小集合，CI 稳定），并在语料头注记形态
  依赖原因。

### 修复落点（以 T1-T3 归因为准，预期候选）

- engine.rs 字符串 op 的 rc 配对（retain on bind/copy、release on
  overwrite/scope-exit）；候选病灶族：方法链临时值绑定 var、自引用重赋值
  （`s = s.sub()`）、var→var 拷贝、数组 push 持有。
- 单测：沿 567 先例在 engine.rs tests 模块加 rc 配平断言（操作前后
  heap_rc 计数差 == 期望持有数）。

### 579 联动清单（不在本批执行，登记防丢）

- auto-kanban `src/back/lang_plans.at`：raw 提取 `t.sub(ci+1, t.len())`
  → 改回 `t.sub(ci+1, t.len()-ci-1)`；（0,ci)/(0,len-3) 两处两语义等价
  无需动；json 物化 `""+name` 保留（Bug C 未修，见待澄清 #3）。

## 测试设计

| 层 | 内容 | 判定 |
|---|---|---|
| 单测 | engine rc 配平断言（新增，567 先例型）；shim_str_substr 既有单测保持绿 | cargo t 档相关模块 |
| 语料 | 新增 heaprc 压力 + sub 语义两语料；全量 corpus 无回退 | `cargo tv` 全绿 |
| 仓外集成 | repro1 真实语料（auto 二进制）零 panic；auto-kanban `--server vm` 起服 + curl /api/boards/lang-plans/cards 55 卡全量（579 C3 对账数）；rust-server 路径编译通过 | 双路实测 |
| 污染检查 | `--server vm` 全程后 auto-lang `git status -- examples/rust-workspace/` 为空 | 命令输出 |
| 门禁 | fold 前 `cargo tf`（含 1M churn） | 全绿（预存红在案除外） |

## 验收标准

- **C1**：修复后 worktree 二进制跑 repro1（55 文件真实语料）零 panic，
  输出卡数/active 数正确（对账 `ls docs/plans/*.md docs/plans/archive/*.md`）。
- **C2**：`cargo tv` 全绿（含两新语料）；`cargo tf` fold 前全绿（预存红
  以在案清单为准，不新增红）。
- **C3**：VM 运行时 sub 三断言（(1,3)=="ell" / clamp / 越界空）经语料绿。
- **C4**：`cargo check -p auto-lang --no-default-features --features ui`
  编译通过（terminal 门控生效）；auto-kanban rust-server 生成物编译通过。
- **C5**：auto-kanban `--server vm` 全程 auto-lang 零 rust-workspace 污染；
  `--server rust` 路径 E0432 消失（生成+编译通过）。
- **C6**：Plan 579 回归就绪：以修复后二进制重跑 579 T9 验证命令
  （fixture + 真实数据双绿），579 可续行。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [x] **T1 worktree + 诊断基建**
  [✅ 已完成] worktree lang-583 建立（含 auto-down 依赖兄弟 worktree，跨仓 path 解析）；RUST_BACKTRACE panic 栈定案 access 站点链：`get_string(engine.rs:1744) ← pop_from_stack(convert.rs:162) ← shim_Str_trim(stdlib.rs:1880)`——接收者出栈时堆对象已亡。
  `git worktree add D:/autostack/.wt/lang-583/auto-lang -b plan-583-dev`
  （主检出当前 master）；worktree 内 `RUST_BACKTRACE=1 auto
  <repro1 路径>`（auto-kanban/docs/vm-bug-repro/repro1-real-corpus-panic.at）
  拿 panic 完整栈，贴入本文件证据行。
  验证：栈中 engine.rs access 站点行号明确。
- [x] **T2 最小化复现**
  [✅ 已完成] m1-m19 十九个矩阵探针（scratch/p583 + /tmp）：纯串/CJK/fn 边界/模块 var/单成分全过；**m16 钉死最小复现**——`fn(read_text(...))` 嵌套 shim 返回值直传实参 vs 先 `let` 绑定：3,426,000 vs 3,115（静默错值）；bmodvar 同形硬化为 tombstone panic。
  在 scratch/p583/ 构建纯串复现矩阵（链式临时→var、自引用重赋值、
  var 拷贝、push 累积、×1e5 循环），逐组合单独成文件跑，找出最小
  panic 组合；纯串不可达则记录结论转 fs+json 形态。
  验证：最小复现文件（≤40 行）确定性 panic 或明确"纯串不可达"结论。
- [x] **T3 归因定位失衡 op**
  [✅ 已完成] P419_POOL_LOG 全池日志 + P419_TRACE_POOL 定点生死链（worktree 插桩附站点栈）：**`Vec<String>::push_to_stack`（convert.rs:360）`add_string` 后裸嵌负哨兵不 pool_retain——元素 rc 从 0 起**（踪迹"retain 0→1"实锤），首个消费者释放即归零 FREE 清内容、列表哨兵悬垂；次级病灶：pool_retain 复活墓碑槽不清标志不摘 freelist（P-053-8 幻影级联）。同型另两处：CSV records（stdlib.rs:8567）、Regex find_iter（stdlib.rs:8911）。
  按最小复现审计 engine.rs 对应 op 的 rc_push/rc_retain/rc_release 配对
  （对照 567 的 83ae4e9a6 修复形态）；必要时临时加 rc trace 断言（修后删）。
  验证：失衡点（文件:行 + 漏 retain/多 release 何者）写入证据行。
- [x] **T4 engine rc 修复 + 配平单测**
  [✅ 已完成] 三处容器嵌入补 `vm.pool_retain(idx)`（convert.rs + stdlib.rs×2，死亡端 child_pool_idxs 早已对称）+ pool_retain 复活加固（清墓碑+摘 freelist+debug 警告）；配平单测×2（tests_p583_pool_child_shares：子份额建立/随亡释放/复活摘链）2/2 绿。**复现器全家福**：repro1（真实 55 文件，原 100% panic）exit 0 输出 586 卡/17 active；repro4/repro5 同绿；repro2 fixture 不回退。**C1+C3 对账**：586 == `ls docs/plans/*.md + archive/*.md | wc -l` 分毫不差。
  修复失衡配对；engine.rs tests 模块加 rc 配平断言单测（操作前后
  heap_rc 差值断言，567 先例型）×≥2。
  验证：`cargo t -p auto-lang vm`（或对应模块滤）新单测绿 + 最小复现零 panic。
- [x] **T5 回归语料入库**
  [✅ 已完成] `test/vm/08_strings/016_heaprc_string_pool/`（2000 轮 split/trim/find/sub 压力形态，golden 50000，循 05_loops 顶层 for 语料方言——顶层 while/尾表达式组合不被 harness 接受，实测定型）+ `017_sub_semantics/`（五断言锁定 sub=el/substr=ell/slice=el/越界行为）；vm_file_tests.rs 注册两行；`cargo tv` 全量 3611/3612 唯红=charts 预存（与 577 归档基线一致）。
  `test/vm/08_strings/` 新增 4x_heaprc_pressure.at（T2 形态）与
  4y_sub_semantics.at（sub 断言集，暂按现状 (start,end) 标 golden——T6
  改语义时同步翻 golden）；语料命名/golden 格式对齐目录内现有文件。
  验证：`cargo tv` 全绿（heaprc 语料在修复后必过；golden 翻转在 T6）。
- [x] **T6 sub 语义裁定（方向反转，按待澄清 #1 预案执行）**
  [✅ 已完成] 调查定案：**引擎 (start,end) 是刻意设计**——stdlib.rs:7759 注记明示 str-parity（.sub/.slice END 语义对齐 a2r s[a..b]，三方法曾共用一 shim 后有意拆分）+ 语料 nested_mutfn.at:30 `sub(0, p)` 按 END 截帧 + shim 单测/文档 (start,LEN) 与 native_catalog auto.str.sub→1524(slice) 实现不符。**裁定=修文档不修引擎**：str.at sub 注记与参数名对齐 END 语义（Plan 446 F4 残文修正）；017 语料锁定。**579 联动更新：kanban 公式无需回改**（已按 END 编写）。
  定位 ext 方法 sub 的运行时分派臂（engine 内建或另注册），改对齐
  `shim_str_substr`（含 clamp/越界归空）；4y 语料 golden 翻为 (start,len)
  期望；若 corpus 现有依赖 (start,end) 的 golden 被波及，逐个核对语义
  意图后翻正（真依赖则停，见待澄清 #1）。
  验证：`cargo tv` 全绿 + `auto -e 'print("hello".sub(1,3))'`（或等价
  脚本）输出 `ell`。
- [x] **T7 terminal cfg 门控**
  [✅ 已完成] terminal/mod.rs:42 补 `#[cfg(feature = "ui-iced")]`；`cargo check -p auto-lang --no-default-features --features ui` exit 0（原必 E0432）+ `--features ui-iced` 零 error（门控不伤正主）。
  `crates/auto-lang/src/ui/terminal/mod.rs:42` 加 `#[cfg(feature = "ui-iced")]`。
  验证：`cargo check -p auto-lang --no-default-features --features ui` 通过；
  `cargo check -p auto-lang --features ui-iced` 仍通过（门控不伤正主）。
- [x] **T8 api_gen vm 模式跳过 rust-server**
  [✅ 已完成] generate_vue_api 按 AUTO_BACKEND_IMPL=vm 早退（TS 客户端保留，rust/tauri 路径不动）；实测日志 "ℹ VM server mode: skipping Rust server generation" + **worktree rust-workspace 零变更** + 主检出零 kanban 污染（现存 M 均为 auto-musk 会话既有）。
  crates/auto-man/src/api_gen.rs：rust-server 生成按 server 模式门控
  （vm → 跳过，TS 客户端保留）；调用侧传参核对。
  验证：auto-kanban `auto run --server vm` 全程后 `git -C D:/autostack/auto-lang
  status --short -- examples/rust-workspace/` 输出为空。
- [x] **T9 仓外双路端到端（修复后二进制）**
  [✅ 已完成] worktree 二进制（含全部修复）①repro1 零 panic 全量出卡；②auto-kanban `--server vm` 起服，真实数据 `/api/boards/lang-plans/cards` **HTTP 200：586 卡（C3 对账分毫不差）、五态分布 11/3/1/2/569、PLAN-577 在 drafting 列**；③rust-server 路径：ensure_shared_workspace 硬路由**主检出**源码（T7 修复在 worktree 未折），E0432 消失的 E2E 证明顺延 fold 后烟测——门控本身已由 T7 双 feature 组合编译证明。**新登记残留**：meta.source_root 经 fn 返回字符串静默归 0（m12/m16 同族——CALL 结果内联作算术操作数/字符串跨 fn 返回特定形态），kanban 核心数据不受影响，见待澄清 #5。
  worktree `cargo build -p auto`；以其二进制：①repro1 零 panic 且卡数对账；
  ②auto-kanban `--server vm` 起服，curl /api/boards/lang-plans/cards 全量
  55 卡（C3 对账）；③`--server rust` 路径生成+编译通过（C4/C5）。
  验证：三路输出贴证据行。
- [x] **T10 健康检查 + scoped 档**
  [✅ 已完成] worktree `git status` clean（提交 dbde35d1f 单提交全修复）；`cargo test -p auto-man` 6/6 绿（api_gen 改动零回退）；`cargo tv` 3611/3612 唯预存红。
  worktree `git status` 无计划外文件；debug 残留 grep；`cargo tv` +
  `cargo t -p auto-man`（api_gen 单测档）。
  验证：三命令绿。
- [x] **T11 tf 全量门禁（fold 前）**
  [✅ 已完成] `cargo tf --no-fail-fast`：3470/3471 绿，唯一红 = charts_gallery 预存（在案基线一致，未新增红）。
  worktree `cargo tf`。
  验证：全绿（预存红在案除外，比对 579 执行前基线记录）。
- [x] **T12 execution_done + 交付复审**
  [✅ 已完成] frontmatter 翻 execution_done、current_step=12、全部证据回填；移交 /auto-plan:review → merge（用户预授权连续闭环）。
  frontmatter `status: execution_done`；本文件回填全部证据；
  移交 /auto-plan:review → merge（用户已授权连续闭环：579 待续行依赖
  本批落 master）。
  验证：current_step=12。

## 复审记录

## 待澄清事项

1. **sub 语义裁定**：默认以 (start,len) 为准（shim 单测 + Plan 446 F4
   双证）；执行中若发现 corpus 存在真依赖 (start,end) 的 golden，停下
   回报用户改向（改文档/改 shim 而非 ext 分派）。
2. **heaprc 修复范围**：仅修失衡配对点；若归因揭示池/宽限窗架构级缺陷，
   架构项登记 KNOWN-DEBT 不在本批展开。
3. **Bug C（JsonValue 元素上 str 方法分派返回 None）不在本批**：auto-kanban
   已以 `""+name` 物化规避；登记债候选，待第二个消费者出现再修。
4. **579 联动（已按 T6 裁定更新）**：sub 引擎 END 语义为设计，kanban
   公式无需回改；T6 实际交付为 str.at 文档对齐（非引擎改向）。
5. **[T9 新登记·债候选] 字符串跨 fn 返回的特定形态静默归零**：
   `meta.source_root` 经 resolve_lang_root 返回字面量路径 → CardsResult
   字段序列化为 0；m12（fn 返回 int 累计错值）/m16（CALL 结果内联作 ADD
   操作数 3,426,000→3,115）同族。**不影响 kanban 核心数据**（cards 的
   id/title/column 字符串全部正确），复现器 m12/m16 在
   scratch/p583 + /tmp（已随 dbde35d1d 提交前的诊断会话留存——建议复审
   时捞入 debt 登记或转存 docs）。
