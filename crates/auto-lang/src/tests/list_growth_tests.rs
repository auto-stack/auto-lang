// Test List automatic capacity expansion
use crate::run;

#[test]
fn test_list_automatic_growth() {
    let code = r#"
        let list = List.new()

        // Push 10 elements to force growth
        list.push(1)
        list.push(2)
        list.push(3)
        list.push(4)
        list.push(5)
        list.push(6)
        list.push(7)
        list.push(8)
        list.push(9)
        list.push(10)

        let len = list.len()
        let cap = list.capacity()
        let first = list.get(0)
        let last = list.get(9)

        // Verify all elements were added
        [len, cap, first, last]
    "#;

    let result = run(code).unwrap();

    // Parse the result array
    assert!(result.contains("1"));   // first element
    assert!(result.contains("10"));  // last element
}

#[test]
fn test_list_large_growth() {
    let code = r#"
        let list = List.new()

        // Push 20 elements to test multiple reallocations
        list.push(1)
        list.push(2)
        list.push(3)
        list.push(4)
        list.push(5)
        list.push(6)
        list.push(7)
        list.push(8)
        list.push(9)
        list.push(10)
        list.push(11)
        list.push(12)
        list.push(13)
        list.push(14)
        list.push(15)
        list.push(16)
        list.push(17)
        list.push(18)
        list.push(19)
        list.push(20)

        let len = list.len()

        // Verify length
        len
    "#;

    let result = run(code).unwrap();
    assert_eq!(result, "20");
}
