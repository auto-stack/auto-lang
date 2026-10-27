# PLAN-824 规范投影独立复审

- 初审对 v104 SHA `60c290f4e75b7f658c161db130f80ad12c647cd6a3a73302c6ede73d6e679b14` 返回 needs_fix：两项证据来源提交未包含文件，投影说明链接未固定。其余5项新增/5项更新及范围、内容、顺序、非目标字节和真实store加载均通过。
- 修复复审：PASS；reviewer `/root/review824_sync`；最终 v104 SHA `e5d4cfe3d8743f393a7433faa2872aabb190d18238ef61176376881442a10bf3`，证据提交 `677b3e9999641c688d118cc3a9e66059f83157ec`。P824-R/P824-REL 两个 source-commit 唯一改为该证据提交，恢复原值后逐字节等于初审候选；两个文件的git blob hash与source-sha256一致。投影说明八个证据链接均可从该证据提交恢复。
- 再次运行真实只读 SpecsStore::load：version104、6 sections、1289项，Python schema 0错误；v102/v103备份与主检出保持原哈希。无新增 finding；合入、归档、清理及批量门仍由merge完成。
