//! PLAN-884 T-06：post-processing fix 函数体 + derive-mask 机制
//!（编排器在 project_fixups.rs:post_process_with，函数体自 rust.rs 原样迁入）。
use super::*;

impl RustTrans {
    /// Apply all post-processing fixes to generated Rust source.
    /// Called after trans() on the final output.
    /// Plan 014 Layer 3: run a post_process fix, tallying whether it rewrote
    /// the output (for A2R_FIX_COUNTS instrumentation).
    pub(super) fn fix_counted(content: &mut String, name: &str, f: fn(&mut String)) {
        let before = content.len();
        f(content);
        if content.len() != before {
            if let Ok(mut m) = FIX_COUNTS.lock() {
                *m.entry(name.to_string()).or_insert(0) += 1;
            }
        }
    }

    /// PLAN-009 T1 (F1): names whose auto-derived comparison set was
    /// restricted; callers thread this into `post_process_with`. (Read-only
    /// accessor; the field itself stays private.)
    pub fn ord_restricted_names(&self) -> &HashSet<AutoStr> {
        &self.ord_restricted_names
    }

    pub fn post_process(output: &mut Vec<u8>) {
        Self::post_process_with(output, &HashSet::new());
    }

    /// Remove `use self::X;` lines that duplicate `pub mod X;` declarations.
    pub(super) fn remove_duplicate_module_uses(content: &mut String) {
        use std::collections::HashSet;
        let pub_mods: HashSet<String> = regex_captures(content, r"pub mod (\w+);");

        if pub_mods.is_empty() {
            return;
        }

        let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        let mut new_lines = Vec::new();
        let mut removed = 0;
        for line in &lines {
            let stripped = line.trim();
            let mut skip = false;
            for mod_name in &pub_mods {
                if stripped == format!("use self::{};", mod_name) {
                    skip = true;
                    removed += 1;
                    break;
                }
            }
            if !skip {
                new_lines.push(line.clone());
            }
        }
        if removed > 0 {
            *content = new_lines.join("\n");
        }
    }

    /// Remove `use` statements that import symbols already defined locally.
    pub(super) fn remove_duplicate_imports(content: &mut String) {
        // Find locally defined symbols (fn, struct, enum, trait, const, static, type names)
        let local_syms: Vec<String> = regex_captures_vec(
            content,
            r"\b(?:pub\s+)?(?:fn|struct|enum|trait|const|static|type)\s+(\w+)",
        );

        if local_syms.is_empty() {
            return;
        }

        let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        let mut new_lines = Vec::new();
        let mut removed = 0;
        for line in &lines {
            let stripped = line.trim();
            if stripped.starts_with("use ") && stripped.ends_with(';') {
                // Extract symbol from use path
                let path = &stripped[4..stripped.len() - 1]; // strip "use " and ";"
                let last_part = path.rsplit("::").next().unwrap_or(path);
                // Handle braced imports: use crate::module::{A, B};
                if last_part.starts_with('{') && last_part.ends_with('}') {
                    let inner = &last_part[1..last_part.len() - 1];
                    let syms: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
                    let any_local = syms.iter().any(|s| local_syms.contains(&s.to_string()));
                    if any_local {
                        removed += 1;
                        continue;
                    }
                } else if local_syms.contains(&last_part.to_string()) && path.contains("::") {
                    removed += 1;
                    continue;
                }
            }
            new_lines.push(line.clone());
        }
        if removed > 0 {
            *content = new_lines.join("\n");
        }
    }

    /// Fix numeric literal .as_str() — numbers should never have .as_str()
    /// E.g., 0.as_str() → 0, 100000.as_str() → 100000
    pub(super) fn fix_numeric_get_as_str(content: &mut String) {
        // Remove .as_str() after any numeric literal (standalone digits)
        // Use \b to avoid matching trailing digits in identifiers like body_str2.as_str()
        if let Some(re) = cached_regex(r"\b(\d+)\.as_str\(\)") {
            let new_content = re.replace_all(content.as_str(), "$1").to_string();
            *content = new_content;
        }
    }

    /// Fix `match expr.get(X) { Some(binding) => { ...` by adding `.cloned()`
    /// to convert `Option<&T>` to `Option<T>` when the binding is used as a value.
    pub(super) fn fix_get_cloned_for_match(content: &mut String) {
        // Pattern: self.field.get(X) { Some(var) => { let ... = var; → add .cloned()
        // Also: self.field.get(X) { Some(var) -> { var.field → add .cloned()
        let fields_needing_cloned = ["sessions", "run", "checkpoint"];
        for field in &fields_needing_cloned {
            let pattern = format!(r"self\.{}\.get\(([^)]+)\) \{{", regex::escape(field));
            if let Some(re) = cached_regex(&pattern) {
                let new_content = re
                    .replace_all(content.as_str(), |caps: &regex::Captures| {
                        format!(
                            "self.{}.get({}).cloned() {{",
                            field,
                            caps.get(1).unwrap().as_str()
                        )
                    })
                    .to_string();
                if new_content != *content {
                    *content = new_content;
                }
            }
        }
        // Also fix: return self.field.get(X); → return self.field.get(X).cloned();
        // HashMap::get returns Option<&T>, but Auto expects Option<T> for return types
        if let Some(re) = cached_regex(r"return self\.(\w+)\.get\(([^)]+)\);") {
            let map_fields = [
                "sessions",
                "run",
                "checkpoint",
                "pages",
                "wiki_dirs",
                "project_locks",
                "professions",
                "souls",
                "agents",
            ];
            let new_content = re
                .replace_all(content, |caps: &regex::Captures| {
                    let field = caps.get(1).unwrap().as_str();
                    let key = caps.get(2).unwrap().as_str();
                    if map_fields.contains(&field) {
                        format!("return self.{}.get({}).cloned();", field, key)
                    } else {
                        caps.get(0).unwrap().as_str().to_string()
                    }
                })
                .to_string();
            if new_content != *content {
                *content = new_content;
            }
        }
    }

    /// Fix Vec/HashMap .insert() where first argument needs to be usize.
    /// Only handles variables with known integer type annotations (u32, i32).
    pub(super) fn fix_insert_usize(content: &mut String) {
        use std::collections::HashSet;
        let mut int_names: HashSet<String> = HashSet::new();
        for ty in &["u32", "i32"] {
            let pat = format!(r"let\s+(?:mut\s+)?(\w+)\s*:\s*{}\s*=", ty);
            if let Some(re) = cached_regex(&pat) {
                for caps in re.captures_iter(content) {
                    int_names.insert(caps.get(1).unwrap().as_str().to_string());
                }
            }
            let pat = format!(r"pub\s+(\w+)\s*:\s*{}\s*,", ty);
            if let Some(re) = cached_regex(&pat) {
                for caps in re.captures_iter(content) {
                    int_names.insert(caps.get(1).unwrap().as_str().to_string());
                }
            }
        }
        // Also collect vars assigned from known u32-returning functions
        if let Some(re) = cached_regex(r"let\s+(?:mut\s+)?(\w+)\s*=\s*self\.ensure_tool_call\(") {
            for caps in re.captures_iter(content) {
                int_names.insert(caps.get(1).unwrap().as_str().to_string());
            }
        }
        if int_names.is_empty() {
            return;
        }

        for name in &int_names {
            let pattern = format!(r"\.insert\(\s*{}\s*(,)", regex::escape(name));
            if let Some(re) = cached_regex(&pattern) {
                let n = name.clone();
                let new = re
                    .replace_all(content.as_str(), move |caps: &regex::Captures| {
                        let comma = caps.get(1).unwrap().as_str();
                        format!(".insert({} as usize{}", n, comma)
                    })
                    .to_string();
                *content = new;
            }
        }
    }

    /// Fix derive macros on structs containing `dyn Trait` fields — `Box<dyn X>`
    /// (spec params) or `Arc<dyn X>` (from `Arc<Spec>` type annotations, Plan
    /// 390 §15.11 L2 转正). `dyn Trait` doesn't implement Clone/PartialEq/Eq/
    /// PartialOrd/Ord (unless the wrapper provides them):
    ///   - `Box<dyn T>`: Clone requires T: Clone → unsafe; Debug requires T: Debug → unsafe.
    ///   - `Arc<dyn T>`: `Arc<T>` is unconditionally Clone for T: ?Sized → Clone SAFE;
    ///                    Debug/PartialEq/Eq/Ord still require T: those → unsafe.
    /// So Box<dyn> fields → allow(dead_code) (all derives unsafe); Arc<dyn>
    /// fields keep Clone but strip the rest. If the user explicitly supplied a
    /// derive omitting the unsafe traits, leave it untouched (Plan 376 override).
    pub(super) fn fix_dyn_trait_derives(content: &mut String) {
        if let Some(re) = cached_regex(
            r"(?s)(#\[derive\(([^)]*)\)\]\npub struct (\w+) \{[^}]*?(?:Box<dyn|Arc<dyn))",
        ) {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    let full = caps.get(0).unwrap().as_str();
                    let derives = caps.get(2).unwrap().as_str();
                    let derive_list: Vec<&str> = derives.split(',').map(|s| s.trim()).collect();
                    // Plan 376: respect a user-supplied derive that already omits the
                    // unsafe traits (e.g. just "Debug", or "Clone, Debug" when the
                    // dyn-Trait is wrapped in an Arc — which IS Clone). Only strip
                    // PartialEq/Eq/PartialOrd/Ord from the AUTO-generated derive.
                    // Plan 376V: Clone AND Debug are unsafe for Box<dyn Trait>:
                    //   - Box<T>: Clone requires T: Clone (dyn Trait never is)
                    //   - Box<dyn Trait>: Debug requires Trait: Debug (specs don't bound it)
                    // So when a struct has a bare Box<dyn> field, replace the whole
                    // derive with #[allow(dead_code)] (matches the hand-written version).
                    // Plan 390 §15.11 (L2): `Arc<dyn T>` keeps Clone — Arc<T> is
                    // unconditionally Clone for T: ?Sized; only Debug/PartialEq/Eq/
                    // Ord remain unsafe (they require T: those).
                    let has_arc_dyn = full.contains("Arc<dyn");
                    let unsafe_traits: &[&str] = if has_arc_dyn {
                        &["PartialEq", "Eq", "PartialOrd", "Ord", "Debug"]
                    } else {
                        &["PartialEq", "Eq", "PartialOrd", "Ord", "Clone", "Debug"]
                    };
                    let needs_fix = derive_list.iter().any(|d| unsafe_traits.contains(d));
                    if !needs_fix {
                        return full.to_string();
                    }
                    // If ALL derives are unsafe (typical for Box<dyn>), use allow(dead_code).
                    let any_safe = derive_list.iter().any(|d| !unsafe_traits.contains(d));
                    if !any_safe {
                        return full
                            .replace(&format!("#[derive({})]", derives), "#[allow(dead_code)]");
                    }
                    // Keep only the safe traits (Clone, Debug, Copy, Default, ...).
                    let kept: Vec<&&str> = derive_list
                        .iter()
                        .filter(|d| !unsafe_traits.contains(d))
                        .collect();
                    if kept.is_empty() {
                        full.replace(&format!("#[derive({})]", derives), "#[derive(Debug)]")
                    } else {
                        let new_derives: Vec<&str> = kept.into_iter().copied().collect();
                        full.replace(
                            &format!("#[derive({})]", derives),
                            &format!("#[derive({})]", new_derives.join(", ")),
                        )
                    }
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }
    }

    /// Fix `return None;` in void (unit-return) functions → `return;`.
    /// Auto's `return` in void functions is parsed as `Return(Nil)` → transpiled as `return None;`
    /// but Rust void functions need plain `return;`.
    pub(super) fn fix_void_return_none(content: &mut String) {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::with_capacity(lines.len());
        let mut in_void_fn = false;
        let mut brace_depth: i32 = 0;
        let mut fn_brace_depth: i32 = 0;

        for line in &lines {
            let trimmed = line.trim();

            // Track function declarations without return type (void).
            // Plan 384: `-> ()` (explicit unit) is also void.
            let is_void_ret =
                !trimmed.contains("->") || trimmed.contains("-> ()") || trimmed.contains("->()");
            if (trimmed.starts_with("pub fn ")
                || trimmed.starts_with("fn ")
                || trimmed.starts_with("pub async fn ")
                || trimmed.starts_with("async fn "))
                && is_void_ret
            {
                in_void_fn = true;
                fn_brace_depth = brace_depth;
            }

            // Track braces
            for ch in trimmed.chars() {
                match ch {
                    '{' => brace_depth += 1,
                    '}' => brace_depth -= 1,
                    _ => {}
                }
            }

            // If we've exited the void function's scope, reset
            if in_void_fn && brace_depth <= fn_brace_depth && trimmed.contains('}') {
                in_void_fn = false;
            }

            // Replace return None; with return; in void functions
            if in_void_fn && trimmed == "return None;" {
                result.push(line.replacen("return None;", "return;", 1));
            } else {
                result.push(line.to_string());
            }
        }

        let new_content = result.join("\n");
        if new_content != *content {
            *content = new_content;
        }
    }

    /// Fix enum == "str" comparisons.
    pub(super) fn fix_enum_str_comparisons(content: &mut String) {
        let enum_fields = [
            "section_type",
            "status",
            "phase",
            "kind",
            "role",
            "stop_reason",
            "source_type",
            "provider",
            "decision",
        ];
        for field in &enum_fields {
            let eq_pat = format!(".{}\\s*==\\s*\"", field);
            if let Some(re) = cached_regex(&eq_pat) {
                let old_eq = format!(".{field} ==");
                let new_eq = format!(".{field}.to_string() ==");
                let new = re
                    .replace_all(content.as_str(), |caps: &regex::Captures| {
                        caps.get(0).unwrap().as_str().replace(&old_eq, &new_eq)
                    })
                    .to_string();
                if new != *content {
                    *content = new;
                }
            }
            let ne_pat = format!(".{}\\s*!=\\s*\"", field);
            if let Some(re) = cached_regex(&ne_pat) {
                let old_ne = format!(".{field} !=");
                let new_ne = format!(".{field}.to_string() !=");
                let new = re
                    .replace_all(content.as_str(), |caps: &regex::Captures| {
                        caps.get(0).unwrap().as_str().replace(&old_ne, &new_ne)
                    })
                    .to_string();
                if new != *content {
                    *content = new;
                }
            }
        }
    }

    /// Fix tuple.get_N() -> tuple.N (Rust tuple indexing)
    pub(super) fn fix_tuple_get_n(content: &mut String) {
        let mut count = 0;
        for n in 0..=9 {
            let pattern = format!(".get_{}()", n);
            let replacement = format!(".{}", n);
            let reduced = content.replace(&pattern, &replacement);
            if reduced != *content {
                count += 1;
                *content = reduced;
            }
        }
        let _ = count;
    }

    /// Fix map.get(key).as_str() → map.get(key).map(|s| s.as_str()).unwrap_or("")
    /// HashMap::get returns Option<&String>, but Auto treats get() as returning the value directly.
    pub(super) fn fix_map_get_as_str(content: &mut String) {
        // Step 1: Replace `let VAR = EXPR.get(KEY);` with
        //         `let VAR = EXPR.get(KEY).cloned().unwrap_or_default();`
        //         ONLY for bootstrap compiler env/state variables (env.*, params.*, state.*)
        //         NOT for use.rust HashMap (those should keep native Option<&V> semantics)
        if let Some(re) = cached_regex(r"let\s+(\w+)\s*=\s*(\w+\.get\([^)]+\));") {
            let mut replacements = Vec::new();
            for caps in re.captures_iter(content) {
                let var = caps.get(1).unwrap().as_str();
                let get_expr = caps.get(2).unwrap().as_str();
                // Only apply to known bootstrap env variables: env.*, params.*, headers.*, state.*
                let is_bootstrap_env = get_expr.starts_with("env.")
                    || get_expr.starts_with("params.")
                    || get_expr.starts_with("headers.")
                    || get_expr.starts_with("state.");
                if is_bootstrap_env
                    && (get_expr.contains(".get(\"") || get_expr.contains(".get(\""))
                {
                    replacements.push((var.to_string(), get_expr.to_string()));
                }
            }
            for (var, get_expr) in &replacements {
                let old = format!("let {} = {};", var, get_expr);
                let new = format!("let {} = {}.cloned().unwrap_or_default();", var, get_expr);
                let replaced = content.replace(&old, &new);
                if replaced != *content {
                    *content = replaced;
                }
            }
        }

        // Step 2: Replace EXPR.get(KEY).as_str() inline patterns
        // Pattern: var.get("key").as_str() → var.get("key").map(|s| s.as_str()).unwrap_or("")
        if let Some(re) = cached_regex(r#"(\w+\.get\("[^"]+"\))\.as_str\(\)"#) {
            let new = re
                .replace_all(content, |caps: &regex::Captures| {
                    let get_expr = caps.get(1).unwrap().as_str();
                    format!("{}.map(|s| s.as_str()).unwrap_or(\"\")", get_expr)
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }
    }

    /// Fix integer.as_str() → integer.to_string().as_str()
    /// i32/u32 don't have .as_str(), but Auto's str() conversion maps to .as_str().
    pub(super) fn fix_int_as_str(content: &mut String) {
        // Track which variables are assigned from integer-returning expressions
        // Pattern: let VAR = ... as i32; or let VAR: u32 = ...;
        let mut int_vars = std::collections::HashSet::new();
        if let Some(re) = cached_regex(r"let\s+(\w+)\s*:\s*(u32|i32|usize)\s*=") {
            for caps in re.captures_iter(content) {
                int_vars.insert(caps.get(1).unwrap().as_str().to_string());
            }
        }
        // Also track: let VAR = expr as i32/u32/usize;
        if let Some(re) = cached_regex(r"let\s+(\w+)\s*=\s*[^;]+\s+as\s+(u32|i32|usize)\s*;") {
            for caps in re.captures_iter(content) {
                int_vars.insert(caps.get(1).unwrap().as_str().to_string());
            }
        }
        // Also track: let VAR: u32/i32;
        if let Some(re) = cached_regex(r"let\s+mut\s+(\w+)\s*:\s*(u32|i32|usize)\s*;") {
            for caps in re.captures_iter(content) {
                int_vars.insert(caps.get(1).unwrap().as_str().to_string());
            }
        }

        if int_vars.is_empty() {
            return;
        }

        // Replace VAR.as_str() with format!("{}", VAR).as_str() for integer vars
        for var in &int_vars {
            let pattern = format!("{}.as_str()", var);
            let replacement = format!("format!(\"{{}}\", {}).as_str()", var);
            let new = content.replace(&pattern, &replacement);
            if new != *content {
                *content = new;
            }
        }
    }

    /// Fix str.split(X).len() → str.split(X).count()
    /// and str.split(X).get(i) → str.split(X).nth(i)
    /// Rust's Split is an iterator, not a Vec.
    pub(super) fn fix_split_methods(content: &mut String) {
        // Pattern: VAR.split(X).len() → VAR.split(X).count()
        if let Some(re) = cached_regex(r"\.split\(([^)]+)\)\.len\(\)") {
            let new = re
                .replace_all(content, |caps: &regex::Captures| {
                    let arg = caps.get(1).unwrap().as_str();
                    format!(".split({}).count()", arg)
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }

        // Pattern: VAR.split(X).get(N) → VAR.split(X).nth(N)
        if let Some(re) = cached_regex(r"\.split\(([^)]+)\)\.get\(([^)]+)\)") {
            let new = re
                .replace_all(content, |caps: &regex::Captures| {
                    let split_arg = caps.get(1).unwrap().as_str();
                    let get_arg = caps.get(2).unwrap().as_str();
                    format!(".split({}).nth({})", split_arg, get_arg)
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }

        // Pattern: VAR.split(X)[N] → VAR.split(X).nth(N).unwrap()
        if let Some(re) = cached_regex(r"\.split\(([^)]+)\)\[(\d+)\]") {
            let new = re
                .replace_all(content, |caps: &regex::Captures| {
                    let split_arg = caps.get(1).unwrap().as_str();
                    let idx = caps.get(2).unwrap().as_str();
                    format!(".split({}).nth({}).unwrap()", split_arg, idx)
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }

        // Pattern: VAR.split(X)[VAR2 as usize] → VAR.split(X).nth(VAR2 as usize).unwrap()
        if let Some(re) = cached_regex(r"\.split\(([^)]+)\)\[(\w+ as usize)\]") {
            let new = re
                .replace_all(content, |caps: &regex::Captures| {
                    let split_arg = caps.get(1).unwrap().as_str();
                    let idx = caps.get(2).unwrap().as_str();
                    format!(".split({}).nth({}).unwrap()", split_arg, idx)
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }
    }

    /// Fix common String/&str mismatch patterns.
    pub(super) fn fix_string_str_mismatches(content: &mut String) {
        // 1. Remove .to_string().as_str() → .as_str()
        let reduced = content.replace(".to_string().as_str()", ".as_str()");
        if reduced != *content {
            *content = reduced;
        }
        // 2. Remove .clone().as_str() → .as_str()
        let reduced = content.replace(".clone().as_str()", ".as_str()");
        if reduced != *content {
            *content = reduced;
        }
        // 3. Remove duplicate .to_string().to_string() → .to_string()
        let reduced = content.replace(".to_string().to_string()", ".to_string()");
        if reduced != *content {
            *content = reduced;
        }
    }

    /// Fix HashMap.keys() used as indexable collection.
    /// Auto: `var keys = map.keys()` returns List<str>, supports keys[i] and keys.len()
    /// Rust: keys() returns an iterator — need to collect into Vec first.
    /// Pattern: `let mut? var = expr.keys()` → `let mut? var: Vec<_> = expr.keys().cloned().collect()`
    pub(super) fn fix_map_keys_indexing(content: &mut String) {
        // Find all .keys() assignments and check if they're used with indexing or .len()
        if let Some(re) = cached_regex(r"(?m)^(\s+let (?:mut )?)(\w+) = (.+?)\.keys\(\)") {
            let captures: Vec<(usize, String, String, String)> = re
                .captures_iter(content.as_str())
                .filter_map(|caps| {
                    let full = caps.get(0)?;
                    let indent = caps.get(1)?.as_str().to_string();
                    let var = caps.get(2)?.as_str().to_string();
                    let expr = caps.get(3)?.as_str().to_string();
                    Some((full.start(), indent, var, expr))
                })
                .collect();

            // Check which vars are used with indexing [i] or .len()
            for (_pos, indent, var, expr) in captures.iter().rev() {
                // Check if var is used with indexing or .len()
                let idx_pat = format!("{}[", var);
                let len_pat = format!("{}.len()", var);
                let needs_fix = content.contains(&idx_pat) || content.contains(&len_pat);
                if !needs_fix {
                    continue;
                }

                let old_line = format!("{}{} = {}.keys();", indent, var, expr);
                let new_line = format!(
                    "{}{}: Vec<_> = {}.keys().cloned().collect();",
                    indent, var, expr
                );
                *content = content.replace(&old_line, &new_line);

                // After converting to Vec, fix map.get(var[i].clone()) → map.get(&var[i])
                // and map.insert(var[i].clone(), ...) → map.insert(var[i].clone(), ...)
                let get_clone_pat =
                    format!(r"\.get\({}\[([^\]]+)\]\s*\.clone\(\)\)", regex::escape(var));
                if let Some(get_re) = cached_regex(&get_clone_pat) {
                    let new = get_re
                        .replace_all(content.as_str(), |caps: &regex::Captures| {
                            format!(".get(&{}[{}])", var, caps.get(1).unwrap().as_str())
                        })
                        .to_string();
                    if new != *content {
                        *content = new;
                    }
                }
            }
        }
    }

    /// Fix Option<String>.unwrap_or("") → Option<String>.unwrap_or_default()
    /// Auto: Option<str>.unwrap_or("") works because "" is str
    /// Rust: Option<String>.unwrap_or("") fails because "" is &str not String
    pub(super) fn fix_option_unwrap_or_empty(content: &mut String) {
        // Pattern: .unwrap_or("") → .unwrap_or_default()
        // This handles Option<String>.unwrap_or("") → unwrap_or_default()
        if let Some(re) = cached_regex(r#"\.unwrap_or\(""\)"#) {
            let new = re
                .replace_all(content.as_str(), ".unwrap_or_default()")
                .to_string();
            if new != *content {
                *content = new;
            }
        }
        // Pattern: .unwrap_or(vec![]) → .unwrap_or_default()
        if let Some(re) = cached_regex(r"\.unwrap_or\(vec!\[\]\)") {
            let new = re
                .replace_all(content.as_str(), ".unwrap_or_default()")
                .to_string();
            if new != *content {
                *content = new;
            }
        }
    }

    /// Fix String passed where &_ is expected.
    /// Uses pattern-based matching instead of variable name tracking.
    pub(super) fn fix_string_to_ref(_content: &mut String) {
        // DON'T blindly add & to all .get(var) — this causes E0277 when var is &str
        // Instead, only fix specific known patterns
        // For now, this is a no-op to avoid regressions
    }

    /// Fix &str assigned to String fields and pushed to Vec<String>.
    /// Pattern 1: `self.field = str_param` where field is String → add .to_string()
    /// Pattern 2: `vec.push(str_param)` where vec is Vec<String> → add .to_string()
    /// Pattern 3: `map.insert(&str_key, ...)` → `map.insert(key.to_string(), ...)`
    pub(super) fn fix_str_to_string_assignments(content: &mut String) {
        // Line-by-line approach: scan for patterns where &str is used where String is needed.
        // This avoids OOM from repeated regex replacements on the entire file.

        let lines: Vec<&str> = content.lines().collect();
        let mut result = String::with_capacity(content.len());

        // Plan 514 W3-15: &str 参数按**当前函数**作用域收集——原全文件
        // 名字池让跨函数同名局部误伤(短名单 d/e/g/... 撞 loop_exit 的
        // i64 局部 d,`self.loop_depth = d;` 被追加 .to_string() E0308)。
        // 发射产物签名单行,行进时遇 fn 头行重置即可。
        let sig_re = cached_regex(r#"fn \w+\([^)]*\)"#).unwrap();
        let param_re = cached_regex(r#"fn \w+\([^)]*(\w+):\s*&str"#).unwrap();
        let mut str_params = std::collections::HashSet::new();

        for line in &lines {
            let mut new_line = line.to_string();
            if sig_re.is_match(line) {
                str_params.clear();
                for caps in param_re.captures_iter(line) {
                    if let Some(m) = caps.get(1) {
                        str_params.insert(m.as_str().to_string());
                    }
                }
            }

            // Pattern: .push(param) where param is &str → .push(param.to_string())
            for param in &str_params {
                let push_target = format!(".push({})", param);
                let push_replacement = format!(".push({}.to_string())", param);
                if new_line.contains(&push_target) && !new_line.contains(&push_replacement) {
                    new_line = new_line.replace(&push_target, &push_replacement);
                }

                // Pattern: self.field = param; → self.field = param.to_string();
                let assign_target = format!("= {};", param);
                let assign_replacement = format!("= {}.to_string();", param);
                if new_line.contains(&assign_target) && !new_line.contains(&assign_replacement) {
                    // Only apply for self.field or var.field assignments
                    if new_line.contains("self.")
                        || new_line.contains("page.")
                        || new_line.contains("s.")
                    {
                        new_line = new_line.replace(&assign_target, &assign_replacement);
                    }
                }
            }

            result.push_str(&new_line);
            result.push('\n');
        }

        // Remove trailing newline if original didn't have one
        if !content.ends_with('\n') && result.ends_with('\n') {
            result.pop();
        }
        if result != *content {
            *content = result;
        }
    }

    /// Plan 373: lower `s.substring(lo, hi)` to Rust slicing.
    /// Auto str has `.substring(lo, hi)`; Rust str has no such method.
    /// `expr.substring(0, cap)` → `&expr[0..cap as usize]`
    /// `expr.substring(lo, end)` → `&expr[lo as usize..end as usize]`
    /// Both args are integer-typed (i32/u32) in the transpiled output, so cast
    /// to usize for byte slicing. (Byte slicing is adequate — Auto sources use
    /// substring only for truncation/summaries, never on multi-byte boundaries
    /// that would split a char.)
    pub(super) fn fix_substring_method(content: &mut String) {
        // `IDENT.substring(a, b)` — IDENT is a simple receiver (str value).
        // Plan 380: add a leading `&` ONLY when the slice isn't chained —
        // `let x = s[lo..hi]` would bind an unsized `str` (E0277), while
        // `s[lo..hi].to_string()` works without `&` (and `&s[..].to_string()`
        // would become `&String`, E0308).
        if let Some(re) = cached_regex(r"(\w+)\.substring\(([^,)]+),\s*([^)]+)\)") {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    let m0 = caps.get(0).unwrap();
                    let after_is_dot = content.as_bytes().get(m0.end()) == Some(&b'.');
                    let recv = caps.get(1).unwrap().as_str();
                    let lo = caps.get(2).unwrap().as_str().trim();
                    let hi = caps.get(3).unwrap().as_str().trim();
                    let inner = format!("{}[{} as usize..{} as usize]", recv, lo, hi);
                    if after_is_dot {
                        inner
                    } else {
                        format!("&{}", inner)
                    }
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }
    }

    /// Plan 373: lower Auto-VM numeric conversion methods to Rust casts.
    /// `expr.to_float()` → `(expr as f64)`
    /// `expr.to_uint()`  → `(expr as u32)`
    /// `expr.to_int()`   → `(expr as i64)`
    /// (Rust ints/floats have no `.to_float()`/`.to_uint()` methods.)
    pub(crate) fn fix_numeric_conversion_methods(content: &mut String) {
        // Match a receiver that is either an identifier or a method chain we can
        // wrap in parens. Keep it conservative: `IDENT.method_chain().to_float()`.
        for (method, cast) in [("to_float", "f64"), ("to_uint", "u32"), ("to_int", "i32")] {
            let pat = format!(r"([\w.()]+)\.{}\(\)", method);
            if let Some(re) = cached_regex(&pat) {
                let mthd = method;
                let new = re
                    .replace_all(content.as_str(), |caps: &regex::Captures| {
                        let recv = caps.get(1).unwrap().as_str();
                        // Avoid double-wrapping if already parenthesized.
                        if recv.starts_with('(') && recv.ends_with(')') {
                            format!("{} as {})", &recv[..recv.len() - 1], cast)
                        } else {
                            format!("({} as {})", recv, cast)
                        }
                    })
                    .to_string();
                if new != *content {
                    *content = new;
                }
                let _ = mthd;
            }
        }
    }

    /// Plan 373: drop residual `Err(Box::new(...))` wrapping.
    /// a2r sometimes wraps error payloads in `Box::new(...)` even when the
    /// enclosing `Result`'s error type is a plain enum/String (not Box<...>).
    /// `Err(Box::new(X))` → `Err(X)`
    pub(super) fn fix_residual_error_box(content: &mut String) {
        let needle = "Err(Box::new(";
        let mut out = String::with_capacity(content.len());
        let mut rest: &str = content;
        loop {
            match rest.find(needle) {
                None => {
                    out.push_str(rest);
                    break;
                }
                Some(pos) => {
                    out.push_str(&rest[..pos]);
                    let after = &rest[pos + needle.len()..];
                    // Find the close paren that balances the Box::new( open,
                    // counting nesting (payload may itself contain parens, e.g.
                    // nested format!(...) from `+` concat). The old regex only
                    // handled ONE paren level — nested format! stayed Box::new
                    // and broke `Result<_, String>` fns (E0308 Box<String>).
                    let mut depth = 1i32;
                    let mut close: Option<usize> = None;
                    for (i, ch) in after.char_indices() {
                        match ch {
                            '(' => depth += 1,
                            ')' => {
                                depth -= 1;
                                if depth == 0 {
                                    close = Some(i);
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                    // `Err(Box::new(` opens BOTH Err( and Box::new( — the Err's
                    // own close paren immediately follows the Box::new close.
                    // Consume it too: `Err(Box::new(X))` → `Err(X)`.
                    match close {
                        Some(i) => {
                            out.push_str("Err(");
                            out.push_str(&after[..i]);
                            out.push_str(")");
                            let skip = if after[i + 1..].starts_with(')') {
                                i + 2
                            } else {
                                i + 1
                            };
                            rest = &after[skip..];
                        }
                        None => {
                            // Unbalanced — leave the text untouched.
                            out.push_str(needle);
                            rest = after;
                        }
                    }
                }
            }
        }
        if out != *content {
            *content = out;
        }
    }

    pub(super) const D_PE: u8 = 1; // PartialEq
    pub(super) const D_EQ: u8 = 2; // Eq
    pub(super) const D_PO: u8 = 4; // PartialOrd
    pub(super) const D_OR: u8 = 8; // Ord
    pub(super) const D_FULL: u8 = Self::D_PE | Self::D_EQ | Self::D_PO | Self::D_OR;

    /// Trait names listed in a type's explicit `#[derive(...)]` attrs.
    /// Attrs without a derive(...) contribute nothing (empty Vec).
    pub(super) fn derive_traits_from_attrs(attrs: &[AutoStr]) -> Vec<AutoStr> {
        let mut out: Vec<AutoStr> = Vec::new();
        for attr in attrs {
            let a = attr.as_str().trim();
            let Some(inner) = a.strip_prefix("derive(") else {
                continue;
            };
            let Some(inner) = inner.strip_suffix(')') else {
                continue;
            };
            for t in inner.split(',') {
                let t = t.trim();
                if t.is_empty() {
                    continue;
                }
                // Path-qualified traits (std::fmt::Debug) count by last segment.
                let name = t.rsplit("::").next().unwrap_or(t);
                if !out.iter().any(|e| e.as_str() == name) {
                    out.push(name.into());
                }
            }
        }
        out
    }

    fn mask_from_traits(traits: &[AutoStr]) -> u8 {
        let mut mask = 0;
        for t in traits {
            match t.as_str() {
                "PartialEq" => mask |= Self::D_PE,
                "Eq" => mask |= Self::D_EQ,
                "PartialOrd" => mask |= Self::D_PO,
                "Ord" => mask |= Self::D_OR,
                _ => {}
            }
        }
        mask
    }

    /// Comparison-trait mask the shape heuristics (TaskRef/dyn/float/map/
    /// enum-field downgrades — same branches as the TypeDecl handler) would
    /// grant a type with these field/payload types, before explicit-derive
    /// intersection.
    pub(super) fn shape_base_cmp_mask(field_types: &[&Type]) -> u8 {
        let has_taskref = field_types.iter().any(|ty| Self::type_is_taskref(ty));
        let has_dyn = field_types.iter().any(|ty| Self::type_contains_dyn(ty));
        let has_eq_unsafe = field_types.iter().any(|ty| {
            Self::type_has_float(ty) || Self::type_has_map(ty) || Self::type_contains_enum(ty)
        });
        if has_taskref || has_dyn {
            0
        } else if has_eq_unsafe {
            Self::D_PE
        } else {
            Self::D_FULL
        }
    }

    /// Names of locally declared types reachable inside `ty` (through
    /// List/Option/Result/Reference/Map/GenericInstance wrappers).
    pub(super) fn collect_local_type_names(
        ty: &Type,
        locals: &HashSet<AutoStr>,
        out: &mut Vec<AutoStr>,
    ) {
        match ty {
            Type::User(td) => {
                let n = td.name.clone();
                if locals.contains(&n) && !out.contains(&n) {
                    out.push(n);
                }
            }
            Type::GenericInstance(inst) => {
                for arg in &inst.args {
                    Self::collect_local_type_names(arg, locals, out);
                }
            }
            Type::List(inner)
            | Type::Option(inner)
            | Type::Result(inner)
            | Type::Reference(inner) => {
                Self::collect_local_type_names(inner, locals, out);
            }
            Type::Map(k, v) => {
                Self::collect_local_type_names(k, locals, out);
                Self::collect_local_type_names(v, locals, out);
            }
            _ => {}
        }
    }

    /// Constraint mask imposed by a referenced type name: its explicit
    /// derive surface if declared with attrs, otherwise its (recursively
    /// computed) auto-derived set; unknown/external types impose none.
    fn type_cmp_mask(&mut self, name: &str) -> u8 {
        if let Some(traits) = self.explicit_derive_traits.get(name) {
            return Self::mask_from_traits(traits);
        }
        if self.local_auto_shapes.contains_key(name) {
            return self.auto_cmp_mask(name);
        }
        Self::D_FULL
    }

    /// Memoized, cycle-guarded recursion: the comparison mask an auto-derived
    /// local type ends up with (shape base ∩ referenced types' masks).
    fn auto_cmp_mask(&mut self, name: &str) -> u8 {
        if let Some(m) = self.auto_cmp_mask_memo.get(name) {
            return *m;
        }
        // Cycle guard: assume unconstrained until proven otherwise.
        self.auto_cmp_mask_memo.insert(name.into(), Self::D_FULL);
        let mut mask = Self::D_FULL;
        if let Some((base, refs)) = self
            .local_auto_shapes
            .get(name)
            .map(|(b, r)| (*b, r.clone()))
        {
            mask = base;
            for r in refs {
                mask &= self.type_cmp_mask(&r);
            }
        }
        self.auto_cmp_mask_memo.insert(name.into(), mask);
        mask
    }

    /// Intersect the candidate mask with the constraints of every local type
    /// referenced inside `field_types` (struct members / enum payloads).
    pub(super) fn field_cmp_mask(&mut self, field_types: &[&Type]) -> u8 {
        let mut mask = Self::D_FULL;
        let mut names: Vec<AutoStr> = Vec::new();
        for ty in field_types {
            Self::collect_local_type_names(ty, &self.local_struct_types, &mut names);
        }
        for n in names {
            mask &= self.type_cmp_mask(&n);
        }
        mask
    }

    /// Rebuild a `#[derive(...)]` attribute keeping only the comparison
    /// traits `mask` allows (other traits and canonical order preserved).
    pub(super) fn filter_cmp_traits(derive_attr: &str, mask: u8) -> String {
        let inner = derive_attr
            .trim_start_matches("#[derive(")
            .trim_end_matches(")]");
        let mut out = String::from("#[derive(");
        let mut first = true;
        for t in inner.split(',') {
            let t = t.trim();
            let keep = match t {
                "PartialEq" => mask & Self::D_PE != 0,
                "Eq" => mask & Self::D_EQ != 0,
                "PartialOrd" => mask & Self::D_PO != 0,
                "Ord" => mask & Self::D_OR != 0,
                _ => true,
            };
            if keep {
                if !first {
                    out.push_str(", ");
                }
                out.push_str(t);
                first = false;
            }
        }
        out.push_str(")]");
        out
    }

    pub(super) fn type_has_float(ty: &Type) -> bool {
        match ty {
            Type::Float | Type::Double => true,
            Type::List(inner) | Type::Result(inner) | Type::Option(inner) => {
                Self::type_has_float(inner)
            }
            _ => false,
        }
    }

    pub(super) fn type_has_map(ty: &Type) -> bool {
        matches!(ty, Type::Map(_, _))
            || matches!(ty, Type::Rust(source) if {
                let name = source.short_name();
                name.starts_with("HashMap") || name.starts_with("BTreeMap")
            })
    }

    pub(super) fn type_contains_enum(ty: &Type) -> bool {
        match ty {
            Type::Tag(_) | Type::Enum(_) => true,
            // Type::User with empty members is a generic type param (T), not a concrete type
            Type::User(td) if !td.members.is_empty() || !td.generic_params.is_empty() => true,
            Type::GenericInstance(inst) => {
                inst.args.iter().any(|arg| Self::type_contains_enum(arg))
            }
            Type::List(inner) | Type::Result(inner) | Type::Option(inner) => {
                Self::type_contains_enum(inner)
            }
            _ => false,
        }
    }

    pub(super) fn type_contains_dyn(ty: &Type) -> bool {
        match ty {
            Type::User(td) => td.name.starts_with("dyn "),
            Type::GenericInstance(inst) => inst.args.iter().any(|arg| Self::type_contains_dyn(arg)),
            Type::List(inner)
            | Type::Result(inner)
            | Type::Option(inner)
            | Type::Reference(inner) => Self::type_contains_dyn(inner),
            _ => false,
        }
    }

    pub(super) fn type_is_taskref(ty: &Type) -> bool {
        matches!(ty, Type::GenericInstance(inst) if inst.base_name == "TaskRef")
    }

    /// Plan 373 G2: fix string-literal args to enum variants that expect String.
    /// e.g. `ToolError::Args("msg")` → `ToolError::Args("msg".to_string())`
    /// Pattern: `Variant("...")` where Variant is Args, Exec, LoopDetected,
    /// Config, Failed, or other common error variant names.
    pub(super) fn fix_string_literal_enum_args(content: &mut String) {
        for variant in &["Args", "Exec", "Config", "LoopDetected", "Failed"] {
            let pat = format!(r#"{}\("([^"]*)"\)"#, variant);
            if let Some(re) = cached_regex(&pat) {
                let var = *variant;
                let new = re
                    .replace_all(content.as_str(), |caps: &regex::Captures| {
                        let msg = caps.get(1).unwrap().as_str();
                        format!("{}(\"{}\".to_string())", var, msg)
                    })
                    .to_string();
                if new != *content {
                    *content = new;
                }
            }
        }
    }

    /// Plan 373 G2: add missing use imports for common crate types referenced
    /// in trait impl blocks. Single-file transpile doesn't know about the
    /// assembled crate's module layout.
    pub(super) fn fix_missing_trait_impl_uses(content: &mut String) {
        for (type_name, use_stmt) in [
            ("JsonValue", "use crate::wire::JsonValue;"),
            ("ToolError", "use crate::error::ToolError;"),
            ("AgentError", "use crate::error::AgentError;"),
        ] {
            // Skip if already imported via `use crate::wire::JsonValue;` or
            // via `use wire: ..., JsonValue, ...` (Auto import syntax).
            //
            // Plan 018: the normal `use error: AgentError` translation emits the
            // brace form `use crate::error::{AgentError};`, which a plain
            // `contains(use_stmt)` (no braces) misses — causing a duplicate
            // injection and E0252. Derive and also check the brace form.
            let brace_form = use_stmt
                .strip_suffix(";")
                .map(|s| {
                    if let Some(pos) = s.rfind("::") {
                        format!("{}::{{{}}};", &s[..pos], &s[pos + 2..])
                    } else {
                        s.to_string()
                    }
                })
                .unwrap_or_else(|| use_stmt.to_string());
            let already_via_rust = content.contains(use_stmt) || content.contains(&brace_form);
            let already_via_auto = content.contains(&format!(": {}", type_name))
                || content.contains(&format!(", {}", type_name))
                || content.contains(&format!("{} ,", type_name));
            if content.contains(type_name) && !already_via_rust && !already_via_auto {
                if let Some(pos) = content.find("use a2r_std::*;") {
                    content.insert_str(pos + "use a2r_std::*;".len(), &format!("\n{}", use_stmt));
                }
            }
        }
    }

    /// Plan 376 Pass 4: Fix `for x in self.field` → `for x in &self.field`
    /// when the enclosing method is `&self` (not `&mut self`). Without this,
    /// iterating a Vec field of `&self` causes E0507 (cannot move out of self.field).
    pub(super) fn fix_for_in_self_field_borrow(content: &mut String) {
        let re = cached_regex(r"for\s+(\w+)\s+in\s+self\.(\w+)\s*\{");
        if let Some(re) = re {
            let mut new_content = content.clone();
            let mut offset = 0;
            for caps in re.captures_iter(content.as_str()) {
                let full = caps.get(0).unwrap();
                let var = caps.get(1).unwrap().as_str();
                let field = caps.get(2).unwrap().as_str();
                let before = &content[..full.start()];
                let fn_line = before.rfind("fn ").map(|pos| &content[pos..full.start()]);
                let is_mut = fn_line.map_or(false, |line| line.contains("&mut self"));
                if !is_mut {
                    let old = format!("for {} in self.{} {{", var, field);
                    let new = format!("for {} in &self.{} {{", var, field);
                    let pos = new_content[offset..].find(&old);
                    if let Some(p) = pos {
                        let abs = offset + p;
                        new_content.replace_range(abs..abs + old.len(), &new);
                        offset = abs + new.len();
                    }
                }
            }
            if new_content != *content {
                *content = new_content;
            }
        }
    }

    /// Plan 376 Pass 2: Fix `.get(key).field` → `.get(key).unwrap().field`
    /// (HashMap.get returns Option, not the value directly).
    pub(super) fn fix_option_get_field_access(content: &mut String) {
        let safe_methods = [
            "is_some",
            "is_none",
            "unwrap",
            "unwrap_or",
            "unwrap_or_default",
            "map",
            "and_then",
            "unwrap_or_else",
            "as_ref",
            "as_deref",
            "copied",
            "cloned",
            "ok",
            "err",
            "iter",
            "into_iter",
            "as_mut",
            // Plan 610 ⑥: Option has expect too — injecting .unwrap() before
            // it broke libloading Symbol resolution (`lib().get(b"..").expect(..)`).
            "expect",
        ];
        if let Some(re) = cached_regex(r"\.get\(([^)]+)\)\.(\w+)") {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    let key = caps.get(1).unwrap().as_str();
                    let field = caps.get(2).unwrap().as_str();
                    if safe_methods.contains(&field) {
                        return format!(".get({}).{}", key, field);
                    }
                    format!(".get({}).unwrap().{}", key, field)
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }
    }

    /// Plan 376 Pass 2b: Fix a2r_std::fs function return type mismatches.
    /// read_to_string returns String (not Result); wrap in Ok() so match works.
    pub(super) fn fix_a2r_std_fs_result_patterns(content: &mut String) {
        // Plan 376W: use non-greedy .*? to handle nested parens in args
        // (e.g. path.to_str().unwrap() contains ')' which broke the old [^)]*).
        let re = cached_regex(
            r"match\s+(a2r_std::fs::(?:read_to_string|write|read_dir|exists|is_dir)\(.*?\))\s*\{",
        );
        if let Some(re) = re {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    let call = caps.get(1).unwrap().as_str();
                    // Plan 380: the bridged read_to_string returns String (not
                    // Result), so we fake a Result with Ok(...) for the match.
                    // Annotate the error type (the Err arm is dead) — bare
                    // `Ok(x)` can't infer E (E0282).
                    format!("match Ok::<String, std::io::Error>({}) {{", call)
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }
    }

    /// Plan 376V: Box spec-trait constructors in functions returning Box<dyn Trait>.
    /// `Some(Assistant())` → `Some(Box::new(Assistant()))` when the enclosing
    /// function returns Option<Box<dyn Role>> (or similar spec-trait). Auto's
    /// `has Role` spec means a concrete `Assistant` value must be boxed to
    /// satisfy `Box<dyn Role>`. Without this, `Some(Assistant {})` fails E0308
    /// (expected Box<dyn Role>, found Assistant).
    pub(super) fn fix_spec_trait_boxing(content: &mut String) {
        // Find functions returning Option<Box<dyn <Trait>>> and box their
        // Some(Constructor()) returns. The signature pattern:
        //   fn name(...) -> Option<Box<dyn Trait>> {
        // Match the trait name so we only box inside the right functions.
        if let Some(re) =
            cached_regex(r"(?ms)(fn \w+\([^)]*\)[^{]*->\s*Option<Box<dyn (\w+)>>\s*\{)(.*?)(^\})")
        {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    let header = caps.get(1).unwrap().as_str();
                    let body = caps.get(3).unwrap().as_str();
                    let close = caps.get(4).unwrap().as_str();
                    // Wrap Some(PascalCase {}) → Some(Box::new(PascalCase {}))
                    // (escaped \) — the unescaped form panicked with "unopened
                    // group" and silently dropped the builtin_roles module).
                    let body_re = regex::Regex::new(r"Some\((\w+)\s*\{\s*\}\)").unwrap();
                    let new_body = body_re.replace_all(body, "Some(Box::new($1 {}))");
                    format!("{}{}{}", header, new_body, close)
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }
    }

    /// Plan 376V: Convert tuple indexing pair[0]/pair[1] → pair.0/pair.1.
    /// Auto uses [] for both list and tuple access, but Rust tuples need .N.
    pub(super) fn fix_tuple_index(content: &mut String) {
        if let Some(re) = cached_regex(r"\bpair\[(\d+)\]") {
            let new = re
                .replace_all(content.as_str(), |caps: &regex::Captures| {
                    format!("pair.{}", caps.get(1).unwrap().as_str())
                })
                .to_string();
            if new != *content {
                *content = new;
            }
        }
    }
}
