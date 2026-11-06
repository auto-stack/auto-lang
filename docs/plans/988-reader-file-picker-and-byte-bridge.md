---
plan_id: PLAN-988
status: executing
feature_name: Reader原生多选与动态字节桥接
plan_revision: 1
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-06
current_step: 1
total_steps: 6
supersedes_spec_components: [docs/specs/auto-lang/ui/design/dialog-dnd-host-capabilities.md, docs/specs/auto-lang/vm/native-fs.md]
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
| T-00 | work | 授权 | 树/依赖确认，新构建复现、ABI与表示根因定位、冻结最小设计 | 源码调用链及P-02红/读回绿；实际命令/版本 |
| T-01 | work | T-00 | 公共多选声明/VM provider/UI OS/生成映射与用例 | P-01、P-04 |
| T-02 | work | T-00 | 动态字节桥与永久正反回归 | P-02及受影响同桥消费者 |
| T-03 | work | T-01/02 | 两端消费者验证、准备SD、工具交接记录 | P-03及冻结受影响检查 |
| T-04 | review | T-03 | 独立复审绑定各仓提交与Specs | AC-01～03 pass |
| T-05 | merge | T-04 | 顺序落地/工具消费/规范/归档清理/009交接 | AC-05 |

## 9. 复审记录

2026-11-06 stage:new | PLAN-988 r1 | outcome:pass | next:work T-00。用户跨仓授权已记录；已独占核号脚本分配988，未实施、未新工具构建。证据首轮以009最小红/绿为入口，不等同新代码回归结果。无需再次请求同范围许可。


### 2026-11-06 work：T-00闭合，字节桥与多选接线进行中

- 权威树：D:/autostack/.wt/lang-988/auto-lang，plan-988-dev，base=a137e8be6；UI同组auto-ui，codex/reader-009-prerequisites，base=22a925c7。auto-down只读58ea33f，auto-web只读bf72baa（UI真实path依赖需要），均git detached，无链接。
- T-00 [x]：新源码编译最小桥回归，原生i32 list控制及坏句柄控制通过，Value整数数组与错误元素分类失败：2 PASS/2 FAIL，cargo exit101（PowerShell包装记录exit1）。故障位置为ffi/convert.rs::VMConvertible for Vec<i32>，仅downcast ListData<i32>，不是磁盘权限。T-01 API冻结为dialog_open_files(filter)->JSON数组字符串，native9942，provider返回Result<Vec<String>,String>；旧provider默认明确unsupported，取消[]，VM parented/生成Rust沿已有unparented边界。
- 语言实现提交82d4b5d50：支持Value整数标签与i32范围检查，非整数写前拒绝；新增provider方法/VM shim、intrinsic、ID/返回类型/保留ID注册。相关核心回归命令 `cargo test --offline --locked -p auto-lang-core --lib plan988 -- --nocapture`：9 PASS/0 FAIL，exit0。包括真实源码字面数组、List.new/push、JSON数组写真实文件[0,127,128,255]；全256字节；无效成员保持既有文件；多路径Unicode/引号/反斜杠JSON往返、取消、旧provider拒绝但单选不变。
- 初次新增多选测试编译因漏import和私有trait路径失败（bridge-dialog.log），修复测试trait公开引用与codegen常量import后9项全绿；首次失败保留，不当功能验收红。
- `cargo test --offline --locked -p auto-lang-core --lib native_catalog -- --nocapture`：5 PASS/0 FAIL，exit0，native编号/名字/返回注册一致。两仓diff --check通过。编译有既有warnings，尚未做最终新增warning差异审计。
- UI接线：os_open_files调用真实rfd.pick_files，复用filter/父窗；provider/生成门面与编译映射接线。`cargo check --offline --locked -p auto-ui-runtime -p auto-ui-compiler` 已完成，exit0、95秒，session=69605已终态，日志scope/ui-check.log；UI编译239 warnings为待最终基线差异审计，非零错误。
- SD-01/02已在语言树准备，未发布。T-01/T-02/T-03仍未完成：真picker/焦点、缺席/覆盖/源码多选、函数参数返回/HTTP消费者、生成Rust真实字节写入、完整受影响组及UI门仍需执行。T-04独立review与T-05 merge均未启动。
- 证据scope D:/autostack/plan-artifacts/plan-988/：byte-baseline.log SHA256=2bcb8a092d9826171b65ac57b42c5abb58c9e5d628defbbf6a3fd4e52f01edd9；bridge-dialog-v2.log=20af796d0783c07f64644f9fdaedbde82377d847942f2d0e0214c9da5b65e8e8；native-catalog.log=1ac3f70ab89290ad33b9cd7695905cf9e2aee75345d7e7c600d81d40a86ec91e。原009探针材料未动。
- stage: goal | plan_id: PLAN-988 | plan_revision: r1 | round: 3 | from: work | to: work | status: executing | outcome: needs_fix (work incomplete, not a final-review finding) | worktree: D:/autostack/.wt/lang-988/auto-lang | commit: 82d4b5d50 | next: work T-01/02/03。

## 10. 待澄清事项

无待用户范围决定。API具体返回形态/native ID/依赖版本由T-00有界核查；若出现超授权架构改造才另提决定。真实交互证据需要可用桌面，不能以模拟provider替代。
