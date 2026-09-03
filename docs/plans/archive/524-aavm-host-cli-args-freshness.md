---
plan_id: PLAN-524
status: archived               # drafting → executing → execution_done → reviewed → archived（终态）
feature_name: aavm-host-cli-args-freshness
author: [zhaopuming]
created_at: 2026-09-03
updated_at: 2026-09-03

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "specs/aavm/design/lib-modularization-map.md: §CLI 入口形态——'clap 拒绝透传/process.args 返回值异常(argc=76) 观察项'已由本计划收口（位置参数直达 ev_run_files + 真实 argv List）；spec 预写的 fn main: process.args → ev_run_files(path) 形态已落地"
new_spec_components:
  - "specs/auto-cli/project.md: 直跑透传 auto <file> [args...]（index=2 trailing_var_arg + allow_hyphen_values；-- 消歧子命令撞名；全局旗标后置吞位语义；process.args() = [程序路径]+透传参数 的 List）"
  - "specs/parity/project.md: --auto-binary 新鲜度闸门（mtime 对账 crates 树硬失败 + --allow-stale 逃生 + 相对路径统一绝对化解析——P517-2 清偿）"
touched_goals:                # 自举（CLI 入口可用性 + 工具链防坑）
  - "GOAL-017: 自举——auto/aavm.at <目标.at> 一条命令直达成立 + parity 陈旧二进制防假红护栏"

affects: [aavm, auto-lang/cli]
current_step: 7
total_steps: 9
---

# [PLAN-524] 宿主小修批：CLI 参数透传 + process.args 修复 + parity 新鲜度

## 变更摘要

打包清偿 517 执行期暴露的三个宿主侧小项：

1. **`process.args()` native 修复**：现返回值异常（无参直跑 argc=76，
   517 W3 实测）——修为返回真实 argv（程序路径 + 透传参数）；
2. **CLI 直跑模式参数透传**：`auto <file>` 直跑形态当前被 clap 拒绝
   额外位置参数——增加透传机制（形态 W0 定案：`--script-arg` 全局选项
   多次收集 vs 位置参数放宽 vs `--` 分隔 trailing var args），使
   `auto auto/aavm.at <目标.at>` **直达形态**成立；`auto/aavm.at` 升级
   位置参数形态（行数协议保留为无参回退）+ **ev_run_files 的 CLI 形态
   解锁**（多文件程序经入口运行）；
3. **parity 二进制新鲜度校验**（P517-2 根治）：parity 启动时校验
   `--auto-binary` 指向的 auto.exe 与 crates/ 树源码新鲜度（mtime 对账
   ——515 G4 ② `auto_exe 陈旧防护` 先例），陈旧则明确报错（防陈旧产物
   伪装回归假红——本会话实证踩过两次的坑：P511-5/P517-2）。

**与 523/525 的关系**：小件，改动面与 523 零交叠——可先于/并行于 523
执行；523 步骤 14（aavm.at a2r 模式）与 525 语料运行将直接受益于透传
形态。

## 目标

1. `process.args()` 返回真实 argv；单测锁。
2. `auto <file> [args...]` 直跑透传成立；`auto/aavm.at <目标.at>` 一条
   命令直达（输出与 517 行数协议形态一致）；`auto/aavm.at` 增 ev_run_files
   多文件形态（目标为主文件路径）。
3. parity 启动新鲜度校验：陈旧 auto.exe → 明确报错退出（不再假红）；
   worktree 相对路径解析顺修（P517-2 后半）。

### 非目标（Out of Scope）

- VM 脚本运行时语义改动（透传仅 CLI 层）；
- stdin 行数协议移除（保留为回退与管道形态）；
- 523 的发射面/金样工作。

## 架构方案

```text
W0 形态选型       W1 process.args + 透传        W2 parity 新鲜度       收尾
──────────       ─────────────────────        ──────────────        ─────
clap 透传形态 →  native 修复+单测          →  mtime 对账校验     →  复审
考古(三选一)  →  CLI 实现+透传单测         →  相对路径顺修          归档
             →  aavm.at 升级(位置参数+多文件)
```

## 需求分析与背景调查

- **process.args 异常实证**：517 W3 探针（scratch/p517/argsprobe2.at）
  无参直跑 argc=76——native 实现位 `auto.process.args`（id 1301）返回
  载荷待考古（疑似返回了未过滤的进程环境/历史 argv）。
- **clap 拒绝实证**：`auto.exe probe.at alpha beta` → "unexpected
  argument 'alpha'"；Cli 结构唯一位置参数 `file: Option<String>`
  （index=1），无 var args。
- **新鲜度坑双实证**：P511-5（master auto.exe 陈旧 + 进程锁定 → 矩阵②腿
  561 假红）；P517-2（worktree 相对路径 `--auto-binary ../../target-p517`
  解析失败 os error 3）。
- **先例**：515 G4 ② auto_exe 陈旧防护（e2e_exe 共用体 mtime 对账 +
  AUTO_FRESH_EXE=1 强制重建 + 三态单测）——直接镜像。

### 风险与对策

| 风险 | 对策 |
|---|---|
| clap 透传形态与既有子命令参数冲突 | W0 三形态（--script-arg/位置放宽/-- 分隔）实测定案；全局选项方案冲突面最小 |
| process.args 修复影响既有消费方 | 全仓 grep `process.args` 消费面盘点（预期零——native 长期异常无人可用） |
| 新鲜度校验误报（合理陈旧=本地 dev 常态） | 镜像 515 三态：陈旧→eprintln 警告+`AUTO_FRESH_EXE=1` 强制/parity 加 `--allow-stale` 逃生；默认硬失败还是警告 W0 定案（倾向硬失败——假红代价>重建成本） |

## 详细设计

### W1 process.args + 透传（worktree）

1. `auto.process.args` native：返回 `vec![程序路径] + 透传参数`；无透传
   时仅程序路径。消费面盘点+单测。
2. CLI：按 W0 定案形态实现透传（缺省倾向 `--script-arg <V>` 全局可多次
   ——`auto auto/aavm.at --script-arg 目标.at`；或 `--` trailing）；
   `run_file` 路径把透传参数交 native 注册表。
3. `auto/aavm.at` 升级：有参数 → `ev_run_files(args[0])`（多文件主入口
   形态）；无参数 → 行数协议 stdin（保留）→ 无 stdin → 冒烟。实测：
   `auto auto/aavm.at crates/.../b07_fib.at` → 55；corpus_use 主文件 →
   多文件输出。

### W2 parity 新鲜度（worktree）

1. `run_auto`/`build_aavm_rust_bin` 前置：auto.exe mtime vs crates 树
   最新源 mtime 对账（镜像 515）；陈旧 → 明确错误（含重建命令提示）；
   `--allow-stale` 逃生旗。
2. `--auto-binary` 相对路径解析顺修（相对 parity cwd 而非 repo root 的
   解析统一/报错信息含绝对路径提示）。

## 测试设计

- 保护网：517/523 终态全绿面不破绿。
- 新增单测：process.args 三态（无参/单参/多参）；透传 clap 解析；新鲜度
  三态（新鲜/陈旧/缺档，515 同款）；aavm.at 位置参数形态端到端
  （b07→55 + corpus_use 多文件）。
- 命令：`cargo t`（涉 CLI crate `auto`）+ `cargo tv` + 矩阵（涉 parity）。

## 验收标准

1. `auto auto/aavm.at <目标.at>` 一条命令直达（b07→55 实测留档）；
   corpus_use 主文件多文件运行绿；无参回退形态不破。
2. process.args 单测三态绿；全仓消费面零回归。
3. parity 陈旧二进制 → 明确报错（非假红）实测留档；worktree 相对路径
   场景可跑；三态单测绿。
4. `cargo t/tv` 绿；无静默丢弃。

## 执行步骤

> 约定：全程 worktree `.worktrees/plan-524-dev`（涉 CLI/parity 代码）；
> 单折叠点。

1. [✅ 已完成]（2026-09-03，W0 实测）W0 形态选型考古：clap 三透传形态实测定案 + process.args native
   实现位考古（76 来源）+ 新鲜度默认硬失败/警告裁定。验证：决策注记。
   **决策注记**：
   - **透传形态定案 = 形态 B 位置参数放宽**：`Cli` 增 `#[arg(index = 2,
     trailing_var_arg = true, allow_hyphen_values = true)] script_args: Vec<String>`。
     独立工程实测（clap 4.5.17 同版）：`auto aavm.at b07.at` 直达 ✓/多参 ✓/
     hyphen 值 ✓/子命令零破坏（new、run -d、--error-limit 前置）✓；裸值撞
     子命令名（`probe.at run`→Run）以 `--` 消歧 ✓（trailing_var_arg 自带 `--`，
     形态 C 免费并入）；已知全局旗标**后置**于 file 时被旗标吞（文档注记项）。
     `--script-arg`（形态 A）落选——与直达形态 `auto auto/aavm.at <目标.at>` 不符。
   - **process.args 76 来源**：native 链路本身健康（codegen `("process","args")`
     →`auto.process.args`→id 1301→`shim_process_args`=`std::env::args().join(" ")`）；
     "argc=76" = 探针对 join 字符串取长度/计数的伪 argc（路径串长度量级）。
     语言契约考古：`examples/a2rs/03_image_scraper.at` 已按 **List** 消费
     （`list.len(args)`/`args[1]`）——契约即 List，join 字符串为 VM 侧异端。
     修复 = shim 返回 `Vec<String>`（VMConvertible 已支持 → VM List）。
     消费面盘点：055 VM 测试 `#[ignore]` 无 golden、c_process_app 测固定串
     不触 native、a2rs 示例本就 List 形态——**零回归面**。
   - **新鲜度默认口径定案 = 硬失败 + `--allow-stale` 逃生旗**（计划缺省倾向；
     515 先例为警告+强制重建档，parity 场景按"假红代价>重建成本"升级硬失败）。
2. [✅ 已完成]（2026-09-03，worktree 3e85293d5）`process.args` native 修复 + 三态单测。
   验证：`cargo t` 绿（scoped：`cargo t -p auto-lang stdlib` 23/23 绿，含新增
   `process_args_three_states`；全量 `cargo t` 归步骤 6）。
   实现：`shim_process_args` 返回 `Vec<String>`（VM List）= [程序路径]+透传；
   lib.rs 增 `SCRIPT_ARGS` 全局 + `set_script_args`/`script_args` +
   `run_file_with_args`（run_file 保持原签名委托）。
   **环境注记**：worktree 深度下跨仓 path 依赖 `autodown-core`（`../../../auto-down`）
   解析断裂 → 本地 junction `.worktrees/auto-down → D:\autostack\auto-down`
   （环境级 workaround，不入库；check-junctions.sh 扫描面覆盖）。
3. [✅ 已完成]（2026-09-03，worktree cf4fe8f82）CLI 透传实现（按 W0 形态）+ clap
    解析单测。验证：单测绿（`cargo test -p auto --bin auto cli_passthrough` 6/6；
    `cargo check -p auto` 干净）。注：CLI crate 不在 `cargo t`（-p auto-lang）
    别名内，单测经 `cargo test -p auto --bin auto` 显式跑（步骤 6 回归重跑）。
4. [✅ 已完成]（2026-09-03，worktree 3208d1d5a）`auto/aavm.at` 升级（位置参数
    ev_run_files + 回退链）+ 端到端实测（b07→55/corpus_use/冒烟）。验证：实测留档——
    - `auto auto/aavm.at crates/.../b07_fib.at` → **55**（rc=0，直达形态）；
    - corpus_use 六用例主文件（001-006）经直达形态全跑通：hi w/3、5、42、1/1、20、9；
    - 无参冒烟（`< /dev/null`）→ **2**（rc=0）；
    - 行数协议回退（b07 管道）→ **55**（517 原形态不破）。
5. [✅ 已完成]（2026-09-03，worktree e53924a8f）parity 新鲜度校验（mtime 对账+
    逃生旗+报错文案）+ 三态单测 + 相对路径顺修。验证：陈旧场景实测留档 + 单测绿。
    - 新增 `parity/crates/auto-parity/src/freshness.rs`：`stale_against`（515 镜像）
      + `resolve_auto_binary`（相对→绝对统一解析）+ `check_freshness` 启动闸门；
      main.rs 接线（--allow-stale 旗 + 全命令前置 + 解析后绝对路径下发）。
    - 三态单测 5 例绿（auto-parity 全量 38/38）。
    - 实测留档：①伪造陈旧 exe（mtime-1h）→ 硬失败 rc=1（文案含陈旧判定+
      指认最新源 + `cargo build -p auto` 重建提示 + 逃生旗提示）；②`--allow-stale`
      → 警告放行 rc=0；③缺档相对路径 → 绝对路径 + cwd 明确报错 rc=1（不再裸
      os error 3）——顺带实锤 P517-2 坑根因：`parity/../../target` 多退一级
      （=`.worktrees/target` 不存在），正确深度 `../target`；④worktree 相对
      路径（`../target/debug/auto.exe`，新鲜）→ 闸门过 rc=0。
6. [✅ 已完成]（2026-09-03，折叠 c14fdf475）全量回归（`cargo t` + `cargo tv` + 矩阵）+
    折叠合入。验证（零新增失败，全部存量红有据）：
    - `cargo t`（ui-iced 日常档）：4456/4459，3 失败全存量——d8_toggle_dark_mode
      与 strips_tags 实证 db9bfc977 基线同红（前者=1f7313e93 暗色默认化后测试未跟，
      仅 ui-iced 档可见故 tf 门禁未拦）；schema_drift 在档（PLAN-041 合入注记）。
    - `cargo tv`：3554/3556，2 失败=在档 aavm2 m4/m5 corpus 存量
      （PLAN-041 注记 + 523 W0 已归属定位 nat#112 镜像桩）。
    - `cargo tf`（pre-fold 门禁）：3397/3398，唯一失败=在档 schema_drift
      （442 期 3396/3397 + 我新增三态单测 1 = 3397/3398 精确对账）。
    - CLI 单测 `cargo test -p auto --bin auto`：8/8；auto-parity：38/38。
    - 矩阵：45/46，唯一 DIFF=b38_for_in_arr **存量**——宿主 for-in over
      fn-returned arr 已演进（输出 14/16/18，master exe 直跑同形实证），
      AAVM ②③⑤ 镜像未跟（engine.at nat#112 桩注释"b38 宿主零输出"为历史
      实证）；auto/lib 基线↔master 逐字节一致（`git diff db9bfc977
      f333a21ad -- auto/lib` 空）→ master 矩阵同形。属 523/525 家族工作面，
      已登记 KNOWN-DEBT P524-1 注记。
    - 折叠：master c14fdf475（合入时 master 已含 523 W0 提交 5f06c240c，
      零冲突）；worktree 已回同步 master。
7. [✅ 已完成]（2026-09-03）文档回写（README 入口用法更新位置参数形态/KNOWN-DEBT
    P517-2 核销注记）。auto/lib/README.md CLI 入口节增位置参数直达形态+透传
    一般用法（-- 消歧/全局旗标吞位注记）（worktree 403fa4fdb，随折叠落地）；
    KNOWN-DEBT P517-2 核销注记（freshness.rs 闸门+四场景留档）+ 新登记
    P524-1（cargo t 两处存量红：d8/strip_html）与 P524-2（CLI crate 单测不在
    cargo t 别名内）。
8. [✅ 已完成]（2026-09-03，复审 PASS）复审（/auto-plan:review）→ status: reviewed。
    复审记录见上节：验收 4/4 PASS（存量红全部有据、零新增失败）+ 遗漏/延后/workaround
    扫描四项全登记 + spec-impact 元数据已填。
9. [✅ 已完成]（2026-09-03，merge 沉淀）merge 沉淀归档。六节 SpecItem P524-1..6 落
    `.autoos/specs.json`（file/related 溯源本档）；module overview 回写
    （auto-cli/parity project.md + lib-modularization-map 宿主小修收口节）；
    worktree/分支已清；KNOWN-DEBT P517-2 核销 + P524-1/2/3 登记。

## 复审记录

**复审**（2026-09-03，/auto-plan:review，独立于执行侧）：结论 **PASS → status: reviewed**。

复审基线：worktree `.worktrees/plan-524-dev`（已回同步 master c14fdf475，含并入的
PLAN-041/523-W0 提交）；计划净落地 diff = `git diff 5f06c240c c14fdf475`（7 文件，
+422/−18，与计划声明面逐一对齐，无越面改动）。auto.exe 于同步树上重建后复验。

### 验收标准逐条复验（verify, don't trust）

| # | 标准 | 判定 | 证据 |
|---|---|---|---|
| 1 | `auto auto/aavm.at <目标.at>` 直达；corpus_use 多文件绿；无参回退不破 | **PASS** | 重建 exe 实跑四形态：b07→**55**/001_use_fn→`hi w`+`3`/冒烟→**2**/行数协议管道→**55**（corpus_use 六用例执行期全跑通留档步骤 4） |
| 2 | process.args 三态单测绿；全仓消费面零回归 | **PASS** | `cargo t -p auto-lang process_args_three` 1/1；消费面 grep 复盘=aavm.at（新，预期）+055（#[ignore] 无 golden）+a2rs 示例（List 契约，VM 侧由异端归正）+c_process_app（测固定串不触 native）——零意外消费方 |
| 3 | parity 陈旧→明确报错；worktree 相对路径可跑；三态单测绿 | **PASS** | 复验实测：陈旧 exe（mtime-1h）→硬失败 rc=1（指认最新源+重建命令+逃生旗文案）；缺档相对路径→绝对路径+cwd 报错 rc=1；`../target/debug/auto.exe`（正确深度）→闸门过；auto-parity 38/38 |
| 4 | `cargo t/tv` 绿；无静默丢弃 | **PASS**（存量红有据） | 复审全量门禁链（同步树重建后）：t 4456/4459（3 存量：d8+strip_html **基线 db9bfc977 实证同红**、schema_drift 在档）；tv 3554/3556（2=在档 aavm2 m4/m5）；tf 3397/3398（1=在档 schema_drift，+1=新增三态单测精确对账）；CLI 8/8；矩阵 45/46（b38 存量：宿主 for-in 演进、AAVM 镜像未跟，master exe 直跑同形+auto/lib 基线↔master 逐字节一致双重实证）——**零新增失败** |

### 遗漏/延后/workaround 扫描

- **遗漏**：无——W1/W2 全部子项与测试设计五件在 diff 内逐一对到（native 修复+单测/CLI
  形态+单测/aavm.at 三回退链/新鲜度三态+逃生旗+文案/相对路径顺修）；diff 内
  TODO/FIXME/HACK 标记扫描零命中。
- **延后**：aavm.at 仅消费 args[1] 主文件（余下透传参数不消费）——计划 W1.3 原文
  "目标为主文件路径"即此范围，文件内注释显式声明，非未批准缩面。
- **Workaround/环境项**（均已登记，不阻断）：
  1. `.worktrees/auto-down` junction（worktree 深度下 autodown-core 跨仓 path 依赖
     解析断裂的环境解）→ **KNOWN-DEBT P524-3**；
  2. CLI crate 单测不在 `cargo t`（-p auto-lang）别名内 → **P524-2**；
  3. `cargo t` 日常档两处 master 存量红（d8=1f7313e93 暗色默认化测试未跟、
     strip_html 空白折叠漂移；仅 ui-iced 档可见故 tf 门禁漏拦）→ **P524-1**；
  4. `#[ignore]` 的 055_process_args.at 未随返回类型更新（无 golden 无闸面，更新
     零收益——留档不 touch）。
- KNOWN-DEBT 回写：P517-2 **核销注记**（闸门四场景留档）+ P524-1/2/3 新登记。

### spec 影响裁定

见 frontmatter 三字段（merge 按此 upsert）：lib-modularization-map.md 的 517 期
"clap 拒/argc=76"观察项收口；auto-cli/parity 两 project.md 增本计划行为面；
GOAL-017 前进（入口直达 + 防假红护栏）。

## 待澄清事项

1. ~~**透传形态**（阻塞步骤 3）：`--script-arg` 全局多次 vs 位置参数放宽
   vs `--` trailing——W0 实测定案；缺省倾向 `--script-arg`（全局选项
   与子命令冲突面最小）。~~ **已定案（W0，步骤 1）**：位置参数放宽
   （index=2 trailing_var_arg），`--` 天然可用作消歧；实测证据见步骤 1 注记。
2. ~~**新鲜度默认口径**（阻塞步骤 5）：硬失败 vs 警告——缺省硬失败
   （假红代价>重建成本），`--allow-stale` 逃生。~~ **已定案（W0，步骤 1）**：
   硬失败 + `--allow-stale` 逃生旗（维持缺省倾向，实测证据见步骤 1 注记）。
