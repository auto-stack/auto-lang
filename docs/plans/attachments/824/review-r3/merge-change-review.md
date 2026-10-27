# PLAN-824 主干同步与规范增量独立复核

- stage: review；review_scope: merge 前补充复核，仅主干同步、依赖漂移、编号与规范落稿；reviewer_identity: `/root/review824_sync`，用户明确授权的独立只读子代理，与实施会话分离。未声称重新完成全部代码最终复审；原独立 R3 通过结论保留，具体代码证据仍见 [review.md](review.md)。
- 初审 `5e762131552f98eea8004e02a5776a4bd5961d97`：needs_fix，仅规范落稿（3 个 824 plans 行脱表；正式 MIR 容器与宽标量入口边界、取消接口范围、错误分态不完整）。未发现因同步带来的新代码差异。
- 修复提交 `fdc6f3a0e472f4e4b9d7aba290cbfc3c19464d90`：独立只读再审 pass。三个计划行归主表；两项规范区分 List/Variant/record 组合、宽标量 entry-param、binding 没有取消回调、portable 各错误分态及错型 take trap。未来 archive 链接须归档后再核。两份提交仅变更 `docs/specs/`。
- 实施基线 `29b6eeea98d1b75e2ea18ea5f62eea96866e9d60` → 同步后 `e2d5c9e081c343f9237f97867e5a85f1e3db82ad`；14/14 提交 [range-diff](merge-range-diff.txt) 均 `=`，未改代码、未改号。主干基线 `5b65aa68c65bd33142e1198b73dfbd6e822fd5a6` 的 profile 2/3/4、AVAD v1/v2/v3、opcode 至 0x2E；824 的 profile 5/v4/0x2F..31 无已合入冲突。823 在途 `ea88414456bf89c98307c8fa2fe99a06dcaa1a5d` 同号，后合入方处理改号并验证 Closure × Dyn 交叉拒绝及合法正控。
- 主干共享依赖变化：根 ecow 启用 serde，`crates/auto-val/Cargo.toml` serde 特性链、`src/value.rs` Op 条件序列化、根及独立 workspace 锁文件变化；probe 的 ecow 经 auto-val→syntax/atom，core 本身无该依赖。原依赖绑定不能直接复用。实施会话针对新依赖运行了 21 项 F03/原错例/恢复/增量定向测试及原 R2 重放，均通过；独立子代理本次只读、未运行测试。所有冻结回归、兼容时序及 finite workspace 合同仍须在 `fdc6f3a0e472f4e4b9d7aba290cbfc3c19464d90` 实测。
- outcome: pass（上述受限范围），next: merge；不代表已合入或整档通过。
