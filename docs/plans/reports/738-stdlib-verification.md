# PLAN-738 验证报告（T-08，2026-10-08）

worktree `D:/autostack/.wt/lang-738/auto-lang`，分支 `plan-738-dev` @ `a4afb3b65`
（T-07 slice3），基面 master `4262f761c` + re-sync `0424673e2`。

## 门禁实测（worktree 内）

| 门 | 结果 | 红分诊（对同命令 master 基线） |
|---|---|---|
| 裸 `cargo t` | 1515/1518（fail-fast 截断） | 3 红全在 musk p053 族；全族红名集 master=worktree 6=6 diff 空 → 预存（他 session WIP）零新增 |
| `cargo tv` | 162/162 | 无红 |
| `cargo tt` | 红名集 worktree 12 ⊂ master 13 | master 独有 plan707_wait 系在案 flake 未复现；零新增 |
| `cargo th --test-threads=1` | 红名集 worktree 1 ⊂ master 2 | back_proxy e2e 环境相关族在案；零新增 |
| plan738 族 | 32/32 | 含 8 个 T-05 缓存族 + 3 个 T-07 见证/矩阵 + 2 反例 + t04/t03/t02 全族 |
| ⑤腿 witness（`-- --ignored`） | 1/1 | rustc 实编实跑 29.8s（cargo 热态） |
| CLI 族（`-p auto --bin auto stdlib -- --test-threads=1`） | 8/8 | 真实二进制冒烟三模式（text/json/check 退出码 3/0/1） |
| auto-man scoped | api_gen 44/44 + freshness 真值表 1/1 | merged_api_client_crud_fallback 系 734 在案预存红 |
| `cargo check` ×3 | auto-lang/auto/auto-man--lib 零错误 | auto-man bin 空 main 系 P736-R4 预存 |

## AC 证据绑定

- **AC-01**（全库清点+完整分母）：inventory 115 层恰清点+t02 无静默遗漏测+CLI `--format json` status=partial（39 parse 失败不删分母，CLI 退出码 3）。
- **AC-02**（共同装配计划）：①VM 管线=T-03 层选择接线+`vm_executes_selected_layer_witness`（选定层 body 真执行 142→42 例）/persistent 对拍；②Rust 腿=⑤腿 rustc 实编实跑 witness（a2r_std::json 路由+真实输出）；③C 腿=trans 入口显式装配目标+t03 candidate 记录（C 实编 witness 明示阻塞→D3b，无 C HTTP provider=Unsupported 不造 server）。
- **AC-03**（声明/实现/native 绑定校验）：validate_core_vm_bindings（resolved+bound 双面）+ID 别名冲突检测（声明组/末段两合法形）+13 处生产冲突冻结基线+sse stub 假名负测+t04 全族。
- **AC-04**（源段错误位置+use 隔离）：target_layer_syntax_error_attributed（T-03）+use_semantics 7/7 回归+同名用户模块干净遮蔽反例+双 body 冲突反例（本轮）。
- **AC-05**（target/environment 严格区分）：六核心矩阵 `738-stdlib-matrix.{json,md}`（四格全在册+非 Supported 必有原因+Browser io/net 全族 Unsupported+json rust claim supported 实证）。
- **AC-06**（缓存失效正确性）：T-05 八测（同 mtime 改层/增删层/双 session/跨装配不串/依赖闭包漂移/命中补全 bytecode+manifest/retarget 守卫/root 守卫）+装配指纹内容敏感性单测+新鲜度真值表。
- **AC-07**（CLI+生成消费 manifest）：inspect 三模式 8 测+真实二进制冒烟+`stdlib_assembly_fingerprint` 进 generation.json receipt+backend_generation_is_fresh 装配比对（改 stdlib 但 api.at 不变→判陈旧→再生）；真实「生成→serve」实跑 witness 归后续（fixture 成员发现流程预存行为在案，非 738 引入）。
- **AC-08**（验证等级/兼容/规范可沉淀）：验证等级诚实（io 第四绑定面=Unverified、http 扫描名=Unverified、json id 面分裂=Unverified、sse=DeclaredStub——全部有原因不冒称）；729/730/734/736 兼容=plan727/729/730 59/59+api_gen 44/44；SD 沉淀稿见下。

## 发现与债务（KNOWN-DEBT 已登记）

- P738-D1：13 处生产 native id 相撞（check 如实上报非零；重编号 D3b）。
- P738-D2：json id 面分裂（catalog canonical 1906 实绑 vs 扫描名 99xx 无 shim）——公共面全量重写下轮。
- 新增（本轮 witness 实勘，建议复审后补登记）：
  - `Json.as_int(Value, key)` 下沉 `as_int_str(value.as_str())` 类型缺口（Option<&str> vs &str）——转译器 json 宿主映射语义 parity 属 D3b；
  - 内嵌 `auto_lang::a2r_std` 与 workspace `a2r-std` crate 内容漂移（前者无 as_int_str；生成服务走 crate 形态=发射头 "from crate" 证实）。

## Spec delta 状态

起草时 7 个 SD（SD-01 新增 assembly-manifest.md + SD-02..07 修改）为提案；
正式 canonical 沉淀在 merge 阶段执行（`/auto-plan:merge`），本报告绑定最终
revision=1、分支 `a4afb3b65`。736 canonical（http-service-deployment.md 等）
已由其会话沉淀，本计划 SD-05/06 与其兼容（receipt 新增字段不改既有键形状）。
