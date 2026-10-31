# 中立 HTTP 宿主端口（PLAN-890）

> **Status**: active；实现：`crates/auto-lang-core/src/vm/host_capabilities/http_service/`。

## 依赖与装配

Core 拥有编译声明、`CompilationMetadataGuard`冻结快照、值/JSON转换、VM/native注册、
request-context与资源hook。协议纯数据/配置/校验来自单源`auto-web-contract`；Core无 Web 实现依赖。
`HttpServiceHost: Send + Sync`只共享provider；返回`HttpServiceFuture`和`Rc<AutoVM>`仍是local/!Send。
`HttpVmOwner`留在原执行线程/LocalSet，网络只接收owned请求、应答、body与取消消息。
禁止将VM指针编码为整数跨线程恢复，不复制AST/Store/VM或传输框架对象。

直接服务消费者固定依赖`auto-web`并调用`install_http_service_host_if_absent()`；它不覆盖自定义provider。
显式替换使用SDK setter。普通无服务入口无需provider；实际服务调用缺席时在监听前具名拒绝，
Core不自装隐式框架、不保留旧运行时回退。Man/UI在各自组合点安装。

## 声明、启动与寿命

编译collector包括依赖模块codegen，在执行/await前冻结到服务owner。
正在运行的服务读取自己的参数/返回值/遮蔽/异步类型快照，不依赖下一次编译的全局重置。
`HttpServiceLaunch`与编译声明分离；配置只在invocation入口捕获，服务内不重新读取环境。
`entry/run_bridge`及`script_entry::run_with_path`的文件线程在spawn前捕获launch并在worker绑定；
显式None保持None，future每次poll恢复自己的上下文，调用方作用域退出后恢复原上下文。
配置与环境端口不同仍以选定配置、profile、hash和Host策略为准。

BlockingListenerKind区分ServerHandle与LegacyStdnet；不支持的provider等级明确失败，不能偷偷替换路径。
返回与cleanup完成分开：网络/logger须真实join，关闭/取消/错误保留原许可、文件/上传/订阅等回收义务。
线程栈、Result/错误传播及ordinary语言行为保持；不新增TLS、认证产品或跨平台承诺。

JSON值编组按当前owner冻结的规范返回声明解释旧native字符串列表；`[]str`/`List<str>`及已解包Future使用语言值转换单源。
未知声明与整数列表不按负数值猜字符串；无效字符串索引拒绝编组。VM集合、迭代及RC ABI保持，middleware短路值不借终端handler类型。

## 相关知识

领域算法与Axum双轨在[Web架构](../../../../auto-web/docs/specs/architecture.md)；
语言协议面见[HTTP server](../stdlib/design/http-server.md)，部署见[deployment](../stdlib/design/http-service-deployment.md)。
原runtime/source/ID/负控与证据边界见[PLAN-890](../../plans/archive/890-auto-web-independent-repository-delivery.md)。
