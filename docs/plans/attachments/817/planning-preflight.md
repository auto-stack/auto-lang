# PLAN-817 r1 起草预检

2026-10-19，stage:new，outcome:pass（契约草案，非实现或独立验收通过）；next:work，T-01可准备，T-02起需D-798及公共注册窗口。drafting 0/10，未执行cargo/性能/资源测试或创建实施树。

## 核查结论

- 使用auto-plan-new及verification-contract；以现状Spec、796交付源码为事实，以Design42/43为目标。798仅只读分支预测，已明确不能充当合入API；来源47份工作区文件和5份branch blob的hash见批次inputs.json。
- 817取号与818由同一mutex+allocation.lock保护，调用new-plan.sh，扫描active/archive一号一文件；max816→817/818，next819。只写本批计划/附件与编号，不提交其他WIP。
- CT执行与796模型分域：admit_as_model不改，真实执行producer及frozen Schema独立；函数输入片段检查→AV→结果回灌consumer，而不是全程序Checked后才能CT。
- 首版i32/bool纯callee与受控FieldCount/HasField参数来源；consumer严格限796有限推导面，CtRef属于recipe IR，不声称生产元编程语法/全量泛型或类型生成已接入。
- 第一个可执行例在T-02，必须同时有真AV、store/restart、consumer回灌、正式AutoDown和span/峰值；T-01只读准备不伪装“首例已经完成”。无注册窗口时不勾T-02。
- 817没有818/797前置，native观察用旧HIR-native；798/796/783只消费，不复制其实现。公共recipe与SDK/工具拆分写面单写，依赖未满足不叠加分支开工。
- 源码不变但helper、Meta集合/负依赖、引擎/权限/target语义变更均有反控；hit禁调AV且仍消费重验，miss必须实际执行，model_compute不参与。
- 32MiB受管/384MiB worker/512MiB请求树为拟议硬预算；分配前计量与峰值/瞬态spike反控、真正不同冷页和512修订+超过水位真实删除都绑定任务。前端/子进程不得扣掉后宣布目标达标。
- 256KiB/100函数、六场景30样本和helper/Meta编辑P95<1s，窗口含准备/IPC/consumer准入/比较；native参考另报，不冒称完整应用编译/热重载亚秒。阈值0和坏raw使gate红。
- 0–10章节、T/AC-01–10、V-01–11、SD-01–05覆盖；8 work+1 review+1 merge，merge-owned门禁与归档清理保持未来项。新代码/命令是未来入口，非测试证据。

## 未决和边界

T-01在本范围内固定recipe/consumer双向绑定、版本及精确预算口径；T-02证明真实AV/冻结/推导/正式runner全路径，失败带具体证据回规划，不用mock/提高阈值补位。完整Auto语法/类型生成/泛型/JIT/发布器及生产迁移不在本合同。

本轮验证仅文档章节/引用/唯一编号/任务及验收对应、相对链接、编码和空白。用户授权起草，不实施、不联系其他会话；规划pass不等于功能、性能或内存已通过。
