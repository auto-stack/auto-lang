这是一份关于 **Auto 语言 (.at)** 标准库跨平台架构的设计规范。该方案的核心在于通过 `ext` 关键字实现 **“编译期符号缝合”**，允许在保持公开接口一致性的前提下，针对不同目标平台进行异构的物理填充。

---

# Auto 语言标准库架构规范：多平台填充机制

## 1. 核心哲学：接口契约与物理补全

Auto 语言将类型的定义拆分为两个阶段：

1. **契约阶段 (Definition)**：在主文件（`.at`）中声明公开字段和方法签名。
2. **补全阶段 (Refinement)**：在平台特定文件（`.vm.at`, `.c.at` 等）中通过 `ext` 关键字填充物理成员和逻辑实现。

---

## 2. 编译期缝合流程 (Symbol Stitching)

编译器根据 `-target` 参数，在逻辑上将分散的文件合并为一个统一的编译单元（Module Unit）。

### 2.1 物理补全规则 (The `ext` Power)

在 Auto 语言中，`ext` 关键字具有根据作用域变化的“二元能力”：

* **同模块填充 (Intra-module Filling)**：
若 `ext` 与 `type` 定义位于同一文件夹（同模块），`ext` 拥有**物理修改权**。它可以为类型添加私有成员变量（字段），从而改变该类型在当前平台下的内存布局。
* **跨模块扩展 (Extra-module Extension)**：
若 `ext` 位于外部模块，它仅拥有**逻辑扩展权**。只能添加方法，严禁添加任何成员变量，以保证 ABI 的安全性。

---

## 3. 标准库实施案例：`auto.io.File`

### 3.1 主接口文件：`stdlib/auto/io.at`

定义跨平台通用的最小集。

```rust
[pub]
type File {
    #[pub]
    path str
    // 此时 File 只有 path 一个字段
}

ext File {
    #[pub]
    static fn open(path str) ?File

    #[pub]
    fn read() ?str

    #[pub]
    fn close()
}

```

### 3.2 平台实现 A：`stdlib/auto/io.c.at`

针对 C 转译器目标。在这里，我们利用 `ext` 填充 C 语言特有的指针。

```rust
[target(c)]
use c.stdio: FILE, fopen, fclose, fgets

ext File {
    // 物理填充：为 File 增加一个私有的 C 指针成员
    _fp *FILE

    #[pub]
    static fn open(path str) ?File {
        let f = fopen(path.to.cstr, c"r")
        if f == nil { return nil }
        
        // 初始化时，编译器允许对填充的私有成员赋值
        return File(path: path, _fp: f)
    }

    #[pub]
    fn read() ?str {
        // 使用填充的私有成员 _fp
        let buf [1024]u8
        if fgets(buf.at.0, 1024, ._fp) == nil { return nil }
        return buf.to.str
    }

    #[pub]
    fn close() {
        fclose(._fp)
    }
}

```

### 3.3 平台实现 B：`stdlib/auto/io.vm.at`

针对解释器目标。此时 `File` 只需要一个索引值（Handle）。

```rust
[target(vm)]
ext File {
    #[pub, vm]
    static fn open(path str) ?File

    #[pub, vm]
    fn read() ?str

    #[pub, vm]
    fn close()
}
```

此时标注里的`#[vm]`项，表示这几个函数或方法需要由VM环境（即解释器提供）

---

## 4. 私有成员填充的约束与特性

### 4.1 内存布局的异构性

* **编译期确定性**：虽然 C 版和 VM 版的 `File` 内存布局不同（一个是 `str+*FILE`，一个是 `str+u64`），但因为同一编译任务只能指向一个 Target，所以不会产生冲突。
* **隐藏性**：外部用户导入 `auto.io.File` 后，通过 IDE 或反射只能看到 `path` 字段，填充的 `_fp` 或 `_handle` 对用户完全透明。

### 4.2 初始化语法 (Constructor Desugaring)

当 `ext` 增加了成员后，该类型的默认构造函数会自动展开以包含这些新成员。

* 在 `file.c.at` 中，构造函数签名为 `File(path, _fp)`。
* 由于这些字段是 `[internal]`，只有同模块的 `open` 函数有权调用此完整构造函数。

---

## 5. 方案总结：为什么使用 `ext` 填充？

1. **消灭句柄转换**：不需要在 `u64` 和 `void*` 之间进行繁琐的强制类型转换，代码更具类型安全性。
2. **极致性能**：在 C 目标下，`File` 结构体直接包含 `FILE*` 指针，访问速度与原生 C 代码无异。
3. **零开销抽象**：用户感知不到底层差异，开发者不需要为多平台编写复杂的宏（`#ifdef`）。

---

## 6. 编译器实现要点 (For a2c Transpiler)

1. **收集阶段**：扫描所有属于该模块的文件，识别出 `type` 块和所有合法的 `ext` 块。
2. **合并布局**：计算 `Primary Type Fields + Refined Fields` 的总和，生成最终的 C `struct`。
3. **方法注入**：将所有的 `ext` 方法翻译为 C 函数，并自动将 `File` 指针作为第一个参数注入。

---
