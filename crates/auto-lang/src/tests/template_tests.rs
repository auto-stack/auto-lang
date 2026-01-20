use std::rc::Rc;

use crate::scope::Meta;
use crate::universe::Universe;
use crate::{eval_template, interpret};
use auto_val::Value;

#[test]
fn test_for_with_mid() {
    let code = r#"$ for i in 0..10 { `${i}`; mid{","} }"#;
    let scope = Universe::new();
    let result = eval_template(code, scope).unwrap();
    assert_eq!(result.result.repr(), "0,1,2,3,4,5,6,7,8,9");
}

#[test]
fn test_for_with_mid_and_newline() {
    let code = r#"
$ for i in 0..10 {
    ${i}${mid{","}}
$ }"#;
    let scope = Universe::new();
    let result = eval_template(code, scope).unwrap();
    let expected =
        "\n    0,\n    1,\n    2,\n    3,\n    4,\n    5,\n    6,\n    7,\n    8,\n    9";
    if let Value::Str(s) = result.result {
        assert_eq!(s, expected);
    } else {
        assert_eq!(result.result.to_string(), expected);
    }
}

#[test]
fn test_simple_template() {
    let code = r#"
            var title = "Students"
            var rows = [
                { name: "Alice", age: 20 }
                { name: "Bob", age: 21 }
                { name: "Charlie", age: 22 }
            ]
        "#;
    let data = interpret(code).unwrap();
    let scope = data.scope.take();
    let template = r#"
<h1>$title</h1>
<table>
$ for row in rows {
    <tr>
        <td>${row.name}</td>
        <td>${row.age}</td>
    </tr>
$ }
</table>"#;
    let interpreter = eval_template(template, scope).unwrap();
    assert_eq!(
        interpreter.result.repr(),
        r#"
<h1>Students</h1>
<table>
    <tr>
        <td>Alice</td>
        <td>20</td>
    </tr>
    <tr>
        <td>Bob</td>
        <td>21</td>
    </tr>
    <tr>
        <td>Charlie</td>
        <td>22</td>
    </tr>
</table>"#
    );
}

#[test]
fn test_eval_template() {
    let code = r#"
        var rows = [
            { name: "Alice", age: 20 }
            { name: "Bob", age: 21 }
            { name: "Charlie", age: 22 }
        ]
        "#;
    let interpreter = interpret(code).unwrap();
    let scope = interpreter.scope.take();
    let template = r#"
$ for row in rows {
{
    name: ${row.name},
    age: ${row.age},
},
$ }
"#;
    let result = eval_template(template, scope).unwrap();
    let expected = r#"
{
    name: Alice,
    age: 20,
},
{
    name: Bob,
    age: 21,
},
{
    name: Charlie,
    age: 22,
},
"#;
    assert_eq!(result.result.repr(), expected);
}

#[test]
fn test_methods_in_template() {
    use crate::ast::{Expr, Store, StoreKind, Type};
    let code = r#"<div>${name.upper()}</div>"#;
    let mut scope = Universe::new();
    scope.set_global("name", Value::str("hello"));
    scope.define(
        "name",
        Rc::new(Meta::Store(Store {
            kind: StoreKind::Var,
            name: "name".into(),
            ty: Type::Str("hello".len()),
            expr: Expr::Str("hello".into()),
        })),
    );
    let result = eval_template(code, scope).unwrap();
    assert_eq!(result.result.repr(), "<div>HELLO</div>");
}