---
plan_id: PLAN-988
status: execution_done
feature_name: Reader原生多选与动态字节桥接
plan_revision: 1
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-06
current_step: 4
total_steps: 6
supersedes_spec_components: [docs/specs/auto-lang/ui/design/dialog-dnd-host-capabilities.md, docs/specs/auto-lang/vm/native-fs.md, docs/specs/runtime/native-vm-app-validation.md]
new_spec_components: []
touched_goals: []
---

# PLAN-988：Reader原生多选与动态字节桥接

## 0. 变更摘要

READER-009 的最小框架前置。补公开原生多文件选择链，修复动态整数列表传入字节文件写入的边界。保留既有单选、取消、父窗与数据身份语义；不扩展阅读器功能或重写 VM。

## 1. 目标

Reader 可通过真实原生多选取得多个路径，并将授权文件/传输解码产生的字节完整落入自有暂存目录；VM 与生成 Rust 的 byte 值一致。独立复审、落地后回到 READER-009 完成真实产品验收。本计划的 provider 假实现单测不替代 009 的真实 picker/UI 证据。

## 2. 架构方案

公开语言接口 → VM native/provider → auto-ui host_adapters/dialog.rs → rfd.pick_files；返回有边界的 owned 多路径列表或 JSON 编码结果（T-00 按现有 ABI 冻结），取消为空集合。缺 provider 必须具名拒绝，不虚构单选多次作为多选。保留 open/save 的既有行为与旧宿主兼容。

字节：列表构造/函数传递/JSON 解码 → 当前 VM 值表示 → native 参数转换 → File.write_bytes；修正确认过的表示不兼容，不以应用层字符串编码规避。若改动通用 Vec<i32> 桥，覆盖实际同桥消费者和非法元素；完整参数验证应发生在写文件之前，不能用错误值归零掩盖损坏。已有范围外整数转 u8 行为先核合同，不能顺手改变不相关兼容性。

仓库写面仅 auto-lang 和 auto-ui。auto-reader 为消费者验证只读；auto-down 等编译依赖兄弟检出只读，不修改。唯一组 D:/autostack/.wt/lang-988/{auto-lang,auto-ui,auto-down}，主语言分支 plan-988-dev，UI 分支 codex/reader-009-prerequisites；无链接。按依赖实际需要补只读兄弟。材料统一 D:/autostack/plan-artifacts/plan-988/；主检出他人文档 WIP 和旧 worktree 不接管。

## 3. 技术栈

现有 Rust VM native/ffi、语言公开声明与生成映射、DialogHostProvider、rfd；沿既有宿主安装和 native 元数据注册，不增加第二套对话框或字节服务。

## 4. 需求分析与背景调查

2026-11-06 用户明确选择“授权最小跨仓修复并继续（推荐）”，具体授权为 auto-lang/auto-ui 独立树内公开原生多选接口及动态字节传输/写入保真修复、对应验证，再继续009；包括本前置完整 work/review/merge。无新增预算。读取技能 auto-plan-new/work 与 auto-lang AGENTS；对应当前 Specs 为 native-fs 与 dialog-dnd-host-capabilities。后者目前仅 open/save（2927/2928），不含多选。

核查语言基线 a63a20b4e69fc39f448b509ec61c046143db0960，UI 22a925c72d7620b929c947ff2bbddead13fda783。当前 DialogHostProvider::open/save 仅 String，UI os_open 使用 pick_file。Byte shim 在 crates/auto-lang-core/src/vm/ffi/stdlib/file_io.rs::shim_file_write_bytes，接 Vec<i32>。native.rs 有 i32/Value 两种 ListData helper；具体失败归属须 T-00 定位，不据相邻 helper 直接改。

READER-009 已实跑最小源：List<int>.new([]) push 0/127/128/255 后 file.write_bytes exit1，Invalid list ID: 4000001；read_bytes→write_bytes 同四字节控制 exit0、SHA256 89273d2f70b93285bb7ddb4bcee86a5347ca7159352e3cbdd20c23e9d1e507d3。旧 installed auto SHA256 54124fd43afc76f3cf41268a57c6cb3defd6b008433390a5f0992166a12871bd；必须在新源码构建重现后判根因。原材料在009 scope，本计划保存最小源与新构建结果到988 scope，不搬动/删除009原件。

## 5. 详细设计

T-00 冻结 native 分配/声明/解析/生成消费实际路径，API 名称建议 dialog_open_files，不得占用已有 ID。沿 provider 当前锁外调用、实例覆盖、进程默认快照规则。旧 provider 未实现新方法时明确 unsupported，不能静默返回单个文件冒支持。父窗策略沿既有 VM parented 与生成路径合同；取消为空列表，不产生空路径成员。JSON 返回如选用必须正确转义中文、反斜杠、引号。

Byte T-00 确认宏生成 FFI 的 FromVm 转换与 File.* 两种注册入口；检查字面量、List.new/push、读回列表、函数参数/返回、JSON 数字列表的实际表示。以最小受影响桥修正，合法0..255精确保持。错误元素/无效句柄应受控失败并保持现有目标字节；不得改写 JSON/数组所有语义。资源限额沿009出版物30MiB、封面8MiB，988不另造更小限制；性能/完整HTTP容量由009 T-01消费冻结。

### 规范增量

| delta_id | action | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify | auto-lang docs/specs/auto-lang/ui/design/dialog-dnd-host-capabilities.md | 仅单选保存 → 多选接口/返回/取消/缺席与旧宿主兼容合同 | 公共接口可消费 | AC-01/03 |
| SD-02 | modify | auto-lang docs/specs/auto-lang/vm/native-fs.md | 未记byte表示 → 合法动态列表写入保真、非法值写前拒绝与兼容边界 | 数据不损坏 | AC-02/03 |
| SD-03 | modify | auto-ui docs/specs/runtime/native-vm-app-validation.md | 增补真实多选宿主消费/验收入口与限制 | 框架与产品证据分账 | AC-01/03 |

## 6. 测试设计

| Case | owner_stage / AC | 输入/路径 | 独立oracle与反例 |
|---|---|---|---|
| P-01 | work AC-01 | provider多路径/中文含引号路径/取消/缺席/旧provider/实例与默认槽；真实源码调用 native | 顺序数量逐字相同、空集合与unsupported分开；旧open/save回归 |
| P-02 | work AC-02 | 字面量、动态push、函数参数返回、JSON整数、read_bytes控制；0/127/128/255及全256值 | Python/Rust直接文件字节与预设数组比较；非法元素/句柄错误不得覆盖预存哨兵 |
| P-03 | work AC-03 | 配套新工具运行最小.at/生成Rust，API调用字节载荷到文件 | 两执行端真实落盘hash一致，错误显式；不得只看生成代码字符串或文件存在 |
| P-04 | work AC-01/03 | 真实UI宿主 picker 多选两份隔离文件/取消/父窗恢复 | 实际选择返回两路径且关闭后宿主可操作；测试provider不当真picker。009继续完整产品UI |

固定语言组按修改选 vm、shim-metadata、a2r 及实际 HTTP 桥受影响时 http；T-00 用 cargo t --list 确认。开发先复现及命名用例；最终 review/merge 按冻结触面并集一次运行，不跑无关全仓。UI 现有 plan804_dialog_dnd_provider/plan804_os_e2e 受影响用例和新增多选 case；编译 UI 实际消费者。独立 final reviewer 在同一权威树冻结提交上复演；允许独立子代理承担该必需复审，实施时不启并行写者。真实桌面不可用时保留 P-04 pending，不假通过。

## 7. 验收标准

- AC-01 work：P-01/P-04 全过，公开原生多选贯穿声明/VM/宿主/生成消费，旧单选与取消不回归。
- AC-02 work：P-02 真文件全字节一致；非法输入先拒绝、原文件保持；有效read控制继续通过。
- AC-03 work：P-03 两端消费者通过，相关固定组/UI检查全过，SD-01～03准备且无范围外变更。
- AC-04 review：独立上下文审核全部AC与反例，精确reviewed commit，不凭旧工具证据。
- AC-05 merge：已审提交落地主线、规范读回、工具刷新或明确消费指令、guard清理归档，009能消费新能力。

## 8. 执行步骤

| ID | owner_stage | 依赖 | 动作 | 关闭证据 |
|---|---|---|---|---|
| T-00 [x] | work | 授权 | 树/依赖确认，新构建复现、ABI与表示根因定位、冻结最小设计 | 源码调用链及P-02红/读回绿；实际命令/版本 |
| T-01 [x] | work | T-00 | 公共多选声明/VM provider/UI OS/生成映射与用例 | P-01、P-04 |
| T-02 [x] | work | T-00 | 动态字节桥与永久正反回归 | P-02及受影响同桥消费者 |
| T-03 [x] | work | T-01/02 | 两端消费者验证、准备SD、工具交接记录 | P-03及冻结受影响检查 |
| T-04 [ ] | review | T-03 | 独立复审绑定各仓提交与Specs | AC-01～03 pass |
| T-05 [ ] | merge | T-04 | 顺序落地/工具消费/规范/归档清理/009交接 | AC-05 |

## 9. 复审记录

2026-11-06 stage:new | PLAN-988 r1 | outcome:pass | next:work T-00。用户跨仓授权已记录；已独占核号脚本分配988，未实施、未新工具构建。证据首轮以009最小红/绿为入口，不等同新代码回归结果。无需再次请求同范围许可。


### 2026-11-06 work：T-00闭合，字节桥与多选接线进行中

- 权威树：D:/autostack/.wt/lang-988/auto-lang，plan-988-dev，base=a137e8be6；UI同组auto-ui，codex/reader-009-prerequisites，base=22a925c7。auto-down只读58ea33f，auto-web只读bf72baa（UI真实path依赖需要），均git detached，无链接。
- T-00 [x]：新源码编译最小桥回归，原生i32 list控制及坏句柄控制通过，Value整数数组与错误元素分类失败：2 PASS/2 FAIL，cargo exit101（PowerShell包装记录exit1）。故障位置为ffi/convert.rs::VMConvertible for Vec<i32>，仅downcast ListData<i32>，不是磁盘权限。T-01 API冻结为dialog_open_files(filter)->JSON数组字符串，native9942，provider返回Result<Vec<String>,String>；旧provider默认明确unsupported，取消[]，VM parented/生成Rust沿已有unparented边界。
- 语言实现提交82d4b5d50：支持Value整数标签与i32范围检查，非整数写前拒绝；新增provider方法/VM shim、intrinsic、ID/返回类型/保留ID注册。相关核心回归命令 `cargo test --offline --locked -p auto-lang-core --lib plan988 -- --nocapture`：9 PASS/0 FAIL，exit0。包括真实源码字面数组、List.new/push、JSON数组写真实文件[0,127,128,255]；全256字节；无效成员保持既有文件；多路径Unicode/引号/反斜杠JSON往返、取消、旧provider拒绝但单选不变。
- 初次新增多选测试编译因漏import和私有trait路径失败（bridge-dialog.log），修复测试trait公开引用与codegen常量import后9项全绿；首次失败保留，不当功能验收红。
- `cargo test --offline --locked -p auto-lang-core --lib native_catalog -- --nocapture`：5 PASS/0 FAIL，exit0，native编号/名字/返回注册一致。两仓diff --check通过。编译有既有warnings，尚未做最终新增warning差异审计。
- UI实现提交518efb0c；语言Spec准备提交739fd451a。UI接线：os_open_files调用真实rfd.pick_files，复用filter/父窗；provider/生成门面与编译映射接线。`cargo check --offline --locked -p auto-ui-runtime -p auto-ui-compiler` 已完成，exit0、95秒，session=69605已终态，日志scope/ui-check.log；UI编译239 warnings为待最终基线差异审计，编译错误为0。
- SD-01/02已在语言树准备，未发布。T-01/T-02/T-03仍未完成：真picker/焦点、缺席/覆盖/源码多选、函数参数返回/HTTP消费者、生成Rust真实字节写入、完整受影响组及UI门仍需执行。T-04独立review与T-05 merge均未启动。
- 证据scope D:/autostack/plan-artifacts/plan-988/：byte-baseline.log SHA256=2bcb8a092d9826171b65ac57b42c5abb58c9e5d628defbbf6a3fd4e52f01edd9；bridge-dialog-v2.log=20af796d0783c07f64644f9fdaedbde82377d847942f2d0e0214c9da5b65e8e8；native-catalog.log=1ac3f70ab89290ad33b9cd7695905cf9e2aee75345d7e7c600d81d40a86ec91e。原009探针材料未动。
- stage: goal | plan_id: PLAN-988 | plan_revision: r1 | round: 3 | from: work | to: work | status: executing | outcome: needs_fix (work incomplete, not a final-review finding) | worktree: D:/autostack/.wt/lang-988/auto-lang | commit: 82d4b5d50 | next: work T-01/02/03。

### 2026-11-06 work：真实选择器与共享字节消费者

- 新增永久真实系统用例 `auto-ui-runtime::ui_tests::plan804_os_e2e::dialog_e2e::plan988_real_multiselect_vm_and_rust_host`，显式ignored防普通门抢桌面。当前UI树含未提交测试与SD-03；语言树含未提交扩展测试，均归988，不是他方WIP。
- UI测试构建 `cargo test --offline --locked -p auto-ui-runtime --lib plan988_real_multiselect --no-run` exit0，177秒。实际测试exe `auto-ui/target/debug/deps/auto_ui_runtime-0e9d1275a5d4351c.exe plan988_real_multiselect --ignored --nocapture --test-threads=1` exit0，1 PASS/0 FAIL/0 ignored，2.22秒。Python外层100秒超时保护未触发，TEMP/TMP在988/tmp。VM shim→真实parented provider与生成Rust使用的真实unparented host各一次多选/取消，选两份隔离文件；返回两路径/空数组、live owner与关闭后owner enabled全部通过。测试直接向自有进程IFileDialog控件输入与按按钮；是实际OS picker，非Reader产品按钮或用户物理IME证明。
- 语言 `cargo test --offline --locked -p auto-lang-core --lib plan988 -- --nocapture` 扩展到12 PASS/0 FAIL/0 ignored，exit0：新增源码函数[]int返回→参数→真实File.write_bytes；真实本地TCP loopback全部256字节逐字一致；HTTP Response凭据body写入0/128/255后非法成员拒绝且旧body不变。只证明对应桥边界，不将其视为完整HTTP产品验收。
- UI生成器追加widget handler多选映射永久用例，尚未运行。SD-03已准备，未发布。
- `cargo t --list`确认固定组后，已启动 `cargo t vm shim-metadata a2r http`；exec session=15562当前仍live，先poll该handle，不能因无新输出重启。外层日志988/affected-groups.log；driver默认输出实际为 `D:/autostack/.wt/lang-988/auto-lang/target/test-groups/run-xgf0njbe`（本轮疏漏未显式--out；终态后核归属/无链接并迁入988唯一scope，记录迁移hash；下一次必须--out）。已经发现目标并进入运行，不能记PASS。验证冻结期间不改/提交语言树以免source-state变更；UI生成器用例等该轮结束后顺序运行。
- 原始材料SHA256：real-picker.log=f26724e0654fa2a8088107e9c9a0aa7456b1249e6e9fddf33e40bf7c3098f235；consumers.log=c9d179492639b9bb69867ebac49db4a9f9fe60279003a94e0e02f1a5f53e7bb1。
- 未闭：生成Rust真实byte消费者、公开源码多选/缺席覆盖、UI generator新例与旧provider相关门、完整组结果及新增warning审计；因此保持executing，不交独立终审。
- stage: goal | plan_id: PLAN-988 | plan_revision: r1 | round: 4 | from: work | to: work | status: executing | outcome: needs_fix (work incomplete) | worktree: D:/autostack/.wt/lang-988 | commit: language739fd451a/UI518efb0c + owned test WIP | next: poll session15562 then finish T-01/02/03。

### 2026-11-06 work：T-01～03完成，冻结独立复审基线

- stage: work | plan_id: PLAN-988 | plan_revision: r1 | outcome: pass | task_ids: T-00～03 complete; T-04/05 pending | worktree_path: D:/autostack/.wt/lang-988/auto-lang + auto-ui | branch: plan-988-dev + codex/reader-009-prerequisites | base_commit: language a137e8be6 / UI 22a925c7 | code_commit: language 65dcdb025c79060a9a80dc9ec99be408eee57970 / UI b0d59400a1c302c040e07c9e4938a1c13b34d937 | dependency_revisions: auto-down 58ea33f / auto-web bf72baa (read-only detached) | blockers: none | next_responsible_role: independent final reviewer。
- 实质实现末提交 language 61d83ee4c、UI 96083340；之后两提交仅格式。通过 rustfmt canonical output 比对，格式前后逐文件等价，因此复用对应运行证据，不伪称格式后重新执行全部测试。两树已提交且无交付WIP，暂停实现写入直至review结论。主检出他方文档及UI测试目录WIP不纳入本计划，merge仍需按规则核归属。

| work AC / task | 当前关闭证据 | 结论与边界 |
|---|---|---|
| AC-01 / T-01 | 语言16个定向例中的多选JSON/取消/缺provider保留参数/旧provider明确拒绝且单选保持/真实源码解析；UI生成器新例1 PASS；旧provider6 PASS；真实系统多选例1 PASS、0 ignored（含VM parented和Rust unparented两端各选择/取消） | pass。真实picker结果与两份隔离文件身份匹配、owner enabled恢复；未冒充Reader按钮、物理IME或焦点键盘事件证明。父窗沿既有端差异。 |
| AC-02 / T-02 | 16例中typed/dynamic/JSON/函数返回参数/全256字节/无效句柄和非整数/HTTP Response body/TCP loopback，目标文件与原始字节独立比较，坏成员不覆盖哨兵 | pass。Vec<i32>仅接受范围内整数标签；合法0..255保持；Rust原有完整i64转低8位行为保持，未增加范围外跨后端等价承诺。 |
| AC-03 / T-03 | 真实生成Rust源→rustc→exe落盘[0,127,128,255]，空列表另编译执行得到零字节；VM真实源码写同四字节；固定受影响组、UI编译与消费者通过；SD-01/02/03已提交 | pass（work范围）。审后冻结并集回归由merge核证据有效性/执行，当前不声称最终合入门完成。 |

- 消费发现并修复：生成Rust原helper仅收Vec<Value>导致实际函数返回Vec<i64>调用E0308；现泛型接受typed/JSON整数列表且先验证全部元素，非法值返回-1并保原文件。原先非法成员归零属于数据损坏，不沿用。直接空数组随后出现推断歧义；两处File.write_bytes生成臂对直接[]发出Vec::<i64>::new()。永久fixture emitter位于auto-generation-core/examples/plan988_byte_emit.rs，使用生产qualify_a2r_std，无手改生成Rust。
- 真实生成消费重放命令：`cargo run --offline --locked -p auto-generation-core --example plan988_byte_emit -- D:/autostack/plan-artifacts/plan-988/rust-final`；empty模式额外参数`empty`、独立rust-empty目录。`rustc --edition=2021 <scope>/main.rs -L dependency=target/debug/deps --extern auto_lang_core=<本树最新rlib> -L native=<本树target/debug/build各out及Cargo registry windows_x86_64_msvc各lib> -o <scope>/consumer.exe` exit0；运行exe exit0；独立读回typed=[0,127,128,255]、empty=[]。首次E0308保留在rust-consumer/compile-red.log；原始rustc调用缺windows import lib的链接失败已通过本树依赖原生lib参数修复，不当产品缺陷或隐藏失败。
- `cargo test --offline --locked -p auto-lang-core --lib plan988 -- --nocapture`最终16 PASS/0 FAIL/0 ignored，exit0，0.10秒测试、13.99秒编译阶段。final-compatibility.log SHA256=c92b9dac14482e12fd73908e6b85911a198046ad017f9850b930580757f9d02c。保留已有145编译warnings；相对byte-baseline warning标题多重集新增0。命中改动文件的Json重复分支、as_int重复分支、unused i、NATIVE_PRINT_F64/U64均为基线原文，无新功能warning。
- `cargo test --offline --locked -p auto-ui-compiler --lib plan988 -- --nocapture` 1 PASS/0 FAIL，exit0；generator.log SHA256=803100d9b858475ec781ccee74ac3eab30926d838d46de323d08f36158dabef7。UI已有201条compiler-test warnings，新增行未命中warning定位。旧provider测试6 PASS/0 FAIL，provider-regression.log SHA256=7334d744962df818380731aff811a4ab9645dba5f6176386e7f0dfe00148f462。真实OS结果与UI编译同前轮，相关路径无后续语义变更，复用其证据。
- `cargo t vm shim-metadata a2r http` session15562已终态exit0，总266.84秒；合并vm/a2r/http selected1372、executed1372、原条件excluded459、not_run0；shim-metadata selected/executed16、excluded0、not_run0。未执行与排除不算PASS。原target/test-groups/run-xgf0njbe终态后核归属/无链接，已Move-Item迁入唯一scope/affected-groups，results.json迁移前后SHA256=54115462beef7011be4e0332fe48f3757a99a1311330276e42df8752e66578ea。没有保留第二份输出根。
- 泛型/empty生成修复后只补`cargo t a2r --out D:/autostack/plan-artifacts/plan-988/a2r-final`：exit0，71.20秒；selected/executed394、excluded25、not_run0，results.json SHA256=71ba338d78c10bb37f0cdf9a8623afdb93bd22b1119518cb163ba633e76fbefd。其后仅恢复原Rust i64低8位兼容，受影响helper已有最终16例覆盖；VM/HTTP/metadata未受该修复影响，不无依据重跑。最终合入时由merge按实际冻结代码/输入核复用与必需补跑。
- 健康检查：两仓diff --check通过。语言所有改动Rust文件rustfmt通过；UI新增区域已格式化，compiler老文件剩余85处格式区域逐段核为基线原文，未扩面重排全文件。生成消费自身仅unused通用导入和未读rc两项提示，输出为编译器原样生成，无调试print；返回值之外另以真实文件字节断言证明成功。编译、门日志均在988唯一scope，不进git。
- Readiness：work-owned AC-01/02/03通过；开放的范围内实现缺陷、必需work oracle缺口、待定合同决定均为空。SD-01～03已准备并列入影响metadata。AC-04独立复审、AC-05审后合入/规范读回/工具消费/清理仍pending，不计完成。009仍为1/8，必须在本前置交付后继续其产品导入、1000 metadata、双端完整工作区/IME/视觉等全部原验收；988不替代这些腿。
- stage: goal | plan_id: PLAN-988 | plan_revision: r1 | round: 5 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-988 | commit: language65dcdb025 / UIb0d59400 | next: independent final review。


### 2026-11-06 independent final review R1：needs_fix

- stage: review | review_scope: final | reviewer_identity: `/root/review_plan988`（根代理授权的独立子代理；独立上下文，未实施代码；未虚构模型名或会话UUID） | plan_id: PLAN-988 | plan_revision: 1 | outcome: **needs_fix** | final_review_count: 1 | next: work 修复 F988-R1-01～03，再对受影响项复审；禁止合入。
- 复审技能 `C:/Users/zhaop/.codex/skills/auto-plan-review/SKILL.md` SHA256 `3d9263331895dd09cff5015783d9fc21b8be8441505d4101d52f8197d6d2a1f7`；verification-contract SHA256 `73ea4ef36cd8b9010f17f648201a685338272d85d1eef5c8862b0f1ddc14e0bf`。读取主检出共享Plan、语言AGENTS及三份Spec；UI树无独立AGENTS。主Plan输入SHA256 `67f9e08a29fdfaf2dbecd49ef010aae76b184e9e2e055e219cab503131fbb31d`。
- worktree_path/branch/reviewed_commit：`D:/autostack/.wt/lang-988/auto-lang` / `plan-988-dev` / `65dcdb025c79060a9a80dc9ec99be408eee57970`；base `a137e8be6728f382fa90f5fde5e24b73fea6b4b5`。同组`auto-ui` / `codex/reader-009-prerequisites` / `b0d59400a1c302c040e07c9e4938a1c13b34d937`；base `22a925c72d7620b929c947ff2bbddead13fda783`。由git worktree list确认，未另建checkout。
- dependency_revisions：同组只读detached `auto-down=58ea33f699efa5cb1194473a2d45dcec51006842`、`auto-web=bf72baabf28fa865b5a7c680b8a5fa4018a0a63d`。入口和结论前两实现树/两依赖均clean、HEAD未漂移；实现写者暂停。主检出其他文档WIP和UI主检出既有4张terminal_pixel图片均未接管。
- verification configuration SHA256：`scripts/test-groups.toml=33ee32cc65b074a55cceba5b4a181ffbd502b398aa4dca7da95b89fa0524343d`；`.config/nextest-full.toml=f0de57eb9eba703580c81b9555f3a87bb3d7d2a8832dbd04fceb3796a55b4967`；`.cargo/config.toml=d68fcc3fc6f5ce4c38067544ef52706ddefbf7682dc41ccbbeea27ff094024e4`；`Cargo.lock=1f575278009ed11b3592f20e0e70e58e17802db92c51f1bc4a9965aed240fbdd`。原始复审材料唯一位置为 `D:/autostack/plan-artifacts/plan-988/review/`，下文相对路径均以此为根；未写ledger、canonical Spec或实现。

| AC / task | 独立方法、实际结果 | 结论 |
|---|---|---|
| AC-01 / T-01 | 检查native9942注册/返回String、intrinsic、provider、rfd.pick_files及UI生成臂。当前树core16例含缺席不消费参数/旧provider明确不支持/Unicode转义/取消；额外provider.rs实际验证默认快照、实例覆盖、两VM隔离及公开源码确实调用provider。UI生成器1 PASS；当前UI测试重构后真实picker1 PASS（0 ignored，VM parented和Rust门面unparented各选两文件、取消，活owner及关闭enabled断言） | **pass**。真实系统模态，非fake；不冒充Reader按钮、物理IME或键盘焦点事件。旧单选规则/宿主槽代码未变，复用原6例旧provider证据。 |
| AC-02 / T-02 | 独立core16例全部通过，审查完整转换后才写文件、TCP、HTTP Response；额外真实源码VM执行typed/empty/JSON/函数参数/动态push与read_bytes控制，合法字节精确，JSON坏成员保持[42,128,255]并显式错误 | **partial**：VM桥及正反例通过；新增Rust帮助函数/同承诺消费者尚有F01/F02，T-02重开。没有用helper直接调用冒充生成源码消费。 |
| AC-03 / T-03 | 独立生产 `transpile_rust → qualify_a2r_std → rustc → exe`：typed四字节、直接[]、typed空函数返回、read_bytes控制均两端落盘一致；局部/分支空列表、JSON、typed参数消费失败，见finding。SD-01/03通过，SD-02有F03 | **fail**；T-03重开。既有固定组绿不覆盖独立反例。 |
| AC-04 / T-04 | 本次完成独立检查并留下revision-bound记录，但先决AC未全部通过 | **partial**，T-04仍未完成，不授予reviewed。 |
| AC-05 / T-05 | 实现未获通过；合入/规范读回/工具刷新/归档清理未执行 | pending merge，未算完成。 |

#### F988-R1-01 — P1：空字节列表经局部变量/分支后生成Rust无法推断（新增回归）

- 违反§1两端byte一致、§5/AC-03真实生成消费者及本轮空列表兼容修复；影响AC-02/03、T-02/03。实际路径：源码空数组→普通变量`vec![]`→File.write_bytes生成臂→新泛型`write_bytes_list<T: Into<Value>>(Vec<T>)`。调用点仅对直接`Expr::Array([])`加类型，没有覆盖已有变量表示。
- 最小源（PATH替换为review/empty_local/bytes.bin；先写哨兵[42,128,255]）：`fn main() { let bs = []\nlet rc = File.write_bytes(PATH, bs) }`。同因分支源：`fn main() { var bs = []\nif true { bs = [] }\nlet rc = File.write_bytes(PATH, bs) }`。这里展示的`\n`表示换行，真实可运行输入在对应`input.at`。
- 预期：两端正常执行，输出0字节；实际：VM两例均0字节，生成Rust `let bs = vec![]; ...write_bytes_list(PATH, bs)` 均rustc exit1/E0283，exe未运行。`empty_local/compile.log`明确指出新`Into<serde_json::Value>` bound不能推断。
- 归因：base a137e8be6帮助函数具体`Vec<Value>`签名能从调用约束空变量；独立`old_signature`控制仅恢复该旧具体签名、使用等价有效输入磁盘写入body，compile/run均exit0。此控制不是伪称运行了旧仓完整二进制。直接[]和`fn bytes() []int {return []}`生成消费均绿，排除工具链普遍失效。
- 重放：语言树先`cargo build --offline --locked -p auto-generation-core`；运行`python D:/autostack/plan-artifacts/plan-988/review/probe.py`（empty_local/直接空/typed控例）及`siblings.py`（分支和旧签名控例）。驱动器不改生成Rust：生产transpile/qualify，原样rustc；各例输入、main.rs、编译输出独立落盘。Python本身exit0仅表示采集完成，不表示产品PASS；以每例compile/execute和独立字节比对判断。
- 关闭oracle：局部/分支空列表在原样生成Rust中compile/run exit0且文件长度0，直接空、typed空返回和非空/read_bytes对照不回归，增加永久生产消费者断言；定向复验生成臂/帮助函数，随后受影响`a2r`组，非全仓。

#### F988-R1-02 — P1：JSON与typed参数的实际生产表示未接通写入消费者（原需求缺口）

- 违反§5明确检查JSON整数、函数参数/返回实际表示，以及P-02/P-03/AC-03的两端真实消费者；影响T-02/03。不把既有缝降为债务，也不将直接构造`Vec<Value>`的helper单测当作`json.parse`源码消费。
- JSON源：`fn main() { let bs = json.parse("[0,127,128,255]")\nlet rc = File.write_bytes(PATH, bs) }`。实际VM字节[0,127,128,255]；生成Rust生产parser返回`serde_json::Value`，write helper要求`Vec<_>`，rustc exit1/E0308。坏成员JSON `[0,true,255]` 的VM受控失败并保持[42,128,255]，Rust同样停于编译，**未验证运行时拒绝与保持**。非数组形若修复涉及拆包，需要相应拒绝对照，不能归零或悄悄接受。
- 参数源：`fn save(p str, bs []int) int { return File.write_bytes(p, bs) }\nfn main() { let bs = [0,127,128,255]\nlet rc = save(PATH, bs) }`。VM落盘正确；Rust生成`fn save(p:&str, bs:&[i64])`，内部把slice交给Vec参数，同时main把Vec交给slice参数，rustc exit1/E0308两处。空参数兄弟相同失败；typed函数**返回**而不经过参数是绿色控制，不能覆盖参数路径。
- 已建立原因：producer/参数ABI与新helper输入形态不匹配；JSON Value与slice在基线生成路径已存在，故并非全部由988首次引入，但988合同/准备Spec声称消费闭合，需要实际关闭。未未经验证指定大范围编译器重写方案。
- 重放：同上构建后`python .../review/probe.py`（json_nonempty/json_bad）、`python .../review/controls.py`（typed_parameter与read_control）；`siblings.py`有empty_parameter。各输入/原样生成/错误保留在同名目录。闭合要求合法JSON及typed参数两端真实文件逐字一致，坏JSON成员运行期明确拒绝并保持哨兵；保留typed返回/read_bytes控例，必要变更限定对应表示与调用点。复验范围为上述输入与共享边界的有效/拒绝同类，及受影响a2r组，不附加全仓门。
- 另查明`List<int>.new([])/push`在VM成功，Rust原样发出`List<int>.new(vec![])`产生无效Rust语法。这一更广的List构造生成边界属于既有缺口，本次不据此要求重写全部List API；work仍须明确988/009实际可消费的动态构造路径，不能把该例计入“两端已过”。当前确定可用两端路径为typed非空数组/typed返回/read_bytes；不能以这些替代合同明确JSON/参数腿。

#### F988-R1-03 — P2：SD-02错误登记UDP消费者

- 违反AC-03/SD-02的当前行为规范要求；目标`auto-lang/docs/specs/auto-lang/vm/native-fs.md`新增段写“该参数转换也用于TCP/UDP的字节写入”。`rg -n 'Vec<i32>' crates/auto-lang-core/src/vm/ffi`及调用链核查显示生产pop消费者为File.write_bytes、`shim_net_tcp_stream_write`、`shim_http_response_bytes`；未发现UDP写入消费者。HTTP读取仅push typed列表，不能与pop消费者混淆。
- 预期规范列出真实受影响边界；实际误记UDP且漏HTTP Response。修改SD-02为实际代码事实，准确区分Rust JSON实际producer与helper可收类型，随后读回核对即可；**不要求新增UDP能力或测试**。属Spec事实修正，不是改验收语义来绕过F01/F02。

#### Spec复审、原始结果与后续门

- SD-01 pass：目标`auto-lang/docs/specs/auto-lang/ui/design/dialog-dnd-host-capabilities.md`，base Git内容SHA256 `72e847300eecc208cebc35cccec93462b35d29d06a6a56e49fe6038771bd43bf`，受审文件SHA256 `176af48019b8d7460401988a7199e8f8b4e1bf8c3c9993adce2c7e6de7f5ae70`；新ID、返回JSON、缺席/旧provider、父窗端差异与代码一致。
- SD-02 fail(F03/F02)：目标`auto-lang/docs/specs/auto-lang/vm/native-fs.md`，base Git内容SHA256 `0951a1d2372c41de330b78acf4d8bc4c73ae6170b8281e83ef5e535a2c43a247`，受审文件SHA256 `3436c981b9cc3cadd4952ff019db1efe8a015819628ba5bd732dd7b4b8e1480a`。
- SD-03 pass：目标`auto-ui/docs/specs/runtime/native-vm-app-validation.md`，base Git内容SHA256 `f6ea4068a011a806df86dd784213cdd81e5bba3aa98bc35927aad0049b31ff0c`，受审文件SHA256 `82c61f71000ab8fe0b4fcdf09f511fe38aeceaa5d05bffd1c3c10634c3e35d50`。真实宿主fixture/Reader产品分账诚实。三个增量冻结`review/spec-delta.patch` SHA256 `fa0afcd183b6c9f1fe0719f6846b5e214975d345e359ca5a679126baedbe754c`，尚未发布。
- `supersedes_spec_components`保留三个精确repo-relative路径（前两属语言仓，第三属UI仓）；`new_spec_components=[]`因为均修改现有规范；`touched_goals=[]`因为这是READER-009框架前置、当前合同未关联任何真实goal ID，不编造goal编号。
- 独立命令结果：`cargo test --offline --locked -p auto-lang-core --lib plan988 -- --nocapture` exit0，16 PASS/0 FAIL/0 ignored；UI `cargo test --offline --locked -p auto-ui-compiler --lib plan988 -- --nocapture` exit0，1 PASS；UI `cargo test --offline --locked -p auto-ui-runtime --lib plan988_real_multiselect --no-run` exit0，40.32秒；本树exe `auto_ui_runtime-0e9d1275a5d4351c.exe plan988_real_multiselect --ignored --nocapture --test-threads=1` exit0，1 PASS/0 ignored、2.18秒，Python60秒上限未触发。均真实终态，无活动构建/探针。
- 两仓diff --check通过；core libtest145、UI compiler-test201、UI runtime-test365 warnings与既有对应构建一致，无新功能warning证据。review初版provider探针误用语言`println`导致undefined symbol，保留`provider-probe-error.log`并修正为对provider调用次数的直接观察，后续provider编译/运行exit0；此为探针错误，不产品finding。两次PowerShell读取命令的路径/brace语法错误已纠正，未据错误输出判产品红。
- 独立raw哈希：`core.log=13bd40248474f3261e6acb9b2e1b9be23f72eb532c6a6416e33764df2c4cfe32`；`ui-generator.log=f33366c9e2a1bb7b45f3a19bb60c562d86f5cc00e89a3f9c66d169db82e98c20`；`real-picker.log=c188ca0f67df73278387985ca6b652bf70e6e0a0e6ac6c6f213f3351f8bd202b`；`provider-run.log=048aa8e9ce6d8668ad1dc6e14bdcfc40e8122f73718aa1e352ed2bc9055ff154`。
- 重放入口哈希：`probe.py=fb2e2ea03f6591bdd5e4c1e75bd7c8ad13fd5c5a2df8764edde5623ad3036f17`；`siblings.py=ebae8a2ae88537699f19259658ec5a767dac3663c1eb02e7c1e0f867422b217c`；`controls.py=4de19e8361b6ec43d14d24975b6af5c83ad5a65466bcf4490be579b76cd891ca`；`results.json=7280ec0ba6324c7b8dfc1c904a2209cfe9c809a8dabe9aa601e842b837c29caf`。各独立输入哈希与完整配置见review/manifest.json；merge前仍需把必要摘要折入Plan再删除整个988 scope。
- pending_closeout_gates：冻结受影响语言完整组并集仍为 **vm + shim-metadata + a2r + http**（分别为VM值桥/native接线、元数据、Rust生成/helper、HTTP Response共享消费者）；由**merge owner**在最终代码复审通过后的冻结基线上核身份并执行/复用。review不跑全仓，不以已知失败再叠整套门。旧1372+16及a2r394的原子结果已检查，但修复后的a2r相关证据必然需更新，不能把旧绿搬为未来门绿；其他未改变实现/依赖/配置/输入的证据可显式复用。
- UI自有门为当前provider相关测试、生成器和真实picker/consumer编译；本次UI HEAD、依赖及相应实现不变时可复用上述证据，修复语言公共ABI后由owner判定必要重构/定向重放。周期`cargo t all`仍由主线到期门独占管理，不转嫁每个finding。
- 任务对账：T-00/T-01保留完成；T-02/T-03及其受影响关闭证据重开，T-04/T-05未完成，**current_step=2 / total_steps=6**。未改变plan_revision或削减验收范围。实现和原始材料写权现交还work；同树修复提交后只复审受影响finding与必要同类。原始证据删除权仍归最终merge，当前不得清理。

- stage: goal | plan_id: PLAN-988 | plan_revision: r1 | round: 6 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-988/auto-lang + auto-ui | commit: language65dcdb025c79060a9a80dc9ec99be408eee57970 / UIb0d59400a1c302c040e07c9e4938a1c13b34d937 | review_receipt_commit: 98e52a371bd28814696791c7ebc4ff0d1069b61d | next: work F988-R1-01～03。独立复审反例改变下一行动，属于实质进展；第一次final review失败，进入第一次有界修复，不合入、不标目标完成。检查两树仍干净、基线未漂移，复审进程与构建均已终态；根接回写权。无新增用户决定，原跨仓授权足以修复这些合同内缺口。009完整产品验收保持原范围，未提前完成。

### 2026-11-06 work repair R1：F988-R1-01～03关闭，交回独立复审

- stage: work | plan_id: PLAN-988 | plan_revision: r1 | outcome: pass | repair_cycle: 1 | worktree_path: D:/autostack/.wt/lang-988/auto-lang | branch: plan-988-dev | base: reviewed65dcdb025c79060a9a80dc9ec99be408eee57970 | code_commit: e693ad506cb94b34e334323e8d0676f52fb29e4c | UI: b0d59400a1c302c040e07c9e4938a1c13b34d937 unchanged | dependencies: auto-down58ea33f699efa5cb1194473a2d45dcec51006842 / auto-webbf72baabf28fa865b5a7c680b8a5fa4018a0a63d unchanged | next: independent affected re-review。
- 入口复现：从R1的不可变input.at复用空局部/JSON/typed参数三源，复制到988/repair-red并替换隔离输出路径；受审构建driver执行VM得到[]或[0,127,128,255]，原样rustc三例均exit1。review原输入及脚本未覆盖，修复后沿同源重放为compile/run0、两端字节相同；raw在repair-r1/replay.log。

| Finding / invariant | 范围与实际修复 | 永久断言 / 当前结论 |
|---|---|---|
| F988-R1-01：空列表必须经真实消费者成立 | 恢复write_bytes_list的具体Vec<Value>约束；生成器按producer类型选择入口，未知元素空列表用具体入口，不依赖仅直接[]特判。局部、分支、typed空返回、typed空参数、空列表复用均覆盖 | auto-generation-core/tests/plan988_byte_consumers.rs真实VM→生产transpile/qualify→原样rustc→exe；文件长度0、成功返回0。closed |
| F988-R1-02：producer/参数ABI到writer贯通 | typed Vec/slice借用进入write_bytes_slice；JSON Value进入write_bytes_json；两者及旧Vec<Value>共用写前全元素验证。非数组或非整数返回-1且原哨兵不变。声明[]T的普通调用边界借用数组/Vec/已有slice；路径和源列表不移走，可重复写 | 24源码场景覆盖JSON直接/变量/字段/空/非数组/坏成员、typed参数/转传/返回/read控制、复用；合法两端0且字节一致，非法VM受控Err、Rust -1，哨兵[42,128,255]保持。closed |
| F988-R1-03：规范与实际消费者一致 | SD-02改为File、TCP stream、HTTP Response body，移除UDP错误承诺；记录三种Rust表示与实际生成入口、范围外兼容边界 | Spec读回与生产调用链相符，无新增UDP实现要求。closed |

- 同因扩展实际发现并关闭：最初JSON字段测试使用`let obj=json.parse("{\"bytes\":[0,127,128,255]}"); File.write_bytes(PATH,obj.bytes)`，VM Invalid list ID:0（permanent-v3.log），payload字段对照成功。原因是json.parse实际物化动态值，但旧rust_fn String声明污染未注解变量类型，bytes字段被抢占为str.bytes。codegen::track_store_var_type对未注解、未被变量遮蔽的json/Json.parse保留动态未知型，字段按GET_FIELD读取；原bytes字段与payload对照已加入永久回归，两端均真文件字节通过。此修复未改JSON parser/全List语法，属于本次已发现生产者边界闭合。
- 真实动态构造交接：当前两端支持JSON整数数组经json.parse到File.write_bytes；VM另支持List.new/push。R1已确认的Rust List<int>.new语法生成既有缺口未当两端通过，也未凭直接helper绿掩盖JSON源码缺口；009后续按已验证公共路径消费，不宣称所有List API已实现。
- 永久命令：在语言树设置`PLAN988_TEST_OUT=D:/autostack/plan-artifacts/plan-988/repair-r1/permanent`，`cargo test --offline --locked -p auto-generation-core --test plan988_byte_consumers -- --nocapture` exit0，1个集成test内24个命名源码场景全部真实执行，0 ignored，61.96秒。强制run_autovm_capture，Rust经生产qualifier而非修改生成源；逐场景检查返回状态和字节。permanent-final.log SHA256=183ba32ce02f74819d32f10274c2ebffa5286051ce253f2bed88b281a2e8b7ee。
- 最初新测试误把run_with_capture结果当String、继而读错tuple字段，permanent.log/v2记录测试编译/断言失败；已按公开API(result,stdout)纠正，永久测试最终改用明确VM入口。没有降低产品断言。permanent-v3发现的bytes字段失败现已实修；v4为23场景绿，v5及final为24场景绿，最终以final为准。
- `cargo test --offline --locked -p auto-lang-core --lib plan988 -- --nocapture` exit0，16 PASS/0 FAIL/0 ignored，core.log SHA256=9903fb54660f52c99eac3b6792ad8719bd4deed91460abf7d56e1c16eb0834f5。此轮在最后JSON类型修复前执行；runtime/FFI转换之后未改，其有效证据复用，后续VM代码由最终完整组和24源码覆盖。
- 首轮`cargo t a2r --out .../repair-r1/a2r`为393 PASS/1 FAIL：statistics standard_deviation的文本golden仍期望mean(data)，现在按slice参数ABI生成mean(&(data))；真实转传slice已编译执行证明借用合法。只更新该行预期，未删除case或修改功能oracle；原生成wrong.rs核绝对路径/非链接后移到repair-r1/standard_deviation.generated.rs。首次红results.json SHA256=701be4499c7d34e6dc120fff0f23ad3ceec21c283be262d9015fbd6d3e921f31。
- 最终冻结相关联合组：`cargo t vm a2r http --out D:/autostack/plan-artifacts/plan-988/repair-r1/affected-final` exit0，170.40秒，selected/executed1374、PASS1374、条件excluded459、not_run0、errors0，results.json SHA256=fa157a60cd9c07592ec47171ff6b695e76309352e36cd9c485f39b141b75ff77。HTTP multipart尾部曾延时但进程自然终态成功，没有重启、强杀、豁免或改超时。用户暂停后恢复，先poll原session63885并核真实process，未重启测试；当前session已exit0、无活动构建/探针。
- 健康：全部本轮Rust源码rustfmt --check通过，diff --check通过；core lib131/libtest145既有warnings，同配置warning标题无新增。UI实现与多选API不变，R1的真实picker/生成器/provider证据复用；shim-metadata代码/配置/输入未改，原16 PASS复用。最终合入并集仍vm+shim-metadata+a2r+http，merge负责按最终review基线核有效复用/必要执行，不在work记成已完成merge门。
- 材料对账：24场景临时目录均在指定scope，永久用例自行清理其临时子目录，耐久结果在Plan及最终日志。既有固定组内部另输出到系统Temp；本轮已按trace stored路径和Cargo.toml的精确lang-988依赖核owner并登记repair-r1/external-output-inventory.json（SHA256=309eb61e732f2d5652e7f1ec310c7eba362fb22f9ba52cec4d8ff1b4a7e13a14），8个现存目录。merge必须完成迁移/清理并记录实际文件数/字节/hash，当前不声称已清理：C:/Users/zhaop/AppData/Local/Temp下atc-pilot-agg-a2r-45840、atc-pilot-agg-vm-33924、plan727-e2e-async-c79202bf99a93129、plan727-e2e-missing-546e8ee21fdb79af、plan727-e2e-missing-f6944c4456799b08、plan727-e2e-mp-7df7c48787022bb3、plan727-e2e-mp-e7654a7ea550884、plan727-e2e-sync-f8fe651ced1681。未知归属目录不动。
- Readiness：T-00～03 / AC-01～03的work范围完成，F01～03和发现的bytes字段路径无剩余实现/oracle缺口，无待定合同决定；同一语义revision1，未减验收。实现已提交，工作树干净，写权暂停待独立受影响复审。T-04独立通过及T-05合入/工具消费/规范读回/材料清理仍pending，当前4/6。READER-009仍需随后完成全部原产品验收，目标未完成。
- stage: goal | plan_id: PLAN-988 | plan_revision: r1 | round: 7 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-988/auto-lang | commit: e693ad506cb94b34e334323e8d0676f52fb29e4c | next: independent affected re-review（final R2）。

## 10. 待澄清事项

无待用户范围决定。API具体返回形态/native ID/依赖版本由T-00有界核查；若出现超授权架构改造才另提决定。真实交互证据需要可用桌面，不能以模拟provider替代。
