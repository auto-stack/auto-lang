# Plan 238: AAVM Bytecode VM — List/Map 数据结构支持

## Context

Plan 237 Phase C 已完成：AAVM 的 bytecode compiler (codegen.at) 和 interpreter (vm.at) 支持 int/str/bool 类型的基本运算。但 BVM 目前无法操作 List 和 Map 对象——这是自举的核心瓶颈（AAVM 自身的 codegen.at 大量使用 `List` 和 `Map`）。

**目标**：为 BVM 添加 List/Map 支持，使其能编译并执行包含数据结构操作的程序。

**架构决策**：BVM 运行在 Rust AutoVM 的 tree-walking evaluator 中，可以直接创建和操作真实的 Auto `List`/`Map` 对象。方案是给 BVM 添加 **heap 结构**（两个 `List`：一个存 List 对象，一个存 Map 对象），通过新的 type tag（2=list ref, 3=map ref）在栈上传递 heap 索引。

## 实施顺序

```
Step 1 (vm.at ~100行) → Step 2 (eval.at ~3行) → Step 3 (codegen.at ~60行) → Step 4 (tests)
```

---

## Step 1: BVM Heap + 新 Opcode — `auto/lib/vm.at`

### 新增 Type Tags

| Tag | 含义 | 栈上存储 |
|-----|------|---------|
| 0 | int | `v{N}` = int 值 |
| 1 | str | `s{N}` = string pool key |
| **2** | **list ref** | **`v{N}` = heap_lists 索引** |
| **3** | **map ref** | **`v{N}` = heap_maps 索引** |

### 新增 Opcode (8个, ID 64-71)

| Opcode | ID | 栈效果 | Auto 等价 |
|--------|-----|--------|----------|
| `OP_LIST_NEW` | 64 | `→ list_ref` | `List.new()` |
| `OP_LIST_PUSH` | 65 | `list_ref, val →` | `list.push(val)` |
| `OP_LIST_GET` | 66 | `list_ref, idx → val` | `list.get(idx)` |
| `OP_LIST_LEN` | 67 | `list_ref → len` | `list.len()` |
| `OP_MAP_NEW` | 68 | `→ map_ref` | `Map.new()` |
| `OP_MAP_INSERT_INT` | 69 | `map_ref, key, val →` | `map.insert_int(k,v)` |
| `OP_MAP_GET_INT` | 70 | `map_ref, key → val` | `map.get_int(k)` |
| `OP_MAP_CONTAINS` | 71 | `map_ref, key → bool` | `map.contains(k)` |

### 修改签名

```auto
// 之前
fn bvm_run(code List, strings Map, state Map) str
fn bvm_step(code List, strings Map, state Map) int

// 之后
fn bvm_run(code List, strings Map, state Map, heap_lists List, heap_maps List) str
fn bvm_step(code List, strings Map, state Map, heap_lists List, heap_maps List) int
```

### 新增辅助函数

```auto
fn bvm_push_list_ref(state Map, heap_idx int) {
    var sp = state.get_int("__sp")
    state.insert_int("v" + int_to_str(sp), heap_idx)
    state.insert_str("s" + int_to_str(sp), "")
    state.insert_int("t" + int_to_str(sp), 2)  // type tag 2 = list ref
    state.insert_int("__sp", sp + 1)
}

fn bvm_push_map_ref(state Map, heap_idx int) {
    var sp = state.get_int("__sp")
    state.insert_int("v" + int_to_str(sp), heap_idx)
    state.insert_str("s" + int_to_str(sp), "")
    state.insert_int("t" + int_to_str(sp), 3)  // type tag 3 = map ref
    state.insert_int("__sp", sp + 1)
}
```

### 修复 Type Tag 传递（3处）

1. **DUP handler** (op==3): 当前 `else` 分支调用 `bvm_push_int` 丢失 tag 2/3
   → 需改为：如果 t==2 调用 `bvm_push_list_ref`，t==3 调用 `bvm_push_map_ref`，否则 `bvm_push_int`

2. **LOAD_LOCAL handler** (op==32): 同理，需要根据 t 值选择正确的 push 函数

3. **RET handler** (op==113): 返回值需要保留 type tag 2/3

### 新 Opcode Handler 示例

```auto
// OP_LIST_NEW = 64
if op == 64 {
    var new_list = List.new()
    var idx = heap_lists.len()
    heap_lists.push(new_list)
    bvm_push_list_ref(state, idx)
    return 1
}

// OP_LIST_PUSH = 65
if op == 65 {
    var val = bvm_pop_int(state)
    var heap_idx = bvm_pop_int(state)
    if heap_idx >= 0 && heap_idx < heap_lists.len() {
        var target = heap_lists.get(heap_idx)
        target.push(val)
    }
    return 1
}
```

**注意**：`heap_lists.get(heap_idx)` 返回的是 List 对象的引用（Auto VM 中 List 是引用类型），所以 `target.push(val)` 会修改 heap 中的原始 List。

---

## Step 2: 入口函数更新 — `auto/lib/eval.at`

修改 `run_bytecode()` 末尾 ~3 行：

```auto
fn run_bytecode(source str) str {
    // ... 现有的 tokenize/parse/typeinfer/codegen 代码 ...
    var state = Map.new()
    bvm_init(state)
    state.insert_int("__nstr", cg.n_strings)
    // Plan 238: heap for List/Map objects
    var heap_lists = List.new()
    var heap_maps = List.new()
    return bvm_run(cg.code, cg.strings, state, heap_lists, heap_maps)
}
```

---

## Step 3: Codegen 扩展 — `auto/lib/codegen.at`

### 新增 Opcode 常量函数（~8行）

在现有 `OP_HALT()` 后添加：

```auto
fn OP_LIST_NEW() int { return 64 }
fn OP_LIST_PUSH() int { return 65 }
fn OP_LIST_GET() int { return 66 }
fn OP_LIST_LEN() int { return 67 }
fn OP_MAP_NEW() int { return 68 }
fn OP_MAP_INSERT_INT() int { return 69 }
fn OP_MAP_GET_INT() int { return 70 }
fn OP_MAP_CONTAINS() int { return 71 }
```

### 扩展 `codegen_call()`（~50行）

在 `callee == "print"` 检查之前添加：

**匹配模式**（基于 AST 分析）：
- `List.new()` → `CallExpr(name="List.new")`, callee = `"List.new"`
- `list.push(5)` → `CallExpr(name="list.push")`, callee = `"list.push"`
- `list.get(0)` → `CallExpr(name="list.get")`, callee = `"list.get"`
- `list.len()` → `CallExpr(name="list.len")`, callee = `"list.len"`
- `Map.new()` → `CallExpr(name="Map.new")`, callee = `"Map.new"`
- `m.insert_int(1,100)` → `CallExpr(name="m.insert_int")`, callee = `"m.insert_int"`
- `m.get_int(1)` → `CallExpr(name="m.get_int")`, callee = `"m.get_int"`
- `m.contains(1)` → `CallExpr(name="m.contains")`, callee = `"m.contains"`

**策略**：检查 callee 的后缀匹配（`.push`, `.get`, `.len`, `.new` 等），提取变量名查找 local slot。

```auto
// 静态构造: List.new() / Map.new()
if callee == "List.new" {
    codegen_emit(cg, OP_LIST_NEW())
    return 0
}
if callee == "Map.new" {
    codegen_emit(cg, OP_MAP_NEW())
    return 0
}

// 实例方法: 提取变量名 + 方法名
// callee 格式: "varname.method"
var dot_pos = callee.find(".")
if dot_pos >= 0 {
    var var_name = callee.substr(0, dot_pos)
    var method = callee.substr(dot_pos + 1, callee.len())
    var slot = codegen_find_local(cg, var_name)
    if slot >= 0 {
        // list/map 方法
        if method == "push" || method == "get" || method == "len"
            || method == "insert_int" || method == "get_int" || method == "contains" {
            codegen_emit(cg, OP_LOAD_LOCAL())
            codegen_emit(cg, slot)
            // 编译参数
            var i = 0
            var pn = node.params.len()
            for i < pn {
                codegen_expr(cg, node.params.get(i), tenv)
                i = i + 1
            }
            // 发射方法 opcode
            if method == "push" { codegen_emit(cg, OP_LIST_PUSH()); codegen_emit(cg, OP_CONST_0()); return 0 }
            if method == "get" { codegen_emit(cg, OP_LIST_GET()); return 0 }
            if method == "len" { codegen_emit(cg, OP_LIST_LEN()); return 0 }
            if method == "insert_int" { codegen_emit(cg, OP_MAP_INSERT_INT()); codegen_emit(cg, OP_CONST_0()); return 0 }
            if method == "get_int" { codegen_emit(cg, OP_MAP_GET_INT()); return 0 }
            if method == "contains" { codegen_emit(cg, OP_MAP_CONTAINS()); return 0 }
        }
    }
}
```

**注意**：`push` 和 `insert_int` 后面加了 `OP_CONST_0()` 因为它们是 void 操作，需要留一个占位值在栈上（与 `print()` 的处理方式一致）。

**需要先实现 `str.find()` 方法**：AAVM 的 codegen.at 使用 Auto 代码运行在 Rust VM 上，`str.find(".")` 需要返回 "." 的位置。如果 `str.find` 不可用，可以用 `str.substr` + 循环手动查找。实际上 Rust VM 中 `str.find` 通过 native 可用，AAVM 的 eval.at 中也可以直接调用。但 codegen.at 运行在 tree-walking evaluator 中，需要确认 `str.find` 是否可用。

**备选方案**（如果 `str.find` 不可用）：使用 `str_get_part` 风格的手动查找，或者直接用 callee 的完整匹配（`callee == var_name + ".push"` 等），避免字符串分割。

实际上最简单的方式是**直接检查 callee 后缀**：

```auto
// 检查 callee 是否以 ".push" 等结尾
// 用 callee 后缀匹配代替 str.find
```

但 Auto 没有 `str.ends_with`... 不过可以用 `str.substr` 来实现。或者更简单——**直接枚举所有可能的 callee 模式**，用 callee 变量名提取辅助函数。

最终决定：用一个辅助函数从 callee 中提取方法名后缀。由于 Auto 有 `str.len()` 和 `str.substr()`，可以用循环从后往前找 "."。

---

## Step 4: 测试

### 测试 066: Bytecode List 操作

`test/vm/99_bootstrap/066_bytecode_list/bytecode_list.at`:
```auto
fn main() {
    var result = run_bytecode("fn main() {\nvar list = List.new()\nlist.push(10)\nlist.push(20)\nlist.push(30)\nprint(list.len())\nprint(list.get(0))\nprint(list.get(2))\n}")
    if result == "3\n10\n30\n" {
        print("ok")
    } else {
        print("fail:" + result)
    }
}
```

### 测试 067: Bytecode Map 操作

`test/vm/99_bootstrap/067_bytecode_map/bytecode_map.at`:
```auto
fn main() {
    var result = run_bytecode("fn main() {\nvar m = Map.new()\nm.insert_int(1, 100)\nm.insert_int(2, 200)\nprint(m.get_int(1))\nprint(m.get_int(2))\nprint(m.contains(1))\nprint(m.contains(3))\n}")
    if result == "100\n200\n1\n0\n" {
        print("ok")
    } else {
        print("fail:" + result)
    }
}
```

### 测试 068: List + 函数交互

`test/vm/99_bootstrap/068_bytecode_list_fn/bytecode_list_fn.at`:
```auto
fn main() {
    var result = run_bytecode("fn sum_list(lst, n int) int {\nvar s = 0\nfor i in 0..n {\ns = s + lst.get(i)\n}\nreturn s\n}\nfn main() {\nvar list = List.new()\nlist.push(1)\nlist.push(2)\nlist.push(3)\nprint(sum_list(list, 3))\n}")
    if result == "6\n" {
        print("ok")
    } else {
        print("fail:" + result)
    }
}
```

### 测试注册 — `vm_file_tests.rs`

```rust
#[test] fn test_aavm_99_bootstrap_066_bytecode_list() { test_aavm("99_bootstrap/066_bytecode_list").unwrap(); }
#[test] fn test_aavm_99_bootstrap_067_bytecode_map() { test_aavm("99_bootstrap/067_bytecode_map").unwrap(); }
#[test] fn test_aavm_99_bootstrap_068_bytecode_list_fn() { test_aavm("99_bootstrap/068_bytecode_list_fn").unwrap(); }
```

---

## 修改文件清单

| 文件 | 修改类型 | 预估行数 |
|------|---------|---------|
| `auto/lib/vm.at` | 修改 | +100 行 |
| `auto/lib/eval.at` | 修改 | +3 行 |
| `auto/lib/codegen.at` | 修改 | +60 行 |
| `test/vm/99_bootstrap/066_bytecode_list/` | 新建 | 2 文件 |
| `test/vm/99_bootstrap/067_bytecode_map/` | 新建 | 2 文件 |
| `test/vm/99_bootstrap/068_bytecode_list_fn/` | 新建 | 2 文件 |
| `crates/auto-lang/src/tests/vm_file_tests.rs` | 修改 | +3 行 |

## 风险

| 风险 | 缓解 |
|------|------|
| Auto 中 `str.find` 不可用 | 用 `str.substr` + 循环手动查找 "." 位置 |
| List 引用语义：`heap_lists.get(idx)` 返回的是引用还是拷贝 | Auto VM 中 List 是堆对象引用，应可直接修改；如果不行为则需要用 `heap_lists.set(idx, modified_list)` |
| void 操作 (push/insert) 的占位值可能被 `ExprStmt` 的 POP 弹走 | 与 print() 处理方式一致，应该正常工作 |
| callee 后缀匹配可能误匹配用户定义的函数 | 只在 codegen_find_local 找到对应 slot 时才处理，否则 fallback |

## 验证

```bash
# 新测试
cargo test -p auto-lang --lib -- test_aavm_99_bootstrap_066
cargo test -p auto-lang --lib -- test_aavm_99_bootstrap_067
cargo test -p auto-lang --lib -- test_aavm_99_bootstrap_068

# 回归
cargo test -p auto-lang --lib -- test_aavm_99_bootstrap
```
