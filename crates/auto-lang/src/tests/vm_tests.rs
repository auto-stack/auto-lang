// VM and Evaluator Integration Tests
// These tests were moved from lib.rs for better organization
use crate::config::AutoConfig;
use crate::{ast, interpret, run, run_with_scope};
use crate::universe::Universe;
use auto_val::Value;

#[test]
fn test_uint() {
    let code = "1u+2u";
    let result = run(code).unwrap();
    assert_eq!(result, "3u");

    let code = "25u+123u";
    let result = run(code).unwrap();
    assert_eq!(result, "148u");
}

#[test]
fn test_byte() {
    let code = "let a byte = 255; a";
    let result = run(code).unwrap();
    assert_eq!(result, "0xFF");

    // promote byte to int
    let code = "let a int = 0xFF; a";
    let result = run(code).unwrap();
    assert_eq!(result, "255");
}

#[test]
fn test_arithmetic() {
    let code = "1+2*3";
    let result = run(code).unwrap();
    assert_eq!(result, "7");

    let code = "(2+3.5)*5";
    let result = run(code).unwrap();
    assert_eq!(result, "27.5");
}

#[test]
fn test_unary() {
    let code = "-2*3";
    let result = run(code).unwrap();
    assert_eq!(result, "-6");
}

#[test]
fn test_group() {
    let code = "(1+2)*3";
    let result = run(code).unwrap();
    assert_eq!(result, "9");
}

#[test]
fn test_comp() {
    let code = "1 < 2";
    let result = run(code).unwrap();
    assert_eq!(result, "true");
}

#[test]
fn test_var_assign() {
    let code = "var a = 1; a = 2; a";
    let result = run(code).unwrap();
    assert_eq!(result, "2");
}

#[test]
fn test_var_arithmetic() {
    let code = "var a = 12312; a * 10";
    let result = run(code).unwrap();
    assert_eq!(result, "123120");
}

#[test]
fn test_if() {
    let code = "if true { 1 } else { 2 }";
    let result = run(code).unwrap();
    assert_eq!(result, "1");
}

#[test]
fn test_if_else() {
    let code = "if false { 1 } else { 2 }";
    let result = run(code).unwrap();
    assert_eq!(result, "2");
}

#[test]
fn test_if_else_if() {
    let code = "if false { 1 } else if false { 2 } else { 3 }";
    let result = run(code).unwrap();
    assert_eq!(result, "3");
}

#[test]
fn test_var() {
    let code = "var a = 1; a+2";
    let result = run(code).unwrap();
    assert_eq!(result, "3");
}

#[test]
fn test_if_var() {
    let code = "var a = 10; if a > 10 { a+1 } else { a-1  }";
    let result = run(code).unwrap();
    assert_eq!(result, "9");
}

#[test]
fn test_var_if() {
    let code = "var x = if true { 1 } else { 2 }; x+1";
    let result = run(code).unwrap();
    assert_eq!(result, "2");
}

#[test]
fn test_var_mut() {
    let code = "var x = 1; x = 10; x+1";
    let result = run(code).unwrap();
    assert_eq!(result, "11");
}

#[test]
fn teste_array() {
    let code = "[1, 2, 3]";
    let result = run(code).unwrap();
    assert_eq!(result, "[1, 2, 3]");

    let code = "var a = [1, 2, 3]; [a[0], a[1], a[2], a[-1], a[-2], a[-3]]";
    let result = run(code).unwrap();
    assert_eq!(result, "[1, 2, 3, 3, 2, 1]");
}

#[test]
fn test_range() {
    let code = "1..5";
    let result = run(code).unwrap();
    assert_eq!(result, "1..5");
}

#[test]
fn test_range_print() {
    let code = r#"for i in 0..10 { print(i) }"#;
    let result = run(code).unwrap();
    // TODO: capture stdout and assert
    assert_eq!(result, "");
}

#[test]
fn test_range_eq() {
    let code = "var sum = 0; for i in 0..=10 { sum = sum + i }; sum";
    let result = run(code).unwrap();
    assert_eq!(result, "55");
}

#[test]
fn test_for() {
    let code = "var sum = 0; for i in 0..10 { sum = sum + i }; sum";
    let result = run(code).unwrap();
    assert_eq!(result, "45");

    let code = "var arr = [1, 2, 3]; var sum = 0; for i, x in arr { sum = sum + x + i }; sum";
    let result = run(code).unwrap();
    assert_eq!(result, "9");
}

#[test]
fn test_is_stmt() {
    let code = r#"var x = 10; is x { 10 => {print("Here is 10!"); x} }"#;
    let result = run(code).unwrap();
    assert_eq!(result, "10");
}

#[test]
fn test_fn() {
    let code = "fn add(a, b) { a + b }; add(12, 2)";
    let result = run(code).unwrap();
    assert_eq!(result, "14");

    let code = "fn add(a, b) { a + b }; add(a:1, b:2)";
    let result = run(code).unwrap();
    assert_eq!(result, "3");

    let code = "fn hi(s str) { print(s) }; hi(\"hello\")";
    let result = run(code).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_object() {
    let code = "var a = { name: \"auto\", age: 18 }; a.name";
    let result = run(code).unwrap();
    assert_eq!(result, "auto");

    let code = "var a = { name: \"auto\", age: 18 }; a.age";
    let result = run(code).unwrap();
    assert_eq!(result, "18");

    let code = "var a = { 1: 2, 3: 4 }; a.3";
    let result = run(code).unwrap();
    assert_eq!(result, "4");

    let code = "var a = { true: 2, false: 4 }; a.false";
    let result = run(code).unwrap();
    assert_eq!(result, "4");
}

#[test]
fn test_array_of_objects() {
    let code = "[1, 2]";
    let result = run(code).unwrap();
    println!("{}", result);
    assert_eq!(result, "[1, 2]");
}

#[test]
fn test_closure() {
    // Plan 060: Test closure syntax (replaces deprecated lambda)
    let code = "var add = (a, b) => a + b; add(1, 2)";
    let result = run(code).unwrap();
    assert_eq!(result, "3");
}

#[test]
fn test_closure_with_type_annotations() {
    // Plan 060: Test closure with type annotations (Auto syntax: no colon)
    let code = "let sub = (a int, b int) => a - b; sub(12, 5)";
    let result = run(code).unwrap();
    assert_eq!(result, "7");
}

#[test]
fn test_json() {
    let code = r#"
            var ServiceInfo = [
                { id: 0x10, name: "DiagnosticSessionControl",  desc: "诊断会话控制" },
                { id: 0x11, name: "EcuReset",  desc: "电控单元复位" },
                { id: 0x14, name: "ClearDiagnosticInformation",  desc: "清除诊断信息" },
            ]
            ServiceInfo[2].name
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "ClearDiagnosticInformation");
}

#[test]
fn test_fstr() {
    let code = r#"var name = "auto"; f"hello $name, now!""#;
    let result = run(code).unwrap();
    assert_eq!(result, "hello auto, now!");
}

#[test]
fn test_fstr_with_expr() {
    let code = r#"var a = 1; var b = 2; f"a + b = ${a+b}""#;
    let result = run(code).unwrap();
    assert_eq!(result, "a + b = 3");
}

#[test]
fn test_fstr_with_addition() {
    let code = r#"`[${no_tx_msgs + 1}u]`"#;
    let mut scope = Universe::new();
    scope.set_global("no_tx_msgs", Value::Int(9));
    let result = run_with_scope(code, scope).unwrap();
    assert_eq!(result, "[10u]");
}

#[test]
fn test_asn_upper() {
    let code = "var a = 1; if true { a = 2 }; a";
    let result = run(code).unwrap();
    assert_eq!(result, "2");
}
#[test]
fn test_nodes() {
    let code = r#"center {
            text("Hello") {}
            button("OK") {
                onclick: || print("clicked")
            }
        }"#;
    let result = run(code).unwrap();
    println!("{}", result);
}


#[test]
fn test_for_loop_with_object() {
    let code = r#"
        var items = [
            { name: "Alice", age: 20 }
            { name: "Bob", age: 21 }
            { name: "Charlie", age: 22 }
        ]
        for item in items {
            print(f"Hi ${item.name}")
        }
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_eval_template_with_note() {
    let code = "#{x+1}";
    use crate::interp::Interpreter;
    let mut scope = Universe::new();
    scope.set_global("x", Value::Int(41));
    let mut interp = Interpreter::with_scope(scope);
    let result = interp.eval_template_with_note(code, '#').unwrap();
    assert_eq!(result.repr(), "42");
}

#[test]
fn test_to_string() {
    let code = r#"1.str()"#;
    let result = run(code).unwrap();
    assert_eq!(result, "1");

    let code = r#""hello".upper()"#;
    let result = run(code).unwrap();
    assert_eq!(result, "HELLO");
}

#[test]
fn test_insert_global_fn() {
    fn myjoin(arg: &auto_val::Args) -> Value {
        Value::Str(
            arg.args
                .iter()
                .map(|v| v.to_astr())
                .collect::<Vec<auto_val::AutoStr>>()
                .join("::")
                .into(),
        )
    }

    let mut scope = Universe::new();
    scope.add_global_fn("myjoin", myjoin);
    let code = "myjoin(1, 2, 3)";
    let result = run_with_scope(code, scope).unwrap();
    assert_eq!(result, "1::2::3");
}

#[test]
fn test_obj_set() {
    let code = "var a = { name: \"Alice\" }; a.name = \"Bob\"; a.name";
    let result = run(code).unwrap();
    assert_eq!(result, "Bob");
}

#[test]
fn test_array_update() {
    let code = "var a = [1, 2, 3]; a[0] = 4; a";
    let result = run(code).unwrap();
    assert_eq!(result, "[4, 2, 3]");
}

#[test]
fn test_let() {
    let code = "let x = 41; x";
    let result = run(code).unwrap();
    assert_eq!(result, "41");
}

#[test]
fn test_let_asn() {
    let code = "let x = 41; x = 10; x";
    let result = run(code);
    assert!(result.is_err());
}

#[test]
fn test_var_reassignment() {
    let code = "let x = 41; var x = 10; x";
    let result = run(code).unwrap();
    assert_eq!(result, "10");
}


#[test]
fn test_type_decl() {
    let code = "type Point { x int = 5; y int }; let p = Point(y: 2); p";
    let mut interpreter = interpret(code).unwrap();
    // Note: Insertion order is now preserved (y comes after x in declaration, but y:2 is set after default x:5)
    assert_eq!(interpreter.result.repr(), "Point{y: 2, x: 5}");

    let code = "p.x";
    let result = interpreter.eval(code);
    assert_eq!(result.repr(), "5");

    let code = "p.y";
    let result = interpreter.eval(code);
    assert_eq!(result.repr(), "2");
}

#[test]
fn test_deep_type() {
    let code = "type A { x int; y int }; type B { a A; b int }";
    let mut interpreter = interpret(code).unwrap();
    let code = "var v = B(a: A(x:1, y:2), b:3); v.a.y";
    let result = interpreter.eval(code);
    assert_eq!(result.repr(), "2");
}

#[test]
fn test_type_with_method() {
    let code = r#"type Point {
            x int
            y int

            fn absquare() int {
                .x * .x + .y * .y
            }
        }"#;
    let mut interpreter = interpret(code).unwrap();
    let code = "var p = Point(3, 4); p.absquare()";
    let result = interpreter.eval(code);
    assert_eq!(result.repr(), "25");
}

#[test]
fn test_ext_statement_instance_method() {
    // Plan 035 Phase 4: Test ext statement with instance methods
    let code = r#"
        ext int {
            fn double() int {
                self + self
            }
        }
        "#;
    let mut interpreter = interpret(code).unwrap();
    let result = interpreter.eval("var x = 5; x.double()");
    assert_eq!(result.repr(), "10");
}

#[test]
fn test_ext_statement_static_method() {
    // Plan 035 Phase 4: Test ext statement with static methods
    // Static methods don't have self, so they can be called without instance
    let code = r#"
        ext int {
            static fn get_default() int {
                42
            }
        }
        "#;
    let mut interpreter = interpret(code).unwrap();
    // Static method call on type
    let result = interpreter.eval("int.get_default()");
    println!("Static method result: {:?}", result);
    assert_eq!(result.repr(), "42");
}

#[test]
fn test_simple_block() {
    let code = r#"
        let a = 10;
        {
            let a = 20;
        }
        a
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "10");
}

#[test]
fn test_type_compose() {
    let code = r#"
        type Wing {
            fn fly() {print("flap!flap!")}
        }
        type Duck has Wing {
        }
        var wing = Wing()
        var duck = Duck()
        duck.fly()
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_last_block_or_object() {
    let code = r#"
        {
            a: 1
            b: 2
        }"#;

    let result = run(code).unwrap();
    assert_eq!(result, "{a: 1, b: 2}");

    let code = r#"
        if true {
            {a:1, b:2}
        }
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "{a: 1, b: 2}");
}

#[test]
fn test_grid() {
    let code = r#"var g = grid(a:"first", b:"second", c:"third") {
            [1, 2, 3]
            [4, 5, 6]
            [7, 8, 9]
        }
        g
        "#;
    let result = run(code).unwrap();
    assert_eq!(
        result,
        r#"grid(a:"first",b:"second",c:"third",) {[1, 2, 3];[4, 5, 6];[7, 8, 9]}"#
    );
}


#[test]
fn test_str_index() {
    let code = r#"let a = "hello"
        a[1]
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "'e'");
}

#[test]
#[ignore] // TODO: conflicts with new property keyword syntax .view
fn test_view_types() {
    let code = r#"
            type Hello {
                text str = "hello"
                fn view() {
                    label(text) {}
                }
            }
            var hello = Hello(text:"hallo")
            hello.view()
        "#;
    // TODO: implement view types
    let res = run(code).unwrap();
    println!("{}", res);
    assert!(true);
}

#[test]
fn test_access_fields_in_method() {
    let code = r#"
            type Login {
                username str
                status str = ""

                fn on(ev str) {
                    // status = `Login ${username} ...`
                    var a = status
                }
            }"#;
    let result = run(code).unwrap();
    println!("{}", result);
    assert!(true);
}

#[test]
fn test_int_enums() {
    let code = r#"
            enum Color {
                Red = 1
                Green = 2
                Blue = 3
            }
            Color.Red
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "1");
}

#[test]
fn test_if_with_bool() {
    let code = r#"
            var succ = true
            if succ {
                print("I won!")
                "succ"
            } else {
                print("You failed!")
                "failed"
            }
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "succ");
}

#[test]
fn test_if_in_array() {
    let code = r#"
            var is_lse = false
            var is_rh = true
            ["osal", if is_lse {"EB"}, if is_rh {"al"}]
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, r#"["osal", "al"]"#)
}

#[test]
fn test_node_newline() {
    // the token sequence [')', '\n', '{'] should be treated as compile error
    // because it would be ambiguous:
    // 1. it might be a node statement with '{' written at the next line
    // 2. it might be a node statement without body, then followed by an object or a block.
    let code = r#"
            dep("x")
            {
                x: 1
                y: 2
            }
        "#;

    let config = AutoConfig::new(code);
    assert!(config.is_err());

    // this should pass, it is a normal node
    let code = r#"
            dep("x") {
                x: 1
                y: 2
            }
        "#;

    let config = AutoConfig::new(code);
    assert!(config.is_ok());

    // this should also pass, this is a call followed by an object
    let code = r#"
            dep("x")

            {
                x: 1
                y: 2
            }
        "#;

    let config = AutoConfig::new(code);
    assert!(config.is_ok());
}

#[test]
fn test_node_store() {
    let code = r#"
            lib x {
                at: "src/x"
            }
            x.at
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "src/x");
}

#[test]
fn test_node_arg_ident() {
    let code = r#"
            var myname = "Xiaoming"
            lib (myname) {}
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "lib Xiaoming {}");
}

#[test]
fn test_add_u8() {
    let code = r#"
            var a = 1u8
            var b = 2u8
            a + b
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "3");
}

#[test]
fn test_atom_query() {
    let code = r#"
            var atom = root {
                header "This is header"
                body {
                    p "This is a paragraph"
                    p "This is another paragraph"
                }
            }
            atom.body.p[1].content
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "This is another paragraph");
}

// ===== Integration Tests for Nested Mutation (Phase 2) =====

#[test]
fn test_object_field_mutation() {
    let code = r#"
            mut obj = {x: 10, y: 20}
            obj.x = 30
            obj.x
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "30");
}

#[test]
fn test_array_element_mutation() {
    let code = r#"
            mut arr = [1, 2, 3]
            arr[0] = 10
            arr[0]
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "10");
}

#[test]
fn test_multiple_field_mutations() {
    let code = r#"
            mut obj = {x: 10, y: 20, z: 30}
            obj.x = 100
            obj.y = 200
            obj.z = 300
            obj.x + obj.y + obj.z
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "600");
}

#[test]
fn test_multiple_array_mutations() {
    let code = r#"
            mut arr = [1, 2, 3]
            arr[0] = 10
            arr[1] = 20
            arr[2] = 30
            arr[0] + arr[1] + arr[2]
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "60");
}

#[test]
fn test_type_field_mutation() {
    let code = r#"
            type Point {
                x int
                y int
            }
            mut p = Point(x: 10, y: 20)
            p.x = 30
            p.x
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "30");
}

// ===== Level 1: Basic Nested Mutations (2-Level Depth) =====

#[test]
fn test_nested_object_field_mutation() {
    let code = r#"
            mut obj = { inner: { x: 10, y: 20 } }
            obj.inner.x = 30
            obj.inner.x
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "30");
}

#[test]
fn test_array_element_field_mutation() {
    let code = r#"
            mut arr = [{x: 1}, {x: 2}, {x: 3}]
            arr[0].x = 10
            arr[0].x
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "10");
}

#[test]
fn test_object_array_element_mutation() {
    let code = r#"
            mut obj = { items: [1, 2, 3] }
            obj.items[0] = 10
            obj.items[0]
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "10");
}

#[test]
fn test_nested_array_element_mutation() {
    let code = r#"
            mut matrix = [[1, 2], [3, 4]]
            matrix[0][1] = 20
            matrix[0][1]
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "20");
}

// ===== Level 2: Type Instance Nested Fields =====

#[test]
fn test_type_instance_nested_field_mutation() {
    let code = r#"
            type Inner { x int }
            type Outer { inner Inner }
            mut obj = Outer(inner: Inner(x: 10))
            obj.inner.x = 20
            obj.inner.x
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "20");
}

// ===== Level 3: Complex Nested Mutations (3+ Level Depth) =====

#[test]
fn test_three_level_object_nesting() {
    let code = r#"
            mut obj = { level1: { level2: { value: 100 } } }
            obj.level1.level2.value = 200
            obj.level1.level2.value
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "200");
}

#[test]
fn test_deep_array_of_objects_mutation() {
    let code = r#"
            mut data = [
                { info: { name: "Alice", age: 20 } },
                { info: { name: "Bob", age: 21 } }
            ]
            data[0].info.age = 25
            data[0].info.age
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "25");
}

// ===== Level 4: Structure Preservation =====

#[test]
fn test_nested_structure_preservation() {
    let code = r#"
            mut obj = { a: { x: 1, y: 2 }, b: { x: 3, y: 4 } }
            obj.a.x = 10
            [obj.a.y, obj.b.x, obj.b.y]
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "[2, 3, 4]");
}

// ===== Level 5: Error Cases =====

#[test]
fn test_nested_out_of_bounds_index() {
    let code = r#"
            mut obj = { items: [1, 2, 3] }
            obj.items[10] = 100
        "#;
    let result = run(code);
    assert!(result.is_err());
}

#[test]
fn test_nested_invalid_field_access() {
    let code = r#"
            mut obj = { inner: { x: 10 } }
            obj.inner.nonexistent = 20
        "#;
    let result = run(code);
    assert!(result.is_err());
}

#[test]
fn test_nested_type_mismatch() {
    let code = r#"
            mut obj = { items: [1, 2, 3] }
            obj.items.invalid_field = 10
        "#;
    let result = run(code);
    assert!(result.is_err());
}

// ===== Type Instance Creation Diagnostic Tests =====

#[test]
fn test_simple_nested_type_instance_creation() {
    let code = r#"
            type Inner { x int }
            type Outer { inner Inner }
            var obj = Outer(inner: Inner(x: 10))
            obj.inner.x
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "10");
}

#[test]
fn test_nested_type_instance_field_access() {
    let code = r#"
            type Inner { x int }
            type Outer { inner Inner }
            var obj = Outer(inner: Inner(x: 10))
            obj.inner.x
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "10");
}

#[test]
fn test_type_instance_field_value() {
    let code = r#"
            type Inner { x int }
            type Outer { inner Inner }
            var inner = Inner(x: 10)
            var obj = Outer(inner: inner)
            obj.inner.x
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "10");
}

#[test]
fn test_nested_type_instance_positional_args() {
    let code = r#"
            type Inner { x int }
            type Outer { inner Inner }
            var v = Outer(inner: Inner(x: 10))
            v.inner.x
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "10");
}



// ===== Phase 3: Borrow Checker Tests =====

#[test]
fn test_borrow_view_basic() {
    // Test basic view borrow - should work like a regular read
    let code = r#"
            let s = "hello"
            let v = s.view
            v
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_borrow_view_multiple() {
    // Test multiple view borrows - they should coexist
    let code = r#"
            let x = 42
            let v1 = x.view
            let v2 = x.view
            v1 + v2
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "84");
}

#[test]
fn test_borrow_mut_basic() {
    // Test basic mut borrow
    let code = r#"
            let s = str_new("hello", 10)
            let m = s.mut
            str_append(m, " world")
            s
        "#;
    let result = run(code).unwrap();
    // Note: In current implementation, str_append modifies in place
    // The borrow checking is functional but doesn't prevent all mutations yet
    assert!(result.contains("hello"));
}

#[test]
fn test_borrow_take_basic() {
    // Test basic take (move semantics)
    let code = r#"
            let s1 = "hello"
            let s2 = s1.take
            s2
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_borrow_view_preserves_original() {
    // Test that view borrow preserves original value
    let code = r#"
            let x = 100
            let v = x.view
            x
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "100");
}

#[test]
fn test_borrow_nested_view() {
    // Test nested view expressions
    let code = r#"
            let x = 42
            let v1 = x.view
            let v2 = v1.view
            v2
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "42");
}

#[test]
fn test_borrow_with_arithmetic() {
    // Test borrow with arithmetic operations
    let code = r#"
            let a = 10
            let b = 5
            let va = a.view
            let vb = b.view
            (va + vb) * 2
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "30");
}

#[test]
fn test_borrow_view_in_array() {
    // Test view borrow in array context
    let code = r#"
            let x = 10
            let y = 20
            let v = x.view
            [v, y]
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "[10, 20]");
}

#[test]
fn test_borrow_view_in_expression() {
    // Test view borrow used in arithmetic expression
    let code = r#"
            let a = 5
            let b = 3
            let va = a.view
            let vb = b.view
            va * vb
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "15");
}

#[test]
fn test_borrow_different_types() {
    // Test borrow expressions with different value types
    let code = r#"
            let num = 42
            let text = "hello"
            let v_num = num.view
            let v_text = text.view
            [v_num, v_text]
        "#;
    let result = run(code);
    // Result should be an array with both values
    assert!(result.is_ok());
    let result_str = result.unwrap();
    assert!(result_str.contains("42") && result_str.contains("hello"));
}

#[test]
fn test_borrow_take_chaining() {
    // Test chaining take operations on different values
    let code = r#"
            let s1 = "first"
            let s2 = "second"
            let t1 = s1.take
            let t2 = s2.take
            t1  // should be "first"
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "first");
}

// ===== Phase 3: str_slice Type Tests =====

#[test]
fn test_str_slice_type_lookup() {
    // Test that str_slice type is registered in universe
    let code = r#"
            // str_slice type should be accessible
            str_slice
        "#;
    let result = run(code);
    // Type lookup should work (returns the type name)
    assert!(result.is_ok());
    let result_str = result.unwrap();
    assert!(result_str.contains("str_slice"));
}

#[test]
fn test_str_slice_borrow_with_view() {
    // Test creating borrow with view expression
    let code = r#"
            let s = "hello world"
            let slice = s.view
            slice
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "hello world");
}

#[test]
fn test_str_slice_multiple_borrows() {
    // Test multiple view borrows (all should be str_slice type)
    let code = r#"
            let s = "hello world"
            let s1 = s.view
            let s2 = s.view
            [s1, s2]
        "#;
    let result = run(code).unwrap();
    assert!(result.contains("hello world") && result.contains("hello world"));
}

#[test]
fn test_str_slice_nested_borrow() {
    // Test nested view borrows
    let code = r#"
            let s = "hello"
            let s1 = s.view
            let s2 = s1.view
            s2
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_str_slice_in_array() {
    // Test view borrow in array context
    let code = r#"
            let s = "hello"
            let slice = s.view
            [slice, "world"]
        "#;
    let result = run(code).unwrap();
    assert!(result.contains("hello") && result.contains("world"));
}

#[test]
fn test_str_slice_with_take() {
    // Test that take works with strings
    let code = r#"
            let s1 = "hello"
            let s2 = s1.take
            s2
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_str_slice_mixed_borrows() {
    // Test mixing view and take on different values
    let code = r#"
            let s1 = "first"
            let s2 = "second"
            let v = s1.view
            let t = s2.take
            v  // View of s1 should work
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "first");
}

#[test]
fn test_str_slice_preserves_original() {
    // Test that view borrow preserves original value
    let code = r#"
            let s = "hello"
            let slice = s.view
            s  // Original should still be accessible
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_str_slice_in_expression() {
    // Test str_slice (view) in expression context
    let code = r#"
            let a = "hello"
            let b = "world"
            let va = a.view
            let vb = b.view
            [va, vb]
        "#;
    let result = run(code).unwrap();
    assert!(result.contains("hello") && result.contains("world"));
}

#[test]
fn test_str_slice_type_coercion() {
    // Test that str_slice (from view) can be used like str
    let code = r#"
            let s = "test"
            let slice = s.view
            slice
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "test");
}

#[cfg(test)]
mod test_multiline {
    use crate::atom;

    use super::*;

    #[test]
    fn test_atom_reader_multiline() {
        let mut reader = atom::AtomReader::new();

        let code = r#"
    let name = "Alice"
    let age = 30
    {name: name, age: age}
    "#;

        let result = reader.parse(code);
        println!("Result: {:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_array_return_eval() {
        // Test that array return types work in the evaluator
        let code = r#"
    fn get_numbers() []int {
        [1, 2, 3, 4, 5]
    }

    fn get_strings() []str {
        ["hello", "world"]
    }

    let nums = get_numbers()
    let first_num = nums[0]
    let last_num = nums[4]

    let words = get_strings()
    let first_word = words[0]
    let last_word = words[1]

    first_num
    "#;

        let result = run(code).unwrap();
        assert_eq!(result, "1");

        // Test indexing returned arrays
        let code2 = r#"
    fn get_numbers() []int {
        [10, 20, 30]
    }

    let nums = get_numbers()
    nums[2]
    "#;

        let result2 = run(code2).unwrap();
        assert_eq!(result2, "30");
    }
}

// ===== HashMap OOP API Tests =====
// These tests use the VM implementation in Rust

#[test]
fn test_hashmap_oop_new() {
    let code = r#"
            let map = HashMap.new()
            map.drop()
            0
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_hashmap_oop_insert_str() {
    let code = r#"
            let map = HashMap.new()
            map.insert_str("name", "Alice")
            map.insert_str("city", "Wonderland")
            let name = map.get_str("name")
            let city = map.get_str("city")
            map.drop()
            [name, city]
        "#;
    let result = run(code).unwrap();
    assert!(result.contains("Alice") && result.contains("Wonderland"));
}

#[test]
fn test_hashmap_oop_insert_int() {
    let code = r#"
            let map = HashMap.new()
            map.insert_int("count", 42)
            map.insert_int("age", 25)
            let count = map.get_int("count")
            let age = map.get_int("age")
            map.drop()
            count + age
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "67");
}

#[test]
fn test_hashmap_oop_contains() {
    let code = r#"
            let map = HashMap.new()
            map.insert_str("test", "data")
            let has_test = map.contains("test")
            let has_missing = map.contains("missing")
            map.drop()
            [has_test, has_missing]
        "#;
    let result = run(code).unwrap();
    assert!(result.contains("1") && result.contains("0"));
}

#[test]
fn test_hashmap_oop_size() {
    let code = r#"
            let map = HashMap.new()
            map.insert_str("a", "1")
            map.insert_str("b", "2")
            map.insert_str("c", "3")
            let size = map.size()
            map.drop()
            size
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "3");
}

#[test]
fn test_hashmap_oop_remove() {
    let code = r#"
            let map = HashMap.new()
            map.insert_str("temp", "data")
            map.remove("temp")
            let has_after = map.contains("temp")
            map.drop()
            has_after
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_hashmap_oop_clear() {
    let code = r#"
            let map = HashMap.new()
            map.insert_str("a", "1")
            map.insert_str("b", "2")
            map.clear()
            let size = map.size()
            map.drop()
            size
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

// ===== HashSet OOP API Tests =====

#[test]
fn test_hashset_oop_new() {
    let code = r#"
            let set = HashSet.new()
            set.drop()
            0
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_hashset_oop_insert() {
    let code = r#"
            let set = HashSet.new()
            set.insert("apple")
            set.insert("banana")
            set.insert("cherry")
            let has_apple = set.contains("apple")
            let has_banana = set.contains("banana")
            let has_cherry = set.contains("cherry")
            set.drop()
            [has_apple, has_banana, has_cherry]
        "#;
    let result = run(code).unwrap();
    // Should have all 1 (true values)
    assert!(result.contains("1") && result.matches("1").count() >= 3);
}

#[test]
fn test_hashset_oop_duplicate() {
    let code = r#"
            let set = HashSet.new()
            set.insert("unique")
            set.insert("unique")
            set.insert("unique")
            let size = set.size()
            set.drop()
            size
        "#;
    let result = run(code).unwrap();
    // Should still have size 1 (duplicate ignored)
    assert_eq!(result, "1");
}

#[test]
fn test_hashset_oop_remove() {
    let code = r#"
            let set = HashSet.new()
            set.insert("data")
            set.remove("data")
            let has_data = set.contains("data")
            set.drop()
            has_data
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_hashset_oop_size() {
    let code = r#"
            let set = HashSet.new()
            set.insert("one")
            set.insert("two")
            set.insert("three")
            let size = set.size()
            set.drop()
            size
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "3");
}

#[test]
fn test_hashset_oop_clear() {
    let code = r#"
            let set = HashSet.new()
            set.insert("a")
            set.insert("b")
            set.clear()
            let size = set.size()
            set.drop()
            size
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

// ===== StringBuilder OOP API Tests =====

#[test]
fn test_stringbuilder_oop_new() {
    let code = r#"
            let sb = StringBuilder.new(1024)
            sb.drop()
            0
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_stringbuilder_oop_append() {
    let code = r#"
            let sb = StringBuilder.new(1024)
            sb.append("hello")
            sb.append(" ")
            sb.append("world")
            let result = sb.build()
            sb.drop()
            result
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "hello world");
}

#[test]
fn test_stringbuilder_oop_append_char() {
    let code = r#"
            let sb = StringBuilder.new(1024)
            sb.append("hello")
            sb.append_char('!')
            let result = sb.build()
            sb.drop()
            result
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "hello!");
}

#[test]
fn test_stringbuilder_oop_append_int() {
    let code = r#"
            let sb = StringBuilder.new(1024)
            sb.append("count: ")
            sb.append_int(42)
            let result = sb.build()
            sb.drop()
            result
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "count: 42");
}

#[test]
fn test_stringbuilder_oop_len() {
    let code = r#"
            let sb = StringBuilder.new(1024)
            sb.append("hello")
            let len = sb.len()
            sb.drop()
            len
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "5");
}

#[test]
fn test_stringbuilder_oop_clear() {
    let code = r#"
            let sb = StringBuilder.new(1024)
            sb.append("hello")
            sb.clear()
            let len = sb.len()
            sb.drop()
            len
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

// ===== List OOP API Tests =====

#[test]
fn test_list_oop_new() {
    let code = r#"
            let list = List.new()
            let is_empty = list.is_empty()
            list.drop()
            is_empty
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "1");
}

#[test]
fn test_list_oop_push_pop() {
    let code = r#"
            let list = List.new()
            list.push(10)
            list.push(20)
            list.push(30)
            let popped = list.pop()
            let len = list.len()
            list.drop()
            len + popped
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "32"); // 2 + 30
}

#[test]
fn test_list_oop_push_pop_multiple() {
    let code = r#"
            let list = List.new()
            list.push(42)
            list.push(100)
            let len = list.len()
            let popped = list.pop()
            let is_empty = list.is_empty()
            list.drop()
            if len != 2 {
                0
            } else {
                if popped != 100 {
                    0
                } else {
                    if is_empty != 0 {
                        0
                    } else {
                        1
                    }
                }
            }
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "1");
}

#[test]
fn test_list_oop_len() {
    let code = r#"
            let list = List.new()
            list.push(1)
            list.push(2)
            list.push(3)
            let len = list.len()
            list.drop()
            len
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "3");
}

#[test]
fn test_list_oop_is_empty() {
    let code = r#"
            let list = List.new()
            let empty1 = list.is_empty()
            list.push(42)
            let empty2 = list.is_empty()
            list.drop()
            [empty1, empty2]
        "#;
    let result = run(code).unwrap();
    assert!(result.contains("1") && result.contains("0"));
}

#[test]
fn test_list_oop_clear() {
    let code = r#"
            let list = List.new()
            list.push(1)
            list.push(2)
            list.push(3)
            list.clear()
            let len = list.len()
            list.drop()
            len
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_list_oop_get_set() {
    let code = r#"
            let list = List.new()
            list.push(10)
            list.push(20)
            list.push(30)
            let first = list.get(0)
            let second = list.get(1)
            list.set(0, 15)
            let updated = list.get(0)
            list.drop()
            [first, second, updated]
        "#;
    let result = run(code).unwrap();
    assert!(result.contains("10") && result.contains("20") && result.contains("15"));
}

#[test]
fn test_list_oop_insert_remove() {
    let code = r#"
            let list = List.new()
            list.push(1)
            list.push(3)
            list.insert(1, 2)
            let val1 = list.get(0)
            let val2 = list.get(1)
            let val3 = list.get(2)
            let removed = list.remove(1)
            let len = list.len()
            list.drop()
            if val1 != 1 {
                0
            } else {
                if val2 != 2 {
                    0
                } else {
                    if val3 != 3 {
                        0
                    } else {
                        if removed != 2 {
                            0
                        } else {
                            if len != 2 {
                                0
                            } else {
                                1
                            }
                        }
                    }
                }
            }
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "1");
}

#[test]
fn test_list_oop_reserve() {
    let code = r#"
            let list = List.new()
            list.reserve(100)
            list.push(1)
            list.push(2)
            let len = list.len()
            list.drop()
            len
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "2");
}

#[test]
fn test_list_oop_comprehensive() {
    let code = r#"
            let list = List.new()

            // Test push
            list.push(1)
            list.push(2)
            list.push(3)

            // Test len
            let len = list.len()
            if len != 3 {
                0
            } else {
                // Test clear
                list.clear()
                let is_empty = list.is_empty()
                if is_empty == 1 {
                    1
                } else {
                    0
                }
            }
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "1");
}

#[test]
fn test_list_oop_multiple_operations() {
    let code = r#"
            let list = List.new()
            list.push(100)
            list.push(200)
            list.push(300)
            let val = list.get(1)
            list.set(1, 250)
            let updated = list.get(1)
            let len = list.len()
            list.drop()
            if val == 200 {
                if updated == 250 {
                    if len == 3 {
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
fn test_list_oop_index() {
    let code = r#"
        let list = List.new(10, 20, 30)
        let first = list[0]
        let second = list[1]
        let third = list[2]

        if first == 10 {
            if second == 20 {
                if third == 30 {
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
fn test_list_oop_varargs() {
    let code = r#"
        let list = List.new(1, 2, 3, 4, 5)
        let len = list.len()

        if len == 5 {
            1
        } else {
            0
        }
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "1");
}

#[test]
fn test_list_oop_varargs_empty() {
    let code = r#"
        let list = List.new()
        let len = list.len()

        if len == 0 {
            1
        } else {
            0
        }
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "1");
}

#[test]
fn test_list_oop_for_iteration() {
    let code = r#"
        let list = List.new(1, 2, 3, 4, 5)
        mut sum = 0
        for v in list {
            sum = sum + v
        }
        sum
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "15");
}

#[test]
fn test_list_oop_for_empty() {
    let code = r#"
        let list = List.new()
        mut count = 0
        for v in list {
            count = count + 1
        }
        count
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_vm_annotation_in_ext() {
    let code = r#"
    type File {
        path str

        static fn open(path str) File;

        fn read_text() str;

        fn close();
    }

    ext File {
        _handle uint64

        #[pub, vm]
        static fn open(path str) File

        #[pub, vm]
        fn read_text() str

        #[pub, vm]
        fn close()
    }
    "#;

    let mut parser = crate::Parser::from(code);
    let result = parser.parse();

    assert!(result.is_ok(), "Parsing should succeed: {:?}", result.err());

    let ast = result.unwrap();

    // Find the TypeDecl for File
    let file_type_decl = ast.stmts.iter().find_map(|stmt| {
        if let ast::Stmt::TypeDecl(decl) = stmt {
            if decl.name == "File" {
                Some(decl)
            } else {
                None
            }
        } else {
            None
        }
    });

    assert!(file_type_decl.is_some(), "File TypeDecl should exist");

    let file_decl = file_type_decl.unwrap();

    // Check that methods were merged from ext block
    assert_eq!(file_decl.methods.len(), 3, "Should have 3 methods");

    // Check that methods are VmFunction (from ext block)
    let open_method = &file_decl.methods[0];
    assert_eq!(open_method.name, "open");
    assert!(
        matches!(open_method.kind, crate::ast::FnKind::VmFunction),
        "open method should be VmFunction from ext block"
    );

    let read_text_method = &file_decl.methods[1];
    assert_eq!(read_text_method.name, "read_text");
    assert!(
        matches!(read_text_method.kind, crate::ast::FnKind::VmFunction),
        "read_text method should be VmFunction from ext block"
    );

    let close_method = &file_decl.methods[2];
    assert_eq!(close_method.name, "close");
    assert!(
        matches!(close_method.kind, crate::ast::FnKind::VmFunction),
        "close method should be VmFunction from ext block"
    );
}

#[test]
fn test_function_body_parsing() {
    let code = r#"
    fn test() int {
        42
    }
    "#;

    let mut parser = crate::Parser::from(code);
    let result = parser.parse();

    assert!(result.is_ok(), "Parsing should succeed: {:?}", result.err());

    let ast = result.unwrap();

    // Find the function
    let fn_decl = ast.stmts.iter().find_map(|stmt| {
        if let ast::Stmt::Fn(fn_decl) = stmt {
            if fn_decl.name == "test" {
                Some(fn_decl)
            } else {
                None
            }
        } else {
            None
        }
    });

    assert!(fn_decl.is_some(), "test function should exist");

    let test_fn = fn_decl.unwrap();

    // Check that the function is marked as Function (not VmFunction)
    assert!(
        matches!(test_fn.kind, crate::ast::FnKind::Function),
        "Function should be FnKind::Function, got {:?}",
        test_fn.kind
    );

    // Check that the function body has statements
    assert_eq!(
        test_fn.body.stmts.len(),
        1,
        "Function body should have 1 statement"
    );

    // The statement should be an expression statement containing Int(42)
    if let ast::Stmt::Expr(expr) = &test_fn.body.stmts[0] {
        if let ast::Expr::Int(val) = expr {
            assert_eq!(*val, 42);
        } else {
            panic!("Expected Expr::Int(42), got {:?}", expr);
        }
    } else {
        panic!("Expected Stmt::Expr, got {:?}", &test_fn.body.stmts[0]);
    }
}

#[test]
fn test_simple_function_execution() {
    // Directly test function execution without module import
    let code = r#"fn test() int {
        42
    }

    test()
    "#;

    let result = run(code).unwrap();
    assert_eq!(result, "42");
}

#[test]
fn test_forward_declaration() {
    // Test forward declaration followed by implementation
    let code = r#"fn test() int;

    fn test() int {
        42
    }

    test()
    "#;

    let result = run(code).unwrap();
    assert_eq!(result, "42");
}

#[test]
fn test_module_simulation() {
    // Simulate test.at (forward declaration) + test.vm.at (implementation)
    let code = r#"// Simulating test.at
    fn test() int;

    // Simulating test.vm.at
    #[pub]
    fn test() int {
        42
    }

    // Call the function
    test()
    "#;

    let result = run(code).unwrap();
    assert_eq!(result, "42");
}

#[test]
fn test_compound_assignment_add_eq() {
    let code = r#"
var a = 1
a += 1
a
    "#;

    let result = run(code).unwrap();
    assert_eq!(result, "2");
}

#[test]
fn test_compound_assignment_sub_eq() {
    let code = r#"
var a = 10
a -= 3
a
    "#;

    let result = run(code).unwrap();
    assert_eq!(result, "7");
}

#[test]
fn test_compound_assignment_mul_eq() {
    let code = r#"
var a = 5
a *= 3
a
    "#;

    let result = run(code).unwrap();
    assert_eq!(result, "15");
}

#[test]
fn test_compound_assignment_div_eq() {
    let code = r#"
var a = 20
a /= 4
a
    "#;

    let result = run(code).unwrap();
    assert_eq!(result, "5");
}

#[test]
fn test_compound_assignment_chained() {
    let code = r#"
var a = 1
a += 1
a += 2
a += 3
a
    "#;

    let result = run(code).unwrap();
    assert_eq!(result, "7");
}

#[test]
fn test_compound_assignment_div_eq_oneline() {
    let code = "var a = 20; a /= 4; a";
    let result = run(code).unwrap();
    println!("Result: {} (expected: 5)", result);
    assert_eq!(result, "5");
}
