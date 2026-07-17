use crate::run;

#[test]
fn test_string_new() {
    let code = r#"
            let s = String.new()
            s.len()
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_string_from() {
    let code = r#"
            let s = String.from("A")
            s.len()
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "1");
}

#[test]
fn test_string_from_multi_char() {
    let code = r#"
            let s = String.from("AB")
            s.len()
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "2");
}

#[test]
fn test_string_push() {
    let code = r#"
            var s = String.new()
            s.push('A')
            s.push('B')
            s.push('C')
            s.len()
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "3");
}

#[test]
fn test_string_push_and_get() {
    let code = r#"
            var s = String.new()
            s.push('H')
            s.push('e')
            s.push('l')
            s.push('l')
            s.push('o')
            s.get(0)
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "72");  // 'H' = 72
}

#[test]
fn test_string_pop() {
    let code = r#"
            var s = String.new()
            s.push('A')
            s.push('B')
            s.push('C')
            let val = s.pop()
            val
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "67");  // 'C' = 67
}

#[test]
fn test_string_pop_reduces_length() {
    let code = r#"
            var s = String.new()
            s.push('A')
            s.push('B')
            s.push('C')
            s.pop()
            s.len()
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "2");
}

#[test]
fn test_string_get() {
    let code = r#"
            var s = String.new()
            s.push('A')
            s.push('B')
            s.push('C')
            s.get(1)
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "66");  // 'B' = 66
}

#[test]
fn test_string_set() {
    let code = r#"
            var s = String.new()
            s.push('A')
            s.push('B')
            s.set(0, 'C')
            s.get(0)
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "67");  // 'C' = 67
}

#[test]
fn test_string_is_empty() {
    let code = r#"
            let s = String.new()
            s.is_empty()
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "true");
}

#[test]
fn test_string_is_empty_after_push() {
    let code = r#"
            var s = String.new()
            s.push('A')
            s.is_empty()
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "false");
}

#[test]
fn test_string_insert() {
    let code = r#"
            var s = String.new()
            s.push('A')
            s.push('C')
            s.insert(1, 'B')
            s.get(1)
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "66");  // 'B' = 66
}

#[test]
fn test_string_insert_at_beginning() {
    let code = r#"
            var s = String.new()
            s.push('B')
            s.push('C')
            s.insert(0, 'A')
            s.get(0)
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "65");  // 'A' = 65
}

#[test]
fn test_string_remove() {
    let code = r#"
            var s = String.new()
            s.push('A')
            s.push('B')
            s.push('C')
            s.remove(1)
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "66");  // 'B' = 66
}

#[test]
fn test_string_remove_reduces_length() {
    let code = r#"
            var s = String.new()
            s.push('A')
            s.push('B')
            s.push('C')
            s.remove(1)
            s.len()
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "2");
}

#[test]
fn test_string_clear() {
    let code = r#"
            var s = String.new()
            s.push('A')
            s.push('B')
            s.push('C')
            s.clear()
            s.is_empty()
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "1");
}

#[test]
fn test_string_reserve() {
    let code = r#"
            var s = String.new()
            s.reserve(100)
            s.push('A')
            s.len()
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "1");
}

#[test]
fn test_string_comprehensive_hello() {
    let code = r#"
            var s = String.new()
            s.push('H')
            s.push('e')
            s.push('l')
            s.push('l')
            s.push('o')

            let len = s.len()
            if len == 5 {
                let first = s.get(0)
                if first == 72 {
                    let last = s.get(4)
                    if last == 111 {
                        1
                    } else {
                        0
                    }
                } else {
                    0
                }
            } else {
                0
            }
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "1");
}

#[test]
fn test_string_multiple_operations() {
    let code = r#"
            var s = String.new()
            s.push('A')
            s.push('B')
            s.push('C')
            s.set(0, 'D')
            s.insert(1, 'E')
            s.remove(2)
            let len = s.len()
            let val = s.get(0)
            if len == 3 {
                if val == 68 {
                    1
                } else {
                    0
                }
            } else {
                0
            }
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "1");
}

#[test]
fn test_string_from_then_push() {
    let code = r#"
            var s = String.from("A")
            s.push('B')
            s.push('C')
            s.len()
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "3");
}
// 19317196129
#[test]
fn test_string_from_then_modify() {
    let code = r#"
            var s = String.from("AB")
            s.push('C')
            s.set(0, 'D')
            s.get(0)
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "68");  // 'D' = 68
}
