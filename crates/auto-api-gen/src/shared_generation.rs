//! Compatibility path; implementation has one owner in auto-generation-core.
pub use auto_generation_core::shared_generation::{
    assembly_freshness, bind_workspace_lock, code_region_mask_text, extract_api_lenient,
    extract_db_fn_names, lock_freshness, post_process_companion_rs, post_process_db_rs,
    primary_type_name_pub, promote_api_types_in_companion, qualify_a2r_std,
    transpile_back_module_to_rs, transpile_back_module_to_rs_with_proofs, transpile_db_to_rs,
    try_full_parse,
};
#[cfg(test)]
pub(crate) use auto_generation_core::shared_generation::{parse_fields, parse_params};
