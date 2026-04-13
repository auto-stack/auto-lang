use crate::{
    error::AutoResult,
    trans::rust::transpile_rust,
};
use std::fs::read_to_string;
use std::path::PathBuf;

fn test_a2r(case: &str) -> AutoResult<()> {
    // Parse test case name: "000_hello" -> "hello"
    let parts: Vec<&str> = case.split("_").collect();
    let name = parts[1..].join("_");

    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_path = format!("test/a2r/{}/{}.at", case, name);
    let src_path = d.join(src_path);
    let src = read_to_string(src_path.as_path())?;

    let exp_path = format!("test/a2r/{}/{}.expected.rs", case, name);
    let exp_path = d.join(exp_path);
    let expected = if !exp_path.is_file() {
        "".to_string()
    } else {
        read_to_string(exp_path.as_path())?
    };

    let mut rcode = transpile_rust(&name, &src)?;
    let rs_code = rcode.done()?;

    if rs_code != expected.as_bytes() {
        // Generate .wrong.rs for comparison
        let gen_path = format!("test/a2r/{}/{}.wrong.rs", case, name);
        let gen_path = d.join(gen_path);
        std::fs::write(&gen_path, rs_code)?;
    }

    assert_eq!(String::from_utf8_lossy(rs_code), expected);
    Ok(())
}

#[test]
fn test_000_hello() {
    test_a2r("000_hello").unwrap();
}

#[test]
fn test_001_sqrt() {
    test_a2r("001_sqrt").unwrap();
}

#[test]
fn test_002_array() {
    test_a2r("002_array").unwrap();
}

#[test]
fn test_003_func() {
    test_a2r("003_func").unwrap();
}

#[test]
fn test_005_pointer() {
    test_a2r("005_pointer").unwrap();
}

#[test]
fn test_006_struct() {
    test_a2r("006_struct").unwrap();
}

#[test]
fn test_007_enum() {
    test_a2r("007_enum").unwrap();
}

#[test]
fn test_008_method() {
    test_a2r("008_method").unwrap();
}

#[test]
fn test_010_if() {
    test_a2r("010_if").unwrap();
}

#[test]
fn test_011_for() {
    test_a2r("011_for").unwrap();
}

#[test]
fn test_012_is() {
    test_a2r("012_is").unwrap();
}

#[test]
fn test_013_while() {
    test_a2r("013_while").unwrap();
}

#[test]
fn test_014_closure() {
    test_a2r("014_closure").unwrap();
}

#[test]
fn test_015_nested_if() {
    test_a2r("015_nested_if").unwrap();
}

#[test]
fn test_016_complex() {
    test_a2r("016_complex").unwrap();
}

#[test]
fn test_017_struct_methods() {
    test_a2r("017_struct_methods").unwrap();
}

#[test]
fn test_018_enum_pattern() {
    test_a2r("018_enum_pattern").unwrap();
}

#[test]
fn test_019_blocks() {
    test_a2r("019_blocks").unwrap();
}

#[test]
fn test_020_comprehensive() {
    test_a2r("020_comprehensive").unwrap();
}

#[test]
fn test_021_indexing() {
    test_a2r("021_indexing").unwrap();
}

#[test]
fn test_022_unary() {
    test_a2r("022_unary").unwrap();
}

#[test]
fn test_023_arithmetic() {
    test_a2r("023_arithmetic").unwrap();
}

#[test]
fn test_024_fstring() {
    test_a2r("024_fstring").unwrap();
}

#[test]
fn test_025_fstring_edge() {
    test_a2r("025_fstring_edge").unwrap();
}

#[test]
fn test_026_ref_expr() {
    test_a2r("026_ref_expr").unwrap();
}

#[test]
fn test_027_range_expr() {
    test_a2r("027_range_expr").unwrap();
}

#[test]
fn test_028_object() {
    test_a2r("028_object").unwrap();
}

#[test]
fn test_029_composition() {
    test_a2r("029_composition").unwrap();
}

#[test]
fn test_030_field_composition() {
    test_a2r("030_field_composition").unwrap();
}

#[test]
fn test_031_spec() {
    test_a2r("031_spec").unwrap();
}

#[test]
fn test_032_delegation() {
    test_a2r("032_delegation").unwrap();
}

#[test]
fn test_033_multi_delegation() {
    test_a2r("033_multi_delegation").unwrap();
}

#[test]
fn test_034_delegation_params() {
    test_a2r("034_delegation_params").unwrap();
}

#[test]
fn test_111_generic_alias() {
    test_a2r("111_generic_alias").unwrap();
}

#[test]
fn test_126_generic_field() {
    test_a2r("126_generic_field").unwrap();
}

#[test]
fn test_127_generic_ptr_field() {
    test_a2r("127_generic_ptr_field").unwrap();
}

#[test]
fn test_128_map_type() {
    test_a2r("128_map_type").unwrap();
}

#[test]
fn test_129_map_func() {
    test_a2r("129_map_func").unwrap();
}

#[test]
fn test_130_option_construct() {
    test_a2r("130_option_construct").unwrap();
}

#[test]
fn test_143_empty_variant_match() {
    test_a2r("143_empty_variant_match").unwrap();
}

#[test]
fn test_131_method_chain() {
    test_a2r("131_method_chain").unwrap();
}

#[test]
fn test_110_const_generics() {
    test_a2r("110_const_generics").unwrap();
}

#[test]
fn test_109_generic_hetero_enum() {
    test_a2r("109_generic_hetero_enum").unwrap();
}

#[test]
fn test_035_inheritance() {
    test_a2r("035_inheritance").unwrap();
}

#[test]
fn test_055_union() {
    test_a2r("055_union").unwrap();
}

#[test]
fn test_014_hetero_enum() {
    test_a2r("014_hetero_enum").unwrap();
}

#[test]
fn test_004_cstr() {
    test_a2r("004_cstr").unwrap();
}

#[test]
fn test_023_borrow_view() {
    test_a2r("023_borrow_view").unwrap();
}

#[test]
fn test_024_borrow_mut() {
    test_a2r("024_borrow_mut").unwrap();
}

#[test]
fn test_025_borrow_move() {
    test_a2r("025_borrow_move").unwrap();
}

#[test]
fn test_026_borrow_conflicts() {
    test_a2r("026_borrow_conflicts").unwrap();
}

#[test]
fn test_016_basic_spec() {
    test_a2r("016_basic_spec").unwrap();
}

#[test]
fn test_017_spec() {
    test_a2r("017_spec").unwrap();
}

#[test]
fn test_117_list_storage() {
    test_a2r("117_list_storage").unwrap();
}

// Plan 120: Option and Result type tests
#[test]
fn test_120_option() {
    test_a2r("120_option").unwrap();
}

// Plan 159 Phase 6B-1: is statement with multi-statement match arms
#[test]
fn test_132_is_multi_stmt() {
    test_a2r("132_is_multi_stmt").unwrap();
}

// Plan 159 Phase 6B-2.6: External crate use statement
#[test]
fn test_133_rust_use() {
    test_a2r("133_rust_use").unwrap();
}

// Plan 159 Phase 6B-2.1: async fn transpilation
#[test]
fn test_134_async_fn() {
    test_a2r("134_async_fn").unwrap();
}

// Plan 159 Phase 6B-2.3: derive attribute passthrough
#[test]
fn test_135_derive_attr() {
    test_a2r("135_derive_attr").unwrap();
}

// Plan 162: Type cast — expr.as(Type)
#[test]
fn test_136_type_cast() {
    test_a2r("136_type_cast").unwrap();
}

// Plan 162: Pointer methods — is_null, is_not_null, add, read, write, .ptr
#[test]
fn test_137_ptr_methods() {
    test_a2r("137_ptr_methods").unwrap();
}

// Plan 162: List-style methods with .as() type casts and or operator
#[test]
fn test_138_list_as_cast() {
    test_a2r("138_list_as_cast").unwrap();
}

#[test]
fn test_139_if_multistmt() {
    test_a2r("139_if_multistmt").unwrap();
}

#[test]
fn test_140_if_return() {
    test_a2r("140_if_return").unwrap();
}

#[test]
fn test_141_func_literal_return() {
    test_a2r("141_func_literal_return").unwrap();
}

// Plan 162: .to(Type) explicit type conversion
#[test]
fn test_142_to_convert() {
    test_a2r("142_to_convert").unwrap();
}

// Plan 163: static fn support
#[test]
fn test_148_static_fn() {
    test_a2r("148_static_fn").unwrap();
}

// Plan 163: pub visibility
#[test]
fn test_149_pub_visibility() {
    test_a2r("149_pub_visibility").unwrap();
}

// Plan 163: #[tokio::main] + async main
#[test]
fn test_150_tokio_main() {
    test_a2r("150_tokio_main").unwrap();
}

// Plan 163: &mut self methods
#[test]
fn test_151_mut_self() {
    test_a2r("151_mut_self").unwrap();
}

// Plan 163: per-field serde attributes
#[test]
fn test_152_field_attrs() {
    test_a2r("152_field_attrs").unwrap();
}

// Plan 164: ext Type for Trait { } — external trait impl
#[test]
fn test_153_ext_for() {
    test_a2r("153_ext_for").unwrap();
}

// Plan 165: Struct destructuring in is match arms
#[test]
fn test_154_struct_destructure() {
    test_a2r("154_struct_destructure").unwrap();
}

// Plan 166: Generic constraints #[with(T as Trait)]
#[test]
fn test_155_with_constraint() {
    test_a2r("155_with_constraint").unwrap();
}

// Plan 159 Phase 6B-2.7: impl From<A> for B via ext-for
#[test]
fn test_156_ext_from() {
    test_a2r("156_ext_from").unwrap();
}

// Plan 6B-3.4: const declaration
#[test]
fn test_157_const_decl() {
    test_a2r("157_const_decl").unwrap();
}

// Plan 6B-4.14: Box::new() / Arc::new() smart pointer constructors
#[test]
fn test_158_box_arc() {
    test_a2r("158_box_arc").unwrap();
}
