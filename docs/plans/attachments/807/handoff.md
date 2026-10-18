# PLAN-807 work → review 交接（handoff）

2026-10-18。work 树：`D:/autostack/.wt/lang-807/auto-lang`，分支
`codex/plan-807-dev`，base `2046a025b`。本文绑定送审冻结 SHA（T-07 提交）
与全部验证结果；review 在**同一工作树**独立复核。

## 1. 代码身份

- base commit：`2046a025b`（master）
- 送审冻结提交（R2 修复后三轮）：`4e3bde4d6`（R2 受审 `1eb8434f5`、R1 受审`1f5e05141`；见 §10/§11 修复记录）
- 依赖修订：workspace +`crates/auto-text-core`（imara-diff 0.2 唯一三方；
  dev-deps serde_json）；auto-lang +path dep auto-text-core、−直接 imara-diff
- 组内兄弟：`D:/autostack/.wt/lang-807/auto-down`（detached @c39ac7f，
  autodown-core 路径解析用，零改动）

## 2. 交付形态

- `crates/auto-text-core`：rope.rs + rope/file_backing.rs + diff/{mod,envelope,dirs}.rs
  逐字迁移（git mv 保历史）+ 冻结最小公开面（见 contract-baseline §3）。
- 预取所有权闭环清偿：worker 改持 Weak；`PageStore::worker_exited_handle()`
  每店退出观测；crate 内永久回归 + auto-lang 层 Windows delete-pending 硬观测。
- 旧路径薄门面（类型/函数身份不变）：`ui/code_editor/core/rope.rs`（7 行）、
  `ui/code_editor/diff/{mod(17),dirs(5)}.rs`；`diff/envelope.rs`（31 行）= UI
  adapter：key→snapshot 查找（`code_editor_with`）+ core 纯形调用。
- 快照端口：`vm/host_capabilities.rs` 第 8 槽 `EditorSnapshotProvider`
  （独立 trait，802 trait 零改动）+ NI/AutoVM set_/accessor + `new_inner`
  构造期快照 + Clone/merge 保留；UI adapter
  `EditorSnapshotRegistryProvider` 挂 `code_editor_provider::install_process_default`
  同四点位 if-absent。
- 四 diff shim（9915/16/17/21）与 a2r_std::diff 四函数 on 臂零 `crate::ui`；
  off 臂：四 VM shim 不取参 raise 逐字；a2r `diff_files/diff_dirs/diff_snapshots` panic 逐字、`diff_files_window` 为仅 on 导出（off=符号缺席，旧 base 同形——R1 勘定）；`ui_gen/rust.rs` 发射串零改动。
- 777：domains +`auto-text-core = lang-core`；基线重采（base_revision 见
  §6）；792：`DELIVERY_CRATES` 12→13。

## 3. work-owned 验证收据（V-01..V-06）

| 项 | 命令 | 结果 |
|---|---|---|
| V-01 身份 | `cargo test -p auto-lang --lib plan807`（TypeId 同型/跨 API 传递/剪断锚/真编辑器门面==纯形逐字节） | **7/7 PASS** |
| V-01/V-06 独立消费 | `cargo test --manifest-path test/fixtures/modularity/text-core-consumer/Cargo.toml` + `bash …/verify-deps.sh` | **3/3 PASS；闭包=auto-text-core→imara-diff，零 auto-lang/man/ui/vm** |
| V-02 字节/持久性 | `cargo test -p auto-text-core`（rope 29+分页 12+diff 24 族随源迁移零漂移，含 1GB 谱 3 个 #[ignore] 显式位） | **65 PASS / 3 ignored（显式谱位）** |
| V-03 资源/保存/退出 | `AUTO_LANG_P728_BENCH=1g cargo test -p auto-lang --lib plan728_bench -- --ignored --nocapture` | **PASS：1,073,741,834B/23,280,107 行；结构驻留 3,033,703B ≤ 12MB 合同；answer 4.15ms；edit_50 8.4ms/远端 max 0.247ms；save 1,464ms；typing-paged-1g p50 1.04ms/max 5.73ms；谱行入 docs/reports/p728-bench.jsonl** |
| V-03 退出观测 | plan807 `paged_base_handle_released_after_last_drop`（Windows）+ crate 内 `prefetch_worker_exits_after_last_external_release` | **双 PASS（snapshot 存活期不退出/可读；最后释放后 worker 受控退出、同名重建成功=句柄关闭硬观测）** |
| V-04 四 native 语义 | `cargo test -p auto-lang --lib plan703|plan710|plan716`（VM catalog 真调用、缺 key、feature 开关臂、窗口/错误值形） | **7/7 + 22/22 + 14/14 PASS（冻结矩阵零漂移）** |
| V-04 端口/隔离 | plan807（构造式缺席/快照隔离/实例覆盖/双 VM/Clone·merge/不动 revision/缺 provider=missing key_a） | **含于 7/7** |
| V-05 diff 语义 | `cargo test -p auto-text-core`（golden 八形、704 D-1/D-2、716 窗口语义、dirs 五态族随源迁移）+ 探针族 | **随 65 PASS + plan716 14/14** |
| V-06 外部发行 | 上行 consumer fixture + 777 strict + 792 driver | **见 §6/§7** |
| 旧臂零漂移复跑 | plan728 supply probes（经门面） | **5 PASS/2 ignored（谱位）** |
| feature-off 编译 | `cargo check -p auto-lang --no-default-features` | **绿（off 臂矩阵完整）** |
| 新 crate 卫生 | `cargo check -p auto-text-core` | **零警告** |

## 4. 去重审计（旧算法零残留）

- 算法符号（imara/poly_digest/histogram_changes/anchor_partition/intern_lines/
  split_lines_universal/group_hunks）在 auto-lang src **零命中**；
- 四门面文件共 60 行（7+17+5+31），唯一算法邻接代码 = envelope adapter 的
  key 查找（UI 状态，设计归属）；
- vm/native.rs、a2r_std.rs 对 `ui::code_editor::diff` **零引用**（剪断锚
  永久回归钉住）；
- ui_gen/rust.rs 发射路径经门面续存（下游金样零扰动）。

## 5. 剩余 UI 边（诚实清单）

1. `ui/code_editor/diff/envelope.rs`——key→snapshot 查找（注册表=UI 状态；
   算法零残留）。
2. `ui/code_editor/core/mod.rs`——编辑器内核消费 rope（分页装载阈值/布局
   窗口/预取触发留 UI，合同 §7 冻结）。
3. `ui/code_editor_provider.rs`——快照 adapter 安装（四点位）。
4. `PageStore::prefetch_byte_range/file_len` 为 UI 消费点升 pub（冻结 API）。

## 6. 777 收据

- domains 登记 + 基线重采：`base_revision=3717b2164`，declared_edges=61
  （新增 auto-lang→auto-text-core，lang-monolith→lang-core 合法抽取边）。
- strict：`python scripts/modularity-audit.py --mode strict --no-cargo-check`
  → **verdict=pass violations=0 grandfathered=2**（既有窄豁免，未放宽）。

## 7. 792 收据

- 登记：`DELIVERY_CRATES` 12→13（`+auto-text-core`，rust_ui.rs:3988）。
- driver：`python test/fixtures/modularity/generated-ui-consumer/driver.py
  --scratch D:/p807-scratch-792`（正式入口、真实生成/fresh 解析/locked
  构建/重放/负控全档；cargo 1.98.0）。
- 结果：**25/25 PASS**——manifest = 13 源 crate + 1 patch 条目；
  auto-text-core **当前受审包身份（R2 程序化提取自实际 manifest，F807-R2-02
  勘定——R1 稿手抄 digest 末字节拼错且未随修复后源字节重绑）**：
  7 files / 229812B / digest `6b7380e59ec9d797c54f4c46b27fcb933922c744d932231307035aa675b1e8b4`
  （R2 复审对 `1eb8434f5` 正式生成入口实测，逐源文件与包内副本字节相同
  [review-r2/delivery-content-result.json.txt]；本 R2 修复提交 `4e3bde4d6`
  仅改 `crates/auto-lang/src/tests/plan807_text_core.rs` 一个文件——
  auto-text-core 源字节零变化，该身份对 `4e3bde4d6` 仍逐字节有效）。
  历史身份（R1 轮，仅作过程记录，非当前交接）：229,843B。V01 fresh resolve（5s，patch 单源 +
  pin 全命中）+ 真实 build --locked（214s，产物在位）；V02 重定位 locked
  build（226s）+ metadata 单源/越界 path=无 + 资源字节一致；V03a-j 全部
  负控确定性红（tamper/missing/corrupt/unlisted/missing-patch/bad-pin/
  foreign-dir/redelivery 保旧/sidecar 外部 path/根相对/staging 兄弟保留/
  raw TOML/他应用交付存活）；V04 dev 模式与交付输出逐字节同源。
- 收据：`attachments/807/receipt-792-driver.json.txt`（driver 原始输出；review-r1 独立重跑另见 `review-r1/delivery.log.txt` 25/25 与`delivery-receipt.json.txt`）。

## 8. 提交清单（work 树）

| SHA | 内容 |
|---|---|
| `bc2fc322c` | T-02 rope+file_backing 迁入 + 闭环修复 + 门面 |
| `75f492fc8` | T-03 diff 三件迁入 + 纯快照形 + 门面/adapter |
| `6a931eb20` | T-03 卫生（envelope mut 警告清零） |
| `d340bd0a8` | T-04 快照端口 + 四 shim 接线 + UI adapter |
| `7cb1239b4` | T-05 a2r_std 四函数接线 |
| `43c077e0d` | T-06a/b consumer fixture + plan807 上层回归 |
| `3717b2164` | T-06c/d/e 1GB 资源测 + 777/792 登记 |
| `1f5e05141` | 777 基线重采（**送审冻结提交**，本 handoff 绑定） |

## 8b. SD 草案（merge 沉淀用；canonical 编辑归 merge）

- SD-01（add `docs/specs/auto-text-core/project.md`）：crate 边界/公开面
  （rope/分页/快照/diff/envelope）、支持面与诚实边界、依赖纪律
  （imara-diff+std）、闭环修复的寿命合同——取材 contract-baseline §3/§7。
- SD-02（modify `paged-rope.md`）：实现路径迁移注记（ui/…/rope →
  auto-text-core::rope，门面身份不变）；全部分页不变式原样保留；
  `worker_exited_handle` 观测面新增；50MB 阈值/2MB 窗口仍留 UI。
- SD-03（modify `vm/overview.md`）：diff 四项 9915..9921 消费面
  （auto-text-core 直调 + EditorSnapshotProvider 端口）；缺席/feature 矩阵
  （缺 provider=无法找到 editor→err envelope 值形；off 臂 raise 逐字）。
- SD-04（modify `ui/overview.md`）：editor/Rust 消费新 core 单源；注册表
  adapter（envelope key 查找 + 快照 provider 安装点）；旧路径薄门面兼容
  （ui_gen 发射零扰动）。

## 9. review 注意事项

- review 入口：同树 `D:/autostack/.wt/lang-807/auto-lang`（独立上下文，
  不新建同步树）；AC-05 复核清单见计划 §7。
- merge-owned 门（**pending，不由 work/review 执行**）：冻结提交上裸
  `cargo t` + `cargo tv`（VM 触面）+ `cargo tu`（Rust 生成接线触面）；
  V-03 资源测绑定条件（源/依赖/配置/输入/编译形态未变）已在 §3 记录。
- 806 会话并行在途（另一工作树）；公共写面本计划只增自己的片段（根
  Cargo.toml members+exclude、tests.rs 一行、host_capabilities 追加节、
  777 基线——后入者取并集）。


## 10. R1 修复记录（2026-10-18，[review-r1](review-r1/REVIEW.md) 全 finding 闭合）

| Finding | 修复 | 重放证据 |
|---|---|---|
| F807-R1-01（快照装配测试依赖全局空槽，703+807 联跑确定性失败） | `snapshot_port_assembly_identity` 重构：显式可区分非空默认 A/B 验证构造期快照隔离（不假设槽初态）；缺席断言仅用构造式 `new_without_host_snapshot`；新增 `compat_install_never_overrides_explicit_default` 真实宿主先安装永久控制（显式默认不被兼容 if-absent 覆盖；空槽补缺装真实 adapter 且对未注册键诚实 None）；FakeSnap 带内容标签可区分实例 | 同进程单线程联跑 `-- plan703 plan807 --test-threads=1`：**15/15**（R1 失败形态 13/1 → 全过）；plan807 单独 8/8 |
| F807-R1-02（类型身份测试缺 feature 门控，关 UI 4 处 E0433） | `plan807_identity` 加 `#[cfg(feature = "code-editor")]`；关门形态的身份/单源证明由独立 consumer fixture 承担（闭包零语言/UI/VM，3/3） | `cargo test --lib --no-default-features plan807 --no-run`：plan807 文件**零错误**（余为旧族既有项，复审明确不归本计划） |
| F807-R1-03（冻结材料误称 a2r 四入口 off 均 panic） | contract-baseline §3 纯快照形签名更正为 `DiffSnapshotsPair` 枚举实参、§6 a2r 矩阵更正（`diff_files_window` 旧/新均仅 on 导出，off=符号缺席 E0425；其余三函数 off panic 逐字）；handoff §2/§7 同步 | 文档面更正（保持既有支持面，未新增 fallback/未扩大启用面） |
| F807-R1-04（fmt/doc/交接未收口） | 触面 `rustfmt --check --config skip_children=true` 全过；`worker_exited_handle` 私有 rustdoc 链接清零（`cargo doc -p auto-text-core --no-deps` 零警告）；2 个随源遗传 unused_mut 清偿；handoff 收据链接 `.json`→`.json.txt`、新包文件数 44→**7**（manifest/digest 勘定） | fmt exit 0；doc 零警告；core 65/3 + consumer 3/3 复跑全绿 |

R1 资源绑定补充（review 独立重放，见 `review-r1/resource-rows.jsonl.txt`/
`rss.json.txt`/`baseline.json.txt`）：1GB 输入 1,073,741,834B / 23,280,107 行、
digest `0x17dcf2bba1afe59f`、结构驻留 3,033,703B ≤12MiB、**独立进程
working-set 峰值 16.5MiB**（RSS 非结构计量阈值）、load 23.77s/save 1.05s；
源/依赖 lock/工具/编译形态（default+code-editor debug）指纹全量入
`review-r1/baseline.json.txt`——未来合流改变任一输入须重放影响面，不得
自动复用。

R1 修复提交：`1eb8434f5`（冻结树 12 文件；修复不触 a2r/native 语义面，
故 §3 表中 VM/a2r 相关收据绑定 `1f5e05141` 仍有效，fmt/doc/mut/测试面
绑定 `1eb8434f5`）。

（R2 勘定：本节 R1 资源身份为**历史过程记录**；当前交接的资源/代码指纹
以 [review-r2/baseline.json.txt]（冻结于 `1eb8434f5`；R2 修复差异=§11 已审单文件，verification_inputs 未变）与 §11 当前记录为准。）


## 11. R2 修复记录（2026-10-18，[review-r2](review-r2/REVIEW.md) 全 finding 闭合）

### 剩余不变量根因/覆盖调整表（review"下一轮就绪要求"逐项）

| 剩余不变量 | 之前为什么漏（根因） | 本轮设计/覆盖调整 |
|---|---|---|
| 全局默认槽写者不得污染同进程真实宿主 reader | R1 只证明写者自身不依赖空初态（显式 A/B 默认）；写者互斥锁 `test_default_slot_lock` 只覆盖**持锁的测试写者**，普通真实宿主用例（plan703 经 VmBridge 的 if-absent 安装 + VM 构造期快照）是不持锁的 reader/写者——共享 libtest 进程下二者交错，FakeSnap（只认 `have`）或空槽被快照进真实 VM（R2 实录 threads=8 首跑 13/2、三独立进程复现；PoisonError 为首个失败的连带）。pair-only 组合 20 次未触发完整调度，故 R1 重放不足以闭合 | **生命周期级隔离**：两个写者用例改自重执行子进程——wrapper 重执行本测试二进制 `--exact --ignored --test-threads=1` 跑用例体，子进程独享全局槽（写完即进程退出），对同进程任何 reader 零共享窗口；用例体带 `P807_SLOT_CHILD` env 守卫（裸 `--ignored` 全量跑空过——防 bench 类显式调度把污染重引入共享进程）；实值 oracle 全部保留（显式 A/B/C/D 默认替换、构造式缺席、兼容 if-absent 补缺、真 adapter 对未注册键 None、Clone/merge/不动 revision）。**不依赖单线程/nextest/跳过 reader/err envelope 替代有效输出** |
| handoff 必须准确指向所审实际源码包/资源身份 | R1 稿人工改文件数与 digest 时末字节拼错（`aad3…fa1a` vs 实际 `aad3…fa1c`），且把格式修复视为无需重绑的源字节变化 | 本稿身份**程序化提取**自 R2 复审实际 manifest（§7：7 files/229,812B/`6b7380e5…e8b4`，提取脚本断言 `source_bytes_equal=true`），并区分历史（R1 过程记录）与当前（对 `4e3bde4d6` 逐字节有效——R2 修复仅改 auto-lang 测试文件，auto-text-core 源零变化）；资源/代码指纹：[review-r2/baseline.json.txt] 冻结于 `1eb8434f5`（其 verification_inputs——lock/配置/工具链/兄弟依赖——在 R2 修复前后未变；代码差异=本表所列已审单文件），1GB 资源 R2 已实跑并绑定当前指纹（见 review-r2 §资源/Spec/修订绑定） |

### F807-R2-01 重放证据（原失败形态全覆盖）

- 并行组合 `-- plan703 plan807`：`--test-threads=8` ×3、`--test-threads=16` ×3
  → **6 连 15/15**（R2 失败形态为该组合首跑 13/2）；
- 单线程 `--test-threads=1`：15/15；序交换（`plan807 plan703`）：15/15；
- 更宽组合 `-- plan703 plan710 plan716 plan807 --test-threads=8`：**51 过/2 ignore**；
- 裸 `--ignored plan807`（守卫路径）：2 用例体空过、共享进程零全局槽写；
- 真实宿主 reader 族复跑：plan728_supply 5 过。

### 验证复跑

plan807 8 过/2 ignore（含 2 个子进程驱动 wrapper）；core 65/3；consumer 3/3；
fmt/doc 复验见 R2 复审已关闭项。R2 修复提交：`4e3bde4d6`（单文件
`crates/auto-lang/src/tests/plan807_text_core.rs`，+68/−29）。
