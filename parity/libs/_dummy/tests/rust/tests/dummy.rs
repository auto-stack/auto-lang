fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn sub(a: i32, b: i32) -> i32 {
    a - b
}

#[test]
fn test_add_basic() {
    assert_eq!(add(2, 3), 5);
}

#[test]
fn test_add_zero() {
    assert_eq!(add(0, 0), 0);
}

#[test]
fn test_add_negative() {
    assert_eq!(add(-1, 1), 0);
}

#[test]
fn test_sub_basic() {
    assert_eq!(sub(5, 3), 2);
}

#[test]
fn test_sub_negative() {
    assert_eq!(sub(0, 1), -1);
}
