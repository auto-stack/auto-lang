# PLAN-785 r1 review — frozen proposed Spec delta

Plan revision 1; reviewed implementation 64c72d20c2e081b563760c1c4fe38761ccc6d00b.
This is proposed knowledge, not a canonical Spec publication or a change to acceptance.

## SD-01 — modify docs/specs/auto-lang/vm/overview.md

Input SHA256: AB378E0B0567A96DF5302CBC96776402BB7481BE7CEF1EFB2CDCCA9FE2C25471.
Before: frame 9918/9919 is the documented scalar provider pilot; clipboard text provider behavior is not documented.
After: add the following text next to the frame host capability section (AC01/02):

文本剪贴板 native 2925/2926（auto.clipboard.text/set_text）通过
vm::host_capabilities::ClipboardTextProvider 供值，core 两 shim 不直接调用
ui::clipboard。接口 text 返回 Option<String>（owned），set_text 接受 &str 并返回 bool；
VM shim 保持 add_string/rc_push_str_idx 与 pop_string_arg/encode_bool 的既有编码及释放路径。
provider 缺席时保留 clipboard_text / clipboard_set_text 的原
“the `ui-clipboard` feature is disabled” VMError；provider 在位但不可用时返回空串/false。
显式默认 set 可替换；兼容 if-absent 在同一写锁内补空槽，不覆盖显式默认。
AutoVM 构造快照默认，实例覆盖及 NI Clone 独立，VM merge 保留目标实例 provider；
provider 注入不改变 binding revision/proof，native 注册/merge 原有失效规则保持。
Send+Sync 约束 provider，不增加 VM 跨线程迁移承诺。

## SD-02 — modify docs/specs/auto-lang/ui/overview.md

Input SHA256: BB13D7246FE96852C1E3ECF21D6778FDFD2E7A48FE7D650212EB86C837827A27.
Before: frame provider section calls the entire clipboard family unmigrated.
After: narrow that boundary to clipboard files/images/editor and add (AC03):

UI 的 ArboardClipboardProvider 仅委托现有 clipboard_get/clipboard_set。
ui+ui-clipboard 可用宿主在 install_ui_host_capabilities、run_file_dynamic_ui_inner、
VmBridge 两构造及 auto CLI ui-iced 入口调用兼容 installer；autovm_persistent 经统一助手覆盖。
原始 AutoVM 仅快照默认，不自动回接 UI。no-ui 宿主可显式注入无 UI 类型 provider。
此批只迁文本 2925/2926；files/images 2934..2937、editor、scroll、terminal/media 未迁。
ui-clipboard 单开现在可编译但无 ui 模块/adapter，缺席错误语义保持；
ui 或 ui+ui-clipboard 不含 iced 的现存构建缺陷另记 F-785-P01。
外部 no-ui 独立 manifest 与 workspace 锁定 UI rlib 重放分别验证；
后者不证明全新独立 UI 锁的发行依赖闭包，既有解析漂移保留。

## Review disposition

Targets and existing GOAL-003/GOAL-007 metadata are retained; no new Spec component.
Delta is consistent with inspected implementation. Final pass is withheld for R1 findings;
merge must reconcile source hashes and deposit only after code review and frozen gates pass.
