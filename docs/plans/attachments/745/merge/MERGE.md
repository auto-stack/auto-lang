# PLAN-745:r1 合入与沉淀收据

stage: merge；completion_kind: delivered。原规划/复审 Codex 上下文执行；用户显式调用 auto-musk 的 auto-plan-merge 技能，落点按用户约定为 auto-lang `v0.6-dev`。本报告随实际检查点补齐，未完成项不预报 pass。

## 交付与规范

最终复审 pass 绑定 `bb8cb4b4330d505059ea20e7bfe561a93c1051f3`、只读 auto-down `fba6563ed2148ce85e68208863159b4ccccac710`、冻结 SD SHA256 `cd027eae6395f8903f41f1623c20af49d67ec5fdf09631b30f52b8f5485bcdb2`。原专属树共用于实现、内部检查、最终复审和沉淀。保留恢复 ref `refs/auto-plan/745/reviewed`。

24 个实现提交 rebase 到主线：range-diff 全部 `=`，无冲突，rebased tip `8f0c1d05b8baa818f39a91884e840b0c47bf990f`。已审实现的实验编译器、验收脚本与语料补丁保持一致。canonical 文档及人工层审计在原树完成，交付提交 `2653e0088be1307119bb106ab0f82a4f4dc004f2`，主检出 ff-only 后 HEAD 实际相等，详见 [landed.json](landed.json) 与 [range-diff](rebase-range-diff.txt)。未创建 merge commit、未重写主线。

SD01/03 落在 [source-core-i32](../../../../specs/auto-ac/source-core-i32.md)、auto-ac/project、auto-hir/stage-contract、auto-acc/project；三模块导航与 overview 更新。仅显式有限源码 profile 已交付；完整语言映射、生产 ABI、ACC 主体/自举仍 open。touched_goals=[]；未造新目标或改 GOAL017。spec-index.py 实际执行，INDEX 生成内容不变。

人工层 MD408/409 仅追加有限数值证据，required/unknown 结论维持。AC 卡片变化使 MD203/205/301/302/304 的旧绑定失鲜，逐项审定描述/代码/消费者后更新对应卡片 hash 与行锚点；MD203/205 区分已存在的 Rust 子集适配器和仍 new 的 AutoACC 主体。没有全量盲 rehash，其余绑定与未知字段保持；首次失败与最终 strict47 CHECK-OK 均留存。

## 验证与限制

原树重跑 verify745：13 步、74 例通过；主检出 root check 通过（既有 warning 保留），strict47 CHECK-OK。最终第七轮独立复审的 114 passed/1 ignored、独立 74/74 与块流 N01..24 等证据仍有效，见 [REVIEW](../review-r7/REVIEW.md)。沉淀没有生产源改动。

到期批回归在主检出单实例串行执行；窗口 3668 路径中 trans20/book47、aavm触发0，因此跑 tf/tt/tb，无 taa。auto-plan-regress 在已安装技能根及仓库未找到，依 AGENTS 明确档位直接执行并记录缺失，不冒称调用该技能。

| 档位 | 总数 | 通过 | 失败 | 跳过 | 测试段秒 |
|---|---:|---:|---:|---:|---:|
| tf | 5994 | 5963 | 31 | 512 | 144.606 |
| tt | 5342 | 5308 | 34 | 1782 | 227.100 |
| tb | 5027 | 4951 | 76 | 1534 | 87.583 |

三档 exit100，均完成、无 timeout。逐名旧日志匹配分别28/33/21；未匹配不能自动视为历史红。[batch-attribution](batch-attribution.json) 保留真实集合，a2vue 金样旧红另有上一批 receipt 依据。stage3 子进程收尾、SSE 时序两项在单 job 隔离各1/1通过；temp_plan009 的硬编码 auto-term 输入不存在，未创建或写外仓。745 所有编译器实现均在独立 experimental/ac-core；根生产 crates、配置、Cargo/lock、auto-lib 在落地前后无差异。这支持“非745生产改动引入”，不声称根档全绿或严格同外部输入基线对拍。

tb 的55个 book 失败均为金样 mismatch，并非上一批七个 parse 失败。未 ignore 的 `generate_book_expected` 同批 PASS 且会写 `D:/autostack/book/rust/listings/**/main.expected.rs`，mismatch 测试也写 main.wrong.rs；并发验证的外部 oracle 可变。book HEAD `62e075b90b5f02fedc7aff42c05879ebe310bf6a`，运行后状态见 [external-book-audit](external-book-audit.json)。未捕获外仓运行前快照，不能逐件判定原 WIP 与此次生成差异，保留现存修改，未 reset/commit 或重新生成来制造绿色。P745-MERGE-ENV-01/VERIFY-01 已入债务簿；这是批量测试环境/验证债务，不改745已审合同。

## 派生账本

实际 `.autoos/specs.json` 在 Git 跟踪，与 README 的 ignored 描述不同。首次按运行态发布的七项投影经读回验证后，及时转回原树提交；不将 ignored 备份强行入库。保留初始本地恢复副本 `.autoos/specs.745-r1.active.before.json` 与前驱 Git 版本。首次 Windows 文件句柄阻止 atomic replace 时，原 bytes 未变；释放读句柄、保留独占操作锁、核验无服务写者及准确前 hash 后成功。

复用 P741-2/P743-1/P743-2，稳定新增 P745-1..4；canonical 当前项指 Specs，review/report 保留原 review SHA/冻结 SD/交付 SHA，归档后改指本 Plan。既有六节及所有不相关项/未知字段保持，不迁移整个旧格式。采用当前 musk SpecItem owned-view 校验、旧本地 lowercase/date 兼容投影；并非声称整份旧 ledger 能直接按新 Rust schema 反序列化。active version7/hash `8aceebb8bd97bf4bc77cef4402409ad52e76ab454b7a033f46bb2baad8f67539`，归档引用更新另加2版本；见 ledger/projection 收据。

## 归档与清理

仅在已落地、账本准确读回后归档。真实归档位置、相对链接、三导航745身份、九任务计数及 ledger canonical/history 身份通过实际状态检查后才清树。恢复外部缺失 wt-guard，以逐层 no-follow 扫描拒绝所有 reparse point；drive-root 与真实 junction 负控均拒绝。源脚本/原始 guard 输出入包，外部入口恢复供此次清理使用。

清理须原树及只读 auto-down 独立新鲜 guard clean、路径归属及 Git clean/已落地核验；只删除普通空组目录。最终 outcome、归档任务9/9与三项负控以 assertions-final.json 和 Plan 最后 merge 记录为准。
