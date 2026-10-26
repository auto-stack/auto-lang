# PLAN-822 SD 草案(T-06 整理;review 按 D 实际 Spec 最终确认,merge 才沉淀)

## SD-01 add docs/specs/auto-av/collections-variants.md

登记 `av.collections/1` profile:身份四元组(AVAD v3/profile id 4/producer plan822/语义版本 (1,1,1));支持矩阵(List 元素域、Variant payload 域、record 不含容器、一层容器上限);15 条指令(0x1D..0x2B)的状态/计量规则(epoch 失效、per-element 计费、转移/回滚);陷阱(list-index/stale-place/variant-branch)与资源(list-elems)分类;§5.4/V-11 集合/分支状态计量、§5.5/V-12 阶段计数与恢复规则。

## SD-02 add docs/specs/auto-mir/collection-place-profile.md

MIR 层合同:MirType::List/Variant(tag 13/14)与类型域检查(`mir.type.list-elem`/`variant-payload`/`nested-container`/variant 无环);guard 事实格点(must-intersect join;PACK/SWITCH/CALL 失效)、payload-moved(may-union);LHS reservation 沿 821;程序化 typed fixture 的未信任输入地位(真 verify 授信)。

## SD-03 modify docs/specs/auto-av/cache-and-conformance.md

A22 collections layer(依赖轴:producer/owner/entry/profile/types-fp/codec×3 语义/snapshot);恢复重验全链(envelope→decode→verify→execute,零信任盘面);好 envelope 坏 payload 拒;世代隔离(v1/v2/v3 互不可读);确定性重放(步数一致)。

## SD-04 modify docs/specs/auto-av/project.md

登记第四 profile 与明确非目标:无 source 面(fixture-only)、无 Map/Set/迭代器/借用元素容器、record 不含容器字段、List 不嵌 List、非 native/CTE。

## SD-05 modify docs/specs/auto-test-case/architecture.md

登记 `ac-av/collections-run-v1` recipe(avworker/1 域,collections:<id> fixture 入口)、av-data-822 语料身份、父侧冻结期望表(COLLECTIONS_CASES)与反控 seam(AV822_COLLECTIONS_EXPECT_SABOTAGE + AV798_FAULT 复用)。

## SD-06 modify docs/specs/auto-av/resident-function-install.md

沿 D828:未改函数跨修订同一 Arc 对象/代际不变(Identity reuse);collections 单元经 admit_unit→link 进入 snapshot;T-09 增量四子case 待完成后按实测登记。
