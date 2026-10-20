# PLAN-815 M815-L T-15 恢复方案、候选数据与反例验证

## 0. L1 复审修复轮(2026-10-20,[review-l1](../review-l1/review.md) 六项全部闭合)

| Finding | 修复 |
|---|---|
| F-L1-01 (P1) 退出码 | main 传播子命令退出码;convert BLOCKED=3(语义显式)、READY=0;apply 0/1(失败已回滚)/2(拒绝)/4(外部更新冲突)。自检全部经**真实 CLI 子进程**断言退出码与文件不变 |
| F-L1-02 (P1) 并发/回滚覆盖 | 写通道=账本文件排它锁(msvcrt/fcntl,锁内 read-CAS-write 同句柄)+每 checkpoint CAS;回滚仅在当前内容**确系本运行写入**(A/B 字节)时恢复精确原件,外部更新保留并报告(exit 4 / rollback-skipped);checkpoint 间外部写入反例(真实探针注入 v777)= exit 4 且逐字保留;写后失败且文件被外部改 = 不回滚不覆盖;deposit 脚本回滚同样加护栏 |
| F-L1-03 (P2) 转换前像/保护 | source 永不被改(改名在深拷贝上执行并以 id 转换行记录,含新 id 的 id 字段声明);原值在改写**前**捕获(字符串化 related 现记原字符串,如 `"['PLAN-656']"`);缺失 vs null 以 `<missing>` 哨兵区分;未知字段搬 sidecar 亦记转换行;protections 拒绝未声明的删除/新增/改值与 section 越界 |
| F-L1-04 (P2) 镜像不完备 | u64 范围(非 bool、0..2^64-1)、列表元素类型、六区精确集合/顺序、id↔PascalCase type 配对;apply 的真实 loader 探针**必填**且核验文档身份(project/version/总数/六区+type),不再只认退出码 |
| F-L1-05 (P2) 双 checkpoint | 恢复(A,含裁定改名)与四条阶段增量(B)拆分:57→58→59,各 checkpoint 独立写+真实探针身份核验;转换行携带 checkpoint 标记;"恢复不递增"的错误声明已撤回 |
| F-L1-06 (P2) 指纹错写 | 本报告全部 hash 由 convert-summary.json 程序化注入(本节由脚本生成,非手抄);§6 命令同步生成 |

**接口演进说明**:apply CLI 按上述修复演进为 `--expected-candidate-a/b-sha256` + 必填 `--loader-probe`;review-l1/review_probe.py 绑定旧单候选接口,其反例在新接口下由本工具自检以等效或更强的真实 CLI 用例覆盖(退出码/外部保留/回滚/前像/schema 负控),下轮审查可按新接口重绑。

2026-10-20。work owner=T-15 交付;自检仅表示可送 T-16,**未写生产账本**(live v57 全程逐字未动,SHA256 `c24b174e…` 每步核验)。实现工具在唯一树 `scripts/plan815_ledger_recover.py`(worktree `codex/plan-815-dev`);本目录为证据/冻结/候选(主检出,镜像审查探针模式)。

## 1. 冻结指纹

| 项 | 值 |
|---|---|
| live v57 | SHA256 `c24b174ee3c0eba25555c0a31f0c988a0a465af880c99388bf0f537925b03038`(version 57,1098 items,6 sections) |
| 历史备份 | `bak-v57-815a`=`669a6998…`(=写前 v56)、`bak-v56-811pre`=`55e30e38…`(t14 审计认定与 v56 其余数据语义相同) |
| store schema 源 | `auto-musk/backend/crates/musk/src/specs.rs` SHA256 `c124c844…`(只读,未修改该仓) |
| 逐字冻结副本 | [freeze/](freeze/)(live+两份备份,字节拷贝) |

## 2. 全库缺陷普查(audit.json)

1098 items + 6 sections 全部不满足 store 必填结构;逐类计数:

| 类 | 计数 | 处置规则(basis) |
|---|---|---|
| status 缺失/null | 218 | → `empty`(工厂默认;无记录≠已验收) |
| status `active` | 133 | → `stable`(旧语义=现行有效;生命周期集合内最近似且不构成验收主张) |
| status `delivered` | 2 | → `done`(旧值本身即交付主张,等强映射,不新增主张) |
| status `accepted` | 6 | → `approved`(同上,接受主张等强映射) |
| status `current` | 6 | → `stable`(同 active) |
| 重复 ID | 1(P014-1×2:designs/reviews 各一,内容不同) | **BLOCKED 待裁定**;T-15 建议 reviews 臂改名 `P014-1-R`(镜像 Pxxx-R 惯例,`--ruling reviews/P014-1=P014-1-R` 已实现) |
| created_at/modified_at 缺失 | 1064/1064 | 旧字段名搬移:580/510 条带 `created`/`last_modified`(ISO 日期字符串)→ 精确解析为 UTC 纪元秒(日精度无损);无旧值 → 0=未知哨兵(不补造时间) |
| depends_on/related/tags 缺失 | 1050/312/764 | → [](容器缺失=无声明);related 为派生字段(rebuild_relations 重算) |
| related 字符串化列表 | 8 | 解析还原为列表,值保留(原值入转换表) |
| Option 字段缺失 | 各 1064- | → null(工厂默认);completed_at → null(不发明完成时间) |
| content 缺失 | 2 | → ""(原即缺失,未补造) |
| 未知字段 | 10 items(created×残留/last_modified×残留/canonical_section/provenance/superseded_by/source_sha256) | **真实 load/save 实测:静默丢弃**(probe 注入实验);全部逐字存 [unknown-fields-sidecar.json](convert-final/unknown-fields-sidecar.json) |
| section 结构缺失 | 6 sections | section_type=**PascalCase**(`Reports` 等,见 §3)、title=工厂显示名、status=`empty`、content=""、depends_on=[]、last_modified=0、last_verified=null |

**§5.6 防护**:唯一 ID、原始 ID 全保留(含重复对)、非目标字段逐字节不变(转换表外改动即拒绝)、全库计数不变、候选再转换稳定(round-trip)、写路径 CAS/备份不覆盖/读回失败回滚。

## 3. 关键发现:SectionType 线格式为 PascalCase(真实 loader 实证)

`SpecsStore::load` 对 snake_case `section_type` **不报错**——`#[serde(other)]` 吞为 Unknown 后被静默滤除,返回"成功"的**空文档**(0 sections/0 items)。python 级 JSON/镜像校验完全无法发现此坑;唯一检出手段是真实 typed load。已修:工具与修复版 deposit 脚本的校验/生成均改 PascalCase(deposit 脚本 self-test fixture 同步修正,现 PASS)。

## 4. 候选数据(T-16 审查对象,hash 由 convert-summary.json 程序化注入)

以 T-15 建议裁定(`--ruling reviews/P014-1=P014-1-R`)+ 四条 M815-A 阶段增量([stage-increments.json](stage-increments.json)=收尾冻结稿逐字)生成,**双 checkpoint**:

| 项 | 值 |
|---|---|
| source(v57) | `c24b174ee3c0eba25555c0a31f0c988a0a465af880c99388bf0f537925b03038` |
| **checkpoint A(结构恢复+裁定改名,v58)** | `d571dbe3f87289c180f7e0d8cbfbe519409162ec860ad8d069ccf9b71bef8f43` |
| **checkpoint B(+四条增量,v59)** | `33830b65bd58aebe8a15f94fb71906f64b56883d89140697e6496f32d8475063` |
| **conversions(全部,sort_keys 规范化)** | `0a1e1cbba1eb7d01ce3904d1da742d48bff961cfa2b107aba01dee41980cb961`(12163 条:A 12135 + B 28;每行含 checkpoint 标记/真实原值/依据) |
| 状态 | **READY**(blocked=0/schema A·B=0/protection A·B=0);不带 `--ruling` 时正确 BLOCKED 于 P014-1 |

产物:[convert-final/](convert-final/)(candidate-a-recovery.json / candidate-b-final.json / conversions.json / sidecar / convert-summary.json)。

## 5. 真实 typed loader 验证(V-L1/V-L2)

probe=[t15-recovery/probe](probe/)(`musk` path 依赖,只读;`SpecsStore::load/save`);二进制 `target/plan815-t15-probe/release/plan815-ledger-loader-probe.exe`。

| 用例 | 结果 |
|---|---|
| V-L1 负控:live v57 | **LOAD-FAILED**(首个拒绝:`unknown spec status 'active'`)——真实 store 拒绝现状 ✓ |
| V-L1 正控:候选 | loaded:true,**6 sections/1098 items 与原账本逐区计数一致** ✓ |
| V-L2 往返:候选 load→save | 1098→1098,ID 全保留,除未知字段(候选已无)外**零差异**;related 不重算(仅 upsert 重算)✓ |
| V-L2 未知字段边界 | 注入 `provenance_note` → 真实 load/save 后**字段消失**、1098 项全保——边界实证,sidecar 为唯一保留通道 ✓ |
| V-L2 状态正控 | 候选(含 stable/verified/published/proposed/done/empty…)整体加载成功;非法值族由负控+转换表覆盖 |
| V-L3 工具自检 | PASS:dup-block/dry-run-intact/unknown-sidecar/iso-time-move/status-map/apply-hash-drift/conversion-drift/cas-backup/read-back-rollback(含真实探针失败的写后回滚) |

## 6. T-17 全链彩排(临时副本,生产零接触;真 .exe 探针身份核验)

recover apply(源/A/B/转换四 hash 绑定 + 必填真实探针,A/B 各自写后身份核验)→ 最终 loader version **59**、6 sections/**1098 items** ✓ → 四条 P815-A 恰好各 1(status=verified/created_at=1792467247)✓ → P014-1-R 改名在场 ✓ → 生产文件 hash 不变 ✓;自检(V-L3)含真实 CLI 退出码族、checkpoint 间外部写入 exit 4 逐字保留、写后外部更新不被回滚覆盖、探针失败精确回滚、未声明删除拒绝、schema 负控(version=-1/bool/tags=[7]/u64 越界/缺区)。**T-17 实际命令**:

```bash
python scripts/plan815_ledger_recover.py apply   --ledger .autoos/specs.json   --expected-source-sha256 c24b174ee3c0eba25555c0a31f0c988a0a465af880c99388bf0f537925b03038   --expected-conversions-sha256 0a1e1cbba1eb7d01ce3904d1da742d48bff961cfa2b107aba01dee41980cb961   --expected-candidate-a-sha256 d571dbe3f87289c180f7e0d8cbfbe519409162ec860ad8d069ccf9b71bef8f43   --expected-candidate-b-sha256 33830b65bd58aebe8a15f94fb71906f64b56883d89140697e6496f32d8475063   --stage-increments docs/plans/attachments/815/t15-recovery/stage-increments.json   --ruling "reviews/P014-1=P014-1-R"   --loader-probe target/plan815-t15-probe/release/plan815-ledger-loader-probe.exe
```

## 7. 待 T-16 裁定/核准清单

1. **P014-1 重复 ID**:建议 reviews 臂改名 `P014-1-R`(designs 保留);不同裁定则候选/转换 hash 需按新规则重生成并重审。
2. **状态映射**:`active/current→stable`(133+6)、`delivered→done`(2)、`accepted→approved`(6)、缺失/null→`empty`(218)——均为"等强或更弱表示、原值入表",请逐项核准或改判。
3. **时间表示**:ISO 日期→UTC 纪元秒(日精度,86400s 粒度);无旧值→0 哨兵;completed_at→null。
4. **未知字段处置**:store 无槽位且真实 load/save 必丢弃 → 仅 sidecar 保留(canonical_section/provenance/superseded_by/source_sha256/残留 created-last_modified)。
5. **PascalCase section_type**(§3)为 store 真实线格式;候选据此生成。
6. 版本策略:恢复**不递增** version(结构投影,非知识变更);T-17 后如需 upsert 语义递增由 deposit 负责(彩排中为幂等跳过)。

## 8. 边界声明

本阶段只恢复兼容投影+四条增量;不改 canonical Spec 正文、历史业务结论、外仓代码或在线服务;`auto-musk` 只读。1064 条旧记录的 created_at 多为 0 哨兵=历史未记录,任何消费方不得将其解读为 1970 年实际事件。tf 6270/6270 等 batch 证据与本恢复验证互不冒充。
