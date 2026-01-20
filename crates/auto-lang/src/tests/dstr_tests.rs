use crate::run;

#[test]
fn test_dstr_new() {
    let code = r#"
            let s = dstr.new()
            s.len()
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_dstr_from_byte() {
    let code = r#"
            let s = dstr.from_byte(65)
            s.len()
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "1");
}

#[test]
fn test_dstr_from_bytes() {
    let code = r#"
            let s = dstr.from_bytes(65, 66)
            s.len()
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "2");
}

#[test]
fn test_dstr_push() {
    let code = r#"
            mut s = dstr.new()
            s.push(65)
            s.push(66)
            s.push(67)
            s.len()
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "3");
}

#[test]
fn test_dstr_push_and_get() {
    let code = r#"
            mut s = dstr.new()
            s.push(72)
            s.push(101)
            s.push(108)
            s.push(108)
            s.push(111)
            s.get(0)
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "72");
}

#[test]
fn test_dstr_pop() {
    let code = r#"
            mut s = dstr.new()
            s.push(65)
            s.push(66)
            s.push(67)
            let val = s.pop()
            val
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "67");
}

#[test]
fn test_dstr_pop_reduces_length() {
    let code = r#"
            mut s = dstr.new()
            s.push(65)
            s.push(66)
            s.push(67)
            s.pop()
            s.len()
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "2");
}

#[test]
fn test_dstr_get() {
    let code = r#"
            mut s = dstr.new()
            s.push(65)
            s.push(66)
            s.push(67)
            s.get(1)
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "66");
}

#[test]
fn test_dstr_set() {
    let code = r#"
            mut s = dstr.new()
            s.push(65)
            s.push(66)
            s.set(0, 67)
            s.get(0)
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "67");
}

#[test]
fn test_dstr_is_empty() {
    let code = r#"
            let s = dstr.new()
            s.is_empty()
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "1");
}

#[test]
fn test_dstr_is_empty_after_push() {
    let code = r#"
            mut s = dstr.new()
            s.push(65)
            s.is_empty()
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_dstr_insert() {
    let code = r#"
            mut s = dstr.new()
            s.push(65)
            s.push(67)
            s.insert(1, 66)
            s.get(1)
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "66");
}

#[test]
fn test_dstr_insert_at_beginning() {
    let code = r#"
            mut s = dstr.new()
            s.push(66)
            s.push(67)
            s.insert(0, 65)
            s.get(0)
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "65");
}

#[test]
fn test_dstr_remove() {
    let code = r#"
            mut s = dstr.new()
            s.push(65)
            s.push(66)
            s.push(67)
            s.remove(1)
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "66");
}

#[test]
fn test_dstr_remove_reduces_length() {
    let code = r#"
            mut s = dstr.new()
            s.push(65)
            s.push(66)
            s.push(67)
            s.remove(1)
            s.len()
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "2");
}

#[test]
fn test_dstr_clear() {
    let code = r#"
            mut s = dstr.new()
            s.push(65)
            s.push(66)
            s.push(67)
            s.clear()
            s.is_empty()
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "1");
}

#[test]
fn test_dstr_reserve() {
    let code = r#"
            mut s = dstr.new()
            s.reserve(100)
            s.push(65)
            s.len()
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "1");
}

#[test]
fn test_dstr_comprehensive_hello() {
    let code = r#"
            mut s = dstr.new()
            s.push(72)
            s.push(101)
            s.push(108)
            s.push(108)
            s.push(111)

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
fn test_dstr_multiple_operations() {
    let code = r#"
            mut s = dstr.new()
            s.push(65)
            s.push(66)
            s.push(67)
            s.set(0, 68)
            s.insert(1, 69)
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
fn test_dstr_from_byte_then_push() {
    let code = r#"
            mut s = dstr.from_byte(65)
            s.push(66)
            s.push(67)
            s.len()
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "3");
}

#[test]
fn test_dstr_from_bytes_then_modify() {
    let code = r#"
            mut s = dstr.from_bytes(65, 66)
            s.push(67)
            s.set(0, 68)
            s.get(0)
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "68");
}
