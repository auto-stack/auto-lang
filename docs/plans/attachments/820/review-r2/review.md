# PLAN-820 r6 — R2 needs_fix

2026-10-24；review_scope: final readiness + affected repair review。本聊天 Codex，独立于实施上下文，未启动代理。上轮未通过；本轮检查修复机制和代表性失败出口后发现未闭合同项，未继续完整 final 验收。

## 基线与结果

- 唯一树 D:/autostack/.wt/lang-820/auto-lang，plan-820-dev，受审 HEAD baf4d7d49492444cfd8063495a0bbd26af1b1d86；修复 base a33538088cd4ce309176d217361edc68a1d2ced8；原交付 base 37e51253689631ffa61a9af58041c5f6646a7569。前后 HEAD 一致、实现 clean、diff-check exit0。
- 入审主检出 Plan SHA256 F153C9B9E202E2B53F78CB5FF7AF54D487D2FDBDFB0BA6FF600A7842988D2F0E。其实际状态为 executing/current_step7，任务勾选与 work 收据不完全一致；不伪称入审是 execution_done。以实际修复和合同核验，不以状态文字替代。
- SD draft SHA256 50B2176B9E9AB83017A26235C0A680B25B771D315C2DC06056FC1B910DF727D8；SD-07 的全链/失败不安装仍不能记 pass。依赖 ancestry、Spec 各目标当前内容/hash 全核仍 pending；未把实施方依赖声明转记独立 pass。

## R1 finding 关闭映射

F820-R1-02 **CLOSED（本机 Windows 采样边界）**：独立命令 `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test av_data_820 av_data_820_worker_ -- --nocapture --test-threads=1`，2 passed/0 ignored。父侧2ms采样：idle905216、load2052096、增量1146880 B（1.1MiB），163/169样本；暂态反控 idle913408/peak68153344，捕获67239936 B（64.1MiB）/122样本。确认真实新 worker 与父侧 PID sampler；采样量化不是任意短峰值的数学保证。

F820-R1-03 **CLOSED（同一首例）**：独立命令 `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test av_data_820 av_data_820_first_case_native_independent_comparison -- --nocapture`，1 passed/0 ignored；核对同源码 record profile native-build/link/run 与手审42/data结果，不再用 scalar 替代。

F820-R1-01 **PARTIAL / remains open**：独立 incremental 族5/5通过，已消除原 edit_body 的 whole compile_record_source，新增单 body 生产工作窗、代码身份与双退化门。仍有下列原合同观测缺口：data_session.rs:1176 `zero_io()` 直接构造磁盘读写0，edit_body:371直接用其作为证据；没有来自实际 IO 的计量，也没有 decode/verify/load 实际事件计数。`parse_fn_items:1`亦由消费者填写，HIR Arc 是 lower_and_admit 从 bundle body 克隆另存的观测副本。当前测试证明这些副本稳定和代码复用，不等于所有声明生产对象/事件已被观测。关闭要求：把要求的事件/身份绑定真实 owner/生产路径，独立故意插入读盘/重验事件可使同一正式 gate 判红；未发生事件的源码检查可作辅助，不能把常量当 raw。并关闭下面新发现的失败出口回归。

## F820-R2-01 P1 — 失败编辑已替换当前会话（AC-09/T-09/V-13）

违反既有 §5 全链合同“失败/取消/迟到不安装”。输入为合法 record 程序，bump 的 `c.inner.x + 1` 改成 `c.inner.x / 0`；该输入通过前端/MIR/ABC，真实执行产生 DivByZero。该 runtime trap 本身符合语言语义；错误是本 API 把编辑作为 Err 退回，却已经发布其会话状态。

实测：[重放源码](failed-edit-replay.rs)、[原始输出](failed-edit-result.txt)。使用受审 probe rlib 编译外部临时重放程序（没有改 worktree 实现）。输出：

```text
before=Ok(I32(42))
edit=Err("run: Trap { kind: DivByZero, ... }")
after=Err("run: Trap { kind: DivByZero, ... }") identities_preserved=false
retained_old_snapshot=Ok(I32(42))
retry=Ok(... generation: 3 ...)
after_retry=Ok(I32(43))
```

已建立原因：edit_body:343..361 先改 outline/src/hir/code/def_bundle/counters，并 relink 发布 snapshot，再 run；错误返回早于 candidate.commit，造成会话/环境事务分裂。保留在调用者手里的旧 snapshot 可执行不等于当前 session 未安装。合法对照：原输入42，后续合法+2重试43；失败时身份已变化。

兄弟路径：apply_source_impl 同样先安装全部状态后在843行 run；apply_caps 在850..863先变 caps/code 后 link/run。这两条源检查显示同类风险，尚未独立逐条触发，不冒称已实测失败。

关闭 oracle：候选的准入、link、运行验证与结构门全部成功才发布；真实失败（link、runtime trap/resource/cancel、结构门/commit失败）后当前 source/env/HIR/code/snapshot/代际/授权仍为失败前值，后续合法编辑可继续；覆盖至少 fast edit 与受影响兄弟入口。guest 已发生副作用不要求回滚，区分 guest 语义与宿主候选安装事务。

## F820-R2-02 P2 — 忽略/漏例可被脚本报绿（AC-06/T-06/V-10/12）

既有 T-06：0例/漏必需用例/skip均非绿。verify-av-data-820.ps1:32..46 只看累计 passed>0 与 failed；不校验必需ID或 ignored。同族一个用例通过即可掩盖其余必需用例 skip 或过滤缺失。

实测 failure-propagation 控制（仅替换本次 shell 中 cargo，未修改脚本/实现，不作为产品运行证据）：

```powershell
function cargo {
  Write-Output 'test result: ok. 1 passed; 0 failed; 8 ignored; 0 measured; 0 filtered out'
  $global:LASTEXITCODE=0
}
& ./scripts/verify-av-data-820.ps1 -Scope first-case
```

实际 exit0、`first-case PASS`。预期非零。关闭：必需ID集合/各选择实际执行、无 ignored/漏项/重复冒计的正反控，不能只校验合计非零；保留合法多target零选择容许边界。

## Health 与待审面

定向 native/worker 两次均出现未处理编译 warning：probe/tests/av_data_820.rs:504 `fn Local0()` non_snake_case。属于本计划新增测试，T-06 健康门待修（更名并定向 check 即可，无须整档）。diff-check通过不代表 fmt/编译零告警完成。

AC01 的 native 缺口关闭但其他完整验收仍 partial；AC05 的采样缺口关闭，其他资源矩阵保留 pending；AC09 fail；AC06 验收聚合 fail；AC02/03/04 未完成独立全核；AC07未过、AC08 pending。SD01..06需后续逐条对现状Spec及行为核验，SD07当前不通过。未声称其他未审面无缺陷。

状态回 executing 5/9，T-06/T-09重开，保留T-01..05工作完成主张（未代表其全部独立AC pass）。两次非pass后，下轮work应先提供具名根因、覆盖/设计调整和关闭表，尤其所有安装失败出口与真实生产计量；再提交修复与定向证据送审。未跑整档/acceptance聚合，未改实现/canonical/ledger、未合入。merge-owned 裸 cargo t+tv+独立workspace完整有限合同档仍 pending。
