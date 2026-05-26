// Plan 212 Phase 3C-v2: syn AST source scanner
//
// Scans Rust crate source code from ~/.cargo/registry/src/ to extract
// public function signatures for automatic FFI shim generation.
//
// Usage:
//   let sigs = scan_crate_signatures("serde_json")?;
//   // sigs = {"from_str": FunctionShim{param_types:[CString], return_type:CString}, ...}

use std::collections::HashMap;
use std::path::PathBuf;

use super::sandbox::{FunctionShim, ShimType};

/// Scan a crate's source code and extract public function signatures.
///
/// Looks for the crate source in `~/.cargo/registry/src/{crate_name}-*/src/lib.rs`.
/// Uses syn to parse the AST and extract `pub fn` signatures.
pub fn scan_crate_signatures(crate_name: &str) -> Result<HashMap<String, FunctionShim>, String> {
    let cargo_reg = find_cargo_registry_src()?;
    let crate_dir = find_crate_source(&cargo_reg, crate_name)?;
    let lib_rs = crate_dir.join("src").join("lib.rs");

    if !lib_rs.exists() {
        return Err(format!(
            "lib.rs not found at {}",
            lib_rs.display()
        ));
    }

    let source = std::fs::read_to_string(&lib_rs)
        .map_err(|e| format!("Failed to read {}: {}", lib_rs.display(), e))?;

    let mut result = HashMap::new();

    // Parse the top-level file
    scan_file_for_fns(&source, &mut result, &crate_dir.join("src"), crate_name);

    log::info!(
        "syn scan: found {} pub fns in {}",
        result.len(),
        crate_name
    );

    Ok(result)
}

/// Find ~/.cargo/registry/src/ directory.
fn find_cargo_registry_src() -> Result<PathBuf, String> {
    let home = dirs::home_dir()
        .ok_or_else(|| "Cannot find home directory".to_string())?;
    let reg = home.join(".cargo").join("registry").join("src");
    if !reg.exists() {
        return Err(format!("Cargo registry src not found at {}", reg.display()));
    }
    Ok(reg)
}

/// Find the source directory for a crate in the cargo registry.
///
/// Searches for `{crate_name}-*` directories (version suffix).
fn find_crate_source(registry_src: &PathBuf, crate_name: &str) -> Result<PathBuf, String> {
    // The registry src has subdirs like "index.crates.io-1949cf8c6b5b557f/"
    for entry in std::fs::read_dir(registry_src)
        .map_err(|e| format!("Failed to read registry src: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read dir entry: {}", e))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        // Look for crate_name-* inside this host dir
        let entries = std::fs::read_dir(&path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        for crate_entry in entries {
            let crate_entry = crate_entry.map_err(|e| format!("Dir entry error: {}", e))?;
            let crate_path = crate_entry.path();
            let dir_name = crate_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            // Match "{crate_name}-" prefix (e.g., "serde_json-1.0.128")
            if dir_name.starts_with(&format!("{}-", crate_name)) {
                return Ok(crate_path);
            }
        }
    }
    Err(format!(
        "Crate source not found for {} in {}",
        crate_name,
        registry_src.display()
    ))
}

/// Parse a Rust source file and extract public function signatures.
fn scan_file_for_fns(
    source: &str,
    result: &mut HashMap<String, FunctionShim>,
    src_dir: &PathBuf,
    crate_name: &str,
) {
    let ast = match syn::parse_file(source) {
        Ok(ast) => ast,
        Err(e) => {
            log::warn!("syn parse error: {}", e);
            return;
        }
    };

    for item in &ast.items {
        match item {
            syn::Item::Fn(fn_item) => {
                if is_public(&fn_item.vis) && !has_receiver(&fn_item.sig) {
                    extract_fn_signature(&fn_item.sig, fn_item.sig.ident.to_string(), result);
                }
            }
            syn::Item::Mod(mod_item) => {
                if is_public(&mod_item.vis) {
                    // If inline mod content is available, scan it
                    if let Some((_, content)) = &mod_item.content {
                        scan_mod_content(content, result, src_dir, crate_name);
                    } else {
                        // External mod file: src/{mod_name}.rs or src/{mod_name}/mod.rs
                        let mod_name = mod_item.ident.to_string();
                        let mod_file = src_dir.join(format!("{}.rs", mod_name));
                        let mod_dir = src_dir.join(&mod_name).join("mod.rs");
                        let path_to_load = if mod_file.exists() {
                            Some(mod_file)
                        } else if mod_dir.exists() {
                            Some(mod_dir)
                        } else {
                            None
                        };
                        if let Some(path) = path_to_load {
                            if let Ok(mod_source) = std::fs::read_to_string(&path) {
                                scan_file_for_fns(
                                    &mod_source,
                                    result,
                                    &path.parent().unwrap_or(src_dir).to_path_buf(),
                                    crate_name,
                                );
                            }
                        }
                    }
                }
            }
            syn::Item::Use(use_item) => {
                // pub use re-exports — scan for function re-exports
                // e.g., "pub use regex::Regex;" makes Regex available at crate root
                // We don't need to handle this for function signatures specifically
                let _ = use_item;
            }
            _ => {}
        }
    }
}

/// Scan module content (inline) for public functions.
fn scan_mod_content(
    items: &[syn::Item],
    result: &mut HashMap<String, FunctionShim>,
    src_dir: &PathBuf,
    crate_name: &str,
) {
    for item in items {
        if let syn::Item::Fn(fn_item) = item {
            if is_public(&fn_item.vis) && !has_receiver(&fn_item.sig) {
                extract_fn_signature(&fn_item.sig, fn_item.sig.ident.to_string(), result);
            }
        }
    }
    let _ = (src_dir, crate_name);
}

/// Check if a visibility modifier is public (pub).
fn is_public(vis: &syn::Visibility) -> bool {
    matches!(vis, syn::Visibility::Public(_))
}

/// Check if a function has a self receiver (method, not free function).
fn has_receiver(sig: &syn::Signature) -> bool {
    sig.inputs.iter().any(|arg| {
        matches!(
            arg,
            syn::FnArg::Receiver(_)
        )
    })
}

/// Extract a function signature into a FunctionShim.
fn extract_fn_signature(
    sig: &syn::Signature,
    name: String,
    result: &mut HashMap<String, FunctionShim>,
) {
    let mut param_types = Vec::new();
    for arg in &sig.inputs {
        if let syn::FnArg::Typed(pat_type) = arg {
            let shim_type = syn_type_to_shim(&pat_type.ty);
            param_types.push(shim_type);
        }
    }

    let (return_type, returns_result) = match &sig.output {
        syn::ReturnType::Default => (ShimType::Void, false),
        syn::ReturnType::Type(_, ref ty) => {
            let is_result = is_result_type(ty);
            (syn_type_to_shim(ty), is_result)
        }
    };

    // Only add if not already present (first definition wins)
    result.entry(name.clone()).or_insert(FunctionShim {
        name,
        param_types,
        return_type,
        body_override: None,
        returns_result,
    });
}

/// Check if a syn type is `Result<T, E>`.
fn is_result_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(type_path) => {
            type_path.path.segments.last().map_or(false, |seg| {
                seg.ident == "Result"
            })
        }
        _ => false,
    }
}

/// Map a syn type to ShimType.
///
/// Conservative: unknown types map to CString (string serialization).
fn syn_type_to_shim(ty: &syn::Type) -> ShimType {
    match ty {
        syn::Type::Path(type_path) => {
            let last = type_path.path.segments.last();
            match last {
                Some(seg) => match seg.ident.to_string().as_str() {
                    "i32" | "u32" => ShimType::I32,
                    "i64" | "u64" => ShimType::I64,
                    "f64" => ShimType::F64,
                    "bool" => ShimType::Bool,
                    "String" => ShimType::CString,
                    // Wrapper types — look at generic args
                    "Option" => shim_from_generic(&seg.arguments, true),
                    "Result" => shim_from_generic(&seg.arguments, false),
                    "Vec" => ShimType::CString, // serialize as JSON string
                    "Cow" => shim_from_generic(&seg.arguments, false),
                    // Value, ValueRef, etc. from serde_json — serialize as string
                    _ => ShimType::CString,
                },
                None => ShimType::CString,
            }
        }
        syn::Type::Reference(ref_type) => {
            // &str, &String, &[u8], etc.
            match ref_type.elem.as_ref() {
                syn::Type::Path(type_path) => {
                    let last = type_path.path.segments.last();
                    match last {
                        Some(seg) => match seg.ident.to_string().as_str() {
                            "str" => ShimType::CString,
                            "String" => ShimType::CString,
                            "OsStr" | "Path" => ShimType::CString,
                            _ => ShimType::CString,
                        },
                        None => ShimType::CString,
                    }
                }
                syn::Type::Slice(_) => ShimType::CString, // &[T] → serialize
                _ => ShimType::CString,
            }
        }
        syn::Type::Tuple(tuple) => {
            if tuple.elems.is_empty() {
                ShimType::Void // () = unit
            } else {
                ShimType::CString // multi-value tuple → serialize
            }
        }
        syn::Type::Ptr(_) => ShimType::CString,
        _ => ShimType::CString,
    }
}

/// For generic types like Option<T> and Result<T, E>, extract the T type.
fn shim_from_generic(args: &syn::PathArguments, is_option: bool) -> ShimType {
    match args {
        syn::PathArguments::AngleBracketed(angle) => {
            if let Some(syn::GenericArgument::Type(ty)) = angle.args.first() {
                let inner = syn_type_to_shim(ty);
                if is_option {
                    // Option<T> → use T's type (the None case is handled at runtime)
                    inner
                } else {
                    // Result<T, E> → use T's type
                    inner
                }
            } else {
                ShimType::CString
            }
        }
        _ => ShimType::CString,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syn_type_mapping() {
        let cases: Vec<(&str, ShimType)> = vec![
            ("i32", ShimType::I32),
            ("u32", ShimType::I32),
            ("i64", ShimType::I64),
            ("u64", ShimType::I64),
            ("f64", ShimType::F64),
            ("bool", ShimType::Bool),
            ("String", ShimType::CString),
            ("&str", ShimType::CString),
            ("()", ShimType::Void),
        ];

        for (type_str, expected) in cases {
            let ty: syn::Type = syn::parse_str(type_str).unwrap();
            let result = syn_type_to_shim(&ty);
            assert_eq!(result, expected, "Type mismatch for {}", type_str);
        }
    }
}
