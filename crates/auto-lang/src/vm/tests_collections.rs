/// Test VecDeque and BTreeMap collections (Plan 085)
/// These tests verify that the VM-native VecDeque and BTreeMap implementations
/// work correctly with AutoLang code

use crate::run;

// ============================================================================
// VecDeque Tests
// ============================================================================

#[test]
fn test_vecdeque_new() {
    let code = r#"
        let deque = VecDeque.new()
        deque.drop()
        0
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_vecdeque_push_back() {
    let code = r#"
        let deque = VecDeque.new()
        deque.push_back("first")
        deque.push_back("second")
        let size = deque.size()
        deque.drop()
        size
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "2");
}

#[test]
fn test_vecdeque_push_front() {
    let code = r#"
        let deque = VecDeque.new()
        deque.push_front("a")
        deque.push_front("b")
        let front = deque.front()
        deque.drop()
        front
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "b");
}

#[test]
fn test_vecdeque_pop_back() {
    let code = r#"
        let deque = VecDeque.new()
        deque.push_back("first")
        deque.push_back("second")
        let last = deque.pop_back()
        deque.drop()
        last
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "second");
}

#[test]
fn test_vecdeque_pop_front() {
    let code = r#"
        let deque = VecDeque.new()
        deque.push_back("first")
        deque.push_back("second")
        let first = deque.pop_front()
        deque.drop()
        first
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "first");
}

#[test]
fn test_vecdeque_front() {
    let code = r#"
        let deque = VecDeque.new()
        deque.push_back("first")
        deque.push_back("second")
        let front = deque.front()
        deque.drop()
        front
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "first");
}

#[test]
fn test_vecdeque_back() {
    let code = r#"
        let deque = VecDeque.new()
        deque.push_back("first")
        deque.push_back("second")
        let back = deque.back()
        deque.drop()
        back
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "second");
}

#[test]
fn test_vecdeque_is_empty() {
    let code = r#"
        let deque = VecDeque.new()
        let empty1 = deque.is_empty()
        deque.push_back("item")
        let empty2 = deque.is_empty()
        deque.drop()
        [empty1, empty2]
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("true") && result.contains("false"), "got: {}", result);
}

#[test]
fn test_vecdeque_clear() {
    let code = r#"
        let deque = VecDeque.new()
        deque.push_back("a")
        deque.push_back("b")
        deque.clear()
        let size = deque.size()
        deque.drop()
        size
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_vecdeque_queue_fifo() {
    // Test FIFO queue behavior
    let code = r#"
        let queue = VecDeque.new()
        queue.push_back("task1")
        queue.push_back("task2")
        queue.push_back("task3")
        let task1 = queue.pop_front()
        let task2 = queue.pop_front()
        queue.drop()
        [task1, task2]
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("task1") && result.contains("task2"));
}

#[test]
fn test_vecdeque_stack_lifo() {
    // Test LIFO stack behavior
    let code = r#"
        let stack = VecDeque.new()
        stack.push_back(1)
        stack.push_back(2)
        stack.push_back(3)
        let top1 = stack.pop_back()
        let top2 = stack.pop_back()
        stack.drop()
        [top1, top2]
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("3") && result.contains("2"));
}

// ============================================================================
// BTreeMap Tests
// ============================================================================

#[test]
fn test_btreemap_new() {
    let code = r#"
        let map = BTreeMap.new()
        map.drop()
        0
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_btreemap_insert_get() {
    let code = r#"
        let map = BTreeMap.new()
        map.insert("name", "Alice")
        map.insert("age", 30)
        let name = map.get("name")
        map.drop()
        name
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "Alice");
}

#[test]
fn test_btreemap_contains() {
    let code = r#"
        let map = BTreeMap.new()
        map.insert("key1", "value1")
        let has_key1 = map.contains("key1")
        let has_missing = map.contains("missing")
        map.drop()
        [has_key1, has_missing]
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("1") && result.contains("0"));
}

#[test]
fn test_btreemap_size() {
    let code = r#"
        let map = BTreeMap.new()
        map.insert("a", 1)
        map.insert("b", 2)
        map.insert("c", 3)
        let size = map.size()
        map.drop()
        size
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "3");
}

#[test]
fn test_btreemap_remove() {
    let code = r#"
        let map = BTreeMap.new()
        map.insert("temp", "value")
        map.remove("temp")
        let has_after = map.contains("temp")
        map.drop()
        has_after
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_btreemap_clear() {
    let code = r#"
        let map = BTreeMap.new()
        map.insert("a", 1)
        map.insert("b", 2)
        map.clear()
        let size = map.size()
        map.drop()
        size
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_btreemap_is_empty() {
    let code = r#"
        let map = BTreeMap.new()
        let empty1 = map.is_empty()
        map.insert("key", "value")
        let empty2 = map.is_empty()
        map.drop()
        [empty1, empty2]
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("true") && result.contains("false"), "got: {}", result);
}

#[test]
fn test_btreemap_first_key() {
    let code = r#"
        let map = BTreeMap.new()
        map.insert("banana", 2)
        map.insert("apple", 1)
        map.insert("cherry", 3)
        let first = map.first_key()
        map.drop()
        first
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "apple");
}

#[test]
fn test_btreemap_last_key() {
    let code = r#"
        let map = BTreeMap.new()
        map.insert("banana", 2)
        map.insert("apple", 1)
        map.insert("cherry", 3)
        let last = map.last_key()
        map.drop()
        last
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "cherry");
}

#[test]
fn test_btreemap_ordered_insertion() {
    // Test that keys are maintained in sorted order
    let code = r#"
        let map = BTreeMap.new()
        map.insert("z", 1)
        map.insert("a", 2)
        map.insert("m", 3)
        let first = map.first_key()
        let last = map.last_key()
        map.drop()
        [first, last]
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("a") && result.contains("z"));
}
