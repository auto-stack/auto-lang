# PLAN-818 golden.md — T-01 独立期望（手审定稿）

2026-10-20，work stage T-01。本文件冻结首例与参考负载的**独立期望**：由
人工按冻结的源语义推演得出，**不经任何后端**（AV、旧 HIR-native、新
MIR-native 均不参与期望的生产）。mark 内建的返回值与 stderr 字节是落地
test-support 合同的冻结事实（`experimental/ac-core/test-support/src/lib.rs`
与 `marks.rs` 字节一致）：`mark_a()` 写 `"a"` 返回 `1`；`mark_b()` 写
`"b"` 返回 `2`；`extern "system"`、kernel32 直调、无 CRT。

后续任务把本文件作为 oracle：新 backend 不生成 expected；三路径（AV、
新 native、旧 native）的 actual 都与本文件核对，不只比较两后端同错。

## 1. 首例（T-02 vertical，源文本冻结）

### 1.1 源文本（冻结，逐字节）

```text
fn gate(v int) int {
    let a = mark_a()
    let t = v * 3 + a
    if t > 40 {
        return t - mark_b()
    }
    return t + a
}

fn main() int {
    let x = gate(9)
    let y = gate(15)
    if x + y != 73 {
        return 1
    }
    return 0
}
```

用例侧提供能力 `hir.test.trace.v1`（源自身只含调用；能力由 case 绑定提供，
与 798 trace parity 用例同形态）。native 腿链接落地 `ac-trace-support`
staticlib（无其他 Rust staticlib 在链中，无重复 panic 符号）。

### 1.2 手算过程（每步列出依据）

**gate(9)**：
1. `mark_a()`：stderr 追加 `'a'`，`a = 1`。
2. `t = 9 * 3 + 1 = 28`（乘先于加；无溢出）。
3. `28 > 40` 为假 → 跳过 then 臂。
4. `return t + a = 28 + 1 = 29`。

**gate(15)**：
1. `mark_a()`：stderr 追加 `'a'`，`a = 1`。
2. `t = 15 * 3 + 1 = 46`。
3. `46 > 40` 为真 → 进 then 臂。
4. `return t - mark_b()`：lhs `t`（46）先求值（无副作用），rhs
   `mark_b()` stderr 追加 `'b'`、返回 2 → `46 - 2 = 44`。

**main**：
1. `x = 29`，`y = 44`。
2. `x + y = 73`；`73 != 73` 为假。
3. `return 0`。

**trap 无涉审计**：全程出现的值为 {0,1,2,9,15,28,29,44,46,73}，远离
±2^31；无除/余运算；乘法 9×3、15×3 无溢出 → 任何合法执行不可能 trap。

### 1.3 冻结期望（oracle）

| 观察面 | 期望 |
|---|---|
| 进程退出码（完整 u32） | `0` |
| stderr 字节（源序） | `"aab"` = `61 61 62` |
| stdout 字节 | 空 |
| AV 腿 | `Success{ value: I32(0) }`，trace 事件序 `a,a,b`，无 trap |
| 旧 native 腿 | 真 PE：退出码 0、stderr `"aab"` |
| 新 native 腿（T-02 判定项） | 真 PE：退出码 0、stderr `"aab"`；与 AV/旧 native/本表一致 |

MIR/COFF 的内部形状**不是** golden 的一部分（798 adapter 的 temp 槽布局
非冻结面，待 D-798）；observable 行为（上表）才是。

### 1.4 首例附带的非行为断言（T-02 一并判定）

- cache：真实 store put（A08 经 798 交付面、A09' 新 COFF）→ **新进程**
  worker 恢复 → 命中时 `codegen_emit_units = 0`（禁调反控）且仍真 link/
  run/父侧比较。
- 正式 AutoDown 首例：`ac/mir-native-scalar-v1` recipe 走通 probe/run、
  atom-data 输入、父侧准入/比较。
- 计量：请求/编译/link/run 分段 trace 与内存峰值首份 raw。

## 2. 参考负载（T-05 冻结实现的设计）

### 2.1 形状（冻结）

- **14 个 leaf** + **100 个 caller**（`fn_000`..`fn_099`）+ `main`；
  无空白填充。
- leaf 模板（冻结）：
  ```text
  fn fn_leaf_N(v int) int {
      if v < 0 {
          return 0 - v
      }
      return v + N
  }
  ```
- caller 模板（冻结，K = **19** 个混合块/函数）：
  ```text
  fn fn_III(x int) int {
      var acc = x * A0
      # 以下块 × 19（常数由 LCG 按发射序抽取）：
      if acc > B {
          acc = acc - B
      } else {
          acc = acc + C
      }
      for acc > E {
          acc = acc - F
      }
      acc = acc + D - D
      # 终端（L = i mod 14 + 1）：
      return fn_leaf_L(acc)
  }
  ```
- main：`var total = 0`，随后 100 行 `total = total + fn_III(X)`（X =
  i mod 7 + 1），`return total`。
- 确定性：LCG(seed = `0x818_5eed`)，抽取序 = 发射序：每 caller 先 A0，
  再按块序 B、C、D、E、F。输出必须字节稳定（cache key 依赖）。范围冻结：
  A0∈[3,40]、B∈[2,60]、C∈[1,20]、D∈[1,25]、E∈[1,30]、F∈[1,12]。
- 规模义务：总源 ≥ 256 KiB（262 144 字节）；K=19 按上述模板估算 ≈270 KiB，
  生成时 assert 实测值 ≥ 262 144，**不得缩 K/函数数/模板来过门**。

### 2.2 无 trap 界分析（手审）

x ≤ 7、A0 ≤ 40 → acc₀ ≤ 280。每块：if 臂 |Δ| ≤ 60；for 环每步减 F ≥ 1
且出口 acc ≤ E ≤ 30（入口已 ≤ E 则不执行）；`+D - D` 抵消。19 块下
acc ≥ 280 − 60×19 = −860，leaf 后 |值| ≤ 860 + 14，main 合计
≤ 100 × 874 = 87 400 ≪ 2^31；无除/余 → **无溢出/除零可能**（base 负载
的 trap 无涉是设计属性；除/余/溢出探针属 T-03 反例族，不入本负载）。

### 2.3 期望推导规则（冻结）

- 期望由生成器的**闭式镜像**推导：与源发射同步用普通 Rust 算术重演同
  模板（if/else、while 折算、leaf 闭式、求和），**不经任何后端**。
- 冻结编辑位：
  - **body 编辑**：`fn_041` 的 A0 加 1（revision n：A0 += n）——源与期望
    均随 n 变化，每样本一个新 n（798 r2 语义）。
  - **签名冲突编辑**：`fn_leaf_2` 增一个参数（`(v int, w int)`）——期望
    是**正确 frontend 诊断**（调用点全部破裂），非崩溃、非静默通过。
- cross-project 场景：同一负载在第二工程 namespace 下复用（内容同、
  project 轴不同）。

### 2.4 手审义务（T-05 回填）

生成器须能打印任一 caller 的常数清单（A0 与 19×{B,C,D,E,F}）。T-05 验收
时至少对 `fn_000` 与 `fn_099` 人工重演 acc 轨迹并与闭式镜像核对，审计
记录（常数清单 + 逐步 acc + 终值）归档进 T-05 回执；不一致即 gate 红，
不得调镜像迁就后端。T-01 先锚定两个模板级手审样例：

- leaf 手审样例：`fn_leaf_3(-5)`：v<0 → `0-(-5)=5`；`fn_leaf_3(5)`：
  `5+3=8`。
- 块轨迹手审样例（acc=50, B=10, C=4, D=7, E=9, F=2）：
  `50>10 → 40`；`40>9 → 38 → 36 → … → 8`（40−2k≤9 ⇒ k=16 ⇒ 8）；
  `+7−7` 抵消 → 8。

## 3. 独立性声明

- 本文件全部期望由人工依据冻结语义（源序求值、i32 检查算术、mark 合同
  1/2/`"a"`/`"b"`）推演；未运行 AV/旧 native/新 native/任何编译器。
- 后端实现的 actual 与本文件不符时，**以后端为疑**（先疑实现，再疑
  oracle，修订须记 golden revision 并重审全部已核对项）。
- 本文件不是性能基准数值的来源（P95/内存门见 plan §5.5 与
  contract-decision §7/§11）；它只冻结正确性 oracle。
