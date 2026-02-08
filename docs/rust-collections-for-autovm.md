# Rust Collections for AutoVM - Analysis and Recommendations

## Current Status (✅ Already Implemented)

| Rust Collection | AutoLang Name | Status | Use Case |
|----------------|---------------|--------|----------|
| `Vec<T>` | `List<T, S>` | ✅ Complete | Dynamic array, growable list |
| `HashMap<K, V>` | `HashMap<K, V>` | ✅ Complete + Types | O(1) hash map, unordered |
| `HashSet<T>` | `HashSet<T>` | ✅ Complete + Types | O(1) hash set, unordered |

---

## Rust's std::collections (Not Yet in AutoLang)

### 1. **BTreeMap<K, V>** ⭐ HIGH PRIORITY

**What it is:** Ordered map sorted by key (O(log n) operations)

**Rust API:**
```rust
use std::collections::BTreeMap;

let mut map = BTreeMap::new();
map.insert("alice", 10);
map.insert("bob", 20);
for (key, value) in &map {  // Iterates in sorted order!
    println!("{}: {}", key, value);
}
```

**When to use vs HashMap:**
- **Need sorted iteration** (keys in order)
- **Range queries** (get all keys between "a" and "c")
- **Ordered operations** (first, last, next, prev)
- **Better cache locality** for small maps

**Performance:**
- Insert: O(log n)
- Lookup: O(log n)
- Delete: O(log n)
- Range query: O(log n + k)

**AutoLang use cases:**
```auto
// Sorted data with range queries
let scores = BTreeMap.new()
scores.insert("alice", 95)
scores.insert("bob", 87)
scores.insert("charlie", 92)

// Keys automatically sorted
for (name, score) in scores.iter() {
    print(`${name}: ${score}`)
}

// Range query: Get all names between "a" and "c"
let range = scores.range("a".."c")
```

**Recommendation:** ⭐ **ADD THIS** - Complements HashMap, fills gap for ordered data

---

### 2. **VecDeque<T>** ⭐ HIGH PRIORITY

**What it is:** Double-ended queue (ring buffer), efficient push/pop from both ends

**Rust API:**
```rust
use std::collections::VecDeque;

let mut deque = VecDeque::new();
deque.push_back(1);
deque.push_back(2);
deque.push_front(0);
// deque is now: [0, 1, 2]

let front = deque.pop_front(); // Some(0)
let back = deque.pop_back();   // Some(2)
```

**When to use vs Vec:**
- **Queue:** FIFO (add to back, remove from front)
- **Stack:** Can also use as stack (add/remove from same end)
- **Both ends:** Need efficient operations from front AND back
- **Sliding window:** Fixed-size buffer

**Performance:**
- push_front: O(1)
- push_back: O(1)
- pop_front: O(1)
- pop_back: O(1)
- Random access: O(1)

**AutoLang use cases:**
```auto
// Queue implementation
let queue = VecDeque.new()
queue.push_back("task1")
queue.push_back("task2")
queue.push_back("task3")

let task = queue.pop_front()  // "task1"

// Stack implementation
let stack = VecDeque.new()
stack.push_back(1)
stack.push_back(2)
let top = stack.pop_back()  // 2

// Sliding window (e.g., moving average)
let window = VecDeque.new()
window.push_back(1.0)
window.push_back(2.0)
window.push_back(3.0)
window.push_back(4.0)
if window.len() > 3 {
    window.pop_front()  // Keep last 3
}
```

**Recommendation:** ⭐ **ADD THIS** - Versatile data structure, fills gap for queue/stack

---

### 3. **BinaryHeap<T>** ⭐⭐ MEDIUM PRIORITY

**What it is:** Priority queue (max-heap by default), always get largest element

**Rust API:**
```rust
use std::collections::BinaryHeap;

let mut heap = BinaryHeap::new();
heap.push(10);
heap.push(30);
heap.push(20);

let max = heap.pop(); // Some(30) - always gets largest
```

**When to use:**
- **Priority queues:** Task scheduling, Dijkstra's algorithm
- **Max/min finding:** Always get largest or smallest element
- **Partial sorting:** Need top K elements

**Performance:**
- push: O(log n)
- pop: O(log n)
- peek: O(1) - view largest/smallest

**AutoLang use cases:**
```auto
// Task scheduler (highest priority first)
let tasks = BinaryHeap.new()
tasks.push((priority: 1, name: "low"))
tasks.push((priority: 10, name: "high"))
tasks.push((priority: 5, name: "medium"))

let next_task = tasks.pop()  // Highest priority first

// Top K elements
let heap = BinaryHeap.new()
for item in items {
    heap.push(item)
}
let top3 = []
for i in 0..3 {
    top3.push(heap.pop())
}
```

**Recommendation:** ⭐⭐ **CONSIDER** - Useful but more specialized, lower priority than BTreeMap/VecDeque

---

### 4. **BTreeSet<T>** ⭐⭐ MEDIUM PRIORITY

**What it is:** Ordered set, sorted by element (O(log n) operations)

**Rust API:**
```rust
use std::collections::BTreeSet;

let mut set = BTreeSet::new();
set.insert(10);
set.insert(20);
set.insert(15);
// Elements are: 10, 15, 20 (sorted!)

if set.contains(&15) { ... }
```

**When to use vs HashSet:**
- **Need sorted elements**
- **Range queries** (all elements between x and y)
- **Ordered operations** (first, last, next, prev)

**Performance:**
- insert: O(log n)
- contains: O(log n)
- delete: O(log n)
- range query: O(log n + k)

**AutoLang use cases:**
```auto
// Sorted unique elements
let numbers = BTreeSet.new()
numbers.insert(30)
numbers.insert(10)
numbers.insert(20)

// Automatically sorted: 10, 20, 30
for n in numbers.iter() {
    print(n)
}

// Range query
let range = numbers.range(15..25)  // [20]
```

**Recommendation:** ⭐⭐ **CONSIDER** - Similar to BTreeMap but set semantics, lower priority

---

### 5. **LinkedList<T>** ❌ NOT RECOMMENDED

**What it is:** Doubly linked list

**Why NOT to add:**
- Rust documentation explicitly recommends **VecDeque** instead
- Poor cache locality (scattered memory)
- High overhead (extra pointers per element)
- Rarely the right choice in practice

**Recommendation:** ❌ **SKIP** - Use VecDeque instead

---

## Comparison Table

| Collection | Order | Operations | Best For | Priority |
|------------|-------|------------|----------|----------|
| **List (Vec)** | Insertion | O(1) push/pop back | General-purpose array | ✅ Done |
| **HashMap** | Unordered | O(1) avg lookup | Fast lookup, unordered | ✅ Done |
| **HashSet** | Unordered | O(1) avg membership | Fast uniqueness check | ✅ Done |
| **BTreeMap** | Sorted | O(log n) | Ordered maps, ranges | ⭐ **HIGH** |
| **VecDeque** | Insertion | O(1) both ends | Queues, stacks | ⭐ **HIGH** |
| **BinaryHeap** | Partial | O(log n) | Priority queues | ⭐⭐ Medium |
| **BTreeSet** | Sorted | O(log n) | Ordered sets, ranges | ⭐⭐ Medium |
| LinkedList | Insertion | O(1) front/back | ❌ Use VecDeque | ❌ Skip |

---

## Recommended Implementation Order

### Phase 1: High Priority (Most Valuable)

1. **VecDeque<T>**
   - Generic: `VecDeque<T>` or `Deque<T>`
   - Methods: `new()`, `push_front()`, `push_back()`, `pop_front()`, `pop_back()`, `front()`, `back()`, `len()`, `is_empty()`, `clear()`
   - Backing: `std::collections::VecDeque<Value>`

2. **BTreeMap<K, V>**
   - Generic: `BTreeMap<K, V>`
   - Methods: `new()`, `insert()`, `get()`, `contains()`, `remove()`, `len()`, `is_empty()`, `clear()`, `iter()`, `range()`, `first()`, `last()`
   - Backing: `std::collections::BTreeMap<String, Value>`

### Phase 2: Medium Priority (Specialized Use Cases)

3. **BinaryHeap<T>**
   - Generic: `BinaryHeap<T>` or `Heap<T>`
   - Methods: `new()`, `push()`, `pop()`, `peek()`, `len()`, `is_empty()`
   - Backing: `std::collections::BinaryHeap<Value>`

4. **BTreeSet<T>**
   - Generic: `BTreeSet<T>`
   - Methods: `new()`, `insert()`, `contains()`, `remove()`, `len()`, `is_empty()`, `clear()`, `iter()`, `range()`, `first()`, `last()`
   - Backing: `std::collections::BTreeSet<String>`

---

## Naming Conventions

| Rust Name | Recommended AutoLang Name | Notes |
|-----------|-------------------------|-------|
| `VecDeque<T>` | `VecDeque<T>` or `Deque<T>` | `Deque` is shorter, but `VecDeque` matches Rust |
| `BTreeMap<K, V>` | `BTreeMap<K, V>` | Matches Rust name |
| `BTreeSet<T>` | `BTreeSet<T>` | Matches Rust name |
| `BinaryHeap<T>` | `BinaryHeap<T>` or `Heap<T>` | `Heap` is simpler, but less specific |

**My recommendation:** Use the full Rust names for clarity (`VecDeque`, `BTreeMap`, `BTreeSet`, `BinaryHeap`).

---

## Integration with Prelude

After adding these, the prelude would be:

```auto
// Collections
use auto.list: List
use auto.hashmap: HashMap
use auto.hashset: HashSet
use auto.btreemap: BTreeMap     // NEW
use auto.vecdeque: VecDeque     // NEW
use auto.binaryheap: BinaryHeap // NEW (optional)
use auto.btreeset: BTreeSet     // NEW (optional)
```

---

## Example: When to Use Which Collection

### Scenario 1: Shopping List (Order Matters)
```auto
// Use VecDeque (maintains insertion order)
let list = VecDeque.new()
list.push_back("milk")
list.push_back("eggs")
list.push_back("bread")
```

### Scenario 2: Phone Book (Alphabetical Order)
```auto
// Use BTreeMap (automatically sorted by key)
let contacts = BTreeMap.new()
contacts.insert("Alice", "555-1234")
contacts.insert("Bob", "555-5678")
contacts.insert("Charlie", "555-9012")

// Iterates in sorted order: Alice, Bob, Charlie
for (name, phone) in contacts.iter() {
    print(`${name}: ${phone}`)
}
```

### Scenario 3: Task Scheduler (Priorities)
```auto
// Use BinaryHeap (highest priority first)
let tasks = BinaryHeap.new()
tasks.push((priority: 1, task: "low priority"))
tasks.push((priority: 10, task: "urgent"))
tasks.push((priority: 5, task: "normal"))

let next = tasks.pop()  // Returns urgent task
```

### Scenario 4: Queue (FIFO)
```auto
// Use VecDeque (efficient both ends)
let queue = VecDeque.new()
queue.push_back("customer1")
queue.push_back("customer2")
queue.push_back("customer3")

let next = queue.pop_front()  // "customer1"
```

---

## Summary

**Add These (High Priority):**
1. ✅ **VecDeque<T>** - Queue/stack, double-ended operations
2. ✅ **BTreeMap<K, V>** - Ordered map, range queries

**Consider Adding (Medium Priority):**
3. ⭐⭐ **BinaryHeap<T>** - Priority queues
4. ⭐⭐ **BTreeSet<T>** - Ordered sets

**Don't Add:**
5. ❌ **LinkedList<T>** - Use VecDeque instead

This gives AutoVM a complete collection library covering all common use cases!
