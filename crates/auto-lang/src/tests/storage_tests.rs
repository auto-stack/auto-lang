use crate::parse_preserve_error;
use crate::error::attach_source;
use crate::ast::{Type, StorageType, StorageKind};

#[test]
fn test_parse_dynamic_storage() {
    let code = r#"
        fn test() {
            let x Dynamic
        }
    "#;

    match parse_preserve_error(code) {
        Ok(_) => println!("✓ Dynamic storage parsed"),
        Err(e) => {
            let err_with_src = attach_source(e, "test.at".to_string(), code.to_string());
            eprintln!("✗ Dynamic storage failed: {}\n", err_with_src);
            panic!("Failed to parse Dynamic storage");
        }
    }
}

#[test]
fn test_parse_fixed_storage() {
    let code = r#"
        fn test() {
            let x Fixed<int>
        }
    "#;

    match parse_preserve_error(code) {
        Ok(_) => println!("✓ Fixed<int> storage parsed"),
        Err(e) => {
            let err_with_src = attach_source(e, "test.at".to_string(), code.to_string());
            eprintln!("✗ Fixed<int> storage failed: {}\n", err_with_src);
            panic!("Failed to parse Fixed<int> storage");
        }
    }
}

// TODO: Re-enable this test when trait bounds are implemented
// #[test]
// fn test_parse_storage_in_type_annotation() {
//     let code = r#"
//         type List<T, S : Storage = Dynamic> {
//             elems: [~]T
//         }
//     "#;
//
//     match parse_preserve_error(code) {
//         Ok(_) => println!("✓ Type with Storage parameter parsed"),
//         Err(e) => {
//             let err_with_src = attach_source(e, "test.at".to_string(), code.to_string());
//             eprintln!("✗ Type with Storage parameter failed: {}\n", err_with_src);
//             panic!("Failed to parse type with Storage parameter");
//         }
//     }
// }

#[test]
fn test_storage_type_display() {
    let storage_dynamic = Type::Storage(StorageType {
        kind: StorageKind::Dynamic,
    });
    assert_eq!(storage_dynamic.to_string(), "Dynamic");

    let storage_fixed = Type::Storage(StorageType {
        kind: StorageKind::Fixed { capacity: 64 },
    });
    assert_eq!(storage_fixed.to_string(), "Fixed<64>");
}
