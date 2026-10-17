# PLAN-802 reviewed Spec delta — R5 frozen text

2026-10-17，plan_revision=1，implementation=b32d4df7d3a4a43746a22ec4b8d41d1856dc9ab2。
以下是 merge 待发布的规范文字；本文件不是 canonical Spec 的替代真源。

## SD-01 modify docs/specs/auto-lang/vm/overview.md

在既有宿主能力族后增补（AC-01/02/03；GOAL-003/007）：

CodeEditor 22个native（2910–2915、2919–2924、2932–2933、2939、9906、
9908–9913）经per-VM CodeEditorHostProvider供值，本族shim不再直接调用
UI editor/scroll实现。19个具名方法使用标准库owned返回值、标量和core
自有DTO，不暴露rope、renderer、UI操作类型或VM堆句柄。光标为0基行/
char列，选区为bytes，fold入口为1基行；负偏移/光标参数的既有边界处理
保留。delta每次调用只消费一次队列，text不消费队列，空delta保持水位。

无provider保持原feature-off语义：scroll_to弹出参数后返回false，其他
项返回原feature-disabled错误（剪切/复制/粘贴使用组合特征文案）。在位
provider的load失败为-1、save失败为false；剪切/复制/粘贴None表示宿主
没有该剪贴板能力，Some(bool)表示执行结果。原ID、签名、栈编码保持。

AutoVM构造快照当前进程默认，实例可覆盖；后改默认不影响旧实例。
兼容默认在同锁下仅空槽补缺，显式默认恒胜；NI Clone保留provider，VM
merge保目标provider。注入不改变binding_revision/proof规则，provider
槽锁不跨操作回调，替换或drop不销毁UI文档。原始AutoVM不反向安装UI。

## SD-02 modify docs/specs/auto-lang/ui/overview.md

在宿主装配说明增补（AC-03/04；GOAL-003/007）：

code-editor feature下，CodeEditorRegistryProvider委托既有editor注册表、
文件IO、scroll controller/intent与OS剪贴板桥，editor registry仍为单一
事实源。剪贴板方法另受ui-clipboard实现级门控。兼容默认安装覆盖统一
install_ui_host_capabilities、高层动态UI运行入口与VmBridge两构造，均为
if-absent补缺。VM和desktop renderer的editor寄宿scroller按同一
editor_scroll_handle约定绑定controller。per-VM provider隔离不承诺UI
全局registry多租户隔离；本批迁移不代表term/media/dialog/dnd等族或整体
UI renderer已独立，须保留其他剩余边的现状说明。

## SD-03 modify docs/specs/auto-lang/ui/design/code-editor-session-endpoints.md

更新VM调用路径与双轨边界，既有端点契约正文保持（AC-02/03）：

VM shim经CodeEditorHostProvider调用UI adapter，adapter与a2r/merged
直接UI臂仍委托同一editor/scroll实现。save保持rope字节直写、无全文VM
字符串往返；BOM/EOL包装归front。光标0基行/char列与选择bytes单位不变。
editor scroll仍以注册表投影为读单源、intent入队为写路径；drain消费中
命令值先行投影，再安排窗口滚动及读回校正，用户滚动后≤2s收敛的原
合同保持。测量使用固定时间戳精度余量，不将查询耗时或通道节拍加到
产品允许延迟；跨阈值且无法证明上界的观测应判不可认证。

保留规范现有其他内容及并行计划新增事实，不整体覆盖文件。无新增Spec
组件；上述路径为supersedes_spec_components，new_spec_components=[]。
