# auto-api-gen — 通用 API 生成语言 adapter（SD-07，as-built）

> **Status**: active（PLAN-831 C2 切源落定）
> 路径：语言仓 `crates/auto-api-gen`

## 职责与边界

auto-api-gen 是语言服务/API 生成 adapter，物理 owner 留语言仓，服务 UI 与 Man 两类消费者。sidecar 声明自 auto-project-model 读取（SidecarDecl/sidecar_decl_from_root 单源），零 Man 管理实现依赖（闭包内无 auto-man-core）。adapter 不拥有第二份 SidecarDecl 或根声明解析，不持有 UI 宿主状态。

## 接口与兼容

真实接口与旧入口兼容由定向语义及类型消费验证（tauri fixture 预存红如实归因保留）。生成器资源与外部工具前置按实际消费者闭包登记。
