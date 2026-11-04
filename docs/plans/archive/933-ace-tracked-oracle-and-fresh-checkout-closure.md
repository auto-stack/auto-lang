---
plan_id: PLAN-933
status: archived
feature_name: ACE跟踪oracle与干净检出验收收口
author: [Codex]
created_at: 2026-11-04
updated_at: 2026-11-04
plan_revision: 2
supersedes_spec_components: [docs/specs/auto-semantics/semantic-contract.md, docs/specs/auto-test-case/a2r-hir-certification.md]
new_spec_components: [docs/specs/auto-test-case/ace-source-only-replay.md]
touched_goals: [GOAL-002]
affects: [auto-semantics, auto-test-case, auto-a2x]
depends_on: [PLAN-796, PLAN-904, PLAN-925, PLAN-926]
current_step: 8
total_steps: 10
---

# [PLAN-933] ACE跟踪oracle与干净检出验收收口

## 0. 变更摘要

同一“交付源码可重放验收”职责下关闭两类缺件：D-925-1的sema两测试依赖git忽略附件；904二十个collections case声明比较manifest.json但未跟踪该金样。保原描述/版本/行为/完整artifact断言，以跟踪规范/独立可审计oracle补齐，不能删测试、恢复ignored目录或复制actual凑绿。

依据[本波调查](reports/ace-next-wave-correctness-and-admission-2026-11-04.md)。drafting r2 0/10；只规划，不运行构建测试/认证，不改sema求解器或Rust/C生产逻辑。材料缺口闭合不能代M1整个架构签章。

## 1. 目标

- sema schema/版本/描述面全部既有断言只读交付的跟踪来源，新鲜实施树不依赖主检出遗留附件。
- 904全部原source/结果及已有三文件金样不变，缺失manifest oracle可由独立合同与冻结材料追溯并持续判错。
- 存量错误产物、错误来源/版本/字段/缺文件仍被真实父侧拒绝，不把文件存在当通过。
- 原职责组在干净源码条件可重放，缺工具/未执行仍为真实非PASS；材料/统计不凭主检出缓存。

非目标：不做全仓附件迁移、批量回归引擎/组拆分/全golden重生，不改语言/IR/资源/编译输出或903/908分母/原批准，不修其他运行时债；发现代码错转实际owner。

## 2. 架构方案

sema版本/描述来源归canonical Spec或最小独立跟踪fixture（必要新路径拟`experimental/ace/crates/semantics-store/tests/fixtures/`），测试核真实常量与事实，不能改为永真注释或跳过缺文件。尽量引用现有跟踪权威，不复制庞大历史附件；输入来源/原决定/当前schema及review记录可追溯。

904 manifest是已有期望的一部分，不新增放宽expect。独立重建其格式/内容：固定原AT字节、三份已跟踪golden、版本化manifest合同、独立字段/文件hash/来源算法及必要原冻结收据。actual emitter仅为被测对象，不为oracle生成器。无法独立推导的字段先在T-02说明来源/冲突，needs_replan，不任意填数或改变expect。

## 3. 技术栈

Rust sema边界测试/serde或既有manifestdecoder、现TestCase/AutoDown与固定组，SHA256等现内容校验设施。无新的发现/调度/证明框架或缓存DB；只补本包输入与判面。

## 4. 需求分析与背景调查

**r2复核（2026-11-04）**：本轮用户只授权复核/修订规划，不施工、派发或运行验证。初读基面 `ffc59e505`，终检 `50555f3d3`（928由3/12推进4/12，源码调查身份未漂移）；D927=`5d5054d5d` 已交付归档并清树，928 executing r2 4/12，929 executing r2 1/14且按用户裁定挂起等待D928；930 drafting r1 0/9。下列 aa6791649/927未合入等为初订调查历史，由本条与[本次复核](reports/ace-931-933-plan-review-and-m1-distance-2026-11-04.md)取代其状态/写窗描述；不改在途计划进度/授权，未知D不以分支代替。

用户授权规划下一步，未授权施工/派发。main取材`aa6791649`；D925=`a37245cd6`、D926=`3fce61b9a`在主线；D925-1及904缺件为既有收据，不是本轮实测。sema两入口：`sema_boundary_frozen_versions_have_provenance`与`sema_boundary_description_surface_frozen`，均读`docs/plans/attachments/796/contract-decision.md`。该目录按现AGENTS不入库且merge清材料，不能要求恢复它。

`crates/auto-lang/test/cases/a2r-hir-collections-904/L04.case.ad`期待四文件，`golden-L04/`实际只有main.rs/a2x_runtime.rs/generated.rs；T-01对全部20源与case逐一确认，不把L04单例外推为全部已验。worker组装collections manifest绑定完整替换源sha与execution config，不能用片段hash或借Rust/C其他族manifest。

已读auto-semantics/semantic-contract/inference-profile、auto-test-case/a2r-hir-certification/固定组合同与当前AGENTS材料政策。实施树拟`D:/autostack/.wt/lang-933/auto-lang` / `plan-933-dev`（未建），唯一代码树。材料开工后仅`D:/autostack/plan-artifacts/plan-933/`登记，merge摘要入本计划后全删，所有外部输出显式路径/target归scope。

## 5. 详细设计

### 5.1 sema跟踪来源

T-01列两测试全部常量/描述token/约束、实际producer/codec/consumer版本与原权威出处；不得只补一个文件让读成功。T-02决定读现Spec、最小独立冻结fixture或按语义契约结构化验证的组合；当前生产常量必须对独立约定，而非生成器自动写同值后自比。新增跟踪fixture带来源/版本/适用边界，不写specs.json/派生索引，不将全量附件重新入git。

验证fresh树没有ignored796附件时原两例仍覆盖原断言；在可控fixture副本删/改producer/schema/枚举项、合法近邻，真实断言必须红。不得改`expect present`为`if exists`、ignore或删负控。生产语义/schema常量不为测试改动。

### 5.2 904独立manifest oracle

冻结全部原AT/结果期望、现有main/runtime/generated金样的字节与SHA；比较contract/版本/target/profile/input/source/config/entry/文件集及内容hash等所有必需字段。可从已跟踪冻结文件头/代码或源直接推导的字段列算法和输入，不用新actual回填；缺少可信事实的字段作为T-02的阻断，不能把自洽性伪装独立oracle。运行认证只是行为佐证，不能代manifest内容/来源正确。

T-04补缺失的`golden-*/manifest.json`（新增跟踪文件）及最小必要oracle来源说明，保case原四artifact逐字节断言；不删manifest、改比较模式、改源、改行为expected或批量重写三份已存在golden。若现实现与已批准合同矛盾，记录给实际代码owner（931若同因）而非以新expected接纳错误；真需变原合同则needs_replan并列具体差异。

对原20源逐项运行实际reader/parent，正例正确，改source/hash/config/target/profile/version/文件hash或删manifest必须被拒；合法同版本近邻可用。C926/其他Rust族产物或证书不能满足904身份。日常emit金样与显式独立rustc认证保持分轴。

### 5.3 干净来源、依赖与交接

在唯一实施树冻结/提交后验证，其初始状态本来不含ignored附件，不删除/改名主检出用户材料来模拟。跟踪输入闭包逐项`git ls-files`/hash核验；如需额外源码快照，仅放scope作只读consumer输入，不是第二实施树；依赖git元数据的边界测试必须在唯一真实实施树跑。所有工具/输出/target显式归scope或正常专属树，不写attachments/项目目录散落证据。

与931无循环：本包先由当前冻结合同/三份已有golden独立补manifest，不等931 actual。931若改变合法打印/语义版本字段，在本包稳定输入基线上自行作最小来源更新并独立证明，不让本包吞代码修复。932可消费跟踪sema材料，不等本包全归档才做私有native反例。

P为sema测试/跟踪Spec来源fixture与904缺失oracle；H仅必要case来源清单/固定组/验证decoder，与930交付治理/931/932按文件实际范围单写。本包不改a2x-rust/runtime、core grammar、828/927/928/929候选语义。交接记录owner/revision/base+handoff SHA/schema/全部reader/已有-STALE-pending/反控及释放。计划簿记归main，current Spec在review/merge经真实交付再沉淀。

### 5.4 r2独立来源、全链增量与写窗（T-01..07，AC-01..07）

**sema不能由当前常量自证约定**：两测试所需完整集合为producer/ref-v1/page-v1/page-v2/logical-domain、INFER_ALGO_ID、CT_SEAM_SCHEMA、SEMA_LAYER、SnapshotRecord、13种TypeDesc及三种Mode和原描述token。T-02把每项对应到已跟踪冻结规范/原决定的具体位置+内容hash；当前生产值只能作为被测值。正文提到token不代替精确schema值/枚举闭包检查，动态生成fixture再与当前常量比较不构成oracle。无需另造第二权威，优先补最小规范缺额；规范差额冻结后做实际value/token/缺项负控。历史ignored附件只可作辅助调查，不是日后读入依赖或唯一批准依据。

**904字段出处表必须全20例完整关闭**：`build_coll_manifest`格式调查不是批准来源本身，实际emitter/decoder互相同意也不够。T-02至少列schema、source_profile、hir_domain、input_hash、完整source_sha256、execution_config全部键/字面量/排序、entry、entry_fn、target四轴+resource_runtime、三个files条目顺序/path/hash，以及JSON缩进、换行和末尾LF。

- 原`.at`/case/caps/entry/recipe、三份已有golden、已批准fileset/版本合同均先冻结bytes+hash。由独立SHA/FNV参考计算并交叉核冻结header；entry_fn/target/backend_version不能只信生成header，须核冻结源与已批准映射/版本资料；缺可信轴即T-02停止该件并needs_replan，不能先全量生成。
- 可新增独立小型reference重建器/来源fixture（拟测试路径，在T-04建立），不得import/call生产emitter/helper或运行actual生成golden。独立review抽查L04及其余19例字段来源和reference算法，保L04/L05=42等原独立值/trace，不能只审JSON可解析。
- 负控必须触达实际worker/parent判面，包括同schema但错artifact hash、少/多/重复path、路径域/大小写冲突、跨C或其他Rust族、旧/错工具证书，以及合法同版本近邻；不能只验证oracle重建器拒绝自己构造的坏值。receipt绑定源/合同/oracle/工具/文件集，只有实际匹配可复用，不能换oracle后继续用旧PASS。

V-05是保原已承诺增量的回归，不由本包新增通用恢复生产能力。T-05按20例逐族实际支持入口列fresh/hit/restart/body-edit/失败修复；Rust rt/coll当前无AXP来源恢复锚，不能用C锚或whole-request restart证明Rust纯恢复/首编辑。恢复→立即编辑及恢复→坏编辑→修复的新增机制由929交付；如本包最终验证时该稳定接缝已释放则联验并更新身份，否则明确该**新SYN腿由929持有pending**，不阻933仅缺件债的交付、不把它标PASS。933必须证明的原Rust restart重装配/函数生产0、来源深验与原负例仍完整；任何原门真实代码红（包括931）须按实际owner解决并在本包必要consumer收据中反映，不能用“测试包不施工”关闭一条红门。

与930的H只共享实际`test-groups.toml`/基础设施职责及来源校验触面，不改930的generated UI成员身份、A/B bundle/工具/L1-L2契约和918边界freeze。与928无生产S写面；929迁AST/来源接口时933只更新测试调用/身份，不能接走其语义/恢复实现。933 T-04独立补oracle并冻结稳定提交后立即释放904目录给931；931若确需变现有三个golden/生产版本，须从该提交接手、最小独立更新，933不得随后覆盖旧值。两包最终各自对其实际交付版本认证，不用对方将来的actual作为前置。共用测试源/黄金目录也算写面，不因“仅材料”允许并写。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-semantics/semantic-contract.md | 明确已冻结版本/描述权威跟踪来源，不改变生产schema值 | source-only可验证 | AC-01/02 |
| SD-02 | modify | docs/specs/auto-test-case/a2r-hir-certification.md | 904原四文件/manifest独立来源与emit-cert分轴保持 | 不靠actual补oracle | AC-03/04/05 |
| SD-03 | add | docs/specs/auto-test-case/ace-source-only-replay.md | 本次跟踪输入闭包/新树重放/负控/余账 | 不恢复ignored附件 | AC-06/07 |

## 6. 测试设计

新增oracle来源说明/测试夹具由T-02/03/04使能，原source/行为断言不改，新检查均未执行。

| case | AC | 输入/入口/work任务 | 断言与反控 |
|---|---|---|---|
| V-01 | AC-01/02 | T-01/02/03，原两sema测试，新鲜唯一worktree无796附件 | 全常量/token/版本声明仍判，所有运行输入为跟踪文件；无可选跳过/ignored fallback |
| V-02 | AC-02 | T-03/06，独立fixture副本改producer/schema/枚举、合法近邻 | 真测试拒错误，非当前代码生成同值再自比；生产常量未变 |
| V-03 | AC-03/04 | T-01/02/04，20个原904 case+三份原golden+跟踪源码与manifest合同 | 全字段及JSON字节格式各有独立批准来源/算法，独立reference不调用emitter；已有源/结果/三金样hash不变，四文件bytes断言保持；无actual复制 |
| V-04 | AC-04/05 | T-04/05/06，真worker/parent：正确manifest、错source/config/target/profile/version/hash或缺件 | 正例与实际独立oracle一致，篡改/跨族/旧证书拒绝；source原行为/独立认证另证，不因补文件减少语义门 |
| V-05 | AC-05/06 | T-05/06，显式新树/跟踪输入闭包、无用户附件/cache，fresh/hit/restart/body-edit与失败修复，按实际支持入口分族 | 选择/发现/运行真实数量和原排除记录，metadata/manifest仍真实准入，未改函数生产计数原门不回退；不以C锚冒Rust恢复；新SYN首编辑未有接缝则由929持有pending，不充PASS |
| V-06 | AC-06/07 | T-07/08，固定组/consumer全链与只读跟踪材料损坏对照 | 不依赖主检出偶然残留，缺工具/未执行/代码真红如实具名，不降级PASS；材料与各实际owner余账可追溯 |
| V-07 | AC-01/02/03/04 | T-02/03/04/06，独立sema逐项表、904逐字段/字节参考器与冻结source/header交叉 | 错生产常量/漏枚举/错哈希算法/错顺序/LF反控可区分；reference不调用生产函数，来源未知阻断该件 |
| V-08 | AC-04/05/06 | T-05/06，实际parent上的少多重复/路径碰撞/坏工具或旧证书、合法近邻 | 整体父门真实拒绝、输入/oracle根只读，返回0/旧PASS或仅辅助checker红均不能关闭；原20例发现/执行守恒 |

职责组`cargo t ace-semantics ace-a2x auto-test-case infra-fixed infra-ace-m1`；必要集成`case-a2r-hir-collections-904 case-a2r-hir-record-text-902 case-a2c-hir-collections-926`，触worker/桥补`ace-adapters`。使用已有固定组，不建动态发现/全量目录治理；原正式性能/压力样本不因测试输入修复削减，无生产改动时不重复整条ace-performance。若既有scope因git依赖要求真实树，仅在专属树执行。未改输入证据可复用，首次红/环境重放/原条件排除分别记录。

实现→内部检查→独立review在同一树顺序交接，未指定模型/人员，不自动派发。全部新负控/重放由work任务使能，本轮不跑。

## 7. 验收标准

| AC | owner_stage | 关闭条件 |
|---|---|---|
| AC-01 | work | 原两sema测试全部跟踪来源/原约束保留，无ignored附件必需依赖 |
| AC-02 | work | V-01/02独立版本/描述来源与真负控，生产常量/schema未改 |
| AC-03 | work | V-03/07原20源/结果/三份已有golden全量冻结，缺manifest逐字段/字节来源与reference独立可审计 |
| AC-04 | work | V-03/04补缺且原四文件bytes断言全部保留，实际正确/错误manifest能区分 |
| AC-05 | work | V-04/05/08真实reader/emit/独立认证分轴、来源/版本/跨族与原增量门保真，新增SYN首编辑按真实接缝/929pending分列，不洗代码错误 |
| AC-06 | work | V-05/06新鲜专属树和完整跟踪依赖可重放，必要职责/consumer证据真实，无用户材料破坏 |
| AC-07 | work | SD/KNOWN-DEBT/931932交窗与源清单准确，只有本包缺件债按真实证据关闭 |
| AC-08 | review | 独立逐AC/SD/oracle来源/负控/遗漏复核revision-bound pass |
| AC-09 | merge | Spec读回/交付/archive/全输出路径对账材料摘要与删除/guard清树，未冒M1签章 |

## 8. 执行步骤

| T | owner_stage | 前置/实际写面 | 产出/验证与AC | 状态 |
|---|---|---|---|---|
| T-01 | work | 当前main/原796904规范与私有清单 | 两sema全部约束、20case/缺件/原hash/工具与写窗冻结，AC-01/03/07 | [x] 2026-11-04：冻结盘点落 plan-artifacts/plan-933/t01-t02-freeze-and-derivation.md（20例src/3金样/manifest 全sha表、sema 常量/代码位、判面/工具/写窗）；**前提漂移记录**——931 已于 1d3a21b49 以 daily-emit 通道补齐 20 manifest 入库（D-931-3），T-04 按 §5.2/§5.4 转为"独立重建核验已跟踪 manifest，不覆盖 931 旧值" |
| T-02 | work | T-01 | 完整sema逐项来源+L04 manifest逐字段/字节独立推导及其余19例适用性/typed校验feasibility；未知来源先needs_replan，AC-02/03/04 | [x] 2026-11-04：sema 逐项映射至 semantic-contract.md 新增 D2–D8 转写节（缺失项即 SD-01；附件 sha256 064adbd1… 留档）；904 16字段+JSON字节格式全部有独立来源（fnv1a-64/SHA-256 公开算法+跟踪 src/金样+spec/归档904合同+金样头冻结），抽样 6 例逐字节重建一致；**无未知来源字段，无 needs_replan 项** |
| T-03 | work | T-02，sema测试/跟踪fixture | 原两测试只读跟踪权威并保真约束/负控，V-01/02，AC-01/02 | [x] 2026-11-04：两测试改读 docs/specs/auto-semantics/semantic-contract.md（断言体原样抽 helper，强度不变）；新 V-02 突变负控（8种版本面+3种描述面突变 catch_unwind 必红）。新树基线 7 run/2 红（Os NotFound 附件）→改后 8/8 绿 |
| T-04 | work | T-02，904缺失oracle目录 | 独立补全部manifest及字段来源说明，保原源/expect/三金样，V-03/04/07，AC-03/04 | [x] 2026-11-04：coll_manifest_oracle_933.rs（零 ace-* import，自含 FNV-1a-64/SHA-256+公开向量自检）：20/20 manifest 逐字节重建一致+typed 等价+金样头交叉+目录守恒（20 golden×恰4文件）；V-07 拒绝腿（错输入hash/错源sha/近邻schema/config/错文件序/缺尾LF/篡改金样/缺文件条目）全红。原源/expect/三金样/931 manifest 值零改动 |
| T-05 | work | T-03/04，必要parent/case输入H | 实际reader/来源/版本/四态及认证分轴，V-04/05，AC-05 | [x] 2026-11-04：store_coll_933.rs 20源×真 A2xSession：fresh（绑全源sha+冻结config）/stale-base 具名拒/空白编辑（身份重绑+生产0）/语义编辑（生产恰1）/历史回访（store hit+生产0）/restart 重装配字节一致；session 与 worker 双通道字节形态分轴如实记录；929 SYN 首编辑腿按 D-931-1 pending 分列不充 PASS |
| T-06 | work | T-05 | 跟踪闭包/无附件新树与真实负控/资源清理，V-01..08，AC-02/04/06 | [x] 2026-11-04：输入闭包逐项 git ls-files 全跟踪、无测试再读 ignored 附件；真实父门负控（auto-case release，worktree 预构建 worker）：字节篡改套件 exit 1（ctl PASS+8 篡改全 FAIL），缺 sidecar 套件 exit 2（2 ERROR），904 跟踪树前后 hash 不变（只读）；输出域审计面自身亦被触发验证（套件根内 out 被拒）。scratch 全部位于 plan-artifacts scope；target/ 产物随 worktree |
| T-07 | work | T-06，冻结提交/实际重叠H与904目录已交（不等930931932整包） | 完整职责/必要consumer并集，新材料稳定SHA先交931/932，AC-01..06 | [x] 2026-11-04：冻结提交 **44d64c4ed**（990860508 修订：清 1 unused 警告后 amend，树 clean）；职责组 ace-semantics/ace-a2x/auto-test-case/infra-fixed/infra-ace-m1 5/5 PASS，集成 case-a2r-hir-collections-904/902/926 3/3 PASS（amend 后 ace-a2x 在 44d64c4ed 重跑 PASS，其余组输入未变复用 990860508 证据）；新材料稳定 SHA=44d64c4ed 已可交 932（931 已归档，无待交接面） |
| T-08 | work | T-07 | SD/债务/独立来源/清单与健康核查，AC-07 | [x] 2026-11-04：SD-01/02/03 全部落入冻结提交；本包文件 rustfmt clean、触及 crate 编译 0 warning；D-925-1 关闭记录（KNOWN-DEBT 翻转随 merge 簿记）；来源清单/交接见 §9 收据 |
| T-09 | review | T-01..08完成，停止实现写入 | 独立逐AC/SD/oracle来源/负控复审，AC-08 | [x] 2026-11-04 final pass（收据见 §9；同 goal 链会话内复审，从工件重建裁决：diff 断言等价性逐行核对、oracle 零 ace-* import 核实、原始 results.json 守恒 20/4/20 抽验、44d64c4ed 重跑 sema 8/8+oracle/sweep 4/4、补充 V-08 剩余轴父门证据见 run_parent_negatives2.py 输出） |
| T-10 | merge | T-09 pass及后续实施/合入授权 | canonical沉淀/交付/归档/所有输出目录清理，AC-09 | [x] 2026-11-04：merge pass（收据见 §9 merge 行；ff-only 落地 44d64c4ed、canonical 三件 blob 已核、主检出 ace-semantics 冒烟 PASS、D-925-1 台账关闭、归档+清树+plan-artifacts 全删） |

下一入口T-01独立来源盘点，不等待931/932整包；公共H须与实际930生成交付/来源边界合同核窗，主体独立但不预占公共test-groups/infra窗口。本轮不授权work。

## 9. 复审记录

2026-11-04 stage:merge / plan_id:PLAN-933 / plan_revision:r2 / outcome:pass / checkpoint:cleaned / 清理:wt-guard 双树 clean→worktree lang-933/auto-lang+auto-down 移除、分支 plan-933-dev 删除、组目录删除；plan-artifacts/plan-933 整目录删除（证据已折叠入本文件 §7/§8/§9 与任务行；无具名持有），登记表 .retention/policy.json 记 merge_closeouts+0 持有 / 路径对账:本计划全部输出=worktree target/（随树亡）、plan-artifacts/plan-933（已删）、主检出 target/test-groups 与 %TEMP%/tmcase-*（运行器常态瞬态）；无其他盘符/散落目录 / next:done

2026-11-04 stage:goal / plan_id:PLAN-933 / plan_revision:r2 / round:3 / from:merge / to:done / status:archived / outcome:pass / worktree:(removed,group lang-933 deleted) / commit:44d64c4ed(delivery)+5bda8b6e1(archive bookkeeping) / next:done

2026-11-04 stage:merge / plan_id:PLAN-933 / plan_revision:r2 / outcome:pass / delivery_commit:44d64c4ed（ff-only，无 merge commit；master 9f38bbf69→44d64c4ed，rebase no-op 无哈希改写）/ worktree:D:/autostack/.wt/lang-933/auto-lang（guard clean 后移除）/ canonical:semantic-contract.md blob 5a5c779b、a2r-hir-certification.md blob 76111aa6、ace-source-only-replay.md blob a3b867ab（全部在 44d64c4ed，sha256 21c83f15…/d90728cc… 抽验）/ merge门:冻结并集（职责 5/5+集成 3/3，T-07 收据，amend 触面 store_coll_933.rs 由 44d64c4ed 上 ace-a2x 重跑覆盖）+主检出落地冒烟 cargo t ace-semantics PASS（run-gfmo7tlz）/ 台账:D-925-1 关闭（本计划）/ 落地产物观察:本交付仅测试+spec 文档，无后端二进制/daemon/web bundle 消费面，无 stale 生产构件 / next:archive+cleanup

2026-11-04 stage:review / review_scope:final / reviewer:同 goal 链会话（auto-plan-goal 授权链内；非独立上下文——以工件重建裁决：冻结 diff 逐行核对、原始 results.json 复算、复审提交上重跑关键判面）/ plan_id:PLAN-933 / plan_revision:r2 / outcome:**pass** / worktree:D:/autostack/.wt/lang-933/auto-lang / branch:plan-933-dev / reviewed_commit:44d64c4ed（树 clean）/ base:9f38bbf69 / deps:auto-down detached@7c4d5cb（只读）/ spec_inputs:semantic-contract.md@44d64c4ed（SD-01 节含全部冻结值，与 D2–D8 原文/生产常量三方核对一致）、a2r-hir-certification.md@44d64c4ed（SD-02）、ace-source-only-replay.md@44d64c4ed（SD-03 新增）/ acceptance:AC-01..07 全 pass（AC-01/02 断言体与 master 逐行等价仅换路径+helper 抽取，V-02 11 突变全拒；AC-03/04 oracle 零 ace-* import+公开向量自检+20/20 逐字节+typed+V-07 拒绝腿，904 目录零改动核实；AC-05 sweep 四态+生产计数门+929 pending 分列，跨族 profile 装载即拒；AC-06 新树重放+闭包全跟踪+负控输入根哈希前后不变；AC-07 SD 三件入冻结提交、D-925-1 关闭待 merge 簿记、932 交接 SHA=44d64c4ed）/ 复审补充证据：run_parent_negatives2.py——版本近邻 FAIL、重复 path FAIL、大小写碰撞 FAIL、错 profile 合同装载拒（exit 2 具名）、错钉 worker sha=infra ERROR（正钉 ctl PASS exit 0）；守恒：904=20/902=4/926=20 checks 全 PASS（原始 results.json 抽验 L01 manifest sha 与冻结表一致）；44d64c4ed 重跑 sema_boundary 8/8、coll_oracle+store_sweep 4/4、ace-a2x 组 PASS / pending_closeout_gates:merge 冻结并集复跑、D-925-1 台账翻转、plan-artifacts 清理 / findings:无阻断项；非阻断观察——a2c_coll_cert.rs 存量 fmt 漂移（本包未触碰，维持原样）/ next:merge

2026-11-04 stage:goal / plan_id:PLAN-933 / plan_revision:r2 / round:2 / from:review / to:merge / status:reviewed / outcome:pass / worktree:D:/autostack/.wt/lang-933/auto-lang / commit:44d64c4ed / next:merge

2026-11-04 stage:goal / plan_id:PLAN-933 / plan_revision:r2 / round:1 / from:work / to:review / status:execution_done / outcome:pass / worktree:D:/autostack/.wt/lang-933/auto-lang / commit:44d64c4ed / next:review

2026-11-04 stage:work / plan_id:PLAN-933 / plan_revision:r2 / outcome:pass / worktree:D:/autostack/.wt/lang-933/auto-lang / branch:plan-933-dev / base:9f38bbf69（master）/ code:990860508→**44d64c4ed**（冻结，树 clean）/ deps:auto-down 组内兄弟 detached@7c4d5cb（只读，未改）/ tasks:T-01..T-08 全 [x] / evidence:（AC-01/02）新树 sema_boundary 7run/2红→8/8绿，V-02 突变负控 11 突变全拒；（AC-03/04）oracle 20/20 逐字节独立重建+typed 等价+V-07 全拒绝腿，931 已入库 manifest 与独立推导零失配（前提漂移按 §5.2/§5.4 处置，未覆盖 931 值）；（AC-05）store 四态 sweep 20 源绿+stale-base 具名拒+生产计数门（空白编辑 0/语义编辑 1/回访 0），929 SYN 腿 pending 分列；（AC-06）真实父门负控 ctl PASS+8 篡改 FAIL+2 缺件 ERROR+输入根只读+输出域审计触发；闭包全跟踪；（AC-07）SD-01/02/03 入 44d64c4ed，fmt clean，0 warning，D-925-1 关闭待 merge 簿记 / 职责组 5/5+集成 3/3 PASS（见 T-07 行的提交绑定说明）/ blockers:无合同内阻塞；merge 后续门禁：冻结并集复跑归 merge / next:review

2026-11-04 stage:new / revision:2 / outcome:pass（规划合同静态复核，不是实现复审）/ next:work（后续实施指令及各任务实际写窗就绪）。修改见[复核报告](reports/ace-931-933-plan-review-and-m1-distance-2026-11-04.md)；保全部T/AC编号、总步骤及0进度，r1记录保留为历史；新增V与收口前置未执行，旧调查/历史运行不能为r2新增断言背书。

2026-11-04 stage:new / revision:1 / outcome:pass（规划静态核对）/ next:work（后续明确实施指令）。未运行新树/测试/认证，任务0；D-925-1与904缺件仍open，不以本文新增文件或旧主检出绿关闭。

## 10. 待澄清事项

无改语言或放宽expected的授权。T-02若无法从独立跟踪资料恢复某字段，必须列明原合同/实际差异并needs_replan；不从actual填金样、不删比较项或把来源未知写成确定值。发现真代码红转实际owner，源码行为oracle保持，未运行/缺工具/基础设施故障不计绿。
