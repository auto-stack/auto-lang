# PLAN-818 r1 起草预检

2026-10-19，stage:new，outcome:pass（契约草案，非实现或独立验收通过）；next:work，T-01可准备，T-02起需D-798及公共注册窗口。drafting 0/10，未执行cargo/性能/资源测试或创建实施树。

## 核查结论

- 以AC/HIR/产物/TestCase Spec和现有native/link执行器为事实；798 MIR仅branch预测，正式接线必须等实际合入及Spec。来源hash及公共写面见批次inputs.json/parallel-contract.md。
- 与817同次排他取号：max816→817/818，next819，一号一文件；不改797/798状态、任务或验收，不触碰SDK/旧VM生产文件。
- 新路径直接消费CheckedMir生成CLIF/COFF，旧lower_object仅参照；首例T-02包含真实多obj/符号/平台link/PE、AV与旧native独立对拍及手审expected。不能用MIR→HIR绕旧后端冒充。
- private i32/bool调用/trace/trap实验ABI与797公共AutoABI分离；817/797都不是前置。完整heap、聚合、异常/公开FFI不支持，未知MIR版本不降级解释或NOP。
- 函数obj/symbol稳定身份与内容版本分离，body改动只重编适用单元，读取签名/类型/配置要入key；实际module级依赖不能漏记后伪称增量。命中真link/run，绝不复用PASS。
- 对象按小页落store，避免16MiB对象的一次双副本写超783的8MiB staging；A09/manifest恢复有内层和身份准入，MIR不是required时可回收后仍link合法对象。
- 失败事务用真实多个旧输出/旧程序观察，含link失败、超时、发布IO失败、子孙/暂存清理；不声称通用发布器或完整增量链接器已交付。
- 48MiB受管/512MiB worker/768MiB请求树为拟议硬门，Cranelift/前端/子进程全报；32MiB真实不同冷对象及512真实修订推动水位、验证实际删除/收敛，不能重复同一块或只测低占用。
- 正式AutoDown从T-02首例，T-06补全完整case/负控/兼容。公共注册/dispatch在797/798和拆分owner窗口串行，不新造执行器/裁判，测试核心零Cranelift依赖。
- 局部body/合法签名编辑请求到真实PE父侧准入/比较P95<1s，含准备/IPC/link/run/等待；参考路径独立正确性suite，性能主路径每样本核独立expected。阈值0/坏raw/spike都必须使gate红。
- 0–10章节、T/AC-01–10、V-01–11、SD-01–05覆盖；8 work+1 review+1 merge，执行树唯一且独立最终review后才跑merge-owned门禁。脚本/代码尚未创建，未以计划文本冒认执行证据。

## 未决和边界

T-01固定实际已交付MIR指令映射、内部ABI/symbol/工具与资源口径；T-02证明新codegen、多obj链接和cache/runner整条可行。798接口不足、预算/硬门失败需要修实现或有证据revision，不改旧native代跑、不上调阈值。公开AutoABI、SDK默认入口、AJ/热重载/发布器另立计划。

本轮只检查文档章节/编号/映射/链接/编码/空白，未编译、测试或跑基准；用户授权起草，不实施或联系其他会话。
