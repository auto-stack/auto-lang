# Web 四仓来源与交付合同（PLAN-890）

> **Status**: active；owner为language/Web/UI/Man，Down是固定依赖而非第五领域owner。

## 单源及消费者

Web领域规范唯一来源：[Web项目](../../../../auto-web/docs/specs/project.md)。
Core仅持语言语义、编译元数据、中立[宿主端口](../language-sdk/http-host-port.md)和纯合同类型。
HTTP服务/客户端生成与模板归Web；IPC/Tauri和非HTTPadapter留原owner。
受限共享`rust_workspace/shared_generation/sidecar/source_owner`单源在`auto-generation-core`，
不形成Web→auto-api-gen大门面/UI/Man/Tauri的反向实现边。
中立photo_service及原十项测试在auto-media-core，UI旧路径仅重导出。

Man保留CLI/工程/依赖和通用进程编排，UI保留renderer/tool/worker/proxy及非HTTP行为。
各宿主消费固定来源，内部显式装配Web，不改变`auto service`与daemon `auto serve`的区别。
消费细节：[Man](../../../../auto-man/docs/web-integration.md)、[UI](../../../../auto-ui/docs/web-integration.md)。

## 来源、SDK及恢复

冻结前后核对源码、锁、17原资源和真实producer身份。显式所选/邻接受管SDK失败不借ambient来源。
Web受管源包包含19实际package、完整stdlib、必要fixture/license与版本/全文件inventory/hash。
服务生成接受Web最小SDK或Man Core/Full；UI生成仍要求Full。
prepare/verify/swap失败不得破坏旧有效workspace/安装或用户文件；成功升级后重读实际产物。
通用事务算法归Man，Web携带逐字且Git来源可验证的不可编辑owner快照，不养第二份安装算法。

原旧架构验收仅认证真实合法Core形态，不据此冒认旧UI/Full。ZIP、fresh/locked/重定位、故障恢复和
当前组合的实际结果见[Web发行](../../../../auto-web/docs/specs/delivery.md)与归档收据。
历史PASS不能认证变更后的SDK字节；源码等价复用须明确对应路径、输入/配置/断言及限制。

## 实际执行与拒绝边界

`python scripts/web-four-repository.py --out <本地目录>`冻结四仓及Down依赖，语言通过896新`cargo t`显式固定组
`integration auto-generation-core auto-api-gen auto-media-core a2r-std auto-test-case`执行；Web独立库/HTTP、
UI/Man保留原owner命令和单进程ui_gen语义。只列举指定Cargo配置，不读取旧叶目录或自动扩大消费者图。
公开`web-repository-fixture.py run --require-complete`核来源/闭包/资源/迁移/执行及发行恢复收据。
list-only/过滤/缺腿/重复/零选择/未执行/错误hash/缺command-output/原始失败不能认证完整交付。

执行端和收据消费端共享原baseline/retirement/migration/frozen-floor规则；清单/输出/ID/数量/hash彼此一致
也不能删去原必需用例。迁移credit只来自原必需ID对应目标的实际PASS，不来自自报count或仅在清单出现。
原ignored/platform/manual/milestone保持NOT_APPLICABLE，不算执行或通过；ui-tu使用同批ui-full原ignore身份
补足libtest清单不标ignore的缺口，exact condition_excluded拒绝任意排除，原floor不降。
用途建议不改变启用条件；最终显式受影响固定组及必要消费者/集成、实际四仓与发行门分列，周期门只在主检出到期执行。

## 支持与证据

固定Blog来源b42a660的原314文件不改；原真实Vue15/15、native22/22及冷构建/API/持久化证据保持原输入边界。
原Windows长路径、测试控制器清理、ignored、warning与三项AAVM历史红不以迁仓宣布消除。
最终交付状态以[PLAN-890](../../plans/archive/890-auto-web-independent-repository-delivery.md)的实际门、landing与guard收据为准；
本地Git交付不代表远端push/registry发布。
