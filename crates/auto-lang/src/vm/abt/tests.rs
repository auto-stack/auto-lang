//! Plan 226 Phase 6: Round-trip integration tests for ABT
//!
//! Verifies: Auto → ABC → ABT → ABC produces equivalent bytecode.

/// Round-trip test helper: compile → disassemble → parse → assemble → compare.
fn assert_roundtrip(source: &str) {
    let (vm, _, _, _) = crate::create_vm_from_source(source).expect("compile failed");
    let original_flash = &*vm.flash;

    // 1. Disassemble ABC → ABT
    let strings = vm.strings.read().unwrap();
    let abt = super::disasm::disassemble_flash(original_flash, Some(&strings));
    let abt_text = abt.to_string();

    // 2. Parse ABT text → AbtProgram
    let parsed = super::parser::parse(&abt_text).expect("parse failed");

    // 3. Assemble AbtProgram → CompiledPackage
    let reassembled = super::asm::assemble(&parsed).expect("assemble failed");

    // 4. Compare key properties
    assert_eq!(
        original_flash.memory.len(),
        reassembled.bytecode.len(),
        "Bytecode length mismatch after round-trip\nOriginal: {} bytes\nReassembled: {} bytes\n\nABT text:\n{}",
        original_flash.memory.len(),
        reassembled.bytecode.len(),
        abt_text
    );

    // Bytecode content should match exactly
    assert_eq!(
        original_flash.memory, reassembled.bytecode,
        "Bytecode content mismatch after round-trip\n\nABT text:\n{}",
        abt_text
    );

    // Exports should match
    assert_eq!(
        original_flash.exports_by_name.len(),
        reassembled.exports.len(),
        "Export count mismatch"
    );
    for (name, &addr) in &original_flash.exports_by_name {
        assert_eq!(
            reassembled.exports.get(name),
            Some(&addr),
            "Export address mismatch for '{}'",
            name
        );
    }

    // Object keys should match
    assert_eq!(
        original_flash.object_keys.len(),
        reassembled.object_keys.len(),
        "Object keys count mismatch"
    );

    // Object types should match
    assert_eq!(
        original_flash.object_types.len(),
        reassembled.object_types.len(),
        "Object types count mismatch"
    );
}

#[test]
fn test_roundtrip_empty_main() {
    assert_roundtrip(r#"
fn main() {
}
"#);
}

#[test]
fn test_roundtrip_hello_world() {
    assert_roundtrip(r#"
fn main() {
    print("Hello")
}
"#);
}

#[test]
fn test_roundtrip_add_function() {
    assert_roundtrip(r#"
fn add(a, b) {
    return a + b
}

fn main() {
    print(add(3, 4))
}
"#);
}

#[test]
fn test_roundtrip_local_vars() {
    assert_roundtrip(r#"
fn main() {
    let x = 10
    let y = 20
    print(x + y)
}
"#);
}

#[test]
fn test_roundtrip_if_else() {
    assert_roundtrip(r#"
fn main() {
    let x = 5
    if x > 3 {
        print(1)
    } else {
        print(0)
    }
}
"#);
}

#[test]
fn test_roundtrip_loop() {
    assert_roundtrip(r#"
fn main() {
    var sum = 0
    for i in 0..3 {
        sum = sum + i
    }
    print(sum)
}
"#);
}

#[test]
fn test_roundtrip_call_nat() {
    assert_roundtrip(r#"
fn main() {
    print(42)
}
"#);
}

#[test]
fn test_roundtrip_float() {
    assert_roundtrip(r#"
fn main() {
    let x = 3.14
    print(x)
}
"#);
}

#[test]
fn test_roundtrip_negative_int() {
    assert_roundtrip(r#"
fn main() {
    let x = -42
    print(x)
}
"#);
}

#[test]
fn test_roundtrip_bool() {
    assert_roundtrip(r#"
fn main() {
    let t = true
    let f = false
    print(t)
    print(f)
}
"#);
}

// ============================================================================
// Plan 226 Phase 6b: Direct ABT execution tests
// ============================================================================

#[test]
fn test_create_vm_from_abt_basic() {
    let abt = r#"
.exports
  main -> @main

.code
main:
  fn.prolog 0, 0
  const.i32 42
  call.nat nat#1
  ret 0
"#;
    let (vm, output, main_entry) = crate::create_vm_from_abt(abt).expect("create_vm_from_abt failed");
    assert_eq!(main_entry, 0, "main entry should be 0");

    let task_id = vm.spawn_task(main_entry, 16384);
    crate::get_global_runtime().block_on(async {
        vm.run_task_loop().await;
    });

    let result = output.read().unwrap().clone();
    assert_eq!(result, "42\n", "Expected '42\\n' in output, got: {:?}", result);
}

#[tokio::test]
async fn test_run_abt_print_i32() {
    let abt = r#"
.exports
  main -> @main

.code
main:
  fn.prolog 0, 0
  const.i32 42
  call.nat nat#1
  ret 0
"#;
    let output = crate::run_abt(abt).await.expect("run_abt failed");
    assert_eq!(output, "42\n", "Expected '42\\n' in output, got: {:?}", output);
}

#[tokio::test]
async fn test_run_abt_print_multiple() {
    let abt = r#"
.exports
  main -> @main

.code
main:
  fn.prolog 0, 0
  const.i32 1
  call.nat nat#1
  const.i32 2
  call.nat nat#1
  const.i32 3
  call.nat nat#1
  ret 0
"#;
    let output = crate::run_abt(abt).await.expect("run_abt failed");
    assert_eq!(output, "1\n2\n3\n", "Expected '1\\n2\\n3\\n' in output, got: {:?}", output);
}

#[tokio::test]
async fn test_run_abt_arithmetic() {
    let abt = r#"
.exports
  main -> @main

.code
main:
  fn.prolog 0, 0
  const.i32 10
  const.i32 20
  add
  call.nat nat#1
  ret 0
"#;
    let output = crate::run_abt(abt).await.expect("run_abt failed");
    assert_eq!(output, "30\n", "Expected '30\\n' in output, got: {:?}", output);
}
