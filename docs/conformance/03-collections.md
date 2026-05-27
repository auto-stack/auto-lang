# 03 — 集合操作语义

## 数组 (Array)

### 数组字面量 `[1, 2, 3]`
- **语义**: 创建固定大小数组，元素类型一致
- **AutoVM**: `OpCode::CREATE_ARRAY` — 从栈上弹出 N 个值创建堆数组
- **a2r**: 转译为 `vec![1, 2, 3]` 或数组表达式
- **类型推断**: `[1, 2, 3]` 推断为 `[]int`
- **示例**:
  - `let arr = [1, 2, 3]` → `[1, 2, 3]`
  - `["a", "b"]` → `["a", "b"]`

### 数组索引 `arr[i]`
- **语义**: 按零基索引访问元素
- **AutoVM**: `OpCode::GET_ELEM` (0x2C)
- **a2r**: `arr[i]`（Rust 索引）
- **越界**: 运行时 panic
- **示例**: `[10, 20, 30][1]` → `20`

### 数组赋值 `arr[i] = val`
- **语义**: 修改指定索引的元素
- **AutoVM**: `OpCode::SET_ELEM`
- **a2r**: `arr[i] = val`
- **越界**: 运行时 panic

### 数组切片 `arr[1..3]`
- **语义**: 返回子数组，左闭右开
- **AutoVM**: 范围切片 opcode
- **a2r**: `arr[1..3].to_vec()` 或切片
- **示例**: `[0, 1, 2, 3, 4][1..3]` → `[1, 2]`

### 固定数组 `[N]T`
- **语义**: 编译时确定大小的数组
- **声明**: `let buf [10]int` — 10 个 int 的固定数组
- **a2r**: 转译为 `[T; N]` (Rust 固定数组)

## List (动态列表)

### 创建
```auto
let list = List.new()          // 空 List
let list = [1, 2, 3]           // 从数组字面量创建（类型推断为 List）
```
- **AutoVM**: `OpCode::CREATE_LIST_INT` / `CREATE_LIST_STR` 等类型特化 opcode
- **a2r**: 转译为 `Vec::new()` 或 `vec![]`

### `.push(val)` → `void`
- **语义**: 追加元素到末尾
- **AutoVM**: `LIST_PUSH_INT` / `LIST_PUSH_STR` 等类型特化
- **a2r**: `.push(val)` (Rust Vec)
- **示例**: `list.push(4)` → list 变为 `[1, 2, 3, 4]`

### `.get(index)` → `?T`
- **语义**: 安全访问，越界返回 None
- **AutoVM**: native shim
- **a2r**: `.get(index).copied()` (Rust Vec)
- **示例**: `list.get(0)` → `Some(1)`; `list.get(99)` → `None`

### `.len()` → `int`
- **语义**: 返回元素数量
- **AutoVM**: native shim
- **a2r**: `.len()` (Rust Vec)
- **示例**: `[1, 2, 3].len()` → `3`

### `.pop()` → `?T`
- **语义**: 移除并返回末尾元素，空列表返回 None
- **a2r**: `.pop()`

### `.remove(index)` → `T`
- **语义**: 移除指定位置元素并返回
- **a2r**: `.remove(index)`

### `.insert(index, val)` → `void`
- **语义**: 在指定位置插入元素
- **a2r**: `.insert(index, val)`

### `.contains(val)` → `bool`
- **语义**: 检查是否包含指定值
- **a2r**: `.contains(&val)`

### `.sort()` → `void`
- **语义**: 原地排序（升序）
- **a2r**: `.sort()`

### `.reverse()` → `void`
- **语义**: 原地反转
- **a2r**: `.reverse()`

## Map (哈希映射)

### 创建
```auto
let scores Map<str, int> = Map.new()
```
- **AutoVM**: `OpCode::CREATE_MAP` 或类型特化
- **a2r**: `HashMap::new()`

### `.set(key, val)` / `.insert(key, val)` → `void`
- **语义**: 插入或更新键值对
- **AutoVM**: native shim
- **a2r**: `.insert(key, val)`（值自动 `.to_string()` 当 Map value 类型为 String）

### `.get(key)` → `?T`
- **语义**: 按 key 查找，不存在返回 None
- **a2r**: `.get(&key).cloned()` 或 `.get(&key).copied()`

### `.contains_key(key)` → `bool`
- **语义**: 检查 key 是否存在
- **a2r**: `.contains_key(&key)`

### `.len()` → `int`
- **a2r**: `.len()`

### `.remove(key)` → `?T`
- **语义**: 移除指定 key 并返回其值
- **a2r**: `.remove(&key)`

## 迭代器

### `for item in collection`
- **语义**: 遍历集合中所有元素
- **AutoVM**: `OpCode::ITERATOR` + `OpCode::ITER_NEXT` 循环
- **a2r**: `for item in collection` (Rust for-in)

### `for i, item in collection`
- **语义**: 带索引遍历
- **a2r**: `for (i, item) in collection.iter().enumerate()`

### `.map(closure)` → `List<T>`
- **语义**: 映射变换，返回新列表
- **AutoVM**: `OpCode::CALL_CLOSURE` 通过 map 适配器
- **a2r**: `.iter().map(|x| ...).collect::<Vec<_>>()`

### `.filter(closure)` → `List<T>`
- **语义**: 过滤，返回满足条件的元素
- **a2r**: `.iter().filter(|x| ...).collect::<Vec<_>>()`

### `.reduce(closure)` → `T`
- **语义**: 累积计算
- **a2r**: `.iter().fold(init, |acc, x| ...)`

## 已知语义间隙

1. **List 类型特化**: AutoVM 使用 `CREATE_LIST_INT` 等特化 opcode，a2r 统一用 `Vec<T>`。语义一致但性能不同
2. **Map value 的 .to_string()**: a2r 在 Map.insert() 时对 String value 自动插入 `.to_string()`，需确认 AutoVM 行为一致
3. **迭代器 lazy vs eager**: AutoVM 的 map/filter 是 lazy adapter，collect 时才执行；a2r 通过 Rust 迭代器天然 lazy。一致
