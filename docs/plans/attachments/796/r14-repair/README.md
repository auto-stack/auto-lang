# PLAN-796 R14 修复轮证据（work，2026-10-19）

修复 commit：worktree `D:/autostack/.wt/lang-796/auto-lang` 分支 `plan-796-dev`
`52aad1fd2`（R14 复审基线 `1bab7e15e` → 本修复提交；R15 核查确认基线未动，
本修复为 R14 finding 的首次实现）。冻结决策 D24 与 spec-delta R14 增量在提交内。

## F-796-R14-01（P2）FnSig 键版本隔离（R13 修复回归）

- `core/query.rs` FnSig 构键增加 `hdr-sem=fn-sig/sig-only/v1` 轴——R13 把
  FnSig 页头语义改为签名级，语义变更随该轴移动键；保留的前轮合法页（签名+
  body 头）落在旧键上不可达：当前查询不服务旧值；当前语义页在新键发布，
  783 无同键 Conflict。不改 Decl 逻辑身份、不改 783 冲突策略、不清空持久
  store。
- 一般规则（D24 重申）：凡改某查询页头/载荷语义而不动整体 schema/codec
  版本，必须为该查询键补显式语义版本轴（ModuleIndex 已由 R13 的
  index-content-fp 换轴隔离；FnSig 本轮补齐）。
- 永久例 `sema_cache_r14_fnsig_key_version_isolates_old_semantics`（Required
  绑定 V-03b）：保留旧语义页（旧头=签名+body 真实哈希、旧键=当前键去
  hdr-sem 轴，复刻 review 原例构造）→ 当前查询 Miss → 当前页发布无
  Conflict → store 重开仍 HIT → body-only 保持 HIT、签名编辑换键 →
  fresh store 正控。r14-test-replay.log.txt。

## 统一验收（acceptance.log.txt）

13 行全 PASS、exit 0：V-01 16、V-02a 15、V-02b 4、V-03 7、V-03b 38（+1）、
V-06 12 例 310.67s、V-04 5、V-05 1、V-09 6、V-08 16、V-08c 2、V-07 重算
lib P95 10.09ms + IPC 完整窗口 P95 432.00ms（30 raw）。selftest 12 控制
PASS。共 121 个不同定向 Rust 测试（较 R14 的 120 +1）。

未跑整档 t/tv/tt/tb/tu/taa/tf（merge-owned，2026-10-14 裁定）。
