# PLAN-836 r1 — draft preflight

2026-10-23；stage: new；outcome: pass；next: work（用户启动后T-01/T-02）。仅静态合同检查，不是实施、独立验收或merge门禁通过。

- 835 archived 10/10，正式merge/cleanup/batch收据存在；R2冻结提交是autographics-dev正式落点祖先。
- 原子取号836，active/archive唯一；.next-id=837；排他allocation锁文件保留。
- 章节0..10、drafting r1、0/10；10个唯一任务（8work/1review/1merge）、9个AC、10个case、4个SD；任务/验收/规范关联已核。
- 20个不可变源码Git blob/SHA256核对通过；16个当前文档输入含未提交状态显式标记；本次观察不是实施冻结。
- 本地Markdown链接全部存在，新附件不被ignore规则屏蔽；无836树/分支，本轮仅写Plan/附件/.next-id，其他会话WIP保留。
- 合同核查：固定相机/0..1深度、2D真实形状/顺序、3D全mesh OBB、真实Auto临时状态/最后端点单提交、取消恢复、焦点/迟到隔离、schema1不变及有界owner一致。例行实现选择由T-02关闭，不假定新接口已有。
- 新tests/probe均为未来任务；未跑Cargo、原生窗口、图形/游戏或独立consumer验收。
- G835满足；GSOURCE由T-01核冻结，T-02先whole-path可行性，再实现。无须等待835合入。I2/I3、831/832启动锁、源码仅合autographics-dev边界保持。

Plan SHA256: 638d9cdef9e0c84bea8db314c1465ce28033e41a2abc5e14a6a5f1f837c11e3b
