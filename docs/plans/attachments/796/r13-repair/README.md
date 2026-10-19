# PLAN-796 R13 修复轮证据（work，2026-10-19）

修复 commit：worktree `D:/autostack/.wt/lang-796/auto-lang` 分支 `plan-796-dev`
`1bab7e15e`（R13 复审基线 `d67901b5b` → 本修复提交）。冻结决策 D23 与
spec-delta R13 增量在提交内。

## F-796-R13-01（P2）模块索引全量新鲜性

- `module_index_bytes`/`module_decl_versions`（core/query.rs，共享 helper）：
  索引页全部实际发布字段（fn 候选+record 候选+按 logical 排序的 decl 签名
  版本）的规范字节；查询键 `index-content-fp`（替代旧域名集
  candidate-set-fp）、页头 content、读侧 target_content 三者同源。
- `admit_page` 增 ModuleIndex 字段准入：package/module、fn/record 候选集、
  decl_versions 逐字段对当前 input 比对——当前头+陈旧体的页被拒。
- 永久例 `sema_cache_r13_module_index_tracks_signatures_and_records`：
  签名编辑 → 旧索引 Miss + 再发布成功（无 783 Conflict）+ 新版本被服务；
  record 新增 → 同；body-only → 幂等复用同 artifact；fresh store 字段一致
  （重启稳定）；当前键+当前头+陈旧体注入 → 读侧 RefVersionMismatch。

## F-796-R13-02（P2）FnSig 签名级新鲜性 + FnMeta 完整准入

- `target_content` 拆分：FnSig=签名 canonical（body 不再混入——body-only
  编辑同键同内容不误失效，§5.4/V-03）；InferBody 保持签名+body。
- `admit_page` 增 FnMeta 分支：嵌入 fn decl 全量名义准入（verify、当前包/
  模块、顶层 Fn 且无 owner、当前声明存在、content=当前签名 canonical）。
- 永久例 `sema_cache_r13_fnsig_page_admission_and_body_independence`：
  当前 decl HIT 正控；真实旧签名版本/外包 decl 拒绝；body-only 输入编辑
  保持 HIT（误失效修复）；签名编辑 Miss（新键）。

## 页种类覆盖映射（review 路由要求）

RecordMeta=shape+methods（admit_shape_refs+admit_method_ref）；RecGroup=成员
shapes+组有效版本（admit_shape_refs+env）；FnMeta=decl（本轮全量准入+签名级
头）；ModuleIndex=全字段（本轮键/头/准入同源）；InferBody/CachedResult=页头
target_content（签名+body）+manifest 新鲜分类。详见 D23。

## 统一验收（acceptance.log.txt）

13 行全 PASS、exit 0：V-01 16、V-02a 15、V-02b 4、V-03 7、V-03b 37（+2）、
V-06 12 例 140.59s、V-04 5、V-05 1、V-09 6、V-08 16、V-08c 2、V-07 重算
lib P95 8.11ms + IPC 完整窗口 P95 251.00ms（30 raw）。selftest 12 控制
PASS。共 120 个不同定向 Rust 测试（较 R13 的 118 +2）。

注：本轮期间 D: 盘曾 100% 满（ENOSPC 清空过一次 query.rs，git 恢复后重放
编辑，最终提交为完整态）；本 worktree 27G 可再生 target 已清理。

未跑整档 t/tv/tt/tb/tu/taa/tf（merge-owned，2026-10-14 裁定）。
