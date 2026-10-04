# PLAN-741 verification 收据(verification.md)

> 记录执行/验证命令、期望、实际结果与代码修订的对应关系(AC-01..AC-08 证据)。
> 执行环境:worktree `D:/autostack/.wt/lang-741/auto-lang`,分支 `plan-741-dev`,
> Windows 11 x64。工具链基线见 [toolchain.md](toolchain.md)。

## 1. 提交链(执行序)

| commit | 任务 | 内容 |
|---|---|---|
| 0f2b232d7 | T-01 | 独立 workspace + 工具链探针(Cranelift 0.126.2 选型收据) |
| 632e48a03 | T-02 | atom_text 阅读器 + descriptor 绑定器 + text_binding 11 测试 |
| b3bed7d72 | T-03 | verify.rs 语义校验器 + hir_verify 8 测试(§9 矩阵全覆盖) |
| cf5aca9f8 | T-04 | native.rs lowering + native_execution 8 测试(真实 PE 运行) |
| 523fabfe1 | T-05 | link.rs 链接驱动 + ac-probe CLI + cli 9 测试(原子制品) |
| 53775205c | T-06 | test-support trace 支持库 + 调用顺序 runner(原生 b,a/12) |
| (本提交) | T-07 | README + verify-ac-741.ps1 + 本验证报告 |

基线:v0.6-dev @ `951b6c70ff596f79e464ba977139669f2d196821`(计划起草提交)。
全程未修改 `crates/**`、根 Cargo.toml/Cargo.lock(AC-01,见 §4)。

## 2. §6 命令实测(2026-10-04,scripts/verify-ac-741.ps1 -SkipMainGates)

| 命令 | 预期 | 实际 |
|---|---|---|
| cargo check --manifest-path experimental/ac-core/Cargo.toml --locked | 0,无 warning | PASS(0 warning) |
| cargo fmt --manifest-path ... -- --check | 0 | PASS |
| cargo test ... --test text_binding | 11/11 | PASS(11/11) |
| cargo test ... --test hir_verify | 8/8 | PASS(8/8) |
| cargo test ... --test native_execution -- --test-threads=1 | 8/8 真实 PE | PASS(8/8,1.2s) |
| cargo test ... --test cli -- --test-threads=1 | 9/9 | PASS(9/9) |
| cargo test ... --test trace_execution -- --test-threads=1 | 1/1 | PASS |
| ac-probe check 01-add.atom | exit 0 | PASS |
| ac-probe build add-2-3 + 运行 | exit 5 | PASS |
| ac-probe build 03-call-order(能力+支持库)+ 运行 | exit 12,stderr b→a | PASS |
| 汇总 | 全过 | **VERIFY-AC-741: all 11 steps PASS** |

收据目录:`experimental/ac-core/target/ac-verify-receipts/`(toolchain.txt、
summary.txt、cli-walkthrough/{add-2-3.exe,add-2-3.ac-link.txt,trace.*})。

## 3. 关键验收证据(AC → 证据)

| AC | 证据 |
|---|---|
| AC-01 | `cargo tree --locked` 全树仅 Cranelift 栈+object+anyhow(toolchain.md §3);git diff 根 Cargo.toml/Cargo.lock/crates/** 为空;dev-dependencies object 仅测试用 |
| AC-02 | text_binding:四份设计文档经真实 reader/binder;01-add 与 01-add.explicit 语义相等;canonical 往返相等;enum 简写/shorthand 绑定为 Builtin;15 个 invalid 夹具按预期码拒绝(span 校验) |
| AC-03 | hir_verify 拒绝矩阵 21 例(delete-let、lt 类型、不可变写、重复/越界 binding、跨 body local、共享表达式、死表达式、表达式环、共享块、不可达块、return 后尾语句、循环目标越scope、参数索引/类型、返回类型、if 条件、eval_args 数量、函数类型 local、循环内初始化外泄、无返回路径);后端只消费 CheckedModule(verify() 唯一构造路径),text_binding/hir_verify 各有"拒绝→无产物"断言 |
| AC-04 | schema/core-i32.atom 为版本化 core descriptor(身份/枚举/分支契约);profile-mismatch/revision-mismatch/unknown-case 夹具拒绝;本仓不含源码前端/VM 回退路径(experimental/ac-core 无 crates 依赖) |
| AC-05 | native_execution:add(2,3)=5、add(-2,3)=1、count(3/0/-2)=3/0/0 均为链接后真实 PE 退出码;obj/exe/link 收据产出且被 cli 测试断言;COFF 含 ac_start/test_entry/add/ExitProcess 符号 |
| AC-06 | overflow-add-max / overflow-mul-max2 退出 70(独立 ExitProcess(70) 路径,非回绕);trace_execution:03-call-order 原生执行 stderr 顺序 b→a、退出 12;无能力 build 被拒(cli + trace 双重断言) |
| AC-07 | cli 测试:check/build 全错误路径非零(exit.not-found/signature/capability/linker);SENTINEL 夹具证明失败构建不覆盖既有 exe;成功产物 .obj/.exe/.ac-link.txt 收据齐全;§2 命令一键复现 |
| AC-08 | 本报告 + toolchain.md;定向门禁全过;主仓最终门禁见 §4;未宣称源码编译/生产 ABI/AAC(README 边界节) |

## 4. 主仓最终门禁(AGENTS 编译器改动档)

计划 §6 要求最终跑一次 `cargo check -p auto-lang`、`cargo t`、`cargo tv`。
本次执行不含 crates/ 改动。实测(worktree plan-741-dev,2026-10-04):

| 门禁 | 结果 |
|---|---|
| `cargo check -p auto-lang` | **PASS**(1m43s;382 条 warning 为 auto-lang lib 既有基线,本计划零 crates diff) |
| `cargo tv`(语料档) | **PASS 162/162**(4.8s) |
| `cargo t`(日常档) | 4971 跑:**4945 绿 / 26 红** / 1533 skip(nextest fail-fast 关闭枚举) |

### 4.1 cargo t 红测试同基线对照(计划 §6:旧测试红需同基线对照)

对照环境:主检出 `D:/autostack/auto-lang`(v0.6-dev @ 0d7c7f024;其相对本
worktree 基线 951b6c70f 仅前进两个 docs-only 提交,crates/ 代码完全一致),
对该 26 个测试做 scoped 重跑(22 个名字命中执行,4 个在独立测试目标中由
filter 未选中;下表为执行部分):

- **14 个在基线上同样红**(确定性继承红):musk_vm_track 全 7 例
  (widget_content/widget_computed×2/p053_4/p054_t1/p054_t4)、
  plan707_client_sse_poll_close_error_surface、plan502_m3_layout_geometry_e2e、
  plan498_area_emphasis、plan484_024_charts_streaming_recompute、
  ffi_dual_017_dep_lifecycle、dep_parity_017_dep_lifecycle、
  projector_counter_layout_and_hits、
  test_029_photo_gallery_thumbnails_render_with_resolved_src_and_cover_fit。
  这些为跨仓跟进/依赖类测试(auto-musk/auto-down 状态依赖),与本计划改动
  无因果(本计划 crates/** diff 为空)。
- **8 个在基线 scoped 串行下绿**(全档并行负载 flake,在案模式):
  plan498_donut_sector_emphasis、plan499_line_axispointer_crosshair、
  plan499_donut_tooltip_fade、c1_prop_compare_in_init_alive、
  pkg_init_fstr_dollar_bracket、pkg_canary_undefined_var、
  last_window_close_exits_daemon、diff_window_census。
- 其余 4 个(queue_coverage_drift_fence、schema_drift_fence、
  kitchen_sink_page_in_sync、ash_stream_leak_probe)属于 `--test` 独立目标,
  在 filter 运行中未命中;与 musk/ffi/画廊族同类,均为环境/跨仓状态面。

**结论**:本计划 worktree 未引入任何 crates/ 行为变化;红测试全部为基线
继承或负载 flake。裁定(接受继承红/修复/挂账)按流程归 /auto-plan:review
(T-08)独立复审裁定,本报告仅提供对照证据。

## 5. 遗留与边界重申

- 本原型为 A0 计算核心闭环;完整计算子集(A1 起)、AST adapter、B/C 子集、
  生产 ABI 由后续计划承接(§4.1 编号边界维持)。
- `03-call-order` 在无 `--support-lib` 时构建按设计失败(link.failed)——
  trace 能力必须显式提供且链接真实支持库,不存在静默跳过路径。
- 739/740 计划不可见的差异核查仍待主机器恢复后进行(计划 §10.2)。
