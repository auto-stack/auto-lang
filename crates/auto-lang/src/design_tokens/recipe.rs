//! PLAN-607: Declarative Style Recipe Language Layer (Design 29 Phase 3)
//!
//! Style recipes provide named, reusable, and composable class string abstractions
//! with compile-time desugaring to standard Tailwind class strings or expressions.
//!
//! ```auto
//! style card_base = "bg-card rounded-xl shadow-sm border border-border"
//! style pill(bg: str = "bg-primary", fg: str = "text-primary-foreground", pad: str = "px-4 py-2") =
//!     "{pad} {bg} {fg} rounded-full text-sm font-medium shadow-sm hover:{bg}/90 transition-colors"
//! style pill_danger = pill(bg: "bg-destructive", fg: "text-destructive-foreground")
//! ```

use crate::ast::ui::{StyleRecipeDecl, StyleRecipeParam};
use crate::ast::{Arg, Expr, Name};
use auto_val::Op;
use std::collections::{HashMap, HashSet};

/// In-memory representation of a style recipe.
#[derive(Debug, Clone)]
pub struct StyleRecipe {
    pub name: String,
    pub params: Vec<StyleRecipeParam>,
    pub body: Expr,
    pub is_pub: bool,
    pub doc: Option<String>,
}

/// Errors occurring during style recipe analysis or expansion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StyleRecipeError {
    UndefinedRecipe(String),
    UnknownParameter {
        recipe: String,
        param: String,
    },
    MissingRequiredParameter {
        recipe: String,
        param: String,
    },
    TooManyArguments {
        recipe: String,
        expected: usize,
        got: usize,
    },
    CircularReference(Vec<String>),
    InvalidExpression(String),
}

impl std::fmt::Display for StyleRecipeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StyleRecipeError::UndefinedRecipe(name) => {
                write!(f, "Style recipe '{}' is not defined", name)
            }
            StyleRecipeError::UnknownParameter { recipe, param } => {
                write!(f, "Unknown parameter '{}' for style recipe '{}'", param, recipe)
            }
            StyleRecipeError::MissingRequiredParameter { recipe, param } => {
                write!(f, "Missing required parameter '{}' for style recipe '{}'", param, recipe)
            }
            StyleRecipeError::TooManyArguments { recipe, expected, got } => {
                write!(f, "Style recipe '{}' expects at most {} arguments, got {}", recipe, expected, got)
            }
            StyleRecipeError::CircularReference(cycle) => {
                write!(f, "Circular reference detected in style recipes: {}", cycle.join(" -> "))
            }
            StyleRecipeError::InvalidExpression(msg) => {
                write!(f, "Invalid style recipe expression: {}", msg)
            }
        }
    }
}

impl std::error::Error for StyleRecipeError {}

thread_local! {
    static REGISTRY: std::cell::RefCell<HashMap<String, StyleRecipe>> = std::cell::RefCell::new(HashMap::new());
}

/// Clear all registered style recipes (e.g. before compiling a new module).
pub fn clear_style_recipes() {
    REGISTRY.with(|r| r.borrow_mut().clear());
}

/// Register a style recipe from an AST declaration.
pub fn register_style_recipe(decl: &StyleRecipeDecl) {
    let recipe = StyleRecipe {
        name: decl.name.as_str().to_string(),
        params: decl.params.clone(),
        body: decl.body.clone(),
        is_pub: decl.is_pub,
        doc: decl.doc.as_ref().map(|d| d.as_str().to_string()),
    };
    REGISTRY.with(|r| {
        r.borrow_mut().insert(decl.name.as_str().to_string(), recipe);
    });
}

/// Look up a style recipe by name.
pub fn get_style_recipe(name: &str) -> Option<StyleRecipe> {
    REGISTRY.with(|r| r.borrow().get(name).cloned())
}

/// Check if a style recipe is registered.
pub fn has_style_recipe(name: &str) -> bool {
    REGISTRY.with(|r| r.borrow().contains_key(name))
}

/// Return all registered style recipes.
pub fn all_style_recipes() -> Vec<StyleRecipe> {
    REGISTRY.with(|r| r.borrow().values().cloned().collect())
}

/// Register all style recipes from AST statements, validate them, and return any lint warnings.
pub fn load_and_validate_style_recipes(stmts: &[crate::ast::Stmt]) -> Result<Vec<String>, StyleRecipeError> {
    clear_style_recipes();
    for stmt in stmts {
        if let crate::ast::Stmt::StyleRecipeDecl(r) = stmt {
            register_style_recipe(r);
        }
    }
    validate_style_recipes()?;

    let mut warnings = Vec::new();
    for recipe in all_style_recipes() {
        warnings.extend(lint_check_recipe(&recipe));
    }
    Ok(warnings)
}

// ============================================================================
// Semantic Validation (T-02)
// ============================================================================

/// Validate all registered style recipes:
/// 1. Detect circular references (e.g. `style a = b` and `style b = a`).
/// 2. Check that any referenced recipes exist and have valid arguments.
pub fn validate_style_recipes() -> Result<(), StyleRecipeError> {
    let recipes = all_style_recipes();
    let recipe_names: HashSet<String> = recipes.iter().map(|r| r.name.clone()).collect();

    // 1. Check references and build dependency graph
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for recipe in &recipes {
        let mut deps = Vec::new();
        let param_names: HashSet<String> = recipe.params.iter().map(|p| p.name.as_str().to_string()).collect();
        collect_recipe_dependencies(&recipe.body, &param_names, &recipe_names, &mut deps)?;
        adj.insert(recipe.name.clone(), deps);
    }

    // 2. Circular reference detection via DFS
    let mut visited: HashSet<String> = HashSet::new();
    let mut on_stack: HashSet<String> = HashSet::new();
    let mut path: Vec<String> = Vec::new();

    for recipe in &recipes {
        if !visited.contains(&recipe.name) {
            check_cycle(&recipe.name, &adj, &mut visited, &mut on_stack, &mut path)?;
        }
    }

    Ok(())
}

fn check_cycle(
    node: &str,
    adj: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    on_stack: &mut HashSet<String>,
    path: &mut Vec<String>,
) -> Result<(), StyleRecipeError> {
    visited.insert(node.to_string());
    on_stack.insert(node.to_string());
    path.push(node.to_string());

    if let Some(neighbors) = adj.get(node) {
        for next in neighbors {
            if on_stack.contains(next) {
                // Cycle detected
                let mut cycle = Vec::new();
                if let Some(idx) = path.iter().position(|p| p == next) {
                    cycle.extend_from_slice(&path[idx..]);
                }
                cycle.push(next.clone());
                return Err(StyleRecipeError::CircularReference(cycle));
            }
            if !visited.contains(next) {
                check_cycle(next, adj, visited, on_stack, path)?;
            }
        }
    }

    path.pop();
    on_stack.remove(node);
    Ok(())
}

fn collect_recipe_dependencies(
    expr: &Expr,
    params: &HashSet<String>,
    known_recipes: &HashSet<String>,
    out: &mut Vec<String>,
) -> Result<(), StyleRecipeError> {
    match expr {
        Expr::Ident(name) => {
            let s = name.as_str();
            if params.contains(s) {
                // It's a parameter of the recipe
                return Ok(());
            }
            if known_recipes.contains(s) {
                out.push(s.to_string());
            } else if !is_standard_identifier(s) {
                return Err(StyleRecipeError::UndefinedRecipe(s.to_string()));
            }
        }
        Expr::Call(call) => {
            if let Expr::Ident(fname) = call.name.as_ref() {
                let s = fname.as_str();
                if known_recipes.contains(s) {
                    // Validate call arguments
                    validate_recipe_call(s, &call.args.args)?;
                    out.push(s.to_string());
                } else if !params.contains(s) && !is_standard_identifier(s) {
                    return Err(StyleRecipeError::UndefinedRecipe(s.to_string()));
                }
            }
            for arg in &call.args.args {
                let arg_expr = match arg {
                    Arg::Pos(e) => e,
                    Arg::Pair(_, e) => e,
                    Arg::Name(_) => continue,
                };
                collect_recipe_dependencies(arg_expr, params, known_recipes, out)?;
            }
        }
        Expr::Array(elems) => {
            for e in elems {
                collect_recipe_dependencies(e, params, known_recipes, out)?;
            }
        }
        Expr::FStr(fstr) => {
            for p in &fstr.parts {
                collect_recipe_dependencies(p, params, known_recipes, out)?;
            }
        }
        Expr::Bina(l, _, r) => {
            collect_recipe_dependencies(l, params, known_recipes, out)?;
            collect_recipe_dependencies(r, params, known_recipes, out)?;
        }
        Expr::If(if_expr) => {
            for b in &if_expr.branches {
                for s in &b.body.stmts {
                    if let crate::ast::Stmt::Expr(e) = s {
                        collect_recipe_dependencies(e, params, known_recipes, out)?;
                    }
                }
            }
            if let Some(eb) = &if_expr.else_ {
                for s in &eb.stmts {
                    if let crate::ast::Stmt::Expr(e) = s {
                        collect_recipe_dependencies(e, params, known_recipes, out)?;
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn is_standard_identifier(s: &str) -> bool {
    // Ignore boolean/state/builtin identifiers
    s.starts_with('.') || matches!(s, "true" | "false" | "nil" | "null" | "self" | "this")
}

/// Validate a call to a recipe with given arguments.
pub fn validate_recipe_call(name: &str, args: &[Arg]) -> Result<(), StyleRecipeError> {
    let recipe = match get_style_recipe(name) {
        Some(r) => r,
        None => return Err(StyleRecipeError::UndefinedRecipe(name.to_string())),
    };

    let mut supplied_params: HashSet<String> = HashSet::new();
    let mut positional_count = 0;

    for arg in args {
        match arg {
            Arg::Pos(_) => {
                if positional_count >= recipe.params.len() {
                    return Err(StyleRecipeError::TooManyArguments {
                        recipe: name.to_string(),
                        expected: recipe.params.len(),
                        got: args.len(),
                    });
                }
                let param_name = recipe.params[positional_count].name.as_str().to_string();
                supplied_params.insert(param_name);
                positional_count += 1;
            }
            Arg::Pair(k, _) | Arg::Name(k) => {
                let key = k.as_str().to_string();
                if !recipe.params.iter().any(|p| p.name.as_str() == key) {
                    return Err(StyleRecipeError::UnknownParameter {
                        recipe: name.to_string(),
                        param: key,
                    });
                }
                supplied_params.insert(key);
            }
        }
    }

    // Check that all required parameters (no default value) are provided
    for param in &recipe.params {
        let pname = param.name.as_str();
        if !supplied_params.contains(pname) && param.default_value.is_none() {
            return Err(StyleRecipeError::MissingRequiredParameter {
                recipe: name.to_string(),
                param: pname.to_string(),
            });
        }
    }

    Ok(())
}

// ============================================================================
// Desugar Engine (T-03)
// ============================================================================

/// Desugar any style recipe references within an expression into a standard class expression.
/// Compatible with:
/// - Bare identifiers: `style: card_base`
/// - Recipe calls: `style: pill(bg: "bg-destructive")`
/// - Array of parts: `style: [pill(), "ml-2"]`
/// - Conditionals: `style: if cond { pill() } else { card_base }`
/// - F-string interpolation: `style: "{card_base} mt-4"`
pub fn desugar_style_expr(expr: &Expr) -> Result<Expr, StyleRecipeError> {
    match expr {
        Expr::Ident(name) => {
            let s = name.as_str();
            if has_style_recipe(s) {
                expand_recipe_call(s, &[])
            } else {
                Ok(expr.clone())
            }
        }
        Expr::Call(call) => {
            if let Expr::Ident(name) = call.name.as_ref() {
                let s = name.as_str();
                if has_style_recipe(s) {
                    return expand_recipe_call(s, &call.args.args);
                }
            }
            // If not a recipe call, keep as-is but desugar argument expressions
            let mut new_args = Vec::with_capacity(call.args.args.len());
            for arg in &call.args.args {
                match arg {
                    Arg::Pos(e) => new_args.push(Arg::Pos(desugar_style_expr(e)?)),
                    Arg::Pair(k, e) => new_args.push(Arg::Pair(k.clone(), desugar_style_expr(e)?)),
                    Arg::Name(_) => new_args.push(arg.clone()),
                }
            }
            let mut new_call = call.clone();
            new_call.args.args = new_args;
            Ok(Expr::Call(new_call))
        }
        Expr::Array(elems) => {
            let mut new_elems = Vec::with_capacity(elems.len());
            for e in elems {
                new_elems.push(desugar_style_expr(e)?);
            }
            Ok(Expr::Array(new_elems))
        }
        Expr::If(if_expr) => {
            let mut new_if = if_expr.clone();
            for branch in &mut new_if.branches {
                for stmt in &mut branch.body.stmts {
                    if let crate::ast::Stmt::Expr(e) = stmt {
                        *e = desugar_style_expr(e)?;
                    }
                }
            }
            if let Some(ref mut else_body) = new_if.else_ {
                for stmt in &mut else_body.stmts {
                    if let crate::ast::Stmt::Expr(e) = stmt {
                        *e = desugar_style_expr(e)?;
                    }
                }
            }
            Ok(Expr::If(new_if))
        }
        Expr::Bina(l, Op::Add, r) => {
            let new_l = desugar_style_expr(l)?;
            let new_r = desugar_style_expr(r)?;
            // If both sides are string literals, fold them into a single string literal
            if let (Expr::Str(sl), Expr::Str(sr)) = (&new_l, &new_r) {
                let mut joined = sl.to_string();
                if !joined.is_empty() && !sr.is_empty() && !joined.ends_with(' ') && !sr.starts_with(' ') {
                    joined.push(' ');
                }
                joined.push_str(sr.as_str());
                Ok(Expr::Str(joined.into()))
            } else {
                Ok(Expr::Bina(Box::new(new_l), Op::Add, Box::new(new_r)))
            }
        }
        Expr::Str(s) => {
            // Check for recipe references in interpolation syntax `{recipe_name}`
            let str_val = s.as_str();
            let resolved = expand_string_recipe_interpolation(str_val)?;
            Ok(Expr::Str(resolved.into()))
        }
        Expr::FStr(fstr) => {
            let mut new_parts = Vec::with_capacity(fstr.parts.len());
            for p in &fstr.parts {
                new_parts.push(desugar_style_expr(p)?);
            }
            Ok(Expr::FStr(crate::ast::fstr::FStr { parts: new_parts }))
        }
        other => Ok(other.clone()),
    }
}

/// Expand `{recipe_name}` inside a string literal if `recipe_name` matches a registered recipe.
fn expand_string_recipe_interpolation(s: &str) -> Result<String, StyleRecipeError> {
    if !s.contains('{') {
        return Ok(s.to_string());
    }

    let recipes = all_style_recipes();
    if recipes.is_empty() {
        return Ok(s.to_string());
    }

    let mut result = s.to_string();
    for recipe in recipes {
        let pat = format!("{{{}}}", recipe.name);
        if result.contains(&pat) {
            let expanded = expand_recipe_call(&recipe.name, &[])?;
            if let Expr::Str(exp_str) = expanded {
                result = result.replace(&pat, exp_str.as_str());
            }
        }
    }

    Ok(result)
}

/// Expand a recipe call by substituting parameters and inlining the body.
pub fn expand_recipe_call(name: &str, args: &[Arg]) -> Result<Expr, StyleRecipeError> {
    validate_recipe_call(name, args)?;
    let recipe = get_style_recipe(name).unwrap();

    // Map: param_name -> Expr
    let mut param_map: HashMap<String, Expr> = HashMap::new();

    // 1. Populate default values
    for param in &recipe.params {
        if let Some(default_expr) = &param.default_value {
            param_map.insert(param.name.as_str().to_string(), default_expr.clone());
        }
    }

    // 2. Populate positional arguments
    let mut pos_idx = 0;
    for arg in args {
        match arg {
            Arg::Pos(e) => {
                if pos_idx < recipe.params.len() {
                    let pname = recipe.params[pos_idx].name.as_str().to_string();
                    param_map.insert(pname, e.clone());
                    pos_idx += 1;
                }
            }
            Arg::Pair(k, e) => {
                param_map.insert(k.as_str().to_string(), e.clone());
            }
            Arg::Name(k) => {
                param_map.insert(k.as_str().to_string(), Expr::Str(k.as_str().into()));
            }
        }
    }

    // Recursively desugar arguments in param_map
    for expr in param_map.values_mut() {
        *expr = desugar_style_expr(expr)?;
    }

    // Substitute parameters into the recipe body
    substitute_and_expand(&recipe.body, &param_map)
}

fn substitute_and_expand(body: &Expr, param_map: &HashMap<String, Expr>) -> Result<Expr, StyleRecipeError> {
    match body {
        Expr::Str(s) => {
            let text = s.as_str();
            // Check if all parameters in param_map are static string literals
            let all_static_strings = param_map.values().all(|e| matches!(e, Expr::Str(_)));

            if all_static_strings {
                let mut replaced = text.to_string();
                for (param_name, arg_expr) in param_map {
                    if let Expr::Str(arg_str) = arg_expr {
                        let pat1 = format!("{{{}}}", param_name);
                        let pat2 = format!("${{{}}}", param_name);
                        replaced = replaced.replace(&pat1, arg_str.as_str());
                        replaced = replaced.replace(&pat2, arg_str.as_str());
                    }
                }
                // Check if the resulting string itself references other recipes
                let final_expanded = expand_string_recipe_interpolation(&replaced)?;
                Ok(Expr::Str(final_expanded.into()))
            } else {
                // If some params are dynamic, construct an array or f-string
                // For simplicity and dual-backend consistency, split into parts
                let mut parts: Vec<Expr> = Vec::new();
                let mut last_idx = 0;
                let chars: Vec<(usize, char)> = text.char_indices().collect();
                let mut i = 0;
                while i < chars.len() {
                    if chars[i].1 == '{' {
                        let start = chars[i].0;
                        let mut end = None;
                        for j in (i + 1)..chars.len() {
                            if chars[j].1 == '}' {
                                end = Some((j, chars[j].0));
                                break;
                            }
                        }
                        if let Some((end_idx, end_pos)) = end {
                            let param_key = &text[start + 1..end_pos];
                            if let Some(replacement) = param_map.get(param_key) {
                                if start > last_idx {
                                    let literal_part = &text[last_idx..start];
                                    if !literal_part.is_empty() {
                                        parts.push(Expr::Str(literal_part.into()));
                                    }
                                }
                                parts.push(replacement.clone());
                                last_idx = end_pos + 1;
                                i = end_idx + 1;
                                continue;
                            }
                        }
                    }
                    i += 1;
                }
                if last_idx < text.len() {
                    let trailing = &text[last_idx..];
                    if !trailing.is_empty() {
                        parts.push(Expr::Str(trailing.into()));
                    }
                }
                if parts.len() == 1 {
                    Ok(parts.into_iter().next().unwrap())
                } else {
                    Ok(Expr::Array(parts))
                }
            }
        }
        Expr::Call(call) => {
            // Composite recipe: e.g. `style pill_danger = pill(bg: "bg-destructive")`
            // Substitute parameters into the call's arguments first
            let mut new_args = Vec::with_capacity(call.args.args.len());
            for arg in &call.args.args {
                match arg {
                    Arg::Pos(e) => {
                        let subbed = substitute_expr(e, param_map);
                        new_args.push(Arg::Pos(subbed));
                    }
                    Arg::Pair(k, e) => {
                        let subbed = substitute_expr(e, param_map);
                        new_args.push(Arg::Pair(k.clone(), subbed));
                    }
                    Arg::Name(k) => {
                        if let Some(subbed) = param_map.get(k.as_str()) {
                            new_args.push(Arg::Pos(subbed.clone()));
                        } else {
                            new_args.push(arg.clone());
                        }
                    }
                }
            }
            if let Expr::Ident(fname) = call.name.as_ref() {
                if has_style_recipe(fname.as_str()) {
                    return expand_recipe_call(fname.as_str(), &new_args);
                }
            }
            let mut subbed_call = call.clone();
            subbed_call.args.args = new_args;
            Ok(Expr::Call(subbed_call))
        }
        Expr::Ident(name) => {
            let s = name.as_str();
            if let Some(subbed) = param_map.get(s) {
                Ok(subbed.clone())
            } else if has_style_recipe(s) {
                expand_recipe_call(s, &[])
            } else {
                Ok(body.clone())
            }
        }
        Expr::Array(elems) => {
            let mut new_elems = Vec::with_capacity(elems.len());
            for e in elems {
                new_elems.push(substitute_and_expand(e, param_map)?);
            }
            Ok(Expr::Array(new_elems))
        }
        other => Ok(other.clone()),
    }
}

fn substitute_expr(expr: &Expr, param_map: &HashMap<String, Expr>) -> Expr {
    match expr {
        Expr::Ident(name) => {
            if let Some(sub) = param_map.get(name.as_str()) {
                sub.clone()
            } else {
                expr.clone()
            }
        }
        Expr::Str(s) => {
            let mut text = s.as_str().to_string();
            for (k, v) in param_map {
                if let Expr::Str(v_str) = v {
                    text = text.replace(&format!("{{{}}}", k), v_str.as_str());
                    text = text.replace(&format!("${{{}}}", k), v_str.as_str());
                }
            }
            Expr::Str(text.into())
        }
        _ => expr.clone(),
    }
}

// ============================================================================
// Tokenization Lint Rules (T-05)
// ============================================================================

const PALETTE_COLORS: &[&str] = &[
    "slate", "gray", "zinc", "neutral", "stone",
    "red", "orange", "amber", "yellow", "lime",
    "green", "emerald", "teal", "cyan", "sky",
    "blue", "indigo", "violet", "purple", "fuchsia",
    "pink", "rose",
];

const COLOR_PREFIXES: &[&str] = &[
    "bg", "text", "border", "fill", "stroke", "ring", "accent", "from", "to", "via", "placeholder",
];

/// Check a style recipe body for hardcoded color palette classes (e.g. `bg-blue-500`).
/// Returns a list of warning messages.
pub fn lint_check_recipe(recipe: &StyleRecipe) -> Vec<String> {
    let mut warnings = Vec::new();
    let mut strings_to_check = Vec::new();
    collect_strings_from_expr(&recipe.body, &mut strings_to_check);

    for text in strings_to_check {
        for cls in text.split_whitespace() {
            // Strip modifiers (hover:, dark:, focus:, etc.)
            let bare_cls = cls.rsplit(':').next().unwrap_or(cls);
            // Strip opacity (/80, /90)
            let base_cls = bare_cls.split('/').next().unwrap_or(bare_cls);

            for prefix in COLOR_PREFIXES {
                let prefix_dash = format!("{}-", prefix);
                if let Some(rest) = base_cls.strip_prefix(&prefix_dash) {
                    for color in PALETTE_COLORS {
                        let color_dash = format!("{}-", color);
                        if let Some(weight) = rest.strip_prefix(&color_dash) {
                            // Check if weight is numeric like 50, 100..950
                            if weight.chars().all(|c| c.is_ascii_digit()) {
                                warnings.push(format!(
                                    "Hardcoded color palette class '{}' in style recipe '{}'. \
                                     Prefer semantic tokens like '{}-primary' or '{}-destructive'.",
                                    cls, recipe.name, prefix, prefix
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    warnings
}

fn collect_strings_from_expr(expr: &Expr, out: &mut Vec<String>) {
    match expr {
        Expr::Str(s) => out.push(s.as_str().to_string()),
        Expr::Array(elems) => {
            for e in elems {
                collect_strings_from_expr(e, out);
            }
        }
        Expr::FStr(fstr) => {
            for p in &fstr.parts {
                collect_strings_from_expr(p, out);
            }
        }
        Expr::Bina(l, Op::Add, r) => {
            collect_strings_from_expr(l, out);
            collect_strings_from_expr(r, out);
        }
        Expr::If(if_expr) => {
            for b in &if_expr.branches {
                for stmt in &b.body.stmts {
                    if let crate::ast::Stmt::Expr(e) = stmt {
                        collect_strings_from_expr(e, out);
                    }
                }
            }
            if let Some(eb) = &if_expr.else_ {
                for stmt in &eb.stmts {
                    if let crate::ast::Stmt::Expr(e) = stmt {
                        collect_strings_from_expr(e, out);
                    }
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use crate::session::CompilerSession;

    #[test]
    fn test_style_recipe_semantic_validation() {
        clear_style_recipes();

        // 1. Direct cycle: a -> b -> a
        let code_cycle = r#"
            style a = b
            style b = a
        "#;
        let mut p = Parser::from(code_cycle).with_session(CompilerSession::ui());
        let ast = p.parse().unwrap();
        let err = load_and_validate_style_recipes(&ast.stmts);
        assert!(err.is_err(), "Expected error for circular references");
        let err_str = err.err().unwrap().to_string();
        assert!(err_str.contains("Circular reference"), "Expected cycle error, got: {}", err_str);

        // 2. Unknown parameter in call
        let code_param = r#"
            style pill(bg: str = "bg-primary") = "{bg} rounded-full"
            style bad = pill(foo: "bar")
        "#;
        let mut p = Parser::from(code_param).with_session(CompilerSession::ui());
        let ast = p.parse().unwrap();
        let err = load_and_validate_style_recipes(&ast.stmts);
        assert!(err.is_err(), "Expected error for unknown parameter");
        let err_str = err.err().unwrap().to_string();
        assert!(err_str.contains("Unknown parameter 'foo'"), "Expected unknown param error, got: {}", err_str);

        // 3. Missing required parameter
        let code_req = r#"
            style pill(bg: str, pad: str = "p-2") = "{bg} {pad}"
            style bad = pill(pad: "p-4")
        "#;
        let mut p = Parser::from(code_req).with_session(CompilerSession::ui());
        let ast = p.parse().unwrap();
        let err = load_and_validate_style_recipes(&ast.stmts);
        assert!(err.is_err(), "Expected error for missing required parameter");
        let err_str = err.err().unwrap().to_string();
        assert!(err_str.contains("Missing required parameter 'bg'"), "Expected missing param error, got: {}", err_str);

        // 4. Undefined recipe reference
        let code_undef = r#"
            style good = missing_recipe
        "#;
        let mut p = Parser::from(code_undef).with_session(CompilerSession::ui());
        let ast = p.parse().unwrap();
        let err = load_and_validate_style_recipes(&ast.stmts);
        assert!(err.is_err(), "Expected error for undefined recipe");
        let err_str = err.err().unwrap().to_string();
        assert!(err_str.contains("is not defined"), "Expected undefined recipe error, got: {}", err_str);
    }

    #[test]
    fn test_style_recipe_desugar_engine() {
        clear_style_recipes();

        let code = r#"
            style card_base = "bg-card rounded-xl shadow-sm"
            style pill(bg: str = "bg-primary", fg: str = "text-primary-foreground", pad: str = "px-4 py-2") =
                "{pad} {bg} {fg} rounded-full"
            style pill_danger = pill(bg: "bg-destructive", fg: "text-destructive-foreground")
            style composite = "{card_base} mt-4"
        "#;
        let mut p = Parser::from(code).with_session(CompilerSession::ui());
        let ast = p.parse().unwrap();
        let res = load_and_validate_style_recipes(&ast.stmts);
        assert!(res.is_ok(), "Validation failed: {:?}", res);

        // Test bare ident expansion
        let ident_expr = Expr::Ident("card_base".into());
        let desugared = desugar_style_expr(&ident_expr).unwrap();
        if let Expr::Str(s) = desugared {
            assert_eq!(s.as_str(), "bg-card rounded-xl shadow-sm");
        } else {
            panic!("Expected Expr::Str, got: {:?}", desugared);
        }

        // Test call with defaults
        let call_defaults = Parser::parse_expr_fragment("pill()").unwrap();
        let desugared = desugar_style_expr(&call_defaults).unwrap();
        if let Expr::Str(s) = desugared {
            assert_eq!(s.as_str(), "px-4 py-2 bg-primary text-primary-foreground rounded-full");
        } else {
            panic!("Expected Expr::Str, got: {:?}", desugared);
        }

        // Test call with overrides
        let call_override = Parser::parse_expr_fragment("pill(bg: \"bg-red-500\")").unwrap();
        let desugared = desugar_style_expr(&call_override).unwrap();
        if let Expr::Str(s) = desugared {
            assert_eq!(s.as_str(), "px-4 py-2 bg-red-500 text-primary-foreground rounded-full");
        } else {
            panic!("Expected Expr::Str, got: {:?}", desugared);
        }

        // Test composite recipe expansion
        let comp_ident = Expr::Ident("pill_danger".into());
        let desugared = desugar_style_expr(&comp_ident).unwrap();
        if let Expr::Str(s) = desugared {
            assert_eq!(s.as_str(), "px-4 py-2 bg-destructive text-destructive-foreground rounded-full");
        } else {
            panic!("Expected Expr::Str, got: {:?}", desugared);
        }

        // Test interpolation in string
        let interp = Expr::Str("{card_base} p-2".into());
        let desugared = desugar_style_expr(&interp).unwrap();
        if let Expr::Str(s) = desugared {
            assert_eq!(s.as_str(), "bg-card rounded-xl shadow-sm p-2");
        } else {
            panic!("Expected Expr::Str, got: {:?}", desugared);
        }

        // Test array mixing
        let arr = Expr::Array(vec![
            Expr::Ident("card_base".into()),
            Expr::Str("border-2".into()),
        ]);
        let desugared = desugar_style_expr(&arr).unwrap();
        if let Expr::Array(elems) = desugared {
            assert_eq!(elems.len(), 2);
            if let Expr::Str(s0) = &elems[0] {
                assert_eq!(s0.as_str(), "bg-card rounded-xl shadow-sm");
            } else {
                panic!("Expected Expr::Str for elem 0");
            }
            if let Expr::Str(s1) = &elems[1] {
                assert_eq!(s1.as_str(), "border-2");
            } else {
                panic!("Expected Expr::Str for elem 1");
            }
        } else {
            panic!("Expected Expr::Array, got: {:?}", desugared);
        }
    }

    #[test]
    fn test_style_recipe_lint_palette_warnings() {
        clear_style_recipes();

        let code = r#"
            style good_recipe = "bg-primary text-primary-foreground border-border"
            style bad_recipe = "bg-blue-500 text-zinc-400 hover:bg-red-600/80"
        "#;
        let mut p = Parser::from(code).with_session(CompilerSession::ui());
        let ast = p.parse().unwrap();
        let warnings = load_and_validate_style_recipes(&ast.stmts).unwrap();

        // bad_recipe should generate warnings for bg-blue-500, text-zinc-400, hover:bg-red-600/80
        assert_eq!(warnings.len(), 3, "Expected 3 warnings, got: {:?}", warnings);
        assert!(warnings.iter().any(|w| w.contains("bg-blue-500")), "Expected bg-blue-500 warning");
        assert!(warnings.iter().any(|w| w.contains("text-zinc-400")), "Expected text-zinc-400 warning");
        assert!(warnings.iter().any(|w| w.contains("hover:bg-red-600/80")), "Expected hover:bg-red-600/80 warning");
    }

    #[test]
    fn test_style_recipe_in_widget_extraction() {
        clear_style_recipes();

        let code = r#"
            style card_base = "bg-card rounded-xl shadow-sm"
            style btn(bg: str = "bg-primary") = "{bg} px-4 py-2 rounded-md"

            widget TestCard {
                view {
                    col {
                        style: card_base
                        button "Click" {
                            style: btn(bg: "bg-destructive")
                        }
                    }
                }
            }
        "#;
        let mut p = Parser::from(code).with_session(CompilerSession::ui());
        let ast = p.parse().unwrap();
        load_and_validate_style_recipes(&ast.stmts).unwrap();

        let widget_decl = ast.stmts.iter().find_map(|s| {
            if let crate::ast::Stmt::WidgetDecl(d) = s { Some(d) } else { None }
        }).unwrap();

        let aura_widget = crate::aura::extract_widget_from_decl(widget_decl).unwrap();
        if let crate::aura::AuraNode::Element { props, children, .. } = &aura_widget.view_tree {
            // The root col should have desugared style
            let root_style = props.get("style").unwrap();
            if let crate::aura::AuraPropValue::Expr(Expr::Str(s)) = root_style {
                assert_eq!(s.as_str(), "bg-card rounded-xl shadow-sm");
            } else {
                panic!("Expected Expr::Str for col style, got: {:?}", root_style);
            }

            // The button child should have desugared style
            if let crate::aura::AuraNode::Element { props: btn_props, .. } = &children[0] {
                let btn_style = btn_props.get("style").unwrap();
                if let crate::aura::AuraPropValue::Expr(Expr::Str(s)) = btn_style {
                    assert_eq!(s.as_str(), "bg-destructive px-4 py-2 rounded-md");
                } else {
                    panic!("Expected Expr::Str for button style, got: {:?}", btn_style);
                }
            } else {
                panic!("Expected AuraNode::Element for child, got: {:?}", children[0]);
            }
        } else {
            panic!("Expected AuraNode::Element for root view_tree, got: {:?}", aura_widget.view_tree);
        }
    }

    #[test]
    fn test_style_recipe_in_vue_gen() {
        clear_style_recipes();

        let code = r#"
            style card_base = "bg-card rounded-xl shadow-sm border border-border"
            style pill(bg: str = "bg-primary") = "{bg} px-4 py-2 rounded-full"

            widget RecipeCard {
                view {
                    col {
                        style: card_base
                        button "Click" {
                            style: pill(bg: "bg-destructive")
                        }
                    }
                }
            }
        "#;
        let mut p = Parser::from(code).with_session(CompilerSession::ui());
        let ast = p.parse().unwrap();
        load_and_validate_style_recipes(&ast.stmts).unwrap();

        let widget_decl = ast.stmts.iter().find_map(|s| {
            if let crate::ast::Stmt::WidgetDecl(d) = s { Some(d) } else { None }
        }).unwrap();

        let aura_widget = crate::aura::extract_widget_from_decl(widget_decl).unwrap();
        let mut gen = crate::ui_gen::VueGenerator::new();
        let sfc = gen.generate_sfc(&aura_widget).unwrap();

        // Check that Vue template contains the desugared class strings
        assert!(sfc.contains("bg-card rounded-xl shadow-sm border border-border"), "Vue output should contain desugared card_base: {}", sfc);
        assert!(sfc.contains("bg-destructive px-4 py-2 rounded-full"), "Vue output should contain desugared pill: {}", sfc);
        // And should NOT contain the raw recipe identifier
        assert!(!sfc.contains("card_base"), "Vue output should not contain raw recipe symbol 'card_base': {}", sfc);
    }
}

