### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/tool-execution/project.md | 分散启动/等待→具名设施/adapter、进程/stdio/取消/发现/bootstrap支持 | 真执行边界 | AC-02, AC-03 |
| SD-02 | modify | docs/specs/auto-cli/project.md | 大量业务/静态UI→薄CLI/真实owner与默认兼容/命令矩阵 | 入口可独立交付 | AC-01, AC-04 |
| SD-03 | modify | docs/specs/auto-man/project.md | 纯核心/服务/旧兼容/语言与UI工具adapter分层 | 无反向UI闭包 | AC-01, AC-04 |
| SD-04 | modify | docs/specs/stdlib/design/http-service-deployment.md | 入口搬迁/执行owner明确，原config/port/ready/exit合同保持 | 防service/serve漂移 | AC-02, AC-04 |
| SD-05 | modify | docs/specs/repository-boundaries/generated-ui-distribution.md | 命令/工具profile与producer/resource/bootstrap消费闭包 | 真独立消费 | AC-03, AC-04 |

