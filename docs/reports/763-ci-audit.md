# PLAN-763：GitHub Actions 审计

检查日期：2026-10-11。通过 Git 历史和 GitHub Actions API 只读核查；没有触发流程或部署。

七个构建/测试定义各取最近五次，35/35 为 failure；deploy-website 的执行次数为零。这是近期样本，不代表其所有历史执行均失败。

| 文件 | 加入时间 / 提交 | 记录总数 | 最近结果 | 处理 |
|---|---|---:|---|---|
| auto-lsp-ci.yml | 2026-05-11 / `8b6bc1d38` | 156 | [最近五次均失败](https://github.com/auto-stack/auto-lang/actions/runs/38122718160) | 删除 |
| build-bps-gallery.yml | 2026-09-18 / `7da53b1a9` | 16 | [最近五次均失败](https://github.com/auto-stack/auto-lang/actions/runs/38122678554) | 删除 |
| build-ui-examples.yml | 2026-07-29 / `c11495447` | 290 | [最近五次均失败](https://github.com/auto-stack/auto-lang/actions/runs/38122678591) | 删除 |
| build-vue-gallery.yml | 2026-06-26 / `0c0bed22e` | 8 | [最近五次均失败](https://github.com/auto-stack/auto-lang/actions/runs/35332729266) | 删除 |
| deploy-website.yml | 2026-04-22 / `68e6c14f2` | 0 | 零次执行 | 删除 |
| http-e2e-ci.yml | 2026-08-07 / `999f9aa55` | 265 | [最近五次均失败](https://github.com/auto-stack/auto-lang/actions/runs/38122718797) | 删除 |
| parity-ci.yml | 2026-07-17 / `688857fdb` | 400 | [最近五次均失败](https://github.com/auto-stack/auto-lang/actions/runs/38122718799) | 删除 |
| vm-files-ci.yml | 2026-08-25 / `2a2e40d75` | 73 | [最近五次均失败](https://github.com/auto-stack/auto-lang/actions/runs/38122678544) | 删除 |

构建/测试流程目前未维护：Vue 流在 setup-node 失败，bps/UI/parity 流在 CLI 构建失败，HTTP 与 VM 流也没有近期成功运行。定义中的 Rust 检出缺少现行 auto-down path dependency（不把这一静态发现当成所有远端失败的唯一原因）。LSP 最近一次没有 job 记录，未臆测失败原因。

网站流不响应当前 master，但可被 workflow_dispatch/repository_dispatch 触发，并含真实 GitHub Pages 发布权限和 deploy action。用户明确未安排自动发布，因此删除三个触发入口及整份定义；手动 scripts/deploy-website.ps1 保留。

来源能追到添加提交和关联 Plan，无法据此确认是 AI 还是人工编写。旧定义及完整触发条件保存在 763-ci-audit.json，并可从 Git 历史恢复。取消旧 CI 不删除本地测试。

其他点目录：.agents（共享技能）、.autoos（账本）、.cargo（构建与测试入口）、.config（nextest 配置）保持约定位置。本机未跟踪的点目录不影响 GitHub 根目录行数。
