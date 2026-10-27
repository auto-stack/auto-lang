# PLAN-824 r8 修复独立复审

- outcome: pass；reviewer: /root/review824_sync（用户授权独立只读上下文）。
- reviewed: 6426c8706987cf3a4c772d69e6605069b0aac83a；parent: 73b998249ebf9773a73431c3aa4c8da57919009f；原修复4bb9e7e18同步后19/19 range-diff等价。
- F-R8-01 CLOSED：默认P95<1000、显式824 P90<2000、无效策略拒绝；campaign与818真实子进程均证明三种环境及恰选1测试，正式脚本要求新增控制用例；默认真实campaign通过、824 incremental 6/6通过。
- F-R8-02 CLOSED：818 body/rebuild/conflict三条真实入口各1/1，独立复算日志与JSON中相同的30样本及P90/P95分别为265.602/284.109、725.4803/752.9488、269.1276/289.5045ms；原始数组均保存。
- 原R3及r8未变部分复审结论保留；新修复无新增规范语义变动，无新finding。
- 本轮未运行整档；合入前须在同一依赖/配置下执行acceptance、裸cargo t和适用cargo tv，保留性能原始样本和机器状态。
