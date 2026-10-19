# PLAN-796 R11 修复轮证据（work，2026-10-19）

修复 commit：worktree `D:/autostack/.wt/lang-796/auto-lang` 分支 `plan-796-dev`
`bb10c9923`（R11 复审基线 `074464f51` → 本修复提交）。冻结决策 D21、
spec-delta R11 增量在提交内（boundary 出处测试随 D21 值通过）。

## F-796-R11-01 方法名义身份含所属 record

- contract：`DeclKind::Method=3`；`DeclRef.owner: Option<String>`（仅 Method
  携带——ownerless 方法/带 owner 的 Fn·Record 一律 verify 拒绝）；
  `method_logical_id` 把 owner 编入 kind 轴（`method::<owner>`），4 分量哈希
  形状不变、既有 Fn/Record logical 字节不变。
- probe：`ensure_record_page` 方法 decl 经 `DeclRef::new_method` 铸造；
  `admit_page` RecordMeta 逐方法校验 kind=Method 且 owner=本页 record，
  外属/无属页 TargetMismatch 拒绝。
- 版本轴（review 要求的同键异值防护）：页 schema 抬至
  `SEMA_PAGE_SCHEMA_V2="ac796/sema-page/v2"`（进全部查询键与页头校验），
  `SEMA_CODEC_VERSION=2`（方法 DeclRef tag3+owner；Fn/Record 字节与 v1 兼容）
  ——payload 变更随版本轴换键，旧 v1 页不可达。
- 永久反控（全部 Required 按名绑定入 verify 脚本 V-01/V-03b）：
  - `sema_identity_r11_method_owner_is_part_of_the_nominal_identity`：A.get/
    B.get/顶层 get 三向不碰撞；同 owner 异名方法不碰撞；重铸稳定；跨模块
    区分；篡改 owner/无 owner/Fn 带 owner 均 verify 拒绝。
  - `sema_identity_r11_method_declref_codec_roundtrip`：owner/kind/logical
    过页编解码保持。
  - `sema_cache_r11_method_owner_identity_distinguishes_a_get_b_get_and_top_get`
    ：review 原例（同模块 A/B/顶层 get）真实 TextRecord 解析→发布→读回；
    新 store 重启稳定；声明序互换同 id；同 owner 签名编辑 logical 不变而
    content 变；body-only 复用同 artifact；owner 改名 A→C 铸新身份、退役 A
    不再服务。
  - `sema_cache_r11_foreign_owner_method_page_rejected_on_read`：注入"B 属
    方法 decl 于 A 页"（shape 携当前版本越过前置校验直达归属门），读侧
    TargetMismatch 拒绝且错误点名外属 owner。

## 统一验收（acceptance.log.txt）

13 行全 PASS、exit 0：V-01 16 例（+2）、V-03b 34 例（+2）、V-06 12 例
334.00s、V-08 16 例、V-08c 2 例（硬门+窗口反控）、V-07 重算 lib P95
9.17ms + IPC 完整窗口 P95 391.00ms（30 raw）。selftest.log.txt 12 控制
PASS。共 117 个不同定向 Rust 测试（较 R11 的 113 +2 contract +2 probe）。

原例重放：original-example-replay.log.txt（两 R11 测试，含 review 原例）。

未跑整档 t/tv/tt/tb/tu/taa/tf（merge-owned，2026-10-14 裁定）。
