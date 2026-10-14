# PLAN-777 第三轮独立 final review

结论：**needs_fix**。R2 两项具体反例关闭，依赖检测/ownership 验收通过；仍有一个输入别名保护遗漏。不是要求扩展审计范围，而是原 AC-01「输入只读、报告另存」在当前已读取的受管 Rust 输入上仍可被破坏。

## 冻结审面与独立性

- plan_revision=1；review_scope=final；唯一实施树 `D:/autostack/.wt/lang-777/auto-lang` / `plan-777-dev`。
- reviewed_commit=`677055420203db3bc3a868a2dca697f8167361dd`；code=`d6e9479124d7aa11a92d5c63cc3df30874c2af31`；base=`17bc8431457848b040bffa5c261e13b057771559`。
- 组内 auto-down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；主检出=`21e50bcd7b1d46bf6e0d4ab6fb2de6b8b7b02f0d`；Python3.14.2/Cargo1.98.0。
- 本评审会话参与规划/R1/R2，未实施；从提交、原反例及独立输入组合重建判定，无额外 agent/worktree。
- repository-layout Spec SHA256=`3800856129eb9b5bd742fef9ea2aa0c94f0db780ab2a8ba01fc7ae8965dde49b`；冻结 SD01 见 `spec-delta-reviewed.md`。不发布 canonical Spec/ledger。
- 实施树干净、HEAD/branch 前后不变，受管输入完整 hash 前后相同，Cargo.lock 未变。全部变异/硬链接只在评审自有临时副本；真实源码、兄弟仓和 worktree 没有创建链接或被改写。`fingerprints.jsonl` 保存全输入身份和判定前复查。

## Finding：F777-R3-01 / P2 / blocking

**根外硬链接指向受管 Rust 输入时，输出守卫仍会放行并覆盖源码。**

原 AC-01；T-01/T-03/T-04；R1-03/R2-02 输入保护不变量的剩余路径，不是新增语义解析或扫描扩面。

当前 `same_file` 支持文件身份比较，但 `guard_output_target` 只对 config/baseline/read_manifests 使用它。对 tracked 输入，第 904–906 行先按目标路径是否在 root 内分类；根外直接 return，第 908 行才按根内相对路径字符串查 tracked。这遗漏了根外路径与受管源码的文件身份相同这一状态。

独立输入矩阵 M01-hardlink-source：复制既有 fixture、Git 提交有效输入，在临时容器中用 `os.link` 建立根外报告路径到 `crates/fixture-lang/src/lib.rs` 的硬链接。断言 `original.samefile(alias)` 为 True，然后执行真实 CLI：

```text
python <implementation>/scripts/modularity-audit.py --root <owned-fixture> --mode observe --output <owned-container>/source-output.json
```

预期非零且输入字节不变；实际 **exit 0，源文件 SHA256 改变，源码被 JSON 报告覆盖**。工具在此前已对该受管 .rs 计算规模/hash并做文本扫描，故它确实是此次审计输入。相同状态的配置及成员 manifest 硬链接均 exit 3/原字节不变；独立根外新报告 exit 0，报告有效且源码未变。矩阵 3 pass / 1 fail，验证脚本整体 exit 1。

完整原输入、目标身份断言、前后 SHA 与真实结果见 `input-matrix.jsonl`。重放：

```text
python docs/plans/attachments/777/review-r3/input-matrix.py
```

修复方向：将所有受管/实际读取输入的文件身份纳入写前保护，再决定根外输出是否可以另存；或采用同等有效、保留合法另存行为的保护方案。统一使用现有身份比较，不能只保护配置/manifest；加入源码（及被读取的锁/其他受管输入）的别名负控，保留配置、manifest 和根外独立报告正控。本问题可在原 r1 合同下局部修复，不因第三次失败自动要求架构升界或 needs_replan。

## 前两轮闭合对账

| Finding | 本轮结果 |
|---|---|
| R1-01 多 target 表 | 原样重放通过；target + inherited alias + build/dev 组合通过 |
| R1-02 workspace path 基点 | 原样重放及组合通过 |
| R1-03 输入覆盖 | 配置旧例/根外 manifest 旧例关闭，完整只读仍 partial，见 R3-01 |
| R1-04 optional 激活 | 原样重放通过；完整声明身份豁免仍有效，无新增豁免 |
| R1-05 ownership 遗漏 | 27 行、7 组和补登路径检查通过 |
| R1-06 外部 SDK 误红 | 合法 SDK 对照通过；惰性旧 C06 与生产规则正控分开记录 |
| R2-01 外置 auto-ui 域映射 | 关闭；生产域表/规则实际 auto-ui=2、auto-man=2、SDK=0，永久 v10 有 normal/build/target/继承矩阵 |
| R2-02 已读根外 manifest 覆盖 | 具体旧例关闭；S02 exit3/hash不变，永久 v11 大小写别名及根外另存通过；通用输入身份路径仍见 R3-01 |

检查两连败后的 work 诊断：已明确「具体反例闭合≠完整不变量闭合」，补了生产配置的真实矩阵及实读 manifest 集合，具备再审入口；但身份比较只接入了部分输入类别，完整输入/目标矩阵仍不齐。本轮不是仅重跑自测数量，而是选择源码同类身份状态作组合验证。

## AC/task/evidence

| AC | 结论 | task / evidence |
|---|---|---|
| AC-01 | partial | T01/T04；真实双扫/规模/hash/revision通过，输入别名矩阵失败 |
| AC-02 | pass | T02/T03/T04；原7控与R2补充5控通过，新增禁止边/合法对照、窄豁免/撤销/过期/宽豁免都有真实结果 |
| AC-03 | pass | T02/T03；ownership完整，candidate分类/限制明确 |
| AC-04 | 审计完成，未获pass | 本报告/提交/输入指纹绑定 |
| AC-05 | pending | merge-owned沉淀/合流/归档/guard-clean；未执行 |

独立 help/self-test 14/14；旧 replay 7/7，R2 supplemental 5/5，输入矩阵 3/4。真实 observe 两次字节相同（评审保存的 Windows 文本 SHA256=`035552106589a3f5aa930f96f843459b2e66855bd52636b29d8f575d93324f49`）；2052 .rs/945894物理行，36声明边与Cargo metadata --locked零差异，strict正控pass/0违例/2grandfathered，ownership27/7组。`primary-runs.jsonl` 与原输出留存，真实树输入不变。`git diff --check` 通过，无Rust源改动/编译/测试；Category A无Rust整档门禁。

工作摘要的R3验证表V01仍写t04r2文件/旧hash，不能复用为R3凭证；本轮直接生成独立R3证据。修复后同步该摘要行即可，不另扩功能范围。

## 状态与下一步

保留已验证 T-02；重开 T-01/T-03/T-04 及其后置证据；Plan **executing / 1/6**，r1合同不变。历史成功证据与旧finding均保留。下一步同树 work 补全输入身份保护及永久控制，提交后重放该矩阵和原控制，再独立复审。本轮不修改实现、不merge、不发布Spec/ledger、不清理树；781/782 WIP 保留。
