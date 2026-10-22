# PLAN-816 T-01 合同基线(命令矩阵、现有执行设施映射与服务/compat DAG 冻结方案)

状态:work T-01 交付物。证据基线:主检出 master `9e80ae0fb`(worktree `D:/autostack/.wt/lang-816/auto-lang`、分支 `codex/plan-816-dev` 自该基面创建;§2 行号除注明外均为 `9e80ae0fb` 时点)。
本文件冻结 T-02 执行设施合一的 crate 布局/语义边界/预算入口、给出 M816-B 薄 CLI 迁移的逐命令矩阵与 adapter 归属、并记录 bootstrap/profile 候选(§5.3 要求的最终布局在 814/815 合入后由门后整链 probe 定稿,本文件先冻结候选与决策门)。

## 1. 冻结决策(T-01 有界 API/DAG probe 结果)

| 决策点 | 冻结值 | 依据 |
|---|---|---|
| 执行设施 crate | **新建 `crates/auto-tool-execution`(workspace 成员)**:统一 owned 子进程执行设施——argv 数组、显式 cwd/env、stdin/stdout/stderr 策略(继承/管道捕获/null)、退出/错误分类(成功/非零/超时/取消/终止/派生失败)、deadline(毫秒)、cancel + owned reap、reader 线程真终止、Windows Job Object / Unix 进程组的**子进程树**退出 | §5.2"复用现有 facility 并将必要共享缺口合一" |
| 零依赖闭包 | `auto-tool-execution` 依赖闭包 = **仅 std**(无 auto-lang/auto-man/auto-val/serde/三方);777 域登记 `man-ecosystem`(与 auto-toolchain-core 同层:man 生态纯规则/设施层);闭包由 T-07 `test/fixtures/modularity/tool-execution-consumer/` 钉住 | 消费方三重性:(a) T-03 管理服务(无 UI 闭包)、(b) T-12 协议客户端独立 consumer、(c) T-07 fixture——模块挂在 auto-man 内会经其 `Cargo.toml:12`(`auto-lang features=["ui-iced"]`)把 UI 实现拖进每个消费方 |
| `auto-man-runtime` 服务包 | **需要**(T-03/M816-B 创建,A 阶段不建):probe 证据——`crates/auto-man/Cargo.toml:12` 硬依赖 `auto-lang` 带 `ui-iced`,vue.rs(14654 行)/rust_ui.rs(8712 行)/api_gen.rs(9562 行)/jet/ark/tauri/vscode 全在 auto-man 内(815 基线 §2.1 逐符号图);非 UI 管理面(http_service/resolver/pac/target/git/builder/lock)无法在不链接 UI 实现的前提下被服务消费。旧 auto-man 保旧名上层 compat(含 UI 公开门面),新建具名包承接非 UI 管理/执行实现;单源类型(Pac/Target 经 809 auto-man-core re-export 保身份),不复制可变工程状态 | §2"是否需要该包由 T-01 有界 API/DAG probe 冻结;禁止保旧 UI 重导出又宣称同一服务包纯净" |
| Windows .cmd/.bat shim | 新设施默认**直接 argv、零 shell 解释**;`cmd /C` shim(现 main.rs:921 run_command_live / pkg.rs:212 run_script_live 模式)作为显式具名选项 `windows_cmd_shim`(仅 Windows),由 T-05/T-06 迁移既有 npm/pnpm 调用点时接线——不是新默认 | §5.2"argv 数组传参";防 shell 注入面 |
| legacy 端口处理隔离 | `kill_process_on_port`(auto-man/src/util.rs:215,含 PLAN-082 保护名单 `process_name_of`/`should_skip_kill`)与 `rust_ui.rs:5942` 调用点、以及 `rust_ui.rs:5886` 的 env 旁路,**专属 legacy `auto run` 链**;新设施零端口清场逻辑;`auto service`(http_service.rs 模块头合同)端口冲突=bind 失败=非零退出、不 kill 占用者、`AUTO_SERVICE_READY` 行无假 ready——此合同为 V-04/V-02 oracle,不得扩散到新工具/fixture | §5.2"旧 run 链已有的显式 legacy 端口处理策略由 T-01 按现行合同具名隔离,不能扩散到新工具/fixture 或 service" |
| 预算入口/单位 | 新设施:deadline=毫秒(`Duration`),入口 `run(spec)`(一次性)与 `OwnedProcess::cancel()`(长运行);诊断=具名错误串(派生失败含程序名+OS 错误;超时报告已运行时长)。**不截断合法输出**——stdout/stderr 捕获无预算默认上限(与 stdlib 64KB 截断相反;该截断属语言 sys.exec 合同,不动);帧级/队列级预算属 815 协议(8MiB/128),T-12 消费时叠加 | §5.2"所有新预算在 T-01 冻结单位/入口/诊断/边界控制,不把既有合法脚本输出无声截断" |
| 语言侧进程 API 现状 | `vm/ffi/stdlib.rs:2726` Process.spawn(status 同步等待)、`:2746` spawn_with_output(64KB 截断)、`:2803` sys.exec(timeout_ms 参数**未实现**,注释"timeout currently unused")——记录为已知缺口,归语言 adapter(T-03+);T-02 不改语言面 | §4"T-01 从真实调用盘点,未调查不能宣称已有完整 supervisor" |
| orphan 文件裁定 | `cmd_vue.rs`(2008 行)/`cmd_tauri.rs`(983 行)未被 `main.rs:9-16` mod 树声明 = **未挂载**,不当作现行命令;cmd_vue 消费 vue_shadcn 离线源(815 基线 §2.1 注),其去留随 815-B 迁移单独决策,本 Plan 不新增/不移除命令 | §4"有 cmd_vue/cmd_tauri 等 orphan 文件,不能当已挂载命令" |

## 2. 真实 clap 命令矩阵(`crates/auto/src/main.rs` @9e80ae0fb)

全局旗标(所有命令):`-e/--error-limit N`、`--format text|json|at`(json ⇒ ai_mode)、`--ai`(JSON+抑制人读)、`-D/--debug`(VM 日志)、`--dump-lowered`(仅脚本形态)。机器输出统一走 `format_error_json`/`format_success_json`(main.rs:24/84;错误 JSON 带 code/severity/spans/help/related)。

| 命令 | 参数/默认 | 进程/env/输出 | 现行 owner(→目标归属) | 直接测试 |
|---|---|---|---|---|
| `auto <file> [args…]`(positional) | `--dump-lowered`;argv 透传 Plan 524(trailing_var_arg;`--` 后强制透传) | `auto_lang::run_file_with_args`(main.rs:1020);AUTO_SOURCE_DIRS 模块搜索 | 语言 VM(语言 adapter) | dump-lowered/透传有内联用例(plan524/555 测试族) |
| (无参数) REPL | — | `autovm_repl::main_loop`(main.rs:2841) | 语言 VM | 手动面;driver T-07 |
| `new/init` | `-t 模板(app/jet/capp/lib/clib)` | `Automan::create_by_template/create_app`(main.rs:1043/1073) | 管理(管理 adapter;模板 803 auto-man-assets) | auto-man 内联 |
| `build`(b) | `-d/-p/-B 8080/-F 3000/-r vue|rust|vm|jet|arkts|tauri/--scene/--gen-only/--strict(默认)/--lenient` | set_var AUTO_HTTP_PORT/AUTO_FRONT_PORT(main.rs:1158/1171);`am.scan()+am.build()` | 管理+UI 工具(815 迁移后 UI 归 auto-ui-tools) | vue/rust_ui 内联 ~3700 行 |
| `rqhost` | `--pipe/$AUTO_RQHOST_WELLKNOWN/--bridge-ws/--bridge-token` | `desktop_protocol::rqhost::run_daemon/run_bridge`(main.rs:1211/1221);锁管道被占=第二实例退出 0 | UI 桌面协议(815/UI 域) | rqhost 测试族 |
| `run`(r) | 同 build + `-q/--server vm|rust/--no-merge/--merged/--theme/--accent/--desktop/--gallery/--apps/args…` | 预算 fail-fast(PLAN-781,main.rs:1279)→ Plan 480 孵化分岔 `cmd_autodesk::run_if_client_entry` → 端口/窗口/title env 注入 → `am.run()` 全编排(VM 进程内/ui 进程/npm 链) | CLI 编排(→T-05 拆薄:管理服务+语言 adapter+815 UI 工具) | run 链 e2e 属 V-04 |
| `service` | `-d/--server vm|rust(必填)/--http-config/--http-config-inline(互斥)/-B` | `cmd_service.rs` → `auto_man::http_service::serve_project`:vm=进程内大栈线程(32MiB)+ready 轮询(120s)+`AUTO_SERVICE_READY`;rust=api_gen 生成+子进程 `cargo run`(AUTO_HTTP_SERVICE_JSON 注入,inherit stdio,wait 收尸,子非零⇒本进程非零);**不 kill 端口占用** | 非 UI 服务(adapter 面;cmd_service.rs:10 是薄 CLI 路由样板) | http_service.rs 内联 5 测;真 TCP 属 `cargo th`/V-04 |
| `test`(t) | `-d/-f 过滤/-v` | 收集 .at/.au → `run_file` 逐个(#[test] 语义) | 语言 VM | 语言测试族 |
| `test:ui` | `-d/-H headed/-f/--report` | Playwright 子进程(npx,外部工具;先 `auto run`) | CLI 测试驱动(→UI 工具) | autoui-verifier 技能面 |
| `clean` | `-d` | 删 `.auto/build` | 管理 FS 效果 | — |
| `watch` | `-d/-B/-F` | cmd_watch.rs:notify watcher+去抖 100ms+内容 hash 缓存(`.auto/build/cache.json`,GENERATOR_VERSION 失效);SFC 增量再生+HMR;backend .at→a2r `gen/back/` | CLI watch(→T-06 接线到设施;扫描/缓存业务移 owner) | 手动面;V-02 watch 变更 |
| `wizard new/add` | name | autoui-skill 模板 include_str 落盘(main.rs:2846) | UI 工程工具(815) | — |
| `add/fetch/deps` | package | pac.at 依赖(auto-package-resolver/resolver.rs;git.rs 子进程) | 管理 adapter | lock/resolver 测试族 |
| `device list/select` | — | 硬件端口枚举(management) | 管理 | — |
| `export` | `-p port/-f fmt(cmake/iar/ghs)` | `am.export()`(main.rs:2035) | 管理工具链(auto-toolchain-core/GHS/IAR 资源 803) | 803 资源测试 |
| `info`(i)/`open`(o) | `-t`/— | info=包/后端/target 展示;open=IDE spawn(`code`/explorer/xdg-open,automan.rs:545..) | 管理展示/进程效果(→迁移到设施) | — |
| `env reset/install/cache{stats/list/prune/clear/inspect/verify}` | — | AmConfig/AutoCache(auto-cache) | 管理 | plan082 cache 测试 |
| `gen` | `-o/-p/--scene/--delivery-root(792)` | kotlin/vue/jet 生成入口 `am.generate()` | 语言+UI 生成(815) | 792 driver 24/24 |
| `ui build/list/inspect/backlog` | `--target vue/--out packages/widgets/registry/--widgets` | cmd_ui.rs→`ui_gen::VueGenerator` | 语言 ui_gen(→815 UI 工具) | ui_gen 测试族(cargo tu) |
| `bp list/show/add/bind/check`(+`--format at` 全局;hidden 别名 `block` PLAN-639) | key/reference/out=src/front/bps/--bind/--dep | cmd_bp.rs:蓝图注册表(复制/bind/静态验收) | UI 蓝图(815 域) | plan075_bp_tests;bp list --format at CI 漂移检查 |
| `docs` | 子命令 | cmd_docs.rs:schema 文档生成 | 语言文档 | cargo tb(docs_gen) |
| `stdlib inspect` | `--module/--target vm|--environment/--actual <file>/--check` | cmd_stdlib.rs:装配清点;退出码 0/1/2/3 语义(PLAN-738) | 语言 stdlib 装配 | plan738 测试族 |
| hidden legacy:`oldrepl/parse/eval/config` | — | TreeWalker/解析/求值(main.rs:2353..) | 语言 | — |
| `trans ts/c/rust/python/js/gd/tscn/godot/auto` | `-o 输出` | `auto_lang::trans_*` 单文件转译(main.rs:2435) | 语言 transpiler | cargo tt |
| hidden 转译别名:`c/rust/r2a/python/javascript/gdscript/a2c-stdlib` | — | 同上(main.rs:2377..2608) | 语言 | cargo tt |
| `debug`(dbg) | `file/--agent(JSON,stdin 命令)` | `debug_file(_agent)`(main.rs:2830) | 语言调试器 | plan199 族 |
| `cffi` | `--header` | C FFI manifest 清单 | 语言 bindgen(auto-bindgen) | plan216 族 |
| `mcp` | — | `McpServer::run()` stdio 循环(main.rs:2661) | 语言 MCP | plan265 族 |
| `serve` | `--foreground/--stdio/--pipe-name autovm/--max-sessions 20/--timeout 1800` | daemon:`AutovmDaemon` 具名管道/stdio;后台模式 **spawn 自身** `serve --foreground`(Windows CREATE_NO_WINDOW 0x08000000,main.rs:2686) | 语言 VM daemon(≠service,两协议不混) | plan269 族 |
| `req` | `-s/-p/--new-session/--inspect/--reset/--delete/--snapshot/--list/--json/code` | `AutovmClient` 具名管道客户端;匿名 eval_one_shot | 语言 VM daemon 客户端 | plan269 族 |
| **orphan(未挂载)** | cmd_vue.rs/cmd_tauri.rs | 无 mod 声明 ⇒ 非现行命令 | — | — |

## 3. 现有执行设施映射(T-02 合一依据 + M816-B 迁移 owner 表)

每行:现行站点 → 语义 → 缺口 → 归属(新设施 API 或显式不动)。

| 现行站点(证据) | 语义 | 缺口(新设施补) |
|---|---|---|
| main.rs:921 `run_command_live`(+cmd /C shim) | inherit stdio+status;where/which 探测(main.rs:900) | argv 数组、捕获、取消、deadline |
| auto-man/src/pkg.rs:212 `run_script_live`+`ChildGuard`(kill-on-drop,:201)+install/run_script | stdin 继承交互(Vite q 退出);drop 兜底 kill | 退出分类、tree kill、deadline;shim 迁移点 T-05 |
| auto-man/src/git.rs:20/42/66/81(4 处同型) | output() 捕获+status 判定 | 派生失败分类/env 显式;T-03 迁移点 |
| auto-man/src/http_service.rs:230 serve_rust | cargo run spawn+env 注入+inherit+wait 收尸 | ready/健康轮询(注释明示 T-05 接管);归语言/服务 adapter,T-06 用设施重写 |
| auto-man/src/automan.rs:545..(IDE/explorer/xdg-open)、:1782(cargo)、:2022(rust_server+清理 kill :2118) | 分散 spawn+选择性 kill | 统一 owned 语义;open 命令 T-05 迁移 |
| auto-man/src/ark.rs:736 hvigorw;vscode/jet 同型 | 工具链子进程 | 工具发现归 815 I-TOOL/管理 adapter |
| auto-man/src/util.rs:215 kill_process_on_port(PLAN-082 保护名单) | **legacy run 链专属**端口清场 | **不迁移**——§1 冻结隔离;只留 run 链 |
| rust_ui.rs:5942(默认 kill+重生成+spawn;:5886 env 旁路可跳过) | run 链后端占位清理 | 同上,legacy 专属 |
| crates/auto/src/main.rs:2686 serve 后台自身 spawn(CREATE_NO_WINDOW) | daemon 化 | spawn 选项(creation_flags)纳入设施 spawn 策略;daemon 协议本身不动 |
| 语言 stdlib stdlib.rs:2726/2746/2803 | 同步 status/output/64KB 截断;sys.exec timeout 参数空置 | 语言 adapter 面(T-03+ 决定消费方式);T-02 不改 |
| watch:cmd_watch.rs(notify+mpsc+去抖) | 文件事件循环;无子进程 | watcher 寿命归 T-06 接线(设施提供 cancel 原语) |
| daemon/pipe:autovm_daemon、rqhost daemon、mcp stdio | 长运行服务寿命 | 各守原线程/管道归属;设施只提供 owned 子进程原语,**不新建第二 runner**(§1 目标:一份设施,上表分散站点在 T-03/T-05/T-06 陆续迁移,不永远并存) |

## 4. 服务与 compat DAG(T-03 依据)

```
auto(CLI,clap+显示/路由)
 ├─ 语言 SDK(814 M814-A 发布后):parse/compile/VM/trans/config
 ├─ 管理服务 auto-man-runtime(T-03 新建,非 UI):
 │   ├─ 809 auto-man-core(Pac/Target 单源声明)+795/800/803/805 已拆 cores
 │   ├─ http_service/resolver/pac/target/git/builder/lock(adapter)
 │   ├─ auto-tool-execution(T-02,本 Plan 设施)
 │   └─ (A 阶段不存在;M816-A 客户端独立消费 815 协议+本设施)
 ├─ UI 工具(815):auto-ui-tools/auto-tool-protocol(I-TOOL)
 └─ 旧 auto-man:上层 compat(含 UI 公开门面 vue/rust_ui/... 815-B 迁移),不进纯服务闭包
```

- CLI 正常闭包现状:auto → {auto-man(→auto-lang+ui-iced+全 UI 生成器)、auto-lang(default=ui-iced+python+autodown 经 auto 侧 feature)、shell-pack}——**UI 实现在闭包内**,V-01 的 Core profile 负控即此现状的对照面。
- cmd_service.rs:10 只消费 http_service(非 UI)——薄 CLI 路由的现行样板;T-05 以此形态推广全命令。
- 服务/daemon 两协议不混:`service`(HTTP 独立服务,PLAN-736)≠`serve`(AutoVM daemon 具名管道,Plan 269)≠`mcp`(stdio MCP)。
- 线程归属:VM 进程内服务守大栈线程(http_service.rs:132 32MiB 样板);!Send VM 不跨 OS 线程;独立 UI 进程各守原归属。

## 5. Bootstrap / Profile / 预算候选(门后定稿项)

| 项 | 候选(现行证据) | 定稿门 |
|---|---|---|
| 发行 profile | 完整(现行默认):auto features [ui-iced,python,autodown](crates/auto/Cargo.toml:29);Core 候选:剥 ui-iced/autodown(语言/管理命令全可用,UI 命令具名失败);独立安装闭包:auto + auto-ui-tool + auto-tool-protocol + 资源/锁(792 DELIVERY_CRATES 增补,815 基线 §1 已注) | §5.3:T-01 结合 814/815 合入后 probe 冻结(M814-A/M815-A 发布后) |
| 工具发现序 | ①已配置/已安装匹配 tool(815 describe/版本)→②源码开发按需构建由**运行期具名开发 adapter/脚本**完成→主检出解析序 env 覆盖→组内兄弟→主检出(AGENTS.md);禁 build.rs 偷编译 UI、禁搜任意兄弟仓 | M815-A 发布(T-04 验证窄能力+缺件负控);完整 bootstrap 等 M815-B |
| 预算入口(现行) | AUTO_HTTP_STEP_BUDGET(run/service fail-fast,PLAN-781);heavy_gate 机器锁(gallery/1M,PLAN-726);815 帧 8MiB/队列 128/进度 1MiB;stdlib sys.exec 64KB 截断+空置 timeout;**新增:设施 deadline(ms)** | T-02 落地设施预算;其余各守原合同 |
| 777 域登记 | `auto-tool-execution`→man-ecosystem(§1);815 两 crate 由其落地轮登记 | 随代码合入 |

## 6. T-02 合同(据本基线冻结)

1. 新建 `crates/auto-tool-execution`(零三方依赖,仅 std;lib+内联测试);workspace 成员登记。
2. API:`ExecutionSpec{argv: Vec<OsString>, cwd, env(增量), stdin/stdout/stderr: StdioPolicy{Inherit,Capture,Null}, deadline: Option<Duration>, windows_cmd_shim: bool}`;`run(spec)->ExecutionResult{exit: ExitKind, stdout/stderr(Capture 时), elapsed}`;`spawn(spec)->OwnedProcess{cancel(), wait(), id()}`。
3. `ExitKind{Success, Failure(code), Timeout{after}, Cancelled, Terminated, SpawnError{program, source}}`——具名分类,诊断含程序名;Windows 无 code(被强杀)≠Success。
4. owned 语义:cancel/超时后**观察到子进程真停**(kill→wait 收尸)才返回;reader 线程 join 后才返回;树退出:Windows Job Object(KILL_ON_JOB_CLOSE+TerminateJobObject)、Unix setsid+killpg——只杀本次拥有的树,不触碰共享 auto.exe/无关服务(§1 legacy 隔离)。
5. 具名测试子进程:测试二进制自 re-exec(env `AUTO_TOOL_EXEC_TEST_CHILD=<mode>`;mode=echo-args/exit:N/sleep:N/grandchild:N/stdout:N/stderr:N/stdin-echo);孙进程观察=heartbeat 文件停止推进(kill 发出≠已停的反例断言)。
6. 正反例面(AC-02 当期):argv 精确透传(OsString/含空格/Unicode)、cwd/env 生效、exit 0/非零分类、派生失败具名(不存在程序)、stdout/stderr 分离捕获(大输出不死锁)、stdin 管道写入、deadline 超时(杀+真停+reader join)、cancel 长运行、grandchild 树退出、inherit 冒烟。所有测试自带截止(≤10s 级),不依赖外部长睡眠。
7. 不改:809 Builder、既有命令路由、legacy run 链端口策略、语言 stdlib、815 面——T-03/T-05/T-06 迁移消费。

## 7. 门后待办(记录,不在本轮执行)

- 整链 probe 定稿(§8 T-01 尾项):814/815 接口发布后,按 §5 候选做一次真实构建/消费 probe,冻结 bootstrap 布局与 profile 闭包清单,回写本文件附录。
- T-07 fixture `tool-execution-consumer/`(目录已留名,§6 闭包钉住)与 CLI 真实进程 driver。

## 8. 门后整链 probe 定稿（M814-B/M815-B 发布后冻结；T-05 依据）

冻结时点：M814-B（delivery `93a25c924`）、M815-A+M815-B（815 归档，生产位
`0f2858e35`）均已发布；816 树同步 master `f6b99189a`（merge `3abde3f1f`）。
Probe 结论（CLI 消费面逐符号盘点 + 工具能力矩阵核对）：

### 8.1 CLI 依赖拓扑（目标态）

| 层 | 依赖 | 说明 |
|---|---|---|
| 基座（Core 形态=基座） | `auto-lang-core`（语言 SDK）+ `auto-man-runtime`（管理服务）+ `auto-tool-client`（I-TOOL 客户端）+ `shell-pack` + `auto-val` + clap/miette/serde 等 | 旧 `auto-lang` 兼容大包与 `auto-man` compat 门面**退出 CLI 闭包**；语言 wrapper 局部入口家族（`run_file_with_args`/`dump_lowered`/`debug_file(_agent)`/`trans_*` 文件入口族）随迁 `auto-lang-core` 单源 |
| `ui-extra` feature（default on） | `auto-ui-compiler` + `auto-ui-runtime` + `auto-ui-dialect` | **五操作协议面之外**的 UI 静态面：`auto ui`/`bp`（ui_gen 注册表/蓝图）、`rqhost`/`autodesk -q`（desktop_protocol）、docs 的 UI 文档生成、`new -t jet` 脚手架（`ui_gen::jet::JetProjectConfig`——实测在 compiler 域非 tools 域）、裸脚本动态 UI 宿主（`has_ui_keywords`→`run_file_dynamic_ui`）、UI 文法/宿主能力装配（镜像 wrapper `install_ui_host_capabilities`：文法表+provider 装配+SDK 钩子） |
| UI 工程 build/run/gen | 经 `auto-tool-client` → 真实 `auto-ui-tool` 进程（I-TOOL v1 五操作） | **任何形态都不静态链接 `auto-ui-tools`**（SD-05-A）；runtime 服务编排的 UI 臂经 `ui_ops` seam 注入 **ToolUiOps**（适配器=纯委托：UiOps 方法→wire 请求→工具进程，算法零复制）；Core 形态下工具缺失=具名失败（T-04 discovery 负控），无静态 fallback |
| `python` | `auto-lang-core/python` | 同旧 |
| `autodown` | `ui-extra` + `auto-ui-runtime/autodown` | VM 真渲染依赖 UI runtime（旧 feature 转发即 `["ui-iced", "auto-ui-runtime/autodown"]`）；Core 形态不可开（诚实降级 textarea=旧无 autodown 行为） |

### 8.2 Profile 语义（显式区分，§5.3）

- **完整 profile（default features）**：全部 33 命令支持面保持；UI 工程命令经
  工具进程，桌面协议/注册表/裸脚本 UI 宿主经 `ui-extra` 静态面（compiler/
  runtime 域，非 auto-ui-tools）。交付=CLI 二进制 + `auto-ui-tool` + 资源/锁
  （792 DELIVERY 闭包，815 交付形态）。
- **Core profile（`--no-default-features --features core`）**：闭包
  normal/build/dev 无 UI 实现（ui-compiler/ui-runtime/ui-tools/ui-dialect）、
  无旧 auto-lang 兼容大包、无 auto-man compat。裸脚本/透传/REPL/语言 trans/
  docs/stdlib/管理命令全可用；UI 命令（build/run/gen UI 后端、ui/bp/rqhost/
  autodesk UI 面）具名失败。V-01 的闭包 oracle 以此形态钉住。

### 8.3 实测依据（probe 证据要点）

- 工具能力矩阵（815 handoff）：generate_ui=vue/rust/jet/ark/tauri/vscode；
  build_ui=vue/rust/jet/ark/vscode（vm 具名拒绝）；run_ui=vue/vm/rust/jet/
  ark/tauri/vscode，run 交互/取消真实链——UiOps 全部 generate/build/run 臂
  可映射五操作；`request.output_root` 字段即 delivery root 通道。
- runtime `ui_ops` 生产调用面（automan.rs 全量盘点）：vue/rust/vm/jet/ark/
  vscode/tauri × generate/build/run + `create_jet_project`；
  `pkg_run_script`/`pkg_display_name` 无 runtime 生产调用方（trait 预留面，
  ToolUiOps 具名拒绝实现，不假成功）。
- CLI 活面消费（排除 orphan cmd_vue/cmd_tauri）：`ui_build_shadcn*` 仅
  orphan 消费（不需迁移）；`aura/route/ui_gen/ui::*` 归 compiler/runtime 域
  可 `ui-extra` 门控；语言面符号全部在 core（入口家族迁移后）。
- wrapper `install_ui_host_capabilities` 三件套（文法表/钩子/inventory）在
  Core 形态由 core 自身 no-op 钩子承担（814 R3-01 语义），CLI `ui-extra`
  形态镜像装配。
