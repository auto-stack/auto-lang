# PLAN-741 r5 proposed Spec delta（SD-09，未沉淀）

modify `docs/specs/auto-ac/project.md`。兑现SD-08既有全出口清理承诺，不改变profile、语言子集、Windows目标或后端。

`link_object_staged`在链接器非零退出及执行器错误（deadline/spawn/containment等）时，必须检查自有暂存exe回收结果。NotFound视为已回收；用户占位目录不删除。清理受阻须同时报告原阶段/错误、自有残留完整路径、实际OS错误、恢复办法，并明确NOT committed；不能仅返回link.failed/link.deadline而吞掉删除错误。

正常可清理失败仍零自有残留；旧成功exe/obj/receipt字节保持不变。文件锁释放后恢复可验证。原publish/restore/COMMITTED诊断保证保持，受控启动/无detach资源保证保持。仅r5独立review pass后的merge沉淀canonical/ledger。
