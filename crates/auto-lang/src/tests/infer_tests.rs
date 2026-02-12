use crate::run_autovm;

#[test]
fn test_infer_int() {
    let code = "42.type";

    let result = run_autovm(code);
    assert_eq!(result.unwrap(), "int", "`42.type` Should return int");

}
