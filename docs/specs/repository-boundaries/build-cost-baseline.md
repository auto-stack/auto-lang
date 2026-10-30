# build-cost-baseline（构建成本基线与 member 拆分决策）

> PLAN-896补充：测试按固定Cargo包/Rust模块选择，Rust模块过滤只限运行，不保证同crate独立编译。
> 必要experimental普通dev/test debug=0，显式full-debug debug=2；opt-level/断言/溢出/panic及正式性能配置保持。
> 无关ACE不准备/编译；同树兼容target复用保regular/batch及来源绑定。成本按真实同工作量的冷构建/运行/稳态磁盘分列。

> **Status**: proposal（PLAN-873 SD-01；work 冻结，canonical 发布归 merge）
> 落地任务：[Plan 873](../../../plans/873-build-cost-and-member-split-decision.md)（THR-D1：先测量、再决定是否按 crate 拆 workspace member）
> 输入 pin（measurement pin 2026-10-29 04:41）：auto-lang `a748017da`（+测量脚本提交 `c310e296f`/`07d9578b6`，非构建输入）、
> auto-ui `4adf2d03`、auto-man `e6cc06f`、auto-down `c3464d0`（detached 只读）；三仓 Cargo.lock 与 T-01 基线一致（等待期零依赖变更）。
> 机器/工具链（冻结）：VISUS / i5-13600KF 14C20T / 31.8GB / Win11 10.0.26200 / rustc 1.98.0 / cargo 1.98.0 / nextest 0.9.138；
> 仓配置原样（jobs=12、rust-lld、RUST_MIN_STACK=16MB），features=各仓默认面。
> 协议：[m873-protocol.md r1](../../../plans/attachments/873/m873-protocol.md)；运行器：`scripts/measure-build-cost-873.sh`（T-02）、
> `scripts/measure-l2-873.sh`（T-03，含 L2e）；原始日志：`D:/autostack/.wt/lang-873/.measure/logs/`（本地，index.md 为索引）。

## 1. 测量口径（摘要）

| 层 | 态定义 | 入口 |
|---|---|---|
| L1 冷 check | 空 target + `RUSTC_WRAPPER` unset + `CARGO_INCREMENTAL=0` | lang `-p auto-lang` / `-p auto-lang-core`；ui、man `--workspace` |
| L1 热 check | 同输入重跑（温 target） | 同上 |
| L2 增量三控制 | 温 target + incr=1：无改动 / 仅 mtime（touch）/ 真实函数体文本变化（锚点行后插 1 行纯内部语句） | 同 L1；7 站点 ×3 样本，每样本后 checkout 复位 + sha256 核验 + restore 构建 |
| L2e 编辑→测试二进制 | L2 变体 + `cargo test --no-run`（温 target，与 T-02 L3 冷 no-run 互补） | lang `-p auto-lang`；ui `-p auto-ui-runtime`；man `-p auto` |
| L3 测试成本分层 | no-run（编译+链接）/ `nextest list` / 定向单测真实启动分开计时 | 同 L2e 入口 |
| L4 完整档墙钟 | 只读既有合规 merge 收据，不新跑整档 | 880 批量回归收据 |

统计：每组 ≥3 有效样本取中位 + 极差；污染守卫 = rustc-only（T-02 裁定：长驻 `cargo run` dev-server 不构成编译争抢）。
跨仓 mtime 级联异常（O5，见 §2.5）单列归因，不按污染拒收。全部样本命令/时间戳/env 在各 log 头尾。

## 2. 基线数字

### 2.1 L1 冷/热 check（T-02，中位秒）

| 组 | 冷 | 热（同输入） |
|---|---|---|
| lang `-p auto-lang` | **80.0**（71.6/80.8/84.3/79.2） | 0.9 |
| lang `-p auto-lang-core` | **55.5**（58.5/55.2/55.5） | 0.5 |
| ui `--workspace` | **84.2**（85.9/80.3/84.2） | 0.6 |
| man `--workspace` | **105.9**（90.2/95.1/131.7/109.9/105.9；方差大，下界 90–95） | 0.7 |

过渡重建（incr0→incr1，记录非样本）：check 面 lang 88.6 / ui 92.7 / man 93.2；no-run 面 80.5 / 98.4 / 60.6。

### 2.2 L2 增量三控制（T-03，7 真实函数体站点，中位秒）

| 站点（仓 / 文件） | 无改动基线 | 仅 mtime | 函数体变化 | restore | body 重编单元 |
|---|---|---|---|---|---|
| lang-parser（core `parser/expressions.rs`） | 0.7 | 5.9 | **6.9** | 6.7 | core+auto-lang+ui-compiler+ui-runtime（4 crate 跨 2 仓） |
| lang-codegen（core `vm/codegen/expr_ops.rs`） | 0.7 | 6.0 | **6.9** | 6.9 | 同上 |
| lang-engine（core `vm/engine/ops_arith.rs`） | 0.7 | 6.0 | **6.9** | 7.0 | 同上 |
| lang-trans（core `trans/python.rs`） | 0.7 | 5.9 | **6.9** | 6.9 | 同上 |
| ui-compiler（`ui_gen/api.rs`） | 0.7 | 3.8* | **4.6** | 4.7 | compiler+runtime+tools+shell-pack（4 crate 仓内） |
| ui-runtime（`ui/iced/renderer.rs`，42,337 行） | 0.7 | 2.8 | **4.6** | 4.6 | runtime+tools+shell-pack（3 crate） |
| man-auto（`crates/auto/cmd_bp.rs`） | 0.7 | 1.2* | **1.2** | 1.2 | auto（1 crate，CLI 叶子） |

\* 干净样本中位（ui-compiler 3.8/3.8/4.2，man 1.1/1.2）；首批样本的跨仓 mtime 级联见 §2.5。
统计披露（终审 F-2/F-3）：ui-compiler mtime 干净中位为 **3.8**（原记 4.0 系均值误入）；O5 级联剔除后三格
（man-auto mtime、L2e ui-runtime/man no-run）仅剩 2 个干净样本，低于协议 ≥3 有效样本，按 n=2 中位披露；
极差/中位 >10% 的组（ui-compiler-restore 4.6–6.1、man-restore 1.2–1.4、L1 lang 冷组 71.6–84.3）归因为
负载方差/多会话时段（T-02 污染风暴与下界记录在案），中位数稳健，不改变任何结论方向。

### 2.3 L2e 编辑→测试二进制（T-03，中位秒）

| 站点 | no-run 入口 | 冷 no-run（T-02 对照） | 温态编辑后 no-run | 重编单元 |
|---|---|---|---|---|
| lang-parser | `test -p auto-lang --no-run` | 196.6 / 226.6 | **27.4**（31.2/27.1/27.4） | 4 crate（同 check 链） |
| ui-runtime | `test -p auto-ui-runtime --no-run` | 276.7 | **22.9**（22.6/23.2） | auto-ui-runtime 单 crate |
| man-auto | `test -p auto --no-run` | 181.1 | **4.9**（4.9/5.0） | auto 单 crate |

即真实开发者"改一行 → 跑一个定向测试"的构建侧成本：lang ≈ 27s、ui-runtime ≈ 23s、man ≈ 5s
（nextest 枚举 1.2–2.9s + 定向启动 1.1–3.2s 另计，T-02 L3）。

### 2.4 L3/L4（T-02 收录）

- nextest list 稳态 1.2–2.9s；枚举数（2026-10-29 终审 F-1 修正为日志实数，原记 511/1/10 为 T-02 提取错误）：
  lang `-p auto-lang` **1559** 测/22 二进制、ui `-p auto-ui-runtime` **1889** 测/2 二进制、man `-p auto` **34** 测/4 二进制；
  定向单测真实启动 1.1–3.2s。
- L4 只读收据（880 批量回归 @`03f851505`，本机主检出单实例，sccache 常态）：tf-language 3202=**84.0s**、tt 3569=**512.4s**、
  tb 3092=**61.0s**；输入差 866 落地，按近似参考标注。未新跑任何整档。

### 2.5 关键结构性观测（O1–O5）

- **O1 语言包 check 拖动整个 UI 图形栈**：`auto-lang` 默认 feature 启用 `auto-ui-runtime/ui-iced`
  （`crates/auto-lang/Cargo.toml` 特性表），`cargo check -p auto-lang` 的闭包含 auto-ui-compiler、
  auto-ui-runtime 及 iced/wgpu/rav1e 等图形三方族——core 冷 55.5s vs 根包 80.0s 的 ~25s 差主要在此。
- **O2 单行改动的爆炸半径=跨 2 仓 4 crate，且链重查（非 253k 行重编）主导**：core 任一家族单行函数体改动
  重编 core+auto-lang+ui-compiler+ui-runtime（6.9s）；仅 mtime 触发同链 5.9s——mtime 判脏级联的"地板"占成本大头，
  内容变化边际仅 ~1s。四站点（parser/codegen/engine/trans）数字不可区分（6.8–7.5s 同链）。
- **O3 成员数量不是冷成本杠杆**：man 仓已 9 成员，`--workspace` 冷 check 反而最慢（105.9s）——冷成本由
  依赖闭包编译主导，与 workspace 成员粒度无关。
- **O4 温态编辑→测试周期为几十秒级，冷 no-run 为分钟级**：冷/温差一个数量级（196–277s vs 5–27s），
  印证 THR-D1 痛感主要落在首次构建/CI，而非日常增量迭代。
- **O5 跨仓 mtime 级联实测**：前序电池 `git checkout` 复位把 core/auto-ui 源 mtime 推新，晚跑仓 target 的
  上游 rmeta 指纹过期 → 首样本多编 6–11 单元（ui-compiler-mtime-S1=6.0s、man-mtime-S1=6.4s、
  ui-runtime-norun-S1=33.7s、man-norun-S1=16.0s）。已按"跨仓 mtime 漂移"归因单列，干净样本另行中位；
  机制本身 = cargo mtime 判脏的既知行为，对多仓 path-dep 开发者是真实成本。

## 3. 决策模型（三方案对照）

规模输入（[crate-inventory.md](crate-inventory.md)，指纹 `668297c8f`/`47792366a`/`e6cc06fbc`）：
auto-lang-core P 301 文件/250,441 行（inline 测试 32,852）、auto-lang 根 16/7,691、auto-ui-compiler 76/104,587、
auto-ui-runtime 246/257,456（renderer.rs 42,337）、man/auto 10/6,393。家族级行数（core 内，本仓实测）：
vm 106,479 / trans 52,058 / parser 12,221 / test_cases 13,993 / 其余 <6k。

### 3.1 收益上界（拆分能省什么；全部为估算，未经拆分试点不冒称实测）

| 路径 | 拆分收益机理 | 上界估算 | 依据/限制 |
|---|---|---|---|
| 冷 check（首次/CI） | 无——第三方依赖闭包不变 | **≈0** | O3 man 反例；core 冷 55.5s 与根包 80.0s 差=闭包而非成员粒度 |
| 温态 check（每编辑） | 编辑的家族成员自身体积缩小（如 parser 253k→12k） | **<2s/编辑**（6.9→~5s） | O2：链地板 5.9s（mtime 级联+3 个依赖者重查）不因拆分消失；成员更多则每 crate 固定开销更多 |
| 温态测试二进制（每编辑） | 只重链所属成员的测试二进制（core lib 测试二进制现含 32,852 inline 测试） | **~10–15s/编辑**（27.4→~13–18s） | L2e 27.4s 未按二进制分解（缺测 §5）；ui-runtime 单 crate 温态 22.9s 提示单 crate 重链本身即 ~20s 量级，收益可能更低 |
| 缓存命中 | 缓存键粒度变细（改 parser 不失效 vm 的缓存） | sccache 场景下部分重叠 | 日常态未测（§5），以历史 79% 收据对照 |

### 3.2 成本（拆分要付什么；结构性、永久性）

- **迁移面**：core 301 文件/250k 行按家族迁到 3+ 成员，32,852 个 inline 测试随迁；`auto_lang_core::*`
  路径恒等（pub use 组合根）须保持或全量改消费者（ui-compiler/runtime/dialect、man 全家、lsp 等 15+ 上游）。
- **结构风险**：core 内 parser/vm/trans/类型系统共享 `types.rs`/`symbols.rs`/`session` 等中心模块——
  跨成员依赖方向若成环需先拆 trait/接口（843 已按文件分族但未解耦中心依赖）；feature 并集变化
  （ui-iced 默认链会在每个成员上并集放大或需逐成员复刻）。
- **维护/CI 面**：manifest ×N、CI 矩阵组合、缓存键管理、发布/安装清单（unified-package-install 面）、
  crate-inventory/规格页/测试档位映射全部随迁更新（D-2：CI/发布成本按估算计入，未细测）。
- **观测到的反压**：O2 链地板与 O5 跨仓级联说明"重编范围"由依赖拓扑而非成员大小决定——
  拆 core 不减少 3 个依赖者的重查；`-p auto-lang` 的 UI 图形闭包（O1）与 core 拆分正交。

### 3.3 三方案对照与触发条件

| 方案 | 内容 | 预期收益（实测/估算） | 成本 | 判定 |
|---|---|---|---|---|
| ① 不拆 + 替代手段 | 保持 crate 粒度；A1 特性门控、A2 共享缓存、A3 测试档位、A4 观察项（§4.2） | A1 估算：lang 冷 check −25s、每编辑链减 2 crate 重查；A2 历史 79% 命中=57s vs 隔离 80s | 低（A1 需独立签核小改；其余已在运行/已交付） | **推荐（现时点）** |
| ② 部分拆（最热子域试点） | 仅当触发条件成立：先拆单一低风险成员（建议 trans：52k 行、对中心模块依赖最浅、下游=UI/Man 消费面窄）验证模型 | 温态测试二进制 −10~15s/编辑（估算）；check <2s（估算） | 中高：迁移+路径恒等+档位映射 | **暂不启动**；触发条件见下 |
| ③ 全拆（per-family crate） | core 按家族全拆（parser/vm/trans/共享面） | 同②叠加，check 收益仍 <2s/编辑 | 高：全量迁移+循环风险+CI/发布面 | **不推荐**（收益/成本比最差） |

**触发条件**（任一成立即重开拆分评估/试点立项；均带实测口径）：

- **T-A 迭代周期**：温态单行函数体编辑 → `cargo t <定向>` 可运行的 P50 > 60s 持续两周（当前实测 ≈ 27s+3s 启动，未触发）。
- **T-B 结构耦合**：core 内家族出现 feature 并集冲突或跨家族循环依赖，文件级分族无法容纳（当前无实例）。
- **T-C 链接分解**：L3 按测试二进制分解的补测显示单二进制重链 > 40s（须先补 §5 缺测项再判）。
- **T-D 反例消除**：同等依赖闭包下多成员 workspace 展示出可测冷/温优势（当前 man=105.9s 冷为最慢，反例成立）。

## 4. 结论（T-05）

### 4.1 决策

**暂不拆**：不立项 workspace member 拆分（②/③ 均不启动）；THR-D1 以"已测量 + 结论闭合"处理——
原观察"任何改动触发整 crate 重编+测试二进制重链"在温态成立但量级为秒级（check 6.9s/4 crate、
no-run 27.4s），冷态痛感（57–80s check、196–277s no-run）由依赖闭包与默认特性面主导（O1/O3/O4），
拆 member 不削减该成本（§3.1 收益上界）。替代手段按 §4.2 登记，唯一建议跟进项为 A1（独立签核）。

适用版本：本结论绑定 measurement pin（文首）与 crate-inventory 指纹；后续结构工作包（869 减法批/872
后续/未编号结构包）设计冻结前须按 Plan 873 §2.6 核对输入差额，影响本模型输入（家族行数/依赖拓扑/
默认特性面）时只复测受影响站点。完整档墙钟为近似参考（输入差 866 落地）。

### 4.2 替代手段登记（不拆方案的行动面）

| ID | 手段 | 状态/owner | 量级 |
|---|---|---|---|
| A1 | `ui-iced` 移出 `auto-lang` 默认特性（语言侧非 UI 工作不再拖 iced/wgpu 闭包与 ui-compiler/runtime 链） | **建议跟进**：属结构性减法批，走独立签核小计划；试点后复测 O1/O2 | 估算：冷 check −25s、每编辑链 −2 crate；须实测验证 |
| A2 | 共享 sccache 常态维持 | 已在运行（历史 79% 命中→57s 样点 vs 本基线隔离态 80s） | 历史对照 |
| A3 | 测试档位分层（tu/tv/tt/tb、批量回归档） | 已交付（fix-test-tiering/PLAN-700 系）；L3 实测 list/启动 1–3s 非瓶颈 | 实测 |
| A4 | 跨仓 mtime 级联缓解（cargo 内容哈希指纹实验特性） | 仅观察项，不建议现网切换（O5 归因已入册） | 观测 |

### 4.3 若拆的范围草案（触发条件成立时的实施计划输入，非授权）

- **试点成员**：`trans` 家族（52,058 行/26 文件）先拆——对中心模块依赖最浅、消费者面窄、可独立验收；
  不首选 vm（106k 行、与 parser/engine 深耦合）。
- **目标形态**：`auto-lang-trans`（家族成员）+ `auto-lang-core`（组合根，保持 `auto_lang_core::trans::*`
  路径恒等 pub use）；manifest/feature/测试档位随迁；后续 parser/vm 视试点验收再议。
- **验收指标**（试点计划的 AC 骨架）：冷 `check -p auto-lang` P50 不劣化（±5%）；温态 trans 站点单行编辑
  check P50 ≤ 6s（现 6.9s）；温态 `test --no-run` P50 与 per-binary 分解对照（补 §5 缺测）；裸 `cargo t`
  墙钟不劣化；crate-inventory/specs/ledger 同步沉淀。
- **风险清单**：inline 测试随迁遗漏、路径恒等破窗（E0432 负控）、feature 并集放大、CI 矩阵/发布清单漂移、
  跨仓消费者（ui/lsp/man）闭包意外扩大。

## 5. 缺测限制与复跑

**缺测/限制**（结论引用时须一并披露）：

1. L3 no-run 未按测试二进制分解计时（无法直接判定"单二进制重链"占比；T-C 触发条件依赖此补测）。
2. L1 warm-sccache 隔离变体未采样（协议 §5 列有定义）；日常开发者 sccache 命中态以 2026-10-02 历史
   收据（79% 命中、57s）对照，非本基线实测。
3. 完整档墙钟未新跑（红线：不为测量跑整档）；引用 880 收据 @`03f851505`，与本测量输入差 866 落地，近似参考。
4. man 冷组方差大（多会话时段，90–131.7s），中位 105.9s 带下界 90–95 标注；结论只使用其与成员数的定性对照（O3）。
5. crate-inventory 指纹（`668297c8f`/`47792366a`/`e6cc06fbc`）与 measurement pin 存在等待期差额，
   家族级行数口径以其为准未复测；差额影响见 §4.1 适用版本段。

**复跑**（骨架；完整纪律见协议 §2–§7）：

```bash
# L1 冷 check（例）
bash scripts/measure-build-cost-873.sh L1-cold-lang-S1 <lang树> <.measure/target-lang> cold incr0 -- check -p auto-lang
# L2/L2e（站点表内置：list 查看全部 7 站点）
bash scripts/measure-l2-873.sh baseline LANG 3
bash scripts/measure-l2-873.sh site lang-parser 3        # mtime/body/restore ×3
bash scripts/measure-l2-873.sh sitebuild lang-parser 3   # 编辑→cargo test --no-run
```
