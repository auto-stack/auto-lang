# generated-ui-distribution

> **Status**: active（PLAN-792 SD-01）
> 范围：生成式 Rust/Iced UI 工程的显式可搬动交付合同——交付布局、消费根
> patch、应用独立锁、资源 allowlist 与 producer 身份；G/H 搬仓硬门的
> 首个可执行发行面

## 交付模式与开发模式

- **开发模式**（现行，PLAN-792 零变化）：框架仓内项目 = 共享
  `examples/rust-workspace/` + workspace 继承清单（P584-D1 语义保留）；
  仓外项目 = 项目仓内 `rust-workspace/`。
- **交付模式**（`auto gen --delivery-root <dir>`，有效前端须为 rust）：
  显式可搬动受管 bundle。其 manifest **不隐式搜索 D:/autostack、不落
  编译期 `CARGO_MANIFEST_DIR` 兜底**；环境覆盖（AUTO_RUST_WORKSPACE/
  AUTO_LANG_CRATE/CARGO_TARGET_DIR 等）不进入生成物。定位待拷贝源的
  producer 根解析序 = AUTO_DELIVERY_SOURCE_ROOT → AUTO_LANG_ROOT →
  邻接安装 SDK → CWD 仓库漫步 → 编译期上溯（仅生成期事实，记录于 delivery-manifest）。
- 两模式共享同一编译相位（auto-ui-tools `compile_rust_ui_app`）：main.rs
  逐字节同源；sidecar/`__InitLoaded` 改写/exe_name 契约同源。

## 交付 bundle 布局（镜像生产仓根布局）

```
<delivery_root>/
  Cargo.toml            # 受管 workspace 根：members=[app+crates/*]、
                        # default-members=[app]、[workspace.dependencies]
                        # 表逐字节副本、[patch.crates-io] iced_widget（消费根）
  Cargo.lock            # 交付阶段 fresh 解析一次后冻结；消费侧 build --locked
  app/                  # 生成应用（自包含清单：相对 path 依赖 + drift pins）
  crates/<交付闭包>      # 原始副本经收据校验，再按声明投影受影响清单
  crates/patches/iced_widget/  # 本地 patch 随包发行（PLAN-043 走查）
  stdlib/、docs/schema/ # 仓库根级编译期输入（include_str! 越界面）
  assets/               # 资源 allowlist（src/front 非 .at 文件 + SHA-256）
  verify.py             # 随包受管资源校验入口（消费清单 hash，独立可执行）
  delivery-manifest.json # producer 身份/闭包规模/资源/pin/投影修剪记录
```

- **镜像布局的硬理由**：auto-lang `build.rs` 以 `../../crates` 与根
  `Cargo.lock` 为编译期输入（缺失即 unwrap 红）；非镜像布局在受管 build
  阶段确定性失败。
- **交付闭包**：当前 24 个受管 crate、patch 与根级输入以 UI tools 的 `DELIVERY_CRATES`、`DELIVERY_PATCHES` 和根输入声明为准；语言包归语言 owner，UI 包归实际选中的 UI owner。生成应用直接依赖 SDK 与 UI runtime；闭包中保留的 auto-lang 不表示应用仍直接依赖旧门面。根 workspace、锁、patch、stdlib/schema 等实际编译输入随包核验。

## 身份与失败合同

- **单一 package 身份**：iced_widget 仅以 patch 单源出现在交付根
  `[patch.crates-io]`（不写在依赖包内）；bundle 内 package.name 全局唯一
  （同包 source 重复显式拒绝）；应用包名不得与 vendored 撞名。
- **投影修剪**（对受管 auto-lang、core、runtime 清单按实际结构处理）：剥交付闭包外
  optional path 依赖（autodown-core 跨仓 / auto-test-case 测试协议——
  cargo 对成员 optional path 依赖仍读 manifest，不剥则解析红）+ feature
  行剥 `dep:` 引用保留 feature 名；合法源码态、安装中间态和已投影态分别按结构准入，不因合法零编辑或部分编辑判红；缺项或异常混态判为 producer 结构漂移。
- **锁与 pin**：交付应用清单对历史漂移家族（wgpu/iced 家族，来源=生成时
  producer 根 Cargo.lock 已验证版本）精确钉扎；与基础依赖同名者（iced）
  并入基础行；多槽成员（windows/windows-core 三槽）TOML 单键不可重复声明
  ——登记为 `pins_multi_slot_lock_frozen`，由冻结锁控制。779/785 记录的
  fresh-lock wgpu-hal/windows-core 漂移为时变生态漂移（2026-10-16 实测
  fresh 解析已收敛至已提交锁版本集），本轮以"独立锁+pin+冻结重放"闭合
  该失败类，不宣称生态漂移被根治。
- **确定性失败（无静默回退）**：资源校验经**随包受管入口 `verify.py`**
  （消费 delivery-manifest.json 的资源 SHA-256；无 producer/原工程也可独立
  执行）——缺资源/等长篡改/坏 hash/未登记资源逐项判红；消费根漏 patch /
  不兼容 pin → `cargo build --locked` 确定性红（锁需更新被拒）；外来非空
  目录（无 delivery-manifest.json 标记）显式拒绝不覆盖；producer 缺闭包
  源/根级输入在**预检阶段**判红；**再交付经独立 staging 生成+全部校验
  成功后整体换装**——准备或生成失败保留旧交付字节（失败不切换任何旧
  生产产物），**不回退主检出、不复制原 target/rlib 作 oracle**。
- **staging/backup 目录属主（R2/R3/R4）**：staging 与 backup（换出残桩）都用
  本次操作独占的**唯一名**（pid+ns）+ 内置标记——staging 标记
  `.delivery-staging.marker`（创建即写，失败由守卫清理）；backup 在换出
  后立即写入**换出事务标记** `.delivery-swap.marker`（回滚时移除）。
  **任何既存目录（含带合法 delivery-manifest.json 的其他应用交付包）都
  不被认领/删除**。换出事务标记只证明「本线曾把旧根换出」，**不证明新包
  已成功提交**；清单+标记双条件是必要非充分。陈残 GC 仅在**成功替代产物
  已在交付根落位**（`delivery_root` 含 delivery-manifest.json）时回收同时
  具备换出事务标记与清单的 `.{name}.delivery-old-*` 目录——根缺失/未完成
  （待恢复唯一旧包）时 GC 为 no-op，失败重试绝不主动删除尚未被新包替代
  的旧包；成功发布后才清理已被替代的冗余残桩。无标记的模式名目录只报告
  不删除；历史固定名（无 `-` 后缀）不再被使用。生成器只删除能证明属于
  本次操作/本线事务且已有成功替代产物的路径。
- **sidecar 边界**：交付模式显式拒绝 rust_sidecar 的外部 path 依赖——
  绝对路径（任意根，含 Windows 根相对 `/x` 与盘符/verbatim 前缀：此类
  形态在 `path::join` 语义下丢弃解析基目录）与越出交付根的相对 `..`；
  registry 版本依赖（`name:version`）与项目内 .rs 模块照常。全部 bundle
  Cargo.toml 的 path 值经 **TOML 语义解析遍历**（解析器归一空格/单双
  引号/内联表写法——不做固定文本模板匹配；无法解析即受管判红
  fail-closed）后应用同一规则：无 RootDir/Prefix 分量、且按 manifest
  目录词法解析后落在交付根内（覆盖任何来源的越界源，含 sidecar 原样
  写入的 `{path='…'}` 形态）。
- **可追溯**：delivery-manifest.json 记录 producer 源根/git HEAD/dirty/
  ui_gen 指纹、producer 根 Cargo.lock 的 SHA-256（根锁为本地实际输入、
  整体被 .gitignore 忽略——以内容 hash 绑定身份而非跟踪状态）、逐 crate
  文件数字节数与目录 digest（blake3——producer 身份）、资源 SHA-256
  （消费侧可独立校验）、pin 表、投影修剪明细与闭包规模披露。
- **消费合同**：交付根即独立 workspace 根（自有应用 Cargo.lock）；重定位
  （A→B，不带 target/）后 `cargo build --locked` 须过；metadata 全部本地
  path source 在交付根内；运行期本地资产按 CWD 相对解析（消费侧自 assets/
  运行，开发模式对应 src/front）。生成器不自动跑 cargo——锁冻结与消费
  构建是交付管线/消费侧职责。

## 门禁

- 生成器内容门（不跑 cargo）：`cargo test -p auto-man --lib delivery -- --test-threads=1`
- 真实交付门（V01 fresh 解析+真实 build / V02 重定位 locked 重放 /
  V03 负控 / V04 开发模式保持）：`python test/fixtures/modularity/generated-ui-consumer/driver.py`

## M815-A:独立窄 tool/协议来源边界(PLAN-815 SD-06-A)

A 阶段新增独立 tool/protocol 及 data-owner codec 来源
(crates/auto-ui-tools、crates/auto-tool-protocol、auto-project-model wire
feature + auto-ui-tool 二进制),**不改变 `rust_ui.rs` 的 792 DELIVERY 表或现行
UI 生成发行资源**。发布收据绑定已审代码、协议/API 版本、真实实现/资源输入指纹
(`docs/plans/attachments/815/m815-a-release.md`)、精确 Spec 补丁与依赖来源;
delivery commit 须为 master 祖先且 consumer 实际解析验证。阶段 review pass 本身
不是发布收据。

阶段不归档 Plan、不清理唯一树。冻结门及实际发布/Spec/ledger 来源收据齐备后,
才可放行 816 消费 A 能力;完整 UI 能力继续依 M815-B(生成主体迁移时 DELIVERY
表增补 auto-tool-protocol/auto-ui-tools)。

## M816-A:工具消费方式与发行闭包不变(PLAN-816 SD-05-A)

自 M816-A 起,`auto-ui-tool` 及后续 I-TOOL 工具宿主只能经**真实子进程 +
auto-tool-protocol 帧**消费:客户端 `crates/auto-tool-client`(经
auto-tool-execution owned 设施 spawn,取消/超时/reader 收尾由该设施统一承担);
消费闭包**不得静态链接工具实现**——由
`test/fixtures/modularity/tool-client-consumer/verify-deps.sh` 钉住(无
auto-ui-tools/auto-man/auto-lang/auto-vm/auto-gen)。M816-A **不改变任何发行/
安装闭包**:792 `DELIVERY 表`与生成 UI 分发清单维持 M815-A 现状——客户端是
workspace 库,供后续薄 CLI 组合(M816-B T-05+)消费,不作为独立发行件;独立安装
profile 的闭包增补(DELIVERY_CRATES 增补等)属 M816-B,定稿门为 M814-A 后的
整链 probe。

## M814-B:canonical 双直依赖与投影随包图(PLAN-814 SD-08)

M814-B（PLAN-814 T-05/T-08）在 M815-A/M816-A 章节基础上增补：

- 生成工程（app/成员/workspace 三 manifest 模板）canonical **双直
  依赖**：`auto-ui-runtime`（含 ui-iced）+ `auto-lang-core`；feature
  转发随 runtime。旧 `auto-lang` 组合依赖形态经映射表对照在档。
- 投影 spec 随包图重构：`PROJECTION_AUTO_LANG` 收窄（test-case 两项，
  autodown 边迁出）；新增 `PROJECTION_AUTO_UI_RUNTIME`（跨仓
  autodown 边剥除 owner=runtime；receipt 增 `projection_edits_
  auto_ui_runtime` 台账）。
- 792 driver V04 断言随 canonical；**24/24 EXIT=0 为 B 基线**
  （V01 fresh 真构建/V02 重定位 locked/V03a..j 负控/V04 dev 同源）。
- 交付闭包 DELIVERY_CRATES 含 auto-ui-runtime；ui_to_language=0
  不变量入 777 基线（runtime+compiler 源根零 `auto_lang::` 文本
  命中）。

## M815-B:tool/协议/source/embed/producer 闭包与身份语义(PLAN-815 SD-06)

新 tool/协议/source/embed/producer 闭包(DELIVERY_CRATES 26+app+根级输入
stdlib//docs/schema/aura.at)与适用失败矩阵(构建失败具名/产物 oracle/
取消树终止/中断恢复按 profile 精确;资源负控四态/漏 patch/坏 pin/外来
目录/失败保留恢复=792 driver 33/33 在案)。消费交接以**冻结源 SHA+二进制
SHA256+支持指纹**为身份,不以源码路径冒充二进制身份。发行形态=792 源码包
workspace(工具由包内 `cargo build --locked -p auto-ui-tools` 可复现建出,
fresh/重定位三形态实证);**确定性身份锚=冻结源 SHA+delivery-manifest
(逐文件 rel/len/blake3 汇总的目录 digest+producer.git_head/
ui_gen_fingerprint,再生成可复验)**;二进制 SHA256 为带 provenance 的
参考产物指纹(release 形态)——无 reproducible-builds 配置,不以某次构建
哈希承诺逐字节复现,也不以互异的 debug 哈希充当身份。交付闭包编译期/
embed 资源完整盘点(Inter 三字重+OFL/壁纸/桌面壳/gallery/wm/drawlist/
shadcn/根级输入):
`docs/plans/attachments/815/t07-acceptance/batch3/resource-audit-closure.md`。（PLAN-815 M815-B，T-09 R2 pass @ 生产冻结位 0f2858e35；终稿正文与证据：attachments/815/handoff.md §规范增量）

## M816-B:profile/发现/SDK 闭包终态(PLAN-816 SD-05,T-04/T-05/T-07)

- **默认完整 profile**:含 `ui-extra` 静态面(五操作外 UI 能力:ui_gen 注册表/
  蓝图/watch SFC 再生/gen 等)——既有 UI 支持面不缩。**纯 Core profile**
  (`--no-default-features --features core`)显式区分:无 UI 具名失败,不冒充
  完整交付。工具发现序见 tool-execution(M816-B):env→workspace→邻接安装,
  缺件/版本/能力具名拒绝,无静态大库偷偷 fallback。
- **安装 SDK 闭包**:rust_workspace 安装邻接 sdk 解析步(env→`<exe>/sdk`→
  CWD 漫步→编译期根);`gen --delivery-root` 默认 producer.source_root=安装
  sdk(零实施树依赖,显式 env 对照同源正确);安装形态 sdk=crates 全集+lock+
  根级输入,外部项目可 check/locked 构建。
- **当期身份**:冻结源 SHA+二进制 SHA256(带 provenance 的参考产物指纹);
  Unix 执行源变化后二进制身份随之刷新,不以旧快照 hash 充当新发行身份
  (确定性身份锚仍为 792 delivery-manifest 语义,M815-B 节保持)。
- DELIVERY_CRATES 表增补(CLI/service/protocol/tool)属发行收口,随 R07 搬仓
  单独定稿(历史注记保持);正式独立安装包资源布局同。
（PLAN-816 M816-B;终稿证据:attachments/816/handoff.md）

## PLAN-831 多源 producer（SD-04，as-built）

受管 UI 五包实际源可由独立 UI 根提供：delivery_crate_source 按显式 UI 根/组内 UI 源取得 owner 源码并投影为 bundle 内相对依赖；语言 SDK 源及其 root inputs 具名登记（探索序=primary 语言根优先再查 UI 根）。canonical auto-ui-runtime + auto-lang-core 双直依赖、vendored patch、版本 pin、多槽锁、资源哈希与陌生目录/旧字节保护不变；交付身份记录各 producer 根及固定 revision/内容 hash，不能只标语言根 SHA 而未绑定 UI 源。缺环境记录不阻塞（r3：jet/ark 真实构建转后续门控），不支持删除/allowlist 扩大。

## PLAN-832 Man 与三仓发行（SD-05，as-built）

`auto` 与 auto-ui-tool 分属 Man/UI owner；792 driver 分别定位两仓实际二进制，producer 仍按固定语言/UI 源取件，不借主 target。Man 的 Core/full 安装复用受管 staging、hash/profile/source 校验、交换/备份和恢复语义，另补 CLI 所需的独立 workspace/锁与双 profile 布局。独立 clone、重定位、升级失败、等长篡改、缺 tool、取消和恢复均有真实正反控；详见 [man-three-repository-delivery](man-three-repository-delivery.md)。jet/ark 真实构建经 2026-10-25 用户正式裁定转 v0.7 环境门控渠道，未实测不冒称通过，其他发行与恢复验收不变。


## PLAN-841 UI 领域模型与三仓来源（SD-04）

受管 bundle 现含 25 个源码 crate；UI owned AST/codec 随 auto-ui-dialect，
compiler/runtime/tools 均消费 `UiCode`。生成 import 只指向 canonical
auto-ui-runtime/auto-lang-core/auto-ui-dialect，不由 Core AST 兼容臂提供
第二来源。Man 的 `scripts/man-distribution.py` 固定 UI revision；792
driver 的收据同时记录 lang/UI/down 实际 HEAD 与随包工具二进制 SHA256。

PLAN-841 验收使用真实外部工程完成 fresh resolve、locked build、A→B
重定位、metadata/path/source 身份、资源与失败负控，并覆盖四生成器、
Iced 窗口、VM 服务/取消和工具协议，39/39 通过。jet/ark 构建仍只按既有
环境门控披露，不把生成成功冒称宿主工具链已安装。

## PLAN-842：多来源冻结与版本退役

`AUTO_DELIVERY_SOURCE_ROOT` 优先于 `AUTO_LANG_ROOT`，显式语言/UI 根权威；安装 SDK 的实际目录身份而非路径大小写决定受管分类。默认邻接安装存在但清单或 SDK 不完整时拒绝，不借外部开发源。合法 Full SDK 作为 producer 支持源码态、`man-install/1` 合法中间态及已投影态；TOML 表作用域识别投影，异常混态拒绝。

语言和 UI 来自不同受管安装时各自冻结 `man-install/1` 收据，检查所选 owner/path、全受管文件集合及内容；每个复制到 staging 的受管 crate/patch 按相应收据核原始字节，根输入及锁按语言收据核验，swap 前复核各收据。可恢复的旧包不因失败重试删除。清单 SHA 为一致性校验而非签名或来源认证。普通开发源的本地 Cargo 路径图核目标包身份、自身版本与声明约束、workspace 继承及普通/构建依赖环；合法 dev 环和 package 重命名保留，该预检不要求 registry 联网。

生成应用显式装配 SDK hook 和 UI runtime，不直接依赖旧 auto-lang；源根、各 UI/语言 revision、指纹和锁身份应指向实际选中来源。`generated-ui-delivery/1` 与 `man-install/1` 保持可用，未知 schema/profile、坏 producer 在改动旧产物前拒绝。真实 fresh/locked、资源校验和失败恢复范围见来源：[PLAN-842](../../plans/archive/842-legacy-compat-and-distribution-retirement.md)
