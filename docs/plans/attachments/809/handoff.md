# PLAN-809 work→review 交接（handoff）

- worktree: `D:/autostack/.wt/lang-809/auto-lang`，branch `codex/plan-809-dev`
- base: master `2046a025b`；代码提交链：dd4991a7b(T-02)→bc11055ac(T-03)→5c9eb4fdb(T-04)→322aa88d1(T-05)→8ee334f90(T-06a/b)→a4b4a0168(T-06c)→(T-06d/777)
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

## 显式 pending / 范围记录（review 须核对）

1. **V-05 真实工具执行**：ninja/cargo/npm spawn 的完整受控 fixture driver 未落（时间预算）；内存模式证据+探针已入。环境不足为 pending（AC-04 允许），merge 前建议补 cargo 族真跑（cargo 必备）。
2. **run 路径再生成**：run_vue/run_jet 等入口的内部再生成仍走 legacy 自扫（gen 路径已全切请求）；R04 迁生成器时统一退役。
3. **跨文件读取留 adapter**：dep 自身 pac.at 的 npm_deps 合并（vue）、sidecar spec 装载（load_sidecar）、api_gen 的 auto_lang resolve_back_dir——单工程合同外，记录归 R04。
4. **777 基线再生成**：module-boundaries.toml 已加 auto-man-core 条目；`--mode strict` 基线 json 重跑归 T-07 收尾/merge 前（与 792 source 清单并集）。
5. plan734_commands_fixture 测试目标在 master 同坏（预存，非本计划）。

## 复验命令（work 树）

- `cargo check -p auto-man-core -p auto-project-model -p auto-man`
- `cargo test -p auto-man-core`（44）/`-p auto-project-model`（38）
- `cargo test -p auto-man --test plan809_management_core`（6）
- `cargo test -p auto-man --lib -- plan809_v02 generation_request_adapter pac::`
- consumer：`cargo test --manifest-path test/fixtures/modularity/{man-core,project-request}-consumer/Cargo.toml` + verify-deps.sh
- 环境注记：aliyun 镜像缺 redox_syscall 0.9.5 → `--config source.aliyun.registry="sparse+https://rsproxy.cn/index/"`；sccache 端口 → `SCCACHE_SERVER_PORT=57323`

下一责任角色：review（T-09 同树独立复审；merge 门显式 pending）。
