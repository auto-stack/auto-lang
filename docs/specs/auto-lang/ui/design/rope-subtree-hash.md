# rope 子树内容哈希与全等查询面

> PLAN-703 T-01（供②，2026-09-27）。auto-edit 下游战略 §2.3「差异侧直
> 接编辑重比」+§4.2「增量一切：diff 增量重比」的地基件——「两 rope 子
> 树全等」判定达 O(1)/O(log n)。落点
> `crates/auto-lang/src/ui/code_editor/core/rope.rs`。

## 摘要契约

每个 rope 节点缓存 64 位**可复合多项式摘要**（Mersenne 素域 2^61−1）：

- **叶节点**：`H(text)` = 逐字节 `h = h·B + byte mod P`，哨兵初值 1
  （使前导零字节显著）。
- **内部节点**：拼接规则 `H(x‖y) = (H(x)−1)·B^|y| + H(y) mod P`——纯
  函数、序敏感、长度敏感。
- **基**：`B = 0x9E3779B97F4A7C15 mod P`（黄金比例奇常数）——**编译期
  约化入域**（mulmod 的 Mersenne 折叠要求两侧操作数 <P；未约化基数曾
  致乘积越 2^122 折叠溢出，实测踩中）。
- **确定性**：固定常数、零进程内随机种子——同输入跨运行/跨进程同摘
  要（diff golden 双跑字节等的基石）。

### 语义性质

| 性质 | 内容 | 用途 |
|---|---|---|
| 内容决定 | 同字节序列 ⇒ 同摘要，**与树形无关**（叶切分/旋转历史不影响） | 「再编辑回原文」判定 O(1) 回绿；跨形状剪枝 |
| 可复合 | 区间摘要 = 分块摘要按拼接规则折叠，恒等于平铺内容摘要 | `range_hash` O(log n) 分件复用 |
| 碰撞域 | 随机输入 ~n/2^61（非对抗性假设；基数固定为跨运行确定性的代价） | Merkle 快路的概率担保 |

### 维护不变式

摘要随**既有编辑路径零成本维护**：全部节点构造点（`Node::leaf` /
`Node::internal` / split / concat / rebalance 旋转）统一重算——无陈旧
路径；编辑复杂度维持 O(log n)（叶 O(k) 哈希与既有文本拷贝同阶）。

## 查询面

| 面 | 签名 | 语义 | 复杂度 |
|---|---|---|---|
| `content_hash` | `Rope/RopeSnapshot → u64` | 根摘要 O(1) 读；空文档=哨兵 1 | O(1) |
| `range_hash` | `(start, end) → u64` | 字节区间摘要=平铺内容摘要（可复合性）；全区间==根摘要；空区间=1 | O(log n) 分件+O(叶) 边界 |
| `subtree_equal` | `(&other) → bool` | **精确**内容全等：ptr_eq（同 Arc 同偏移）快路→整节点摘要+summary 快路→对齐双指走查（双边界三段分解） | O(1) 共享 / O(divergence spine) |
| `prune_spans` | `(&other) → PruneSpans`（pub(crate)） | 对齐走查出 shared/diverged 四元组（绝对坐标）——diff 引擎快照剪枝的预处理面（T-04） | O(1) 共享 / O(diverged region) |

### subtree_equal 判定纪律

判定**精确不猜测**：快路命中即真（结构共享/摘要+长度+summary 全等），
快路未中落入逐叶精确比较——假阳性仅剩摘要碰撞域（2^61 分之 n），假阴
性不存在。同内容异形（编辑后回退）由摘要内容决定性直接 O(1) 判等。

### prune_spans 语义边界（v1）

shared/diverged 为**位置对齐**分类：diverged 跨度是「同位置内容可能不
同」的保守包络。跨编辑的结构共享（同 Arc 移位出现）**不做移位追踪**——
ptr_eq 快路在 prune 走查中弃用（同 Arc 异位时记录的绝对坐标失真，实测
教训），整节点摘要快路承载剪枝载荷。剪枝漏检仅损性能不损正确性（diver
ged 区交行级 diff 完整处理）。移位追踪重同步=后续增量重比件的留痕面。

## 验证锚

- `cargo test -p auto-lang rope`：differential 随机对照（每 op 根摘要+
  随机区间==平铺多项式）；AC-01 三族（共享/分叉/再编辑回全等）；快照
  摘要冻结；naive 分类器对照（prune spans 共享集必须是 naive 共享 run
  的子集且逐段内容相等）。
- 编辑路径零回退：既有 rope 单测族全绿（differential 600 op×4 种子等）。
