# generated-cargo-build-lifecycle

> **Status**: active（PLAN-910 SD-01/SD-05）
> 范围：生成式 Rust 消费者（UI Rust 前端成员、Web Rust 服务后端成员）的
> **受管生成构建上下文**——producer 归属与缓存根、成员/bin 身份、内容收据、
> 发布件与发现、并发/lease、清理准入，以及本包有限验收口径与 UI007 移根
> 交接要求。单源实现：`crates/auto-generation-core/src/generated_build.rs`
> （经 `auto-api-gen` / `auto-web::rust_workspace` facade 再导出）。
> 不覆盖：交付模式（见 generated-ui-distribution）、Tauri `gen/back/rust`、
> 语言仓工具自身 target、全机/跨 worktree 可写 target 统一。

## 1. 三个对象

| 对象 | 落点 | 说明 |
|---|---|---|
| 生成源码 workspace `<ws>` | `AUTO_RUST_WORKSPACE`（权威）→ 项目在 producer 树内=`<producer>/examples/rust-workspace`（**以项目目录为锚**，cwd 无关）→ 项目本地 `<project>/rust-workspace` | 成员表、`.cargo/config.toml`、`Cargo.lock`、`.generation.lock` 在此 |
| 成员身份 | `<ws>/<basename>-<id8>`（front）/ `<ws>/<basename>-<id8>-back`（back） | `id8=sha256("generated-member/v1"‖canonical 小写规范化工程路径)[..8]`；`member.json` 记 role+project_path 二次一致性校验 |
| 受管缓存根（同 producer 兼容消费者共用） | 开发：`<producer>/target/generated-consumers/<ctx8>/`；安装 SDK：`<ws>/target`（不写 producer）；显式：`AUTO_GENERATED_TARGET_ROOT`（兼容 `CARGO_TARGET_DIR`） | `ctx8=sha256("generated-build-context/v1"‖canonical producer_root‖flags 策略‖[调试策略键])[..8]`（PLAN-1003：缺省 `c` **无**第 4 段——哈希输入与旧三段逐字节一致，既有根/release 缓存存活；显式 `full` 或用户 `CARGO_PROFILE_DEV_DEBUG` 局部覆盖作第 4 段隔离根）；`context.json` 记 schema/ctx/mode/producer/target/rustflags/用户覆盖/toolchain/producer_origin/debug_policy(+source)（可复验，非键） |

producer = `select_language_producer` 单选（显式 `AUTO_DELIVERY_SOURCE_ROOT`/
`AUTO_LANG_ROOT`/`AUTO_LANG_CRATE` → exe 旁安装 SDK → 项目上溯/兄弟发现 →
编译期），与 `[workspace.dependencies]` 锚点同一选择，二者不可能分歧。
混合 ws（成员引用 `auto-web`）时核心 ensure 自行解析并写入
`auto-web = { path, version = "=0.1.0" }`——不再依赖 facade 事后注入，任一侧
的后续 ensure 不得丢失它。

## 2. flags 单源与 cwd 无关

受管派生（UI build/run、split 后端、Man `serve_rust`）统一：cwd=`<ws>`；env 钉
`CARGO_TARGET_DIR=<根>` 与 `CARGO_ENCODED_RUSTFLAGS="-C\x1flink-arg=/STACK:268435456"`
（windows-msvc；256MiB 深视图栈保持，语言仓 `.cargo/config.toml` 的 lld/32MiB
不渗入生成单位）。用户已设 `RUSTFLAGS`/`CARGO_ENCODED_RUSTFLAGS` 时**不覆盖**，
ctx8 并入其值 hash 形成具名隔离根并记入 `context.json.user_flags_override`。
显式根须真实绝对路径，拒绝 producer 工具 target 与另一检出（含 `.git` 的非
producer/非项目树）。linker 不钉（不影响依赖 rlib 指纹；终端链接随 cwd 发现的
config，记为边界）。`<ws>/.cargo/config.toml` 同步写入同一 target-dir 与栈
flags 供手工 cargo 使用；**在受管根内运行未钉 flags 的手工 cargo 会造成指纹
漂移并触发全量重编**（实测 582 单位）。

## 2b. 受管调试策略（PLAN-1003）

受管 ws 根 manifest 携带受管调试策略块（默认策略 C：成员 line-tables、
非成员依赖 debug=false；`full`=仅 marker 省段），配置落点=根 manifest 而非
`.cargo/config.toml`（config 发现锚 cwd；manifest 与 cwd 无关）。策略在
ensure 任何写入前解析，未知值/显式策略与 `CARGO_PROFILE_DEV_DEBUG` 并存=
具名失败、受管文件不动。定义/接口/迁移与实测见
[generated-debug-policy-production](generated-debug-policy-production.md)。

## 3. 内容收据与新鲜度

- 前端成员 `generation.json`（`generated-front-receipt/v1`）：inputs=pac.at + front
  目录 .at 集 + rust_sidecar 模块源的 sha256；**App 声明官方主题包时另含
  `theme/content` + `theme/identity` 统一输入身份键（PLAN-036，UI owner
  加法语义：键值由 UI compiler 单源算法计算；缺键=不新鲜=安全重建；必需
  主题输入读/验失败=具名拒绝，不签 reduced 身份）**；generator=`auto-ui-tools@<ver>
  front-layout/v2 producer=<canonical>`；workspace_lock 三态（absent/空=811c9dc5/
  内容 hash；读错=不可核验）；context_id/target_root；artifact{exe, exe_sha256,
  bin_name, cargo_bin, profile}。
- 新鲜度（`front_needs_regeneration`）：无 Cargo.toml/main.rs/收据 或清单与按
  pac+身份重算的字节不一致 → 全量；inputs/generator/context/lock 任一不一致 →
  代码重生成；mtime 永不参与。生成期冻结输入快照，构建后输入若变则收据
  withheld（同 Web "receipt withheld"）。空 .at 集但成员已有生成物 → 具名拒绝。
- 生成产物（main.rs/Cargo.toml/sidecar 复制/ws 清单与 config/context/member/
  收据）一律**写前比较**，未变不改 bytes/mtime。Web 后端收据改 park-rename
  （`generation.json.pending`）：未变原件原 mtime 还原，失败则无有效收据；
  `backend_generation_is_fresh` 为只读（不再 ensure 重写）。Web 收据新增
  `build_context{context_id,target_root}`（schema_version 1 只增）。
- 边界：main.rs 层生成器变更须手动 bump `FRONT_LAYOUT_VERSION`；含 sidecar
  deps 的成员不做清单字节比对（依赖版本常量）。

## 4. 发布件与发现

- Windows/MSVC cargo 对 bin **不加 metadata hash**：同名 bin 在同一根共享
  `deps/<bin>.exe` 与 `<profile>/<bin>.exe`，fresh 单位亦不重 uplift——同
  basename 劫持发生在链接输出层。故 cargo `[[bin]] name = "<published>-<id8>"`
  （Cargo.toml 内 `# published = "<name>"`），取件路径取自
  `--message-format=json-render-diagnostics` 的本包/本 bin `compiler-artifact`，
  发布到 `<member>/bin/<profile>/<published>.exe`（**用户可见名在此恢复**：pac
  `exe_name` 否则 snake(pac name)）。exe 复制（硬链接共 inode 会让运行中的发布
  件锁死 cargo 重链接）、pdb 硬链接；运行中的旧发布件 rename 让位；发布前校验
  PE 头/可读，坏件具名拒绝并保留上一好副本（cargo fresh 不检测任意篡改，恢复
  =删该单元 `.fingerprint/<pkg>-*` 或整根重建）。bin 名解析清单优先、收据回退。
- 受管 cargo 派生 cwd 恒为 `<ws>`（不从缓存根反推；显式根亦然），config 链只
  来自 ws 自身及其祖先。`auto run` = 受管 build+发布 + 直接运行发布件（应用
  cwd=前端目录）。pdb 取 cargo `compiler-artifact.filenames` 所报路径（rustc
  以下划线命名 `counter_<id8>.pdb`），发布为 `<published>.pdb`，**并以 cargo
  报告名硬链接一份别名**（PLAN-1003：发布 exe 的 RSDS 记录该名，dbghelp 按
  exe 目录回退查找；只发改名 pdb 会使原样发布件 0 成员符号可解析；别名随
  重链接刷新）。前端收据含 `debug_policy` 标签；发现身份=收据 `context_id`==当前
  ctx8 **且**标签一致（与 `front_needs_regeneration` 同一规则——标签单独可被
  改写，不构成身份）。`discover_front_exe`（执行/交出产物）另校验发布件字节=
  记录 sha256；`discover_front_exe_identity`（注册表扫描等分类）不读字节。
- `context.json.producer_origin` 为首见值：同一 producer 经不同选择臂（env/
  发现/编译期）再写入时不翻转该字段。
- 发现：`generated_build::discover_front_exe(app_dir)`（收据优先）；auto-ui
  `app_registry::convention_native_exe` 与 runtime `launcher_compiled_exe` 先走
  `published_native_exe`，约定路径仅作 910 前布局回退。
- 旧 basename 成员：新身份目录与之不同名，永不认领/覆写/删除；新身份目录
  无 `member.json` 却有内容 → 拒绝。

## 5. 并发、lease 与清理

- `<ws>/.generation.lock`（`File::lock`，线程内可重入）保护 workspace ensure
  （成员扫描→清单/config 重写）；生成产物与收据本身靠 tmp+rename 原子替换与
  生成期输入快照保证不出混合体，不在该锁临界区内。两不同工程并发不丢成员，
  同工程并发由 cargo 目标锁串行（争用可观测）。
- `<root>/.lease/<pid>-<ms>.json`：受管 cargo 派生期间、split 后端与 serve
  服务生命周期内持有；外杀留 dead-pid 件，下次 acquire 清扫；清理器对活 pid
  lease 拒绝、对过期 lease 报告并忽略。owned 树（Job/ProcessTree）随父进程
  全灭。
- `scripts/test-governance/build_storage.py`：精确 `<ROOT>/target/generated-consumers/
  <ctx8>` 根在 `context.json` schema v1 且 `producer_root`=本检出、无活 lease 时
  准入；他 producer/无记录/篡改/嵌套/证据混入/reparse/活动编译/execution lease
  一律拒绝；默认预览零删除，`--apply --idle-confirmed` 删前复核。

## 6. 有限验收口径（2026-11-02 实测，lang-910 producer，同 flags/profile）

| 工作集 | 隔离 A + 隔离 B（unique） | 共享 A+B（unique） | B 在共享根编译单位 / 复用 |
|---|---|---|---|
| UI debug（002-counter A/B） | 10.51 + 10.51 = 21.02 GB | **11.72 GB**（-44%） | 1 / 729÷730 单位 id 相同 |
| Web release（auto-chat A/B） | 1.59 + 1.59 = 3.19 GB | **1.744 GB**（-45%） | 7 / 548÷557 |

unique=file-ID 去重（非 CoW 物理块）；NTFS 分配=unique（无压缩/稀疏）；
行为 oracle=发布件字符串表字面量（UI）、真实 HTTP 响应（Web）。无固定阈值
承诺。

## 7. UI007 移根交接要求

- 接本包四仓最终提交与 canonical 本页；移根视为**新工程来源**：新 id8/新成员
  /重新生成，不按旧 mtime/旧 basename 复用；旧成员字节级不动、不自动删除，
  清理显式预览/apply。
- 保持语言→UI/Web 零依赖方向；发现改收据优先；`auto build -d <dir> -r rust`
  仍走 pac 分支（既有 CLI 缺陷，债务候选）。
- 007 须重跑其 V-09/10 与受影响 fresh/locked（本页 V-01/02/03/08/09 为基线）。

## 7b. 中立交付原语（PLAN-1006）

`auto-generation-core` 拥有两个交付侧中立原语（无 Web/Man/UI 领域知识）：

- `commit` 模块：staging 标记（schema `auto-staged-commit/1`，pid；仅同 pid 残留
  被清扫，外来 staging 一律保留并具名报告）、拒未知非空根、prepare→verify→
  backup/swap 提交事务（fail-point 注入、字节保全还原、`.auto-failed-*` 保留）、
  洁净树拷贝（拒 junction/symlink，跳 target/.git/node_modules/__pycache__）与
  确定性文件映射（sha256）。owner=auto-lang；消费方自带载荷构造与严格校验。
- `source_owner::local_build_closure`：本地 path 依赖闭包枚举（normal/build 边，
  workspace=true 解析，身份/版本校验+环检测；dev-dependency 不入闭包）。与
  `validate_development_dependencies` 共用同一走法，开发模式行为不变。

C/full 策略与开发生成路径不变；Web 成员交付（`auto service-export`）按
[auto-web/docs/specs/generated-service-member-delivery.md] 消费这些原语。

## 8. 集中 workspace 研究附注（PLAN-942，2026-11-06）

本节附研究事实；**910 默认（受管共享 target + 各应用独立生成 ws）保持不变**，
不声称任何新默认已生效。完整方法/证据见
[generated-workspace-sharing-study](generated-workspace-sharing-study.md)与归档 PLAN-942。

- 增量收益未获证明：同兼容族集中 workspace/锁（候选）相对本规范的臂内共享
  target，主口径 ΔAB 为 −286,600B（UI，B 反大）/ +557,736B（Web 累计，超匹配
  重建界仅 12,008B 且分量不可归因），ε_meter=0、E_delta=545,728B。可归因的
  UI 分量为负——不为空间目的切换生产布局。
- 归因限制：Web 域逐阶段锁复演未采纳集中解析的 9+ 传递依赖选择，其差额仅
  描述性；UI 域复演完全采纳（分量可归因）。
- 候选导出实证边界：PLAN-792 交付入口在 producer 921bfa518 生成/主题腿可用，
  但 bundle 未收 `experimental/ace/crates/semantics-contract` → 消费侧
  `--locked` 红（P1，930 原 oracle 同红）；Web 成员移根受 workspace 继承的
  开发根路径依赖阻断（P2）。**P1 已于 PLAN-1003 关闭**（vendored 该 crate +
  交付根输入补 `stdlib/contracts/{operations,compatibility}.toml`，930 driver
  43 项全绿，见 generated-debug-policy-production §6 与 KNOWN-DEBT 登记）；
  P2 仍开放。P2 关闭前，交付链消费侧验证以
  [generated-delivery-regression](generated-delivery-regression.md) 的既有
  绿基线为准，不以上述红推断本规范行为回归。
- 迁移：ctx8 不变时集中 ws 复用既有共享 target、id8 不变，净新增 ~0（假设
  列明于研究页）；未实施、未迁移任何用户目录。
