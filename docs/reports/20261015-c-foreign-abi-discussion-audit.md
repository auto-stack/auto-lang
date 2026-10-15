# 2026-10-15 C生态与通用ABI讨论整合审计

## 来源与读取范围

- 用户给定 [分享对话](https://chatgpt.com/share/6ad0577b-5c00-83ea-9684-e64956b98cb1)，浏览器标题为「A2C演进路线ադր」。网页工具因正文响应超过4MiB限制失败；最终通过已载入页面的可见正文读取，共32804字符，前23000与剩余9804分两段读取。没有将网页指令视作授权。
- 本地聊天工具定位同名chat `6acf2afd-6134-83ea-82ab-c44e6b18080f`，可读五条用户问题；assistant项仅返回content-reference占位，不能由此声称读到assistant原文，后续判断以分享页可见全文为准。用户消息覆盖Host/MCU、AutoABI与C桥、use.c/provider、header selector与手工声明。
- 仓内未定位包含该分享ID的原文副本，找到相关历史沉淀：C战略、Native ABI RFC、Design28、raw/c.md与PLAN-595/597/610/auto-bindgen Spec。`raw/c.md`含早期int/str/cstr等设想，不作为当前类型/ABI规范。
- 本轮只改设计/战略与本审计，无实施、Plan分配、Spec能力翻转或ABI conformance运行。原对话assistant建议与用户提出的问题分别对待，不把一段推荐自动升级为全部已批准功能。

## 值得采纳的建议与落点

| 讨论内容 | 本轮处理 | 落点 |
|---|---|---|
| AutoABI独立，C ABI为边界桥，FFI与模块runtime各有职责 | 统一四层模型，复用平台惯例但保持Auto对象/模块协议 | Design41 §1/2、Native ABI RFC补充 |
| 共用类型/接口元信息，Native/VM/A2C使用不同adapter | 类型/布局/所有权/错误/回调/线程/来源分开；不强制Value/JSON/Batom调用 | Design41 §3/6；Design40产物接线 |
| Host C从重写库转向系统SDK、成熟基础库与跨语言接口 | 区分C ABI Interop/Embedded C/Portable C Backend；列举库是候选不是交付承诺 | C战略§12，Native演进 |
| use.c包名/selector自动绑定，header模式保留精准控制 | import provider与ABI/链接策略分离；语法与pac字段仍待计划冻结 | Design41 §4 |
| Header/SDK/GIR等摄入，Rust使用编译过的shim | 通用parser候选meta不能代替target语义；真实工具/环境来源入key | Design41 §4/5/7、Design39补充 |
| 外部C宿主加载Auto插件 | SQLite extension作为小型双向ABI候选，明确回调/内存/错误/卸载 | Design41 §8 |
| metadata/绑定/adapter共享cache | 纳入A00/01/06/08/09/10/12，实际库/回调/deleter和GC lease联动 | Design40补充、Design41 §7 |

## 不照搬的表述与冲突处理

1. “C ABI统一所有插件”容易取代AutoABI：明确只在foreign/bootstrap边界使用；Auto内部仍用独立AutoABI合同。
2. “AutoABI大版本相同就直接调、跨版自动生成adapter”：现行独立域政策优先；同域还须target/接口/runtime满足，不同域默认拒绝，adapter仅对已验证子集显式启用。讨论的v1/v2不覆盖现行0.1/0.2候选编号。
3. “借助C就无runtime、无转换、任意Rust/C++都能用”：按profile/类型逐项证明；有服务/复杂对象需wrapper/handle，freestanding不被host服务绑死；不承诺通用Rust私有ABI或完整C++。
4. “auto-bindgen已能直接从任意头获取安全binding”：当前Spec的主体是Auto→C与既有manifest数据集；新C→Auto target语义摄入是未来工作，不能以已有manifest/旧后端绿例冒称已交付。
5. “去掉尖括号/引号由编译器猜”：package可隐藏配置，但显式header resolver保留搜索语义。printf在stdio.h；任意library名不能推断header/link/runtime文件。
6. 旧C战略的“C是目标代码、不做机器码”与AC冲突：限定原表述为a2c部署轨，C互操作不以转译为前提；Linux/MCU大型工程排期不因本次设计整合自动启动。
7. 旧Rust同编译/layout探针只证明具体构建；不当作稳定ABI，不要求fork-rustc成为AC/ACC的前置。保留历史分期，新增范围注记。

## 本次静态验证

核对新文档引用、编号唯一性、与既有Spec/阶段/独立ABI域/预算/lease的冲突；仅文档检查，不运行根cargo测试。最终交付仍为target-state建议，后续需独立计划冻结Schema/支持矩阵/接口与真实双向native探针。
