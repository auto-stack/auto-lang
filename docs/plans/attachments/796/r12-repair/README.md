# PLAN-796 R12 修复轮证据（work，2026-10-19）

修复 commit：worktree `D:/autostack/.wt/lang-796/auto-lang` 分支 `plan-796-dev`
`d67901b5b`（R12 复审基线 `bb10c9923` → 本修复提交）。冻结决策 D22 与
spec-delta R12 增量在提交内。

## F-796-R12-01（P2）方法 ref 完整名义准入

- `admit_method_ref`（store.rs，置于 `admit_ref` 旁）：对 RecordMeta 内每个
  方法 decl 施加与 `admit_ref` 同等义务——package/module 匹配当前 input、
  方法在当前 owner 下仍声明（`<method no longer declared>` 拒绝）、content
  == 当前签名 canonical（旧版本拒绝）；与 R11 结构门（kind+owner）串联。
- 同入口复核（review 要求）：shape.decl 与字段 record ref 走 `admit_ref`
  （全量）；RecGroup 成员同；FnMeta/Infer/CachedResult 页按页头
  target_content 全量版本等价——仅方法 ref 原有缺口。
- 永久反控 `sema_cache_r12_method_ref_full_admission_matrix`（Required 绑定
  V-03b）：当前键/当前 shape 版本/当前 group 有效版本的注入页，仅变方法
  ref 单轴——全当前 HIT 正控；外包（"foreign"）/外模块（"elsewhere"）
  TargetMismatch；除名（"removed"）RefVersionMismatch；**真实旧
  `get(x int)` 签名内容版本** RefVersionMismatch。admission-matrix-replay.log.txt。

## F-796-R12-02（P3）7 项测试警告清零

- first_case：未用 `SemaInput` 导入、兜底后不可达 HitStaleProvenance 分支；
  more：`publish_main` 无效 store/project 参数（调用点同步）、未用
  `ManagedStore` 导入、三处不可达分支。断言无弱化、行为不变。
- warnings-clean.log.txt：全 target check 后计划命名空间 0 警告；唯一剩余
  为 review 明确排除的 shim-metadata/classify.rs:213 既有项（归属证据在
  log 外壳的完整 warning 块）。

## F-796-R12-03（P3）覆盖矩阵同步 v2

- metadata.rs 十处页载体 schema `ac796/sema-page/v1*` → v2（六 Implemented
  + 四 SchemaOnly）；semaref/CT 独立 schema 与历史 V1 常量未动。
- `matrix_invariants` 新增不变量：凡以 `ac796/sema-page/` 为载体的类别必须
  登记**当前**页 schema（`SEMA_PAGE_SCHEMA_V2`）——矩阵漂移在库级不变量
  与 V-01 双层拦截。

## 统一验收（acceptance.log.txt）

13 行全 PASS、exit 0：V-01 16、V-02a 15、V-02b 4、V-03 7、V-03b 35（+1）、
V-06 12 例 136.93s、V-04 5、V-05 1、V-09 6、V-08 16、V-08c 2、V-07 重算
lib P95 9.68ms + IPC 完整窗口 P95 250.00ms（30 raw）。selftest.log.txt
12 控制 PASS。共 118 个不同定向 Rust 测试（较 R12 的 117 +1）。

未跑整档 t/tv/tt/tb/tu/taa/tf（merge-owned，2026-10-14 裁定）。
