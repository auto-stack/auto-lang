# PLAN-832 T-01 合同基线：Man/CLI 包图、命令、来源与支持矩阵

状态：work/T-01 实际源码基线；不表示后续迁移、发行或整 Plan 验收已通过。
证据形态分为 **observed**（本任务对当前源/二进制/元数据重采）与 **inherited**（上游已交付收据，后续任务按触面复验）；不把假设写成事实。

## 0. 启动门、固定版本与执行位置

### observed

- G832 成立：PLAN-815 r3（delivery `33c9451fbe667dde2412d71036a19ac334b03691`）、PLAN-816 r2（delivery `ac7c1369c74eab41c9f1a6911d74e2029a772a12`）及 PLAN-831 r3（lang delivery `d7f378b676e3a9dac7257f609936455e5d8248fe`、UI delivery `aee499e87093ff727f68ac8467c506e9ecf6ecdc`）均有 final review、merge 门、Spec/ledger、归档及清理闭环；上述语言提交是当前 master 祖先，UI 提交是 auto-ui master HEAD。
- 本基线源码：auto-lang `45558ab628bb29a8719f8467046b3ae0d5e36d7a`；旧 auto-man `51fafe82e8e0c4e66325ee14088eed8c3270e07e`；UI fixed `aee499e87093ff727f68ac8467c506e9ecf6ecdc`；down fixed `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- 唯一执行组：`D:/autostack/.wt/lang-832/{auto-lang,auto-man}`，分支均为 `codex/plan-832-dev`；`auto-ui`/`auto-down` 为上述提交的 detached 只读依赖树。四树根与分支/HEAD 已读回。
- `docs/plans/attachments/831/ui-release.md` 固定版本表的 lang `e1c640477884cf8720c5a53ca3b9df355085a786` 实际是历史 T-09 R6 提交，并非当前 master 祖先；该文件同节收尾读回及归档记录指向实际 delivery/收尾链。本 Plan 以可核验的实际提交为源，不用错误字段替换 delivery，T-09 须在三仓 handoff 纠正继承口径。
- 主检出的其他会话 WIP/未跟踪文件未纳入本基线；未修改旧 Man 默认代码树，未 push、更名或发布 registry。

## 1. 包归属与 DAG（observed：`cargo metadata --no-deps`，41 个 workspace 成员）

| package（版本） | 当前源与直接 Auto 依赖 | 832 目标 owner / profile | 必要输入、消费者与边界 |
|---|---|---|---|
| `auto 0.1.0` | `crates/auto`；lang-core、man-runtime、tool-client/execution/protocol、project-model、auto-val；可选 UI compiler/runtime/dialect | **Man**；default=完整，显式 `core`=无 UI | CLI/测试/资源随包；五操作经 client→真实 UI tool；`ui-extra` 静态面仍消费 fixed UI source |
| `auto-man-runtime 0.1.0` | `crates/auto-man-runtime`；lang-core、project-model、val、man-core、package-lock/resolver、man-assets、toolchain-core、api-gen、gen/cache、tool-execution | **Man / Core 管理实现** | Pac/Target/Automan、builder/git/http_service/UiOps 单源；不得复制 UI 算法；normal 闭包无 UI 实现 |
| `auto-man 0.1.0` | `crates/auto-man`；runtime、旧 auto-lang、auto-ui-tools、api-gen 等 | **Man / compat facet，不是 Core** | 旧公开路径和 UiOps 默认/显式双槽；当前仍静态重导出 UI，只有独立 Core 闭包可作无 UI 证据 |
| `auto-man-core 0.1.0` | `crates/auto-man-core`；atom/project-model/toolchain-core/val | **Man / Core 纯规则** | pac/target/closure/selection/planning；lang/UI/进程/网络禁止边；man-core-consumer |
| `auto-package-lock 0.1.0` | `crates/auto-package-lock`；auto-val | **Man / Core** | pac.lock 模型/TOML/JSON/verify；package-lock-consumer |
| `auto-package-resolver 0.1.0` | `crates/auto-package-resolver`；无 Auto 直依 | **Man / Core** | 已解析依赖视图/路径规则；package-resolver-consumer |
| `auto-toolchain-core 0.1.0` | `crates/auto-toolchain-core`；auto-val | **Man / Core** | 工具链模型、flag mapper、命令模板；toolchain-core-consumer |
| `auto-man-assets 0.1.0` | `crates/auto-man-assets`；无 Auto 直依 | **Man / Core 资源** | 五族 templates、GHS/IAR embed 与纯路径规划；man-assets-consumer；许可随源映射 |
| `auto-tool-client 0.1.0` | `crates/auto-tool-client`；project-model、tool-protocol、tool-execution | **Man / Core client/discovery** | 真子进程 framed I-TOOL；发现、版本/能力/选项核对及寿命；tool-client-consumer |
| `auto-tool-protocol 0.1.0` | `crates/auto-tool-protocol`；project-model、val | **留 auto-lang，中立单源** | I-TOOL v1 数据/framing；Man 不复制协议 |
| `auto-tool-execution 0.1.0` | `crates/auto-tool-execution`；std-only | **留 auto-lang，中立设施** | UI/Man 双消费；owned tree/reader/writer/timeout/cancel 真停，不迁成 Man 独占 |
| `auto-project-model` / `auto-lang-core` / AST/Value 族 | 当前语言仓 | **留 auto-lang，固定 source** | I-PROJECT、parse/compile/VM/trans、类型身份单源；Core consumer 不依赖 UI/Man checkout |
| `auto-api-gen` / `auto-gen` / `auto-cache` | 当前语言仓 | **留 auto-lang** | api-gen 是语言服务 adapter；gen/cache 是 runtime 的既有依赖，不按名字误搬 |
| UI 五包与 `shell-pack` | fixed auto-ui | **留 auto-ui** | Man 不 vendoring UI 生产实现；完整 profile 以固定 UI release/tool/资产组合 |

目标依赖方向：

```text
auto (Man CLI)
 ├─ fixed auto-lang-core / project-model / tool-protocol / tool-execution
 ├─ auto-man-runtime (Man)
 │   ├─ Man cores + auto-api-gen/auto-gen/auto-cache (fixed lang)
 │   └─ tool-execution (fixed lang)
 ├─ auto-tool-client (Man) ──真实进程──> auto-ui-tool (fixed auto-ui)
 └─ ui-extra ──fixed source──> UI compiler/runtime/dialect/shell-pack

auto-man compat (Man, 非 Core) → runtime + UI 门面
```

正常、build、dev 三种 metadata 均须分别验证；dev 依赖不得被误计入 Core normal 闭包，也不得因漏迁 dev 输入而丢测试。同名 SDK/协议/AST/Value 不得出现第二 source 身份。

## 2. 完整 CLI 命令与 flags（observed）

实际入口：`cargo build -p auto` 后执行 `target/debug/auto.exe`；版本输出为 `auto 0.1.0+v0.5.0-1870-g45558ab62`。二进制 SHA256=`5a8010f7dc2dd033b0e1b3cd64ee5b3a74601cfb054bc18c7f6c98e0eba5a685`（仅本基线参考产物，不承诺跨构建复现同 hash）。

全局：`-e/--error-limit N`、`--format text|json|at`、`--ai`、`-D/--debug`、`--dump-lowered`、`-h/--help`、`-V/--version`；位置形 `auto <file> [ARGS...]` 使用 trailing var arg/allow hyphen，`--` 可强制透传。完整原文（含 30 个可见命令及 hidden/alias 帮助；仅去除行尾空格）见 [cli-help.txt](cli-help.txt)，1214 行，SHA256=`5695795d071b01d1c62e3a07ff83e687e9eb00fcf353e0c621a553e0f33391f0`。

| 命令 | 参数/默认与输出或寿命合同 | 目标 owner / 既有 oracle |
|---|---|---|
| 无参数 / `auto <file> [args…]` | REPL / AutoVM；脚本 argv 透传；`AUTO_SOURCE_DIRS`；`--dump-lowered` | lang SDK + Man 薄路由；cli_passthrough、plan524/555 |
| `new` / `init` | `name`（init 无 name）；`-t app|jet|capp|lib|clib` | Man 管理+assets；脚手架测试 |
| `build`（`b`） | `-d/-p/-B 8080/-F 3000/-r/--scene/--gen-only/--strict/--lenient` | Man 编排+真实 UI tool；命令/driver+792 |
| `run`（`r`） | build 参数 + `-q/--server vm|rust/--no-merge/--merged/--theme/--accent/--desktop/--gallery/--apps/args…` | Man 编排+lang/UI；真实运行/取消；legacy 端口策略只属此链 |
| `service` | `-d/--server vm|rust`（必填）、互斥 `--http-config|--http-config-inline`、`-B` | Man http_service；ready/端口冲突/退出；VM 线程与 Rust owned tree |
| `test`（`t`） | `-d/-f filter/-v` | lang 执行 + Man 路由；语言测试族 |
| `test:ui` | `-d/-H/-f/--report`；外部 Playwright/npx，需先启动 app | Man CLI 驱动；autoui-verifier |
| `clean` | `-d`；删除 `.auto/build` | Man FS adapter |
| `watch` | `-d/-B/-F`；100ms 去抖、内容 hash、SFC/HMR、终止原语 | Man watch + full UI；真实变更/退出 |
| `wizard new|add` | `name`；include_str 模板 | Man CLI/full UI 输入 |
| `add` / `fetch` / `deps` | `add <package>`；依赖解析、fetch/pull、图展示 | Man resolver/git/lock |
| `device list|select` | `select <port>` | Man 设备/端口管理 |
| `export` | `-p <port> -f cmake|iar|ghs`（不是全局 `--format`） | Man exporter+toolchain/assets |
| `info`（`i`） / `open`（`o`） | info `-t target`；open 调 IDE/explorer | Man 展示/进程 adapter |
| `env reset|install|cache …` | install `<file>`；cache `stats|list [--type_ --limit 50]|prune|clear|inspect <name>|verify` | Man config+cache（cache 留 lang） |
| `gen` | `-o output/-p/--scene/--delivery-root`；rust delivery 显式 | Man CLI→I-TOOL；792/815/816 负控 |
| `ui build|list|inspect|backlog` | build `--target vue --out packages/widgets/registry --widgets …` | full `ui-extra`；UI compiler |
| `bp list|show|add|bind|check`（hidden `block`） | catalog key/reference、out `src/front/bps`、`--bind/--dep`；支持全局 `--format at` | full `ui-extra`；bp 测试/CI 漂移门 |
| `docs …` | schema 文档子命令 | Man CLI 路由 + lang 文档；docs_gen/tb |
| `stdlib inspect` | `--module/--target vm|rust|c/--environment native|browser/--actual file/--check`；exit 0/1/2/3 | lang stdlib 装配；plan738 |
| hidden `oldrepl|parse|eval|config` | 旧 TreeWalker/parse/eval/config | lang 兼容；不得冒充 orphan |
| `trans ts|c|rust|python|js|gd|tscn|godot|auto` | `-p <path>`，各目标 `-o`；rust 另 `--merge` | lang transpiler；tt |
| hidden `c|rust|r2a|python|javascript|gdscript|a2c-stdlib` | 文件/输出参数；旧转译别名 | lang transpiler；tt |
| `debug`（`dbg`） | `<file>`、`-a/--agent`（stdin JSON 命令） | lang debugger；plan199 |
| `cffi` | `--header <name>` | lang bindgen；plan216 |
| `mcp` | stdio MCP 循环 | lang MCP；plan265 |
| `serve` | `--foreground/--stdio/-p autovm/--max-sessions 20/--timeout 1800` | lang daemon + Man spawn_detached；不是 `service` |
| `req` | `-s/-p autovm/--new-session/--inspect/--reset/--delete/--snapshot/--list/--json [code]` | lang daemon client；命名管道/session |
| `rqhost` | `--pipe`（env `AUTO_RQHOST_WELLKNOWN`）、`--bridge-ws`、`--bridge-token`（requires bridge-ws） | full UI 桌面协议；rqhost 测试 |

`cmd_vue.rs`/`cmd_tauri.rs` 没有挂入当前 mod 树，仍为 **orphan**，不计作第 34/35 个命令，不在本 Plan 新增或删除其支持承诺。

机器输出合同：`--format json`/`--ai` 为单 JSON，模板进度走 stderr；错误含可用的 code/severity/spans/help/related，成功为 `{status:"success",result:…}`；正常退出 0、错误具名非零。`stdlib inspect` 的 0/1/2/3 特殊分类、daemon req 的 `--json` 与全局格式不得混并。

## 3. 环境、发现、来源与安装入口

### observed / inherited 待 T-05..T-07 重验

- 工具发现：`AUTO_UI_TOOL_BIN`（显式缺席即错）→ workspace `target/{debug,release}` → 当前 exe 邻接；每个候选以真实 describe 验 `auto-ui-tool`、protocol=1、所需操作，shutdown 失败也非零；禁 build.rs 偷编、任意兄弟仓搜索、静态 UI fallback。
- I-TOOL v1：describe/validate_project/generate_ui/build_ui/run_ui + input/cancel；generate 选项 `output/project/delivery_root/strict`、run `args`、build `strict`。发送前能力核对、报告落点核实、工具 StrictGuard 三层守门保持。
- 开发 profile 才允许 env→组内兄弟→主检出解析；独立 clone/安装/第三方 consumer 必须清 `AUTO_*` 根、parent workspace、原 target 与任意邻仓 fallback，使用固定 rev/锁或内容寻址源码包。
- 现行安装证据（inherited 816）：双 exe 邻接 + 安装邻接 `sdk/`（crates+lock+根级输入）；`rust_workspace` producer 解析 env→exe/sdk→CWD→编译期，默认安装 producer.source_root=安装 sdk。正式 Man 独立安装资源布局、DELIVERY_CRATES 增补与升级交换是本 Plan T-05..T-07 必做，不能沿用“R07 后续”而再次延期。
- 792 受管交付：staging→完整性/来源/资源校验→交换；唯一 staging/backup 标记、陌生目录不删、准备失败保旧字节、成功后才回收已替代残桩。CLI 安装若缺少受管交换，只可在同一语义内有界扩展。
- 锁来源：lang 根 `Cargo.lock` SHA256=`103ea5b71e6fa86fc915a8e6a9c58c13c94c36cd34ab80d081725538a6d834e1`；fixed UI/down 各有受管锁。T-04 必须生成 Man 独立锁并记录固定 Git/source、包版本、补丁及 hash，不能借语言主锁冒充 Man 锁。

清单 SHA256（observed）：

| 文件 | SHA256 |
|---|---|
| `crates/auto/Cargo.toml` | `b7d020ce3ac69d802fa6a3606fc01c7144a81918ab8b8787a7e4240481668a98` |
| `crates/auto-man/Cargo.toml` | `525d222e689f2c1cdbb0effb5e8ed7322da6a141a5a22fe453b60b8722bd8e47` |
| `crates/auto-man-runtime/Cargo.toml` | `a96e8db1eb5dc669b110e7689500bdfe953489916acdd8c403749203630a6245` |
| `crates/auto-man-core/Cargo.toml` | `8701c1a29868900da6c3ef34f55f36f56bdbdba5b77d38da82f77f27207328bc` |
| `crates/auto-package-lock/Cargo.toml` | `f06101944ba4c190e3ebf188442eab3f81bab5819818cdd3d2494d8f3a05b98d` |
| `crates/auto-package-resolver/Cargo.toml` | `a9545441d6abff9ee86d27c07a879f569dee13a1dbf3f57435c376ea731fa15e` |
| `crates/auto-toolchain-core/Cargo.toml` | `d474f88f4690eeb0bac97032c0be5d0ac7fe820526e80073fc7c3722f3cbe0d3` |
| `crates/auto-man-assets/Cargo.toml` | `bedf832b7fb7f08214657037e2c02906ab70b30018ba5aa929b38426f23ceff2` |
| `crates/auto-tool-client/Cargo.toml` | `4e0966ad050a21d51dc37a9e880d3ba6b0b972602db3f9113722364e947a43bf` |
| `crates/auto-tool-protocol/Cargo.toml` | `56fa0b131bfad2919094b3e9be775160728312f848d6b1e3b88ef1386217f266` |
| `crates/auto-tool-execution/Cargo.toml` | `a5ede6f258251dbe670beac75cdbc571eeed61b46a0167b565aca2ae4498d42f` |

## 4. 旧 Man、资源与迁移边界（observed）

- 旧 auto-man 仓当前是单 crate workspace，`auto-man 0.1.4`，提供旧 `am` 命令及旧 auto-lang/auto-val/auto-gen sibling path；master 干净。它不是当前 0.1.0 管理实现的来源权威，不得与 lang 现行实现双生产。
- 现行权威源是上表 lang 包；T-04 从固定 lang source 导入，建立 source commit/path/hash→Man commit/path 映射。旧 `am`/automan/config/docs/release.ps1 是否保留为历史兼容输入须逐项映射，不能用旧实现覆盖新实现。
- 需要随 owner 的输入：各 crate `src/`、`tests/`、`build.rs`（若存在）、assets/embed、`Cargo.toml` 特性与 dev-dependencies、consumer fixtures、777 域/基线、792 producer/交付表、cross-repo tiers、文档和许可。生成物/target/主仓锁不是可复制生产源。
- `auto-man` compat 与 `auto-man-runtime` 必须分闭包验证；以 compat 能链接 UI 为由宣称 Core 无 UI 是无效 oracle。UI 与 lang 不得新增反向依 Man 实现。

## 5. 寿命与支持矩阵（inherited 合同，T-05..T-07 实测）

| 面 | 必须保持的可观察行为 | 负控/边界 |
|---|---|---|
| 语言/Core | 无 UI checkout 的 parse/compile/VM/trans、脚本 argv、REPL、docs/stdlib、管理命令 | UI 请求具名失败；同名双 source/隐式 path 拒绝；不只 cargo check |
| UI generate/build/run | fixed UI tool 经 client；generate 六 profile，build/run 按 tool 实际能力；Vue/VM-Iced/Rust 生成运行 | 错版本/能力/producer/缺件在副作用前拒绝；Core 不拉 UI |
| service | VM=进程内大栈/线程亲和；Rust=生成+cargo owned tree；ready 行与真实健康/bind；正常/失败/取消退出 | 不杀端口占用者、不假 ready；配置/端口/生成失败非零；只清本次 owned 对象 |
| serve/req/mcp | 原 daemon/stdio/命名管道与 session 角色 | 不合并成 service，不改协议；后台 spawn 与前台 stdio 各自寿命 |
| watch/HMR | 变更再生、缓存、终止原语、reader/child 真停 | 旧 SFC 内容负控；阻塞/超时/外部持管道不得假返回 |
| Windows/Unix owned 执行 | Windows Job、Unix 负 PGID 组信号；取消/超时/EOF/非零/读错后收尸+join | DirectChildOnly 不误杀；外部 writer 不归我方终止；未实测平台不得外推 |
| 安装/升级/重定位 | fresh→locked；A→B 无 target；fixed source/tool/资产/锁可重放；升级保旧 | 坏锁/hash/pin/source/patch、半安装、取消、交换失败均具名非零且可恢复 |
| 平台工具链 | 按现行支持矩阵逐项尝试并记录工具存在性、生成、构建、运行 | 缺环境的最终必需正控记 pending，不能勾整体验收；不得静默删后端 |

## 6. 测试与 CI 基线

### observed 当前选择面

- `cargo test -p auto --bin auto -- --list`：24 tests / 0 benchmarks；原文 [cli-test-list.txt](cli-test-list.txt)，SHA256=`2ed10d2edd03de19a2b73cbc8f57b45943a8be7b8cfa5d004ce4bef27e2dd75c`。
- `cargo test -p auto-man-runtime --lib -- --list`：122 tests / 0 benchmarks；原文 [man-runtime-test-list.txt](man-runtime-test-list.txt)，SHA256=`3c4c5df941c82eacfa77afd5b64927b49d431ec2b39cf40c031451ffa5c888d2`。
- `cargo test -p auto-man --lib -- --list`：4 tests / 0 benchmarks；原文 [man-test-list.txt](man-test-list.txt)，SHA256=`dfab8d9c67f26b01e9b0b33a49ede7d28cf69494621d3709033a96e90b8fc937`。
- 集成源：`auto/tests/{cmd_ui,plan816_cli_driver,autodesk_client_smoke}.rs`；runtime 三个 plan816 集成目标；auto-man 十个集成目标（plan803/805×2/809/811/815/816×4）。T-03 要生成逐测试 ID 映射而非只比较总数。

### inherited 需接线的既有门

- 管理/core：man-core、package-lock、package-resolver、toolchain-core、man-assets 独立 consumer 及各自 `verify-deps.sh`；plan803/805/809/811/815/816 回归。
- 执行/协议/client：tool-execution std-only consumer；tool-client consumer（真实 UI tool/te-child，discovery、options、pump、寿命）；plan816 CLI driver；协议 fixture 与深度/帧负控。
- 语言反向控制：language-sdk-consumer 无 UI/Man 的真实 parse/compile/VM/trans。
- 发行：generated-ui-consumer 792 driver、UI distribution/switch drill、legacy-preservation inventory/preserve/verify/recover-drill、777 `modularity-audit.py --mode strict --no-cargo-check`。
- CI：两仓均无受管 `.github/workflows`；本地受管入口即 CI 入口。现行 `scripts/cross-repo-tiers.py run|list` 编排 lang/UI/down，T-08 扩为含 Man 的固定来源队列并做迁前后 ID 对拍；tf 仍主语言检出单实例批量，重型资源不进入普通并行队列。

## 7. 后续任务的具名输入（不是已完成声明）

- **T-02**：重采旧 Man refs/WIP/ignored（已知历史 ignored=`Cargo.lock`、`.vscode/iar-vsc.json`，必须刷新而非照抄）、状态寻址保全与 recover-drill；确认 784 旧准备树/分支 owner，不强移。
- **T-03**：建立 three-repository-consumer 与安装/失败/命令 driver，固定 V-01..V-06 输入和测试 ID 映射；真实正控不可用 fake 代替。
- **T-04/T-05**：按 §1 导入并生成 Man workspace/lock/CI/Core 安装；复核 normal/build/dev 闭包、资源与许可。
- **T-06/T-07**：绑定 fixed UI release，完成三仓真 tool、服务/watch、fresh/locked/重定位/升级失败与恢复；最终缺环境正控不得假绿。
- **T-08/T-09**：先存在可消费 Man 目标提交，再切 lang/删原件；更新 777/792/CI/docs、统计 source/generated/third-party，冻结 SD-01..06 与 handoff。
- **review/merge**：T-10 独立全 AC/SD 审查后，T-11 才运行冻结整档并有序落地；T-12 才发布 canonical Spec/ledger、归档和 guard 清理。
