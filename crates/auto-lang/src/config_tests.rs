use crate::{config::AutoConfig, eval_config};
use auto_val::Obj;

#[test]
fn test_node_props() {
    let code = r#"
            parent("parent") {
                size: 10
                kid("kid1") {}
            }
        "#;
    let interp = eval_config(code, &Obj::new()).unwrap();
    let config = interp.result.as_node();
    let parent = config
        .kids_iter()
        .filter(|(_, kid)| matches!(kid, auto_val::Kid::Node(_)))
        .map(|(_, kid)| {
            if let auto_val::Kid::Node(n) = kid {
                n
            } else {
                unreachable!()
            }
        })
        .nth(0)
        .unwrap();
    let size = parent.get_prop_of("size").to_uint();
    assert_eq!(size, 10);
}

#[test]
fn test_config() {
    let code = r#"
name: "hello"
version: "0.1.0"

exe hello {
    dir: "src"
    main: "main.c"
}"#;
    let interp = eval_config(code, &Obj::new()).unwrap();
    let result = interp.result;
    assert_eq!(
        result.repr(),
        r#"root {name: "hello"; version: "0.1.0"; exe hello {dir: "src"; main: "main.c"}}"#
    );
}

#[test]
fn test_config_with_node() {
    let code = r#"
        name: "hello"

        var dirs = ["a", "b", "c"]

        lib hello {
            for d in dirs {
                dir(d) {}
            }
        }
        "#;

    let conf = AutoConfig::new(code).unwrap();

    assert_eq!(
        conf.root.to_string(),
        r#"root {name: "hello"; lib hello {dir a {}; dir b {}; dir c {}}}"#
    );
}

#[test]
fn test_config_with_deep_data() {
    let code = r#"let dirs = ["a" , "b", "c"]
for d in dirs {
    dir(id: d) {
        at: d
    }
}
"#;
    let interp = eval_config(code, &auto_val::Obj::new()).unwrap();
    assert_eq!(
        interp.result.repr(),
        r#"root {dirs: [dir a {at: "a"}, dir b {at: "b"}, dir c {at: "c"}]}"#
    );
}