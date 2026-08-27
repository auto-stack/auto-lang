---
plan_id: PLAN-461
status: in-progress
feature_name: Python 科学计算库（numpy/pandas/matplotlib/torch）use.py 调用 parity
author: [zhaopuming]
created_at: 2026-08-28T00:30:00+08:00
updated_at: 2026-08-28T00:30:00+08:00

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 0
total_steps: 8
---

# [PLAN-461] Python 科学计算库 use.py 调用 parity

## 变更摘要

在 Plan 214/300/369 建成的 Python FFI（内嵌 CPython + `use.py` +
`PyObjectHandle` + `py_call`/`py_getattr` 内建）之上，为四个常见科学计算库
**numpy / pandas / matplotlib / torch** 建立 `libs/python/` 下的三方 parity
用例（AutoVM vs a2py vs 原生 Python oracle），验证 AutoVM 可以直接调用
Python 科学计算生态，并把可用的调用惯用法固化为文档与测试基线。

## 目标 / 架构方案

### 定位

不新造任何 FFI 机制。本计划是"既有桥的能力验证 + 惯用法固化"：沿用
py_math 等已验证的 Python-parity 套件结构（`tests/auto/<mod>.at` + 
`tests/python/test_<mod>.py` + README，TAP 双端对名）。

### 调用粒度约定（写入 README 与用例设计）

- **数据生在 Python 侧、活在 Python 侧**：数组/表/张量全程以
  `PyObjectHandle` 存续，仅标量/字符串跨越封送边界；
- 用例结果一律收敛为标量或确定性字符串后再断言（沿用 py_math 的
  `.to(int)` / `.to(str)` 精确比较惯例）;
- 全部用例确定性：不用随机、不用时间、不用依赖版本的格式化输出
  （如 pandas `describe()` 的表格渲染）。

### 两个候选调用惯用法（T2 探针定案）

- A：`use.py numpy: arange, sin, sum` 项导入直呼（py_math 已验证）；
- B：`use.py numpy` 无 items 取模块 + `py_call(numpy, "arange", ...)` /
  `py_getattr(arr, "T")`（Plan 369 Task 12 内建，覆盖方法链与下标）。

### 已知风险与预案

| 风险 | 预案 |
|---|---|
| a2py 不下沉 `py_call`/`py_getattr` | 惯用法 B 的用例暂缓，先落仅用 A 可表达的子集；补 a2py 下沉作为后续小计划 |
| `sum`/`max` 等导入项与 Auto 内建同名冲突 | 探针验证；冲突则改用 B 或换名（`np.sum`）|
| torch 导入 10-20s 拖慢 runner | 用例数控制在 ≤8；README 注明 |
| matplotlib 渲染非确定性 | 固定 Agg 后端，只断言 `savefig` 产物存在且 PNG 魔数正确（`os.path.getsize`/读文件头经 use.py os）|

### 新增目录（沿用 Plan 460 分类结构）

- `libs/python/py_numpy/` — 纯函数与归约：sin/sqrt 标量、arange.sum/mean/max、
  list→array→sum 往返、dot
- `libs/python/py_pandas/` — DataFrame 构造 + 列聚合标量（sum/mean/count）
- `libs/python/py_matplotlib/` — Agg + plot + savefig + 产物断言（1-2 例）
- `libs/python/py_torch/` — 张量标量运算 + `.item()` 收敛（≤8 例）

### 工具与文档

- `main.rs` phase 表新增 `p8`（四库），使 phase 视角覆盖科学计算批次；
- `parity/README.md` python 类表格补 4 行；`parity-guide.md` phase 表同步；
- `.gitignore` 增 matplotlib 产物目录通配（`libs/*/*/py_matplotlib_tmp/`
  及平铺变体）。

## 测试设计与验收标准

- 每库三方（AutoVM / a2py / 原生 Python）`run` 全绿；
- `phase p8` 一次跑通四库；
- `list` 计数 34→38；
- parity workspace `cargo test` 保持 33 绿（phase 表为纯数据变更）；
- README/guide 与实际用例一致；无 TODO/debug print 残留；
- 惯用法结论（A/B、坑、封送行为）写入 `libs/python/` 各 README 与
  `parity/README.md` 的 python 类说明。

## 执行步骤

- [ ] T1 计划文档 + worktree（branch plan-461）
- [ ] T2 语法/封送探针：numpy 小样，定 A/B 惯用法、同名冲突、句柄方法链
      与 a2py 支持面
- [ ] T3 py_numpy 用例 + oracle + README
- [ ] T4 py_pandas 用例 + oracle + README
- [ ] T5 py_matplotlib 用例 + oracle + README（含 .gitignore）
- [ ] T6 py_torch 用例 + oracle + README
- [ ] T7 phase p8 注册 + parity/README + parity-guide + 索引表更新
- [ ] T8 全量验证（run ×4、phase p8、cargo test、list）+ 复审 + 合并归档
