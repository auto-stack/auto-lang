# generated-debug-policy-production

> **Status**: active（PLAN-1003 SD-02）
> 范围：生成式 Rust 消费者（UI dev 前端成员、Web 服务后端经 ws 根清单、显式
> 交付根）的**受管调试信息策略**——定义、落点、身份、覆盖/恢复接口、迁移与
> 回退边界、实测口径。研究对照见
> [generated-debug-policy-study](generated-debug-policy-study.md)（1001 研究
> 与"采用C"裁定）；构建身份与缓存根见
> [generated-cargo-build-lifecycle](generated-cargo-build-lifecycle.md)。
> 单源实现：`crates/auto-generation-core/src/generated_build.rs`
> （`resolve_debug_policy`/`managed_profile_toml`），受管根清单由
> `rust_workspace::try_ensure_workspace_for` 写入，交付根由 auto-ui
> `generate_delivery_root_manifest` 消费同一块。

## 1. 策略与落点

| 策略 | 受管根 manifest 内容 | 语义 |
|---|---|---|
| `c`（缺省） | `[profile.dev] debug="line-tables-only"` + `[profile.dev.package."*"] debug=false` | 成员保留行号表；全部非成员（含 path 依赖）无调试信息；增量不受影响；release 永不写任何 `[profile]` |
| `full`（显式） | 仅 marker 注释，无任何 `[profile.dev]` 段 | 完整恢复=cargo 默认 dev（成员+依赖 debug=2、host-only 延迟关闭）。`"*"` 无环境变量覆盖形式，故完整恢复必须由生成器省段，不能靠 `CARGO_PROFILE_DEV_DEBUG=2` 只改成员 |

落点=受管 ws 根 manifest（非 `.cargo/config.toml`）：cargo 的 config 发现锚定
cwd，`--manifest-path` 手工运行发现不到 ws 内 config（F-910-R1-01/905 同源）；
manifest 落点与 cwd 无关，受管/手工一致生效。成员/非成员按**实际成员集**判定：
dev ws 成员=生成应用（producer crates 全为 path 依赖=非成员）；交付根成员=
app+vendored 闭包、registry 依赖为非成员——两类成员集的有效值不同属预期。

## 2. 接口与优先级

- `AUTO_GENERATED_DEBUG_POLICY`：缺省/空=`c`（来源 managed-default）；`c`/`default`
  =显式 c（同标签，同效配置不分叉缓存）；`full`；未知值=具名错误，**在任何受管
  文件写入前失败**（manifest/config/收据字节不变）。
- `CARGO_PROFILE_DEV_DEBUG`（cargo 原生，仅成员级）：缺省策略下=合法用户局部
  覆盖，保留+记录（标签 `c+user:CARGO_PROFILE_DEV_DEBUG`，值入身份键）；与
  **显式**策略并存=具名冲突拒绝（并存结果不再是所断言的策略）。
- 调用方不得以任何 override 臂签 `c`；其他 `CARGO_PROFILE_DEV_*` 键不在本策
  略身份内（cargo 指纹自理）。

## 3. 身份与新鲜度

- 默认 `c` **不改变** `ctx8` 哈希输入（schema‖producer‖flags 三段逐字节同旧）：
  既有根与 release 缓存存活，dev 单元由 cargo 指纹原地迁移。
- 非默认有效策略（显式 `full` 或用户局部覆盖）作为第 4 段进 `ctx8` → 独立
  缓存根+独立 lease；多并存策略物理隔离，同名旧件不可能串用。
- `context.json` 加性记录 `debug_policy`/`debug_policy_source`；旧根无字段
  =legacy 判定。前端收据 `generation.json` 加性记录 `debug_policy`：缺字段
  （legacy/旧工具产物）或异标签=不新鲜=重建重发布；`discover_front_exe`
  同时校验策略标签与发布件 sha256——策略切换/换名同 exe/伪造标签均不发现。
- 无变更 ensure/重跑：manifest、config、context.json、收据 bytes 与 mtime
  稳定（写前比较）；失败/取消/坏配置不覆盖 last-good 发布件。

## 4. 发布件行号（pdb 别名）

发布 exe 的 RSDS 记录 pdb 名为 cargo 报告名（`<bin>_<id8>.pdb`，相对名），
dbghelp 的 exe 目录回退按该名查找。publish_artifact 除用户可见 stem 名外，
**必须**再以 cargo 报告名硬链接一份别名（随重链接刷新）；只发布改名 pdb 会
使原样发布件解析 0 成员符号（1003 实测 163 导出符号/0 成员）。

## 5. 迁移、回退与残留

- 旧→C（原地，默认根）：cargo 重建 dev 单位（1003 实测 002+013 工作集 596
  单元/124s），旧 A 变体 rlib/pdb **残留**在 deps（实测残留≈9.85 GiB），不受
  1001 增量治理工具管辖（其治理单元=incremental 子树）——回收=整根守卫删除
  （generated-cargo-build-lifecycle §5 的 context.json 证明+lease+闲置预览/
  apply），删除后 C 冷重建即稳态。禁止自动广扫/全局 cargo clean；不为收尾强
  迫用户即时清旧缓存。
- C→`full` 回退=隔离新根冷编（首次），再回 `c` 即时（默认根仍暖）。代价如实
  计（新根≈full 闭包大小），不得以原地混用替代隔离换速度。
- 实测口径（PLAN-1003 冻结工具，002-counter+013-todo 工作集，隔离双根、
  sccache 禁用、分配字节按卷 file-ID **跨根全局去重**——ws 内发布 pdb 是
  target 根 pdb 的硬链接，root+ws 分别相加会重复计数）：原默认闭包
  11.64–11.70 GiB → C 5.08 GiB（−56%），重复冷臂差 E=3,089B；冷编 128.6s vs
  168.2s；成员行号 oracle 两臂同解析（`main.rs:300`），C 下依赖符号无行号
  （裁定已知代价）；迁移瞬态峰 16.0 GiB（采样 3s）。Web release 不受
  `[profile.dev]` 影响（切策略 0 重编实证）。

## 6. 交付根（显式可搬动交付）

交付根 manifest 消费核心单一策略块（不抄取 producer 工具根自身
`[profile.dev]`——该抄取丢 `package."*"` 覆盖）；块随包静态携带、消费侧不读
环境；`delivery-manifest.json` 记 `debug_policy{label,source}`。策略在任何
staging 写入前解析，未知/冲突失败时既往交付原样保留。移根包按实际成员集
生效（实测：app/vendored 成员=line-tables、registry 依赖与 exclude 的
vendor 树=无调试信息）。
