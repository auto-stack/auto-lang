# 给 GLM 的下一轮指示：828 第一批五项一批修齐

在 `D:/autostack/.wt/lang-828/auto-lang`、`plan-828-dev` 当前 `d93a9e37386fc6f8cb079b31c546df6ce37fdb5e` 继续 work。主 Plan r3、executing 0/11 不变。先读本包 review.md、artifact-controls.rs、roundtrip-controls.rs 和 raw；不要把历史第一批 `6f482efaf` 当当前 HEAD，也不要 reset 或撤销 T-05/06/T-07 已有提交。下一责任角色 work；本轮不继续添加新功能任务。

本批 needs_fix 的五项是原有合同，不是新语言/性能要求：

1. **B1-C01 P1**：恢复的 HIR 真正按当前上下文完整准入，再进入 resident/history。canonical 只用于内容身份，不能代替 semantic verify；不可信依赖字段不能不经核对。修复须涵盖 invariant，不限 owner/index 特判。
2. **B1-C02 P2**：核实际解码 body 的来源，不能比较两个自报/冷摘要就采纳；C01 的 verifier 单独不能修它。
3. **B1-C03 P2**：补真实 body-fact query → 依赖记录 → 当前消费失效机制与探针，满足 C01-03 的已读变化失效、未读变化不失效及只读接口 caller 保留；不新增 CTE/inline 语法，不用自报 counter 替代查询。
4. **B1-C04 P2**：单函数 HIR 与冷模块局部表不是同一张表。先确认 producer/持久化/恢复的 DefId/TypeId/owner 映射，修正稳定引用和实际 intrinsic 事实；mixed mark_a/mark_b 的 main 等宽编辑必须存读采纳成功。该修复应与 C01 当前上下文校验一起设计，避免“直接把坏表交 verify”误拒合法非零函数。保留原共享 store 规则；若 payload/key/语义变更需版本化隔离，不覆盖旧行或吞 Conflict 自证成功。
5. **B1-C05 P2**：close 显式释放 T-03 新 cold_module 的整套 owned body；新增观察覆盖这个 owner，保留外部 pin/候选/PausedRun 合法寿命。旧 Weak 控制只看原有 Arc，不能证明新深副本释放。

先补一张紧凑的数据流/所有权表：候选 body 使用哪张 def/type 表 → 工件如何保存身份/事实 → 恢复如何建立当前上下文 → 校验后谁拥有 body → hit 如何核适用性 → close 谁释放。将与本轮五项有关的当前规则/真实 API/永久断言回写现有合同和决策，不重写整份 Plan，不改 AC/V/C 含义或借 T-07 延后 T-03 缺项。

开发期间只跑 finding 控制、必要 scoped check/测试。将本包 12 个控制（含合法 owner=0/1、普通及混合 intrinsic、context/literal/entry 反控）收编或以同输入同 oracle 重放；补 body-fact 必需探针与新增 owner 释放控制。对恢复准入新增状态至少核二次恢复、后续 edit/hit 与 code/HIR 一致、source/trace 不漂移。保留既闭 C01..C05 和 F-05b 的旧控制，不重做未变的全部兼容面。

送审前五项一起闭合；每项列“根因/受影响路径/机制变化/原反例和合法正控/永久断言/剩余边界”。核 HEAD clean、源码与依赖/config 身份、测试实际选择数和 exit_code，给出唯一新冻结 SHA。raw 失败不可丢弃。整体仍 executing 0/11；后批和最终验收不能借本轮修复声明已过。最终审查前不跑整档 t/tv/tt/tb/tu/taa，merge-owned 门禁保持 pending。
