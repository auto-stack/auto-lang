# PLAN-824 共享台账恢复独立复核

- 结论：PASS；复核者 `/root/review824_sync`；用户明确授权按草案恢复并要求独立复核。
- 原版/备份 SHA256 `a5c23f40350b87587602d1fecb432cd6f2da4b08b94e78f7acc90cf1a39c0516`；候选 SHA256 `509363374b432fe4d880cefd258a73a45357cd0944581c73d205616c0b2be1ea`；草案 SHA256 `873ced993dddb530b704a1749dd9cb55d4e5e07999eff87e437d16dd31e9b19c`。
- 独立核对54条记录、210字段逐项符合用户批准草案，日期映射 UTC 秒，5条 active→stable、缺失时间0及缺字段均如实记录；所有ID、顺序、未改字段原始 JSON token、其他记录及版本之外字节保持。备份与原版完全一致可恢复。
- 独立运行当前真实 `SpecsStore::load`：原版缺 `created_at` 被拒；候选成功加载 version103、6 sections、1284 items，store模型源 SHA256 `c124c8449cbf16fe2a856a35910f13d22651a09645ecbac2413dbef47cd914fd`；测试替身只禁止写，不替代加载逻辑。完整日志见 [loader](ledger-real-store-check.log.txt)。Python schema零错误。
- 复核者环境不能读取工作树 .git；实施方补核 `git status --short` 仅 `.autoos/specs.json`，`git diff --check`通过，未发现额外暂存或未暂存修改。共享主检出台账在本复核时仍保持原版。
