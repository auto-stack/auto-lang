# PLAN-809 work→review 交接（handoff）

- worktree: `D:/autostack/.wt/lang-809/auto-lang`，branch `codex/plan-809-dev`
- base: master `2046a025b`；代码提交链：dd4991a7b(T-02)→bc11055ac(T-03)→5c9eb4fdb(T-04)→322aa88d1(T-05)→8ee334f90(T-06a/b)→a4b4a0168(T-06c)→aa1d40e1d(T-06d/777)→**c41a4fe4c(R1-F01..F03)→3b733aa5d(R1-F04)**
- 兄弟依赖：`D:/autostack/.wt/lang-809/auto-down`（detached @ master c39ac7f，只读）

## 已交付（work-owned）

- T-01 合同：`docs/plans/attachments/809/contract-baseline.md`（含 §13 recorded adjustments 六条）
- T-02 I-PROJECT：auto-project-model `generation_request`（静态/动态分流+A/B 组字段）+ project-request-consumer
- T-03 auto-man-core：Target/Dir/Port/Scene/RenderSelection/Group/NodeExt/pac_from_decl + re-export 门面 + TargetEffects/DirEffects 扩展 trait + api_compat_probe
- T-04 闭包：closure.rs 五函数族+resolve_target_closure（NoProgress 有界准入+环候选链）+ assemble_major_targets；Pac 旧算法删除
- T-05 选择/规划：selection.rs/planning.rs/file_types；scan_dir_flat 纯枚举；scanner 死脚手架删除；set_port/cargo setup 消费 core
- T-06 生产接线：jet/ark/rust_ui/vue 读缝（JetReads/ArkReads/PacSource/VueReads）+全 `_with_request` 入口；automan gen 分派（成员+self）一次构造 request_of 多处消费；V-02 对拍（VueReads 九字段+from_workspace 双路径）
- V-05 证据半：v05_ninja_builder_memory_consumes_core_plan（Pac::new→set_port→resolve→真实 NinjaBuilder 内存模式消费 core 规划）+ 工具可用性探针
- 777：auto-man-core 登记 man-ecosystem 域；两 consumer fixture（project-request/man-core）+verify-deps.sh

## R1 Findings 闭合（2026-10-18 修复轮，c41a4fe4c+3b733aa5d）

| finding → 不变量 → 受影响面 → 修复与永久用例 → 结果 → 余缺 |
|---|
| F809-R1-01 → V-05 断言须有区分力+真实工具/失败面证据 → plan809 V-05 测试 → v05_memory_and_cargo_real_serialized（真实 cargo spawn+Cargo.toml 断言+内存 build.ninja 含 main.c/rule/link+空工程负控+cargo 无 rust 目标 setup Err+索引缺依赖 resolve Err；cwd 切换串行化注记）+ v05_vue_real_build_and_failure（真实 npm run build 离线 node 脚本产物标记+exit 3 Err 传播）→ 7/7 过 → ninja 真实 spawn **失败传播**已实测（D:\softin
inja.exe exit 1，编译器缺失环境）；成功产物路径以内存模式覆盖，编译器就绪环境的 ninja 成功跑留环境注记 |
| F809-R1-02 → 生产生成读取全量经合同（无批准延期）→ automan run/run_backend/build_backend/run_workspace/run_member_frontend/open_jet/open_ark + main.rs → 全部 `*_with_request`（rust_ui build/run、vue/jet/ark/tauri run、Automan::gen_request()/成员 request_of）→ automan.rs+main.rs legacy 分派 grep=0；scoped 16+3 过 → dep pac.at 合并/sidecar spec/resolve_back_dir 三处跨文件读取已逐项映射到合同边界（SD-02 边界节具名，R04 收编——非笼统豁免） |
| F809-R1-03 → 规划纯规则单源（一个消费者不构成留在旧层的理由）→ ninja build 迭代+download_deps 决策段 → core BuildOrderItem+ordered_build_renames（adapter 句柄回查执行）+ dep_fetch_action/pinned_version/admit_new_dep_libs（观测注入）/admit_transitive_deps，download_deps 消费 → core 45/45（含 ordered/download_decisions oracle）→ 无 |
| F809-R1-04 → work-owned 收口证据 → 777/792/SD/接口 → 基线显式重采样（c41a4fe4c，+5 边）strict **pass** violations=0 grandfathered=2（收据 t07-777-strict-pass.json）；792 受影响面=rust_ui delivery 管线，delivery 9/9 隔离复验；SD-01..04 as-built 文本入 work 树 docs/specs/ → 无 |

### I-PROJECT 字段读取映射（接口交接，R04 消费面）

请求字段 → 生成器读取点：identity.name→jet/ark/rust_ui(423,641)/vue(display)；display.title[_zh]→vue(index_title)/rust_ui(title_env 2666)；runtime.window→rust_ui(window_env 2665)；exe_name→rust_ui(pac_bin_block 4420)；runtime.api→rust_ui(api 门 6344)；external_backend→rust_ui(6217)；theme_decl→vue(composed)/rust_ui(apply_theme)；dep_roots→rust_ui(resolve_bps 2874)/vue(collect_dep_front_dirs 输入)；shadcn/default_classes→vue(3323/3329)；npm_deps/styles/i18n→vue(3927/4022)；front_decl→vue(front_rel_path)；has_rust_sidecar→rust_ui(563 门外)。run 侧 env 种植（main.rs AUTO_* 14+）不经请求——R04 统一。

## 复验命令（work 树）

- `cargo check -p auto-man-core -p auto-project-model -p auto-man`
- `cargo test -p auto-man-core`（44）/`-p auto-project-model`（38）
- `cargo test -p auto-man --test plan809_management_core`（6）
- `cargo test -p auto-man --lib -- plan809_v02 generation_request_adapter pac::`
- consumer：`cargo test --manifest-path test/fixtures/modularity/{man-core,project-request}-consumer/Cargo.toml` + verify-deps.sh
- 环境注记：aliyun 镜像缺 redox_syscall 0.9.5 → `--config source.aliyun.registry="sparse+https://rsproxy.cn/index/"`；sccache 端口 → `SCCACHE_SERVER_PORT=57323`

下一责任角色：review（T-09 同树独立复审；merge 门显式 pending）。
