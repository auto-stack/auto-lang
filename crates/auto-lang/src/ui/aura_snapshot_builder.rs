//! # AURA Source-Style Snapshot Builder (Plan 279)
//!
//! Generates MCP snapshots directly from the `AuraNode` tree, preserving
//! original AURA syntax: tag names (`center`, `col`, `button`), tailwind
//! class strings, and event attribute names (`onclick`, `oninput`).
//!
//! All state references are evaluated to their current values, making
//! the snapshot a "static rendering" of the dynamic AURA code.

use std::collections::HashMap;

use crate::ast::Expr;
use crate::aura::{AuraNode, AuraPropValue, AuraTextContent};
use crate::ui::render_support::{self, SupportLevel};
use crate::ui::style::BoxLayout;

/// Type alias for layout bounds map: widget ID → (x, y, width, height).
type LayoutBoundsMap = HashMap<String, (f32, f32, f32, f32)>;

/// Builds an AURA source-style snapshot from an AuraNode tree.
pub struct AuraSnapshotBuilder<'a> {
    state: &'a HashMap<String, auto_val::Value>,
    include_status: bool,
    viewport: Option<(f32, f32)>,
    layout_bounds: LayoutBoundsMap,
}

impl<'a> AuraSnapshotBuilder<'a> {
    pub fn new(state: &'a HashMap<String, auto_val::Value>) -> Self {
        Self {
            state,
            include_status: true,
            viewport: None,
            layout_bounds: LayoutBoundsMap::new(),
        }
    }

    /// Set whether to include render status annotations (default: true).
    pub fn with_status(mut self, include: bool) -> Self {
        self.include_status = include;
        self
    }

    /// Set viewport dimensions (width, height) in logical pixels (Plan 281).
    pub fn with_viewport(mut self, width: f32, height: f32) -> Self {
        self.viewport = Some((width, height));
        self
    }

    /// Set actual layout bounds from iced renderer (Plan 282).
    pub fn with_layout_bounds(mut self, bounds: LayoutBoundsMap) -> Self {
        self.layout_bounds = bounds;
        self
    }

    /// Build the complete AURA snapshot string.
    pub fn build(&self, widget_name: &str, root: &AuraNode) -> String {
        let mut out = String::new();

        // Header
        out.push_str("AURA Snapshot v2\n");
        out.push_str(&format!("widget: \"{}\"\n", widget_name));

        // Viewport (Plan 281)
        if let Some((w, h)) = self.viewport {
            out.push_str(&format!("viewport: {}x{}\n", w as i32, h as i32));
        }

        // State section
        out.push_str("\nstate:\n");
        let mut state_keys: Vec<_> = self.state.keys().collect();
        state_keys.sort();
        for key in state_keys {
            if let Some(val) = self.state.get(key) {
                let type_hint = match val {
                    auto_val::Value::Int(_) => "int",
                    auto_val::Value::Float(_) => "float",
                    auto_val::Value::Bool(_) => "bool",
                    auto_val::Value::Str(_) => "str",
                    auto_val::Value::Null => "null",
                    _ => "val",
                };
                out.push_str(&format!("  {}: {} ({})\n", key, Self::format_state_value(val), type_hint));
            }
        }

        // Tree section
        out.push_str("\ntree:\n");
        self.traverse(root, 0, &mut out);

        out
    }

    /// Recursively traverse an AuraNode tree and emit AURA-style text.
    fn traverse(&self, node: &AuraNode, indent: usize, out: &mut String) {
        let pad = "  ".repeat(indent);

        match node {
            AuraNode::Element {
                tag,
                props,
                events,
                children,
                debug_id,
                ..
            } => {
                // Reverse-map: col + "w-full h-full justify-center items-center" → center
                let display_tag = if tag == "col" {
                    if let Some(AuraPropValue::Expr(Expr::Str(s))) = props.get("style") {
                        if s == "w-full h-full justify-center items-center" {
                            "center"
                        } else {
                            tag
                        }
                    } else {
                        tag
                    }
                } else {
                    tag
                };

                let id_str = match debug_id {
                    Some(id) => format!(" #{}", id),
                    None => String::new(),
                };

                // Check for shorthand text label (e.g., button "Add")
                let label = Self::extract_label(props).map(|l| self.eval_interpolated_str(&l));

                // Extract box layout from style/class props (Plan 281)
                let layout_inline = if display_tag != "center" {
                    self.extract_layout_inline(props)
                } else {
                    None
                };

                // Look up actual layout bounds from iced renderer (Plan 282)
                let bounds_str = debug_id.as_ref().and_then(|id| {
                    let key = format!("aura_{}", id.0);
                    self.layout_bounds.get(&key).map(|(x, y, w, h)| {
                        format!("@rect({},{},{},{})", x.round() as i32, y.round() as i32, w.round() as i32, h.round() as i32)
                    })
                });

                // Build annotation suffix: @rect(...) [layout_info]
                let rect_part = bounds_str.as_ref().map(|r| format!(" {}", r)).unwrap_or_default();
                let layout_part = layout_inline.as_ref().map(|l| format!(" [{}]", l)).unwrap_or_default();
                let suffix = format!("{}{}", rect_part, layout_part);

                // Opening tag
                let has_body = !children.is_empty() || !events.is_empty() || Self::count_display_props(props) > 0;
                if let Some(ref lbl) = label {
                    if has_body {
                        out.push_str(&format!("{}{}{} \"{}\"{} {{\n", pad, display_tag, id_str, lbl, suffix));
                    } else {
                        out.push_str(&format!("{}{}{} \"{}\"{}\n", pad, display_tag, id_str, lbl, suffix));
                        return;
                    }
                } else if !has_body {
                    out.push_str(&format!("{}{}{}{}\n", pad, display_tag, id_str, suffix));
                    return;
                } else {
                    out.push_str(&format!("{}{}{}{} {{\n", pad, display_tag, id_str, suffix));
                }

                // Render status annotation (Plan 280)
                if self.include_status {
                    // Use original tag (not display_tag) for support lookup
                    let support = render_support::get_support(tag);
                    if support.level != SupportLevel::Full {
                        let icon = match support.level {
                            SupportLevel::Fallback => "\u{26a0} FALLBACK",
                            SupportLevel::Partial => "\u{26a0} PARTIAL",
                            SupportLevel::Unsupported => "\u{2717} UNSUPPORTED",
                            SupportLevel::Full => unreachable!(),
                        };
                        out.push_str(&format!(
                            "{}// {} {}\n",
                            "  ".repeat(indent + 1),
                            icon,
                            support.note
                        ));
                    }
                }

                // Props (evaluated)
                // For center (reverse-mapped from col), skip the default centering style
                let skip_style = display_tag == "center";
                self.emit_props(props, indent + 1, out, skip_style);

                // Events (original attribute names)
                let mut event_keys: Vec<_> = events.keys().collect();
                event_keys.sort();
                for event_name in event_keys {
                    if let Some(aura_event) = events.get(event_name) {
                        out.push_str(&format!(
                            "{}{}: {}\n",
                            "  ".repeat(indent + 1),
                            event_name,
                            aura_event.handler
                        ));
                    }
                }

                // Children
                for child in children {
                    self.traverse(child, indent + 1, out);
                }

                out.push_str(&format!("{}}}\n", pad));
            }

            AuraNode::Text(content) => {
                let evaluated = self.eval_text(content);
                out.push_str(&format!("{}\"{}\"\n", pad, Self::escape_str(&evaluated)));
            }

            AuraNode::ForLoop {
                var,
                iterable,
                body,
                debug_id,
                ..
            } => {
                let id_str = match debug_id {
                    Some(id) => format!(" #{}", id),
                    None => String::new(),
                };

                // Try to expand the loop from state.
                // Plan 370 D-GAP-4: store fields are merged into root state as
                // bare names, so `.store.notes` must flatten to `notes`.
                let state_name = Self::flatten_state_path(iterable.trim_start_matches('.'));
                if let Some(auto_val::Value::Array(items)) =
                    Self::resolve_state_ref(self.state, &state_name)
                {
                    for (i, _item) in items.iter().enumerate() {
                        out.push_str(&format!(
                            "{}/* for {} in {}[{}] */\n",
                            pad, var, iterable, i
                        ));
                        for child in body {
                            self.traverse(child, indent, out);
                        }
                    }
                } else {
                    // Cannot expand — show the loop header with unevaluated reference
                    out.push_str(&format!(
                        "{}for {} in {}{}\n",
                        pad, var, iterable, id_str
                    ));
                    for child in body {
                        self.traverse(child, indent + 1, out);
                    }
                }
            }

            AuraNode::Conditional {
                condition,
                then_body,
                else_body,
                ..
            } => {
                let result = self.eval_condition(condition);
                if result {
                    for child in then_body {
                        self.traverse(child, indent, out);
                    }
                } else if let Some(else_nodes) = else_body {
                    for child in else_nodes {
                        self.traverse(child, indent, out);
                    }
                }
            }

            AuraNode::Component { name, .. } => {
                out.push_str(&format!("{}/* component: {} */\n", pad, name));
            }

            AuraNode::Outlet => {
                out.push_str(&format!("{}/* outlet */\n", pad));
            }

            AuraNode::Link { to, text, .. } => {
                out.push_str(&format!("{}link \"{}\" -> {}\n", pad, text, to));
            }
        }
    }

    /// Emit properties, evaluating StateRef values.
    fn emit_props(&self, props: &HashMap<String, AuraPropValue>, indent: usize, out: &mut String, skip_style: bool) {
        let pad = "  ".repeat(indent);

        // Collect displayable props in order: class/style last, then others
        let mut display_order: Vec<String> = Vec::new();
        let mut style_value: Option<(String, String)> = None; // (key, value) — preserve "style" vs "class"

        for (key, prop_val) in props {
            match key.as_str() {
                // Skip internal-only props and "text" (used as shorthand label)
                "label" | "id" | "text" => continue,
                "class" | "style" => {
                    let val = self.eval_prop_value(prop_val);
                    style_value = Some((key.clone(), val));
                }
                _ => {
                    display_order.push(key.clone());
                }
            }
        }

        // Emit non-class props
        display_order.sort();
        for key in &display_order {
            if let Some(prop_val) = props.get(key) {
                let val = self.eval_prop_value(prop_val);
                out.push_str(&format!("{}{}: {}\n", pad, key, Self::format_prop_val(&val)));
            }
        }

        // Emit style/class last (preserve original key name)
        if !skip_style {
            if let Some((key, val)) = style_value {
                if !val.is_empty() {
                    out.push_str(&format!("{}{}: \"{}\"\n", pad, key, val));
                }
            }
        }
    }

    /// Evaluate an AuraPropValue to a string (public for MCP inspect tool).
    pub fn eval_prop_value(&self, prop: &AuraPropValue) -> String {
        match prop {
            AuraPropValue::Expr(expr) => self.eval_expr(expr),
            AuraPropValue::StyleBinding(bindings) => {
                // Style bindings like { completed: todo.done }
                // Evaluate each condition and collect active style names
                let active: Vec<String> = bindings
                    .iter()
                    .filter(|b| self.eval_expr_bool(&b.condition))
                    .map(|b| b.style_name.clone())
                    .collect();
                active.join(" ")
            }
        }
    }

    /// Evaluate a base AST `Expr` to a string.
    fn eval_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Str(s) => s.to_string(),
            Expr::Int(i) => i.to_string(),
            Expr::Float(f, _) => f.to_string(),
            Expr::Double(f, _) => f.to_string(),
            Expr::Bool(b) => b.to_string(),
            // State reference: an identifier whose name starts with "." is a
            // state-ref (e.g. ".count"). Other identifiers are also treated as
            // state references (mirroring the old AuraExpr extraction).
            Expr::Ident(name) => {
                let trimmed = name.as_str().trim_start_matches('.');
                self.state
                    .get(trimmed)
                    .map(|v| Self::format_eval_value(v))
                    .unwrap_or_else(|| format!("{}", name))
            }
            Expr::Array(items) => {
                let vals: Vec<String> = items.iter().map(|e| self.eval_expr(e)).collect();
                format!("[{}]", vals.join(", "))
            }
            _ => String::from("..."),
        }
    }

    /// Evaluate a base AST `Expr` as boolean (for style bindings and conditionals).
    fn eval_expr_bool(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Bool(b) => *b,
            Expr::Ident(name) => {
                let trimmed = name.as_str().trim_start_matches('.');
                self.state
                    .get(trimmed)
                    .map(|v| v.as_bool())
                    .unwrap_or(false)
            }
            _ => false,
        }
    }

    /// Evaluate a text content node.
    fn eval_text(&self, content: &AuraTextContent) -> String {
        match content {
            AuraTextContent::Literal(s) => s.clone(),
            AuraTextContent::Interpolated { template, bindings } => {
                let mut result = template.clone();

                // Strategy 1: Use explicit bindings if available
                if !bindings.is_empty() {
                    for var in bindings {
                        let var_clean = var.trim_start_matches('.');
                        let val = self
                            .state
                            .get(var_clean)
                            .map(|v| Self::format_eval_value(v))
                            .unwrap_or_default();
                        // Replace ${.var}, ${var}, ${.var }, ${var } patterns
                        for pattern in &[
                            format!("${{{}}}", var),
                            format!("${{.{} }}", var),
                            format!("${{{}}}", var_clean),
                            format!("${{.{} }}", var_clean),
                        ] {
                            result = result.replace(pattern, &val);
                        }
                    }
                    return result;
                }

                // Strategy 2: Regex-based fallback — extract all ${...} from template
                // This handles cases where bindings is empty but template contains ${.xxx}
                let mut pos = 0;
                let mut evaluated = String::new();
                let bytes = result.as_bytes();
                while pos < bytes.len() {
                    if bytes[pos] == b'$' && pos + 1 < bytes.len() && bytes[pos + 1] == b'{' {
                        // Find closing }
                        let _start = pos;
                        pos += 2;
                        let mut var_buf = String::new();
                        while pos < bytes.len() && bytes[pos] != b'}' {
                            var_buf.push(bytes[pos] as char);
                            pos += 1;
                        }
                        if pos < bytes.len() {
                            pos += 1; // skip '}'
                        }
                        let var_name = var_buf.trim().trim_start_matches('.');
                        let val = self
                            .state
                            .get(var_name)
                            .map(|v| Self::format_eval_value(v))
                            .unwrap_or_else(|| format!("${{{}}}", var_buf));
                        evaluated.push_str(&val);
                    } else {
                        evaluated.push(bytes[pos] as char);
                        pos += 1;
                    }
                }
                evaluated
            }
        }
    }

    /// Evaluate a condition string (for Conditional nodes).
    ///
    /// Supports patterns:
    /// - `.field == "lit"` / `.field != "lit"` — string comparison
    /// - `.field > N` / `<` / `>=` / `<=` — numeric comparison
    /// - `.array_field.len() > N` — list-length numeric comparison
    ///   (Plan 370 D-GAP-4: needed for store arrays like `.store.notes`)
    /// - `.field` — bare state ref truthy check
    /// - `true` / `false` — boolean literals
    ///
    /// Plan 370 D-GAP-4: store fields are merged into root state as bare
    /// names, so `.store.X` paths are flattened to `X` before lookup.
    pub(crate) fn eval_condition(&self, condition: &str) -> bool {
        let cond = condition.trim();

        // Boolean literals
        if cond == "true" {
            return true;
        }
        if cond == "false" {
            return false;
        }

        // Split into lhs op rhs at the first top-level comparison operator.
        // (Conditions here are simple — no nested parens/boolean combinators in
        // the snapshot path; AuraViewBuilder handles those for the live render.)
        if let Some((lhs, op, rhs)) = Self::split_comparison(cond) {
            let lhs_val = self.state_value_string(lhs);
            let rhs_val = rhs.trim().trim_matches('"');
            return Self::compare(&lhs_val, op, rhs_val);
        }

        // Bare state ref (with optional leading dot) — truthy check
        let name = cond.trim_start_matches('.');
        let name = Self::flatten_state_path(name);
        self.state
            .get(name)
            .map(|v| v.as_bool())
            .unwrap_or(false)
    }

    /// Split a condition into `(lhs, op, rhs)` at the first comparison operator
    /// found (` == `, ` != `, ` >= `, ` <= `, ` > `, ` < `). Returns None if no
    /// operator is present. Checks two-char operators before one-char ones so
    /// `>=` is not mis-split as `>`.
    fn split_comparison(cond: &str) -> Option<(&str, &str, &str)> {
        for (op, len) in [(" == ", 4), (" != ", 4), (" >= ", 4), (" <= ", 4), (" > ", 3), (" < ", 3)]
        {
            if let Some(pos) = cond.find(op) {
                return Some((&cond[..pos], op.trim(), &cond[pos + len..]));
            }
        }
        None
    }

    /// Resolve a condition lhs (e.g. `.store.notes.len()` or `.active_folder`)
    /// to its display string, flattening `.store.` and evaluating `.len()`.
    fn state_value_string(&self, lhs: &str) -> String {
        // Normalize spaces the parser may insert inside the call parens.
        let normalized = lhs.replace(" ( ", "(").replace("( ", "(").replace(" )", ")");
        let (field_expr, want_len) = if let Some(stripped) = normalized.strip_suffix(".len()") {
            (stripped, true)
        } else {
            (normalized.as_str(), false)
        };
        let name = Self::flatten_state_path(field_expr.trim_start_matches('.'));
        let Some(val) = self.state.get(name) else {
            return String::new();
        };
        if want_len {
            return match val {
                auto_val::Value::Array(a) => a.values.len().to_string(),
                // VmRef/other already materialized to Array by the MCP sync
                // path; if not, fall back to a truthy-but-unknown length of 0.
                _ => 0.to_string(),
            };
        }
        Self::format_eval_value(val)
    }

    /// Compare two display-string operands under `op`. Numeric for the ordering
    /// operators; string for == / !=.
    fn compare(lhs: &str, op: &str, rhs: &str) -> bool {
        match op {
            "==" => lhs == rhs,
            "!=" => lhs != rhs,
            ">" | "<" | ">=" | "<=" => {
                let l: f64 = match lhs.parse() {
                    Ok(n) => n,
                    Err(_) => return false,
                };
                let r: f64 = match rhs.parse() {
                    Ok(n) => n,
                    Err(_) => return false,
                };
                match op {
                    ">" => l > r,
                    "<" => l < r,
                    ">=" => l >= r,
                    "<=" => l <= r,
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// Flatten a `.store.X` / `store.X` state path to the bare root-state name
    /// `X`. Store fields are merged into the root state object as bare names, so
    /// a `store.notes` reference must become `notes` before a HashMap lookup.
    fn flatten_state_path(name: &str) -> &str {
        name.strip_prefix("store.").unwrap_or(name)
    }

    // ── Helpers ──

    /// Format a Value for display in state section (with quotes for strings).
    fn format_state_value(v: &auto_val::Value) -> String {
        match v {
            auto_val::Value::Int(i) => i.to_string(),
            auto_val::Value::Float(f) => format!("{:.2}", f),
            auto_val::Value::Bool(b) => b.to_string(),
            auto_val::Value::Str(s) => format!("\"{}\"", s),
            auto_val::Value::Null => "null".to_string(),
            other => other.to_string(),
        }
    }

    /// Format a Value for prop evaluation (no extra quoting — format_prop_val handles that).
    fn format_eval_value(v: &auto_val::Value) -> String {
        match v {
            auto_val::Value::Int(i) => i.to_string(),
            auto_val::Value::Float(f) => format!("{:.2}", f),
            auto_val::Value::Bool(b) => b.to_string(),
            auto_val::Value::Str(s) => s.to_string(),
            auto_val::Value::Null => "null".to_string(),
            other => other.to_string(),
        }
    }

    fn extract_label(props: &HashMap<String, AuraPropValue>) -> Option<String> {
        // Shorthand text label: button "Add" { ... } or text .city { ... }
        // In AuraNode, this is stored as a "text" or "label" prop
        // Returns the raw value (may contain ${...} patterns or be a state ref dot-name)
        props
            .get("text")
            .or_else(|| props.get("label"))
            .and_then(|pv| match pv {
                AuraPropValue::Expr(Expr::Str(s)) => Some(s.to_string()),
                AuraPropValue::Expr(Expr::Ident(name)) => Some(name.to_string()),
                _ => None,
            })
    }

    /// Evaluate ${.state_ref} and .state_ref patterns in a string literal.
    fn eval_interpolated_str(&self, s: &str) -> String {
        // Pure state ref like ".city" → evaluate directly
        if s.starts_with('.') && !s.contains(' ') && !s.contains('$') {
            let var_name = &s[1..];
            if let Some(val) = self.state.get(var_name) {
                return Self::format_eval_value(val);
            }
            return s.to_string();
        }

        if !s.contains("${") {
            return s.to_string();
        }
        let mut pos = 0;
        let mut result = String::new();
        let bytes = s.as_bytes();
        while pos < bytes.len() {
            if bytes[pos] == b'$' && pos + 1 < bytes.len() && bytes[pos + 1] == b'{' {
                pos += 2;
                let mut var_buf = String::new();
                while pos < bytes.len() && bytes[pos] != b'}' {
                    var_buf.push(bytes[pos] as char);
                    pos += 1;
                }
                if pos < bytes.len() {
                    pos += 1; // skip '}'
                }
                let var_name = var_buf.trim().trim_start_matches('.');
                let val = self
                    .state
                    .get(var_name)
                    .map(|v| Self::format_eval_value(v))
                    .unwrap_or_else(|| format!("${{{}}}", var_buf));
                result.push_str(&val);
            } else {
                result.push(bytes[pos] as char);
                pos += 1;
            }
        }
        result
    }

    fn count_display_props(props: &HashMap<String, AuraPropValue>) -> usize {
        props
            .iter()
            .filter(|(k, _)| *k != "label" && *k != "id" && *k != "text")
            .count()
    }

    /// Extract box layout inline string from style/class props (Plan 281).
    fn extract_layout_inline(&self, props: &HashMap<String, AuraPropValue>) -> Option<String> {
        // Get style string from "class" or "style" prop
        let style_str = props.get("class")
            .or_else(|| props.get("style"))
            .and_then(|pv| match pv {
                AuraPropValue::Expr(Expr::Str(s)) => Some(s.to_string()),
                _ => None,
            })?;

        let layout = BoxLayout::from_class_string(&style_str);
        layout.format_inline(self.viewport)
    }

    fn format_prop_val(val: &str) -> String {
        // If it looks like a plain identifier (state ref), keep as-is
        // Otherwise wrap in quotes
        if val.starts_with('.') || val == "true" || val == "false" || val.parse::<i64>().is_ok() {
            val.to_string()
        } else {
            format!("\"{}\"", Self::escape_str(val))
        }
    }

    fn escape_str(s: &str) -> String {
        s.replace('\\', "\\\\").replace('"', "\\\"")
    }

    fn resolve_state_ref<'b>(
        state: &'b HashMap<String, auto_val::Value>,
        name: &str,
    ) -> Option<&'b auto_val::Value> {
        state.get(name)
    }
}
