//! PLAN-843 结构拆分（自 codegen.rs 原样搬移；算法/行为零变化）。
use super::*;

impl Codegen {
    /// Track a generic type instantiation during compilation
    /// Returns the monomorphic name for this instantiation
    ///
    /// Example:
    /// ```ignore
    /// let list_type = Type::List(Box::new(Type::Int));
    /// let mono_name = codegen.track_generic(&list_type);
    /// assert_eq!(mono_name, "List_int");
    /// ```
    pub fn track_generic(&mut self, ty: &crate::ast::Type) -> Option<String> {
        if let Some(instance) = extract_generic_instance(ty) {
            let mono_name = self.generics.register(instance);
            Some(mono_name)
        } else {
            None
        }
    }

    /// Check if a type is a generic instantiation and get its monomorphic name
    pub fn get_monomorphic_name(&self, ty: &crate::ast::Type) -> Option<String> {
        if let Some(instance) = extract_generic_instance(ty) {
            Some(instance.monomorphic_name())
        } else {
            None
        }
    }

    /// Get all tracked generic instantiations
    pub fn get_generic_instantiations(&self) -> Vec<crate::vm::generic::GenericInstance> {
        self.generics.all().into_iter().cloned().collect()
    }

    /// Get all List instantiations (e.g., List<int>, List<string>)
    pub fn get_list_instantiations(&self) -> Vec<crate::vm::generic::GenericInstance> {
        self.generics
            .list_instantiations()
            .into_iter()
            .cloned()
            .collect()
    }

    // Plan 076 Phase 2: Monomorphization methods

    /// Perform monomorphization pass on all tracked generics
    /// Generates specialized bytecode for each generic instantiation
    pub fn monomorphize(&mut self) -> Vec<MonomorphizedModule> {
        let mut monomorphizer = Monomorphizer::new();

        // Transfer all tracked generics to the monomorphizer
        for instance in self.generics.all() {
            monomorphizer.register_generic(instance.clone());
        }

        // Generate specialized bytecode
        monomorphizer.monomorphize()
    }

    /// Check if a monomorphic module exists for a given name
    pub fn has_monomorphic_module(&self, name: &str) -> bool {
        self.generics.contains(name)
    }

    /// Get monomorphic name for a type if it's a generic instantiation
    pub fn get_monomorphic_name_checked(&self, ty: &crate::ast::Type) -> Option<String> {
        if let Some(instance) = extract_generic_instance(ty) {
            Some(instance.monomorphic_name())
        } else {
            None
        }
    }

    /// Falls back to a heuristic based on common naming patterns
    ///
    /// For standard library types, we can map variable names to types:
    /// - "list", "arr" -> "List"
    /// - "str", "s" -> "String"
    /// Build fn_return_types from the native registry + intrinsic extras.
    ///
    /// The native registry (`BIGVM_NATIVES`) carries return types for registered
    /// native functions.  We import those here and add a handful of intrinsics
    /// that don't go through the registry (e.g. `int.to_str`, `List.join`).
    pub(super) fn build_fn_return_types() -> HashMap<String, Type> {
        use crate::vm::native_registry::NativeRetType;

        let mut map = HashMap::new();

        // 1. Bulk-import from the native registry
        let registry = crate::vm::native_registry::BIGVM_NATIVES.lock().unwrap();
        for (name, ret) in registry.get_all_return_types() {
            let ty = match ret {
                NativeRetType::Void => Type::Void,
                NativeRetType::Int => Type::Int,
                NativeRetType::Float => Type::Float,
                NativeRetType::Bool => Type::Bool,
                NativeRetType::String => Type::StrOwned,
                NativeRetType::I64 => Type::I64,
                NativeRetType::List => Type::List(Box::new(Type::Unknown)),
                NativeRetType::Map => Type::Map(Box::new(Type::Unknown), Box::new(Type::Unknown)),
            };
            map.insert(name.clone(), ty);
        }
        drop(registry);

        // Plan 378: Import return types for lazily-registered natives. Plan 250's
        // lazy resolve_qualified() registers IDs on first use but NOT return
        // types, so the registry bulk-import above misses natives like
        // `str.to_uint`. NATIVE_RET_ENTRIES is the full catalog's authoritative
        // (name, ret) table; merging it here makes those return types visible to
        // codegen type inference (contains_u64 / is_u64_expr / print routing).
        //
        // Void entries are intentionally SKIPPED. The catalog's Void tag is
        // ambiguous: it is used both for genuinely void natives AND as a stale
        // default for many #[rust_fn] FFI shims that actually return a value
        // through a side channel (e.g. File.exists returns bool via the FFI
        // bridge, not the native stack). Importing such Void tags would make
        // codegen drop real return values (regression: test_18_ffi_001). We
        // therefore only trust non-void tags and leave Void entries unset
        // (unknown) — which exactly preserves the pre-Plan-378 lazy behaviour
        // for those natives. Correcting the stale Void tags in the catalog is
        // tracked as a separate cleanup (see plan 378 §10.4).
        for (name, ret) in crate::vm::native_catalog::NATIVE_RET_ENTRIES {
            let ty = match ret {
                NativeRetType::Void => continue,
                NativeRetType::Int => Type::Int,
                NativeRetType::Float => Type::Float,
                NativeRetType::Bool => Type::Bool,
                NativeRetType::String => Type::StrOwned,
                NativeRetType::I64 => Type::I64,
                NativeRetType::List => Type::List(Box::new(Type::Unknown)),
                NativeRetType::Map => Type::Map(Box::new(Type::Unknown), Box::new(Type::Unknown)),
            };
            // Don't clobber entries already set (e.g. TypeStore-enriched I64 for
            // str.split). Only insert if absent.
            map.entry(name.to_string()).or_insert(ty);
        }

        // 2. Intrinsics that aren't in the registry but codegen needs type info for
        map.insert("int.to_str".to_string(), Type::StrOwned);
        map.insert("int_str".to_string(), Type::StrOwned);
        map.insert("uint.to_hex".to_string(), Type::StrOwned);
        map.insert("List.join".to_string(), Type::StrOwned);
        // str.split returns List<str>
        map.insert(
            "str.split".to_string(),
            Type::List(Box::new(Type::StrOwned)),
        );
        map.insert(
            "auto.str.split".to_string(),
            Type::List(Box::new(Type::StrOwned)),
        );
        // Map.new → auto.hashmap.new (alias for Auto syntax)
        map.insert("Map.new".to_string(), Type::Unknown);
        map.insert("HashMap.new".to_string(), Type::Unknown);

        map
    }

    /// Plan 198 Phase 1: Enrich fn_return_types from TypeStore's #[vm] declarations.
    ///
    /// Reads function declarations from TypeStore (populated by parsing stdlib .at files)
    /// and adds/overrides return types. This is the authoritative source — TypeStore
    /// return types override registry-derived types because they come directly from
    /// the source declarations.
    pub(super) fn enrich_fn_return_types_from_type_store(
        type_store: &types::TypeStore,
        map: &mut HashMap<String, Type>,
    ) {
        for (name, fn_decl) in type_store.all_fn_decls() {
            let ret_type = fn_decl.ret.clone();
            if matches!(ret_type, Type::Void | Type::Unknown) {
                continue;
            }

            let fn_name = name.to_string();

            if let Some(parent) = &fn_decl.parent {
                let parent_str = parent.to_string();

                // lowercase.type: "str.char_at"
                let lower = format!("{}.{}", parent_str.to_lowercase(), fn_name);
                map.insert(lower, ret_type.clone());

                // TitleCase: "Str.char_at"
                let mut chars = parent_str.chars();
                if let Some(first) = chars.next() {
                    let titled: String = first.to_uppercase().collect::<String>() + chars.as_str();
                    let title_key = format!("{}.{}", titled, fn_name);
                    map.insert(title_key, ret_type.clone());
                }
            }

            // Standalone function name
            map.insert(fn_name, ret_type);
        }
    }

    /// Try to resolve a method call on a collection variable to a typed native function.
    ///
    /// When a variable has known generic type parameters (e.g., `Map<str, int>`),
    /// this resolves `m.insert("k", 42)` to `HashMap.insert_int` by picking the
    /// correct suffix based on the value type.
    ///
    /// Returns `None` if:
    /// - The base type is not a supported collection
    /// - The method does not have a typed variant
    /// - The type parameter index is out of bounds
    /// - The resolved native name does not exist in the registry
    pub(super) fn try_mono_dispatch(
        &self,
        base_type: &str,
        method: &str,
        type_args: &[Type],
    ) -> Option<String> {
        // Map Auto base type names to their native registry module path
        let native_module = match base_type {
            "Map" | "HashMap" => "auto.hashmap",
            "String" => "auto.str",
            other => {
                let lower = other.to_lowercase();
                let canonical = format!("auto.{}.{}", lower, method);
                if BIGVM_NATIVES
                    .lock()
                    .unwrap()
                    .peek_qualified(&canonical)
                    .is_some()
                {
                    vm_debug!(
                        "DEBUG: Mono dispatch resolved {}.{} -> {}",
                        base_type,
                        method,
                        canonical
                    );
                    return Some(canonical);
                }
                vm_debug!("DEBUG: Mono dispatch failed for {}.{}", base_type, method);
                return None;
            }
        };

        // HashMap insert/get/set have genuinely different shims for str vs int.
        // Try type-suffixed canonical first (e.g., auto.hashmap.insert_str).
        // Note: "set" maps to "insert" (Auto syntax uses map.set(), natives use insert_str/insert_int)
        let dispatch_method = if method == "set" { "insert" } else { method };
        if matches!(dispatch_method, "insert" | "get") {
            if !type_args.is_empty() {
                let suffix = type_to_native_suffix(&type_args[type_args.len().saturating_sub(1)]);
                if !suffix.is_empty() {
                    let typed = format!("{}.{}{}", native_module, dispatch_method, suffix);
                    if BIGVM_NATIVES
                        .lock()
                        .unwrap()
                        .peek_qualified(&typed)
                        .is_some()
                    {
                        vm_debug!(
                            "DEBUG: Mono dispatch resolved {}.{} -> {}",
                            base_type,
                            method,
                            typed
                        );
                        return Some(typed);
                    }
                }
            }
        }

        // Fallback: generic canonical name
        let canonical = format!("{}.{}", native_module, method);
        if BIGVM_NATIVES
            .lock()
            .unwrap()
            .peek_qualified(&canonical)
            .is_some()
        {
            vm_debug!(
                "DEBUG: Mono dispatch resolved {}.{} -> {}",
                base_type,
                method,
                canonical
            );
            Some(canonical)
        } else {
            vm_debug!("DEBUG: Mono dispatch failed for {}.{}", base_type, method);
            None
        }
    }

    /// Check if a method requires extracting the 'id' field from the instance
    ///
    /// Plan 077 Phase 5: With unified heap registry, heap objects (List, HashMap, etc.)
    /// are now referenced directly by their ID (u64) instead of being wrapped in
    /// Value::Instance with an 'id' field. So we NO LONGER need to extract 'id' for these types.
    ///
    /// This function now only returns true for legacy types that still use Value::Instance.
    ///
    /// Legacy examples (when this returns true):
    /// - `Iterator.next` → extract `id` field (iterators still use old format)
    ///
    /// Examples (now returns false, no extraction needed):
    /// - `List.push` → NO extraction needed, use list_id directly
    /// - `List.len` → NO extraction needed, use list_id directly
    /// - `List.iter` → NO extraction needed, use list_id directly
    pub(super) fn needs_id_extraction(&self, method_name: &str) -> bool {
        // Plan 077 Phase 5: With unified heap registry, List/HashMap/HashSet don't need id extraction
        // They are now stored as heap objects with direct IDs

        // Only iterators still use the old Value::Instance format with id field
        if method_name.starts_with("auto.iterator.") {
            return matches!(
                method_name,
                "auto.iterator.next"
                    | "auto.iterator.map"
                    | "auto.iterator.filter"
                    | "auto.iterator.collect"
                    | "auto.iterator.reduce"
                    | "auto.iterator.find"
            );
        }

        // All other types now use direct heap object IDs - no extraction needed
        false
    }

    // ========== Plan 073: Type Registry Helper Methods ==========

}
