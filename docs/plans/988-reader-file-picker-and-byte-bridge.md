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

## 10. 待澄清事项

无待用户范围决定。API具体返回形态/native ID/依赖版本由T-00有界核查；若出现超授权架构改造才另提决定。真实交互证据需要可用桌面，不能以模拟provider替代。
