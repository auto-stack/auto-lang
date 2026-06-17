# Plan 325: AutoVM 基础缺陷修复（enum 方法 + 跨模块字符串 + print）— 高优先级

> **类型**：Bugfix（基础缺陷，**高优先级**）
> **严重度**：阻断性——比 Plan 316（#[api] panic）更基础，**阻塞所有后端 Auto 代码**（不只是 IO 层）
> **来源**：auto-musk Spec 数据模型层实现（路径 A 后端逻辑先行）实测发现
> **复现 commit**：auto-lang `17118eab`，auto.exe 构建 2026-06-16 16:58

---

## 背景与影响

auto-musk 决定用 AutoVM 脚本运行模式做后端。为绕开 #[api] server panic（Plan 316），我们尝试"后端逻辑层先行"——用 AutoVM 跑纯逻辑代码（不含 HTTP/SSE）。结果在**最基础的后端数据模型测试**上连续撞到 3 个 AutoVM 缺陷。

**核心结论**：AutoVM 当前在"跨模块 + enum + 字符串"这个组合下不稳定，连"跨模块调一个返回字符串的函数"都不可靠。这**不只是 #[api] 那一个点的问题，而是 AutoVM 基础成熟度问题**，阻塞了 auto-musk 的**所有**后端 Auto 代码（逻辑层和 IO 层都受阻）。

---

## 缺陷 1【最基础】：enum 实例方法不被调用

### 现象
enum 上定义的实例方法（`fn name() { is self {...} }`，隐式 self）**调用后方法体根本不执行**，直接返回 None。

### 最小复现（`enum_method_bug.at`）
```auto
pub enum Color {
    Red
    Green

    fn name() str {
        print("  [inside name method]")   // 这行【从不打印】
        return "called-ok"
    }
}

pub fn main() {
    let n = Color.Red.name()
    print(n)   // 输出 None（期望 called-ok）
}
```

### 实际输出
```
calling Color.Red.name():
result:
None
```
方法体内的 `print("inside name method")` **从未执行**，证明方法体未被调用；返回 None 而非 `called-ok`。

### 证据与对照
- stdlib `stdlib/auto/result.at:19-24` 的 `Result<T,E>` 用同样的 `fn is_ok() bool { is self {...} }` 写法。若这些方法同样不工作，则 stdlib 的 Result/May/Cmp 的实例方法在 AutoVM 下全部失效——影响面巨大，建议一并验证。
- `is self` 本身**没问题**：模块级函数 + `is` 匹配 enum 参数能正常工作（见缺陷 1 的绕过验证）。问题专属于"enum 实例方法"这个绑定机制。

### 影响
auto-coder 蓝本（`coder/forge/specs.at`）和 auto-musk 的数据模型，所有 `to_str`/`from_str`/`as_str` 都依赖 enum 方法——全部失效。

---

## 缺陷 2：跨模块（use）字符串返回错乱

### 现象
跨模块（`use specs`）调用返回字符串的函数，返回值损坏（`<invalid string index: 19>`）。

### 最小复现（`cross_module_str_bug.at`）
```auto
// specs.at（被 use 的模块）
pub enum SpecStatus { Empty Proposed Draft /* ... */ }

pub fn status_to_str(s SpecStatus) str {
    is s {
        SpecStatus.Empty -> return "empty"
        SpecStatus.Proposed -> return "proposed"
        SpecStatus.Draft -> return "draft"
    }
    return "draft"
}

// main（use specs）
use specs

pub fn main() {
    let s = status_to_str(SpecStatus.Proposed)
    print(s)   // 期望 proposed
}
```

### 实际输出
```
direct print s2:
<invalid string index: 19>
```
（`<invalid string index: 19>` 是 AutoVM 内部字符串操作错误的泄露）

### 关键对照
- **同模块内**字符串函数正常：缺陷 1 的绕过测试里，`color_name(Color.Red)`（同文件、模块级函数）正确返回 "red"。
- 问题专属于**跨模块**（`use specs` 后调用 specs 的字符串函数）。

### 影响
任何"模块定义数据模型/字符串转换 + 另一模块 use 它"的结构都不可靠——这是组织后端代码的基础模式。

---

## 缺陷 3：print 字面量重复 + 字符串拼接取错值（跨模块场景）

### 现象（与缺陷 2 同场景）
```auto
use specs
pub fn main() {
    let s1 = section_type_as_str(SectionType.Goals)
    print("direct print s1:")    // 打印了【两次】自己
    print(s1)
    print("got=" + s1)           // 得到 "got=direct print s1:"（取了 print 的字面量而非 s1）
}
```

### 实际输出
```
direct print s1:
direct print s1:        ← 重复
direct print s2:
<invalid string index: 19>
concat:
got=direct print s1:    ← 拼接取错值
```

### 影响
基础 IO（print）+ 字符串拼接在跨模块场景损坏。这让任何带输出的调试/测试都不可信。

---

## 附带观察：类型重复注册警告

每次跨模块 `use specs`，都打印：
```
Warning: Failed to register generic template 'SpecItem': Generic type 'SpecItem' already registered
```
（SpecItem/SpecsSection/SpecsDocument/SpecChange 各一次）

虽非致命，但说明模块加载有重复注册问题（可能是缺陷 2/3 的根因之一——重复注册导致符号/字符串表错乱）。建议修复时一并排查。

---

## 三个缺陷的共性

缺陷 1/2/3 都指向 **AutoVM 的模块系统 + 类型/字符串表管理在跨模块边界不可靠**。可能是同一根因（模块加载/符号注册的内存管理 bug）的不同表现。建议**作为一组排查**，而非三个独立 bug。

### 优先级判断
- **比 Plan 316（#[api] panic）更基础**：316 阻塞 IO 层（HTTP server），本组缺陷阻塞**逻辑层**（连纯数据模型测试都跑不动）。
- 修复本组缺陷是 auto-musk **任何后端工作**的前提（无论逻辑层还是后续 IO 层）。
- 建议与 316 并列为最高优先级，甚至先于 316（因为逻辑层工作量更大、更早需要）。

---

## 修复后请验证（auto-musk 阻塞项）

1. 缺陷 1 的复现脚本：`Color.Red.name()` 返回 `"called-ok"`，方法体 print 执行。
2. stdlib 的 `Result.is_ok()`/`May` 等实例方法是否本来就能工作（若不能，是更广的回归）。
3. 缺陷 2 的复现：跨模块 `status_to_str` 返回正确字符串。
4. 缺陷 3 的复现：print 不重复、拼接取对值。
5. **端到端**：auto-musk 的 `src/back/specs_test.at`（在 auto-musk 仓库）全绿——这是数据模型层的完整测试，覆盖 SectionType/SpecStatus 往返、SpecItem/SpecsDocument 工厂、tags 字段。

---

## auto-musk 侧的相关文件（修复后用于回归）

- `D:\autostack\auto-musk\src\back\specs.at` — Spec 数据模型层（已实现，用模块级函数绕过了缺陷 1，但缺陷 2/3 仍阻塞测试）
- `D:\autostack\auto-musk\src\back\specs_test.at` — 数据模型测试（全绿即本组缺陷已修）

注：auto-musk 已决定，待本组缺陷修复后，specs.at 的模块级函数写法（绕过缺陷 1）可以保留（它本就是合理的函数式风格），或回退为 enum 方法（若缺陷 1 修复且 enum 方法更符合语言惯例）——由 auto-musk 届时决定。

---

## 复现环境

- auto-lang commit `17118eab`，`target/debug/auto.exe`（2026-06-16 16:58 构建）
- auto-musk 的 specs.at / specs_test.at 在 `D:\autostack\auto-musk\src\back\`
