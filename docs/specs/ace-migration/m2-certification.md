# ACE M2 认证（A阶段）——规范与证据边界

> 适用范围：M2-A 阶段出口（M2A-08）的认证口径、分母账本与组合证据。B 阶段
> （W1/W2/W3/OS 波与全产品、安装/退役）在 [PLAN-985](../../plans/985-ace-m2-certification.md)
> 及 986/987 按原合同收口；本文件只沉淀 A 阶段**已验证**事实，未交付面保持 B-pending，
> 不以计划代现状。来源基线：PLAN-985 r29（work f93508199＋ecc282df1，独立phase复审
> pass；M2A-08 正式收据与 delivery SHA 见该计划 §5/§9）。

## 1. 阶段口径（WAVES/EVENT-SPLIT，2026-11-16 用户裁定链）

- M2 拆 A/B 逐切片交付：A＝普通语言/新库/非图形 FFI/SDK 原范围＋MIN-A 三件
  （MIN-A-UI＝980/NATIVE、MIN-A-AGE＝981 SOURCE/SESSION/LANGUAGE/PROOF 的 A 部分、
  MIN-A-OS＝OS064 HOST/PROOF 的 A 部分）＋RG-A01/02/06；完整 UI/Widgets/图形三波、
  OS 全量、安装/退役归 B，逐行保 owner/关闭 oracle，不删分母。
- 985 是消费者/认证者：正式来源＝**已合入 master＋已复审收据**（repo/SHA、
  源闭包 hash、Schema/SDK 身份、独立 oracle、producer→consumer）；工作树 WIP、
  整包 executing 不充接口缺失，也不以整包未勾拒收已交切片。
- #6i 冻结身份：980 WEB 语法扩展造成的 grammar-id 漂移由 981 事件 A 切片关闭
  （冻结测试 `plan981_grammar_scope` 新基线绿）；985 拒过期来源，旧身份按 MISS 规则失效。
- R69-2 替代口径（已裁定接受）：带参/带载荷事件派发在 A 以**一律不可执行的具名拒**
  （fail-closed＋零状态/UI 效果）兜底；错 arity/type、重复 binder、未绑定名的逐项
  诊断归 B-W1（`D-981-EVENT-A-REMAINDER`）。A 业务引入带参时按 EVENT-SPLIT 提升规则
  只提升所用最小子面。

## 2. A 分母与义务账本（autolang.ab-obligations/1）

- 载体：`crates/auto-lang/test/cases/ace-m2/m2-cert-985/ab-obligations.tsv`
  （162 父行＝959 冻结 L/S/A/B/C/D/E/F/K＋Design56 FFI/LIB 增补；每行
  successor×phase×disposition×evidence_state×evidence_ref）。
- 校验：`cargo test -p auto-lang-core --lib obligation_ledger`——缺行/重复/错
  source/非收据路径/pending 虚报（证据膨胀）必红；`landed` 行必须锚定
  `docs/specs/` 或 `docs/plans/archive/` 实文件且 sha16 前缀匹配当前内容
  （陈旧/伪造收据红）；收据漂移随 master 沉淀重钉，不以旧 pass 冒当前来源。
- 词表（r29 扩展）：disposition 含 `user-deferred`，仅允许具名 Godot 目标行
  （B-04/B-06，M2-TARGET-SCOPE-20261116 用户裁定；allowlist 外＝红），
  且必须保持 pending（deferred 永不记 PASS/NA）。
- 已裁定归 B 的 A 行（映射理由记 PLAN-985 §5 r26/r29）：L-45（NavCall，A-min
  条件位空→W1 起）、S-20/S-21（media/terminal，产品宿主域 successor＝980/984/OS064，
  A 见证零消费→波次）、S-25（c 面，route C：GAP-982-CONFIG-ACE→B）。
- 准入四族负控（旧 binary/陈旧身份、emit 冒 run、假 worker PASS、timeout 冒负例）
  绑定 `m2-cover-985` 语料与 av_protocol 具名控件；父侧独立裁判，worker 不见期望。

## 3. 组合认证（MIX-01..07 A 腿）

- **MIX-07（同一工程组合，A 实跑）**：`ace-abi conformance f0_t16_mix07` 两例——
  1043 平台文件选择（suffix 装配 TargetOs＝Windows 选 `.rs.at` 实现）→配置文件
  真机读取＋钉版 serde_json=1.0.151 解析（use.rs 公开 API，真 cargo 单元机器码，
  独立 oracle＝进程内另一份 serde 副本）→被证等值流入 use.c SQLite（钉版 3.46.0
  融合库 cl 真码）写入/聚合回读→双车道协作取消（Cancelled 分类独立，取消后项目
  coherent）；V-08 负控三具名（删实现文件＝`load.no-provider-impl` 红、坏配置
  文档＝解析失败值 -1＋恢复、缺路径＝-8）。单例 ≤10s 门。装配道支持 dep 标注
  走真依赖 cargo 构建（dep-free 原路不变）。owned/读帧轴绑定 `out_frame_1064`
  （AV OutBytes）已交证据；仓外安装/退役复核归 986/987。
- MIX-01..06 A 腿绑定 owner 正式证据（不重跑未变输入）：MIX-01→960/962/963/964
  归档套件＋m2-cover owned-relay；MIX-02→966＋978（t14 精确失效）；MIX-03→968＋
  974＋983/A（PORT/PROOF 真 TCP），完整 UI×HTTP×Web 产品组合＝B；MIX-04→961＋
  975＋976＋967 六路径；MIX-05→981 PROOF-A 组合包，direct/RQ 两视图＝B W1+；
  MIX-06→984/A B-EMBED＋OS064/A MIN-A-OS，真实 OS 终验＝B。

## 4. M2A-01..08 门级（A 出口现值）

| 门 | 现值（收据回链见 PLAN-985 §5 r29） |
|---|---|
| M2A-01 分母/矩阵 | 959 冻结＋162 行重冻＋ledger 8/8 |
| M2A-02 语言 | 962-970/972 归档＋976/977-PY/979/980-A/981 MIN-A-AGE 接收 |
| M2A-03 标准库 | 947/948/972＋973/974/975→1064/1065 归档（S-07..S-18 landed） |
| M2A-04 FFI/装配 | 966/967/968＋File(1065)＋978＋T-09 审计＋MIX-07 实跑 |
| M2A-05 RG-A01..06 | 971/A＋RG-A01/02/06＋MIN-A-UI/AGE/OS 三件 |
| M2A-06 T-A01..A03 | 闭环（T-A01 矩阵/T-A02 cover/T-A03 重冻＋并集） |
| M2A-07 | 966/970/978 归档 |
| **M2A-08** | **成立**（A批 phase fold 落主；正式收据＋delivery SHA 见 PLAN-985 §9） |

A-required 无 unknown/NOT_RUN；Linux 全部实跑认证归 1063（NOT_RUN 不充 PASS）。

## 5. B-pending（具名，不因 A 出口消失）

977/B（REVERSE 等）、980 B 批（B-AV/W1 起、WEB/TAURI/RUST 产品）、981-B
（W1/W3 余量、EVENT-B 三件、`D-981-PROOF-AV-SCENE` 等）、982-984 B、OS064 B 波
（A01-28/ASSEMBLY）、1066-B（载体迁移/250 编译）、1067/1068（AC 补齐/三端一致）、
986/987（安装/退役）、1063（Linux 实跑）。矩阵 B 行＋user-deferred 两行＋
KNOWN-DEBT 台账为索引，关闭以各自原 owner 收据为准。

## 6. 来源

[PLAN-985](../../plans/985-ace-m2-certification.md)（r29 A批 work/review/fold
收据链）、[Design55](../../design/55-ace-m2-complete-language-and-product-replacement.md)、
各 owner 归档计划与 `docs/specs/ace-migration/m2-*.md` 已沉淀规范。
