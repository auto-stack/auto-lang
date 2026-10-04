//! Versioned core descriptor (`core-i32-draft`) and the Schema-bound binder.
//!
//! The descriptor is declared in `schema/core-i32.atom` (parsed by this same
//! crate's Atom reader). It lists the allowed node tags, their primary slots,
//! named fields with types, containers, statements and per-branch
//! required/forbidden field sets. The binder then walks an Atom source:
//!
//! 1. structural validation: unknown tags/fields, duplicate fields across
//!    head/args/body, enum cases, literal ranges, branch contracts;
//! 2. reference resolution: every textual reference is bound to a typed,
//!    category-correct ID scoped to its owning table (module types/defs/
//!    bodies, body locals/places/exprs/blocks/loops). Dangling references and
//!    category mismatches are reported with source spans.
//!
//! The output is a `hir::Bundle` of unchecked modules. Nothing here performs
//! semantic (type/dataflow) validation — that is `crate::verify`.
//!
//! Primary slots may equivalently be written as named fields (`module m_add`
//! vs `module(id: "m_add")`, `expr e_a read_local` vs
//! `expr(id: "e_a", kind: read_local)`); presenting both forms is rejected.

use crate::atom_text::{Diagnostic, IntLit, NamedValue, Span, Stage, SyntaxNode, Value};
use crate::hir;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Descriptor model
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Descriptor {
    pub schema: String,
    pub revision: u32,
    pub profile: String,
    enums: BTreeMap<String, Vec<String>>,
    nodes: BTreeMap<String, NodeSpec>,
    branches: BTreeMap<String, BranchSpec>,
}

#[derive(Clone, Debug, Default)]
struct NodeSpec {
    slots: Vec<SlotKind>,
    fields: BTreeMap<String, FieldType>,
    containers: BTreeMap<String, Vec<String>>,
    inline: Vec<String>,
    statements: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
enum SlotKind {
    Id,
    TypeUse,
    TypeRef,
    Enum(String),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum FieldType {
    Text,
    U32,
    I32,
    Bool,
    TypeUse,
    TypeUseList,
    TextList,
    ExprRefList,
    BindingList,
    TypeRef,
    DefRef,
    BodyRef,
    LocalRef,
    ExprRef,
    PlaceRef,
    BlockRef,
    LoopRef,
    LoopDecl,
    Enum,
}

#[derive(Clone, Debug, Default)]
struct BranchSpec {
    require: Vec<String>,
    forbid: Vec<String>,
}

fn builtin_field_type(word: &str) -> Option<FieldType> {
    Some(match word {
        "Text" => FieldType::Text,
        "U32" => FieldType::U32,
        "I32" => FieldType::I32,
        "Bool" => FieldType::Bool,
        "TypeUse" => FieldType::TypeUse,
        "TypeUseList" => FieldType::TypeUseList,
        "TextList" => FieldType::TextList,
        "ExprRefList" => FieldType::ExprRefList,
        "BindingList" => FieldType::BindingList,
        "TypeRef" => FieldType::TypeRef,
        "DefRef" => FieldType::DefRef,
        "BodyRef" => FieldType::BodyRef,
        "LocalRef" => FieldType::LocalRef,
        "ExprRef" => FieldType::ExprRef,
        "PlaceRef" => FieldType::PlaceRef,
        "BlockRef" => FieldType::BlockRef,
        "LoopRef" => FieldType::LoopRef,
        "LoopDecl" => FieldType::LoopDecl,
        _ => return None,
    })
}

fn builtin_slot_kind(word: &str) -> Option<SlotKind> {
    Some(match word {
        "Id" => SlotKind::Id,
        "TypeUse" => SlotKind::TypeUse,
        "TypeRef" => SlotKind::TypeRef,
        _ => return None,
    })
}

impl Descriptor {
    pub fn parse(src: &str) -> Result<Descriptor, Diagnostic> {
        let root = crate::atom_text::parse(src)?;
        if root.tag != "descriptor" {
            return Err(Diagnostic::new(
                Stage::Bind,
                "descriptor.invalid-root",
                format!("expected `descriptor` root, found `{}`", root.tag),
                root.tag_span,
            ));
        }
        let mut schema = None;
        let mut revision = None;
        let mut profile = None;
        for f in root.all_fields() {
            match f.name.as_str() {
                "schema" => schema = str_value(&f.value),
                "revision" => revision = int_value(&f.value).and_then(|l| l.to_u32()),
                "profile" => profile = str_value(&f.value),
                other => {
                    return Err(Diagnostic::new(
                        Stage::Bind,
                        "descriptor.unknown-field",
                        format!("unknown descriptor field `{}`", other),
                        f.name_span,
                    ))
                }
            }
        }
        let (Some(schema), Some(revision), Some(profile)) = (schema, revision, profile) else {
            return Err(Diagnostic::new(
                Stage::Bind,
                "descriptor.missing-field",
                "descriptor requires schema, revision and profile",
                root.tag_span,
            ));
        };

        let mut desc = Descriptor {
            schema,
            revision,
            profile,
            enums: BTreeMap::new(),
            nodes: BTreeMap::new(),
            branches: BTreeMap::new(),
        };

        for child in root.child_nodes() {
            match child.tag.as_str() {
                "enums" => {
                    for en in child.child_nodes() {
                        if en.tag != "enum" {
                            return Err(bad_descriptor(en));
                        }
                        let name = match en.primary.first() {
                            Some((Value::Ident(s), _)) => s.clone(),
                            _ => return Err(bad_descriptor(en)),
                        };
                        let mut cases = Vec::new();
                        for f in en.all_fields() {
                            if f.name == "cases" {
                                for s in ident_list(&f.value) {
                                    cases.push(s);
                                }
                            }
                        }
                        desc.enums.insert(name, cases);
                    }
                }
                "nodes" => {
                    for n in child.child_nodes() {
                        if n.tag != "node" {
                            return Err(bad_descriptor(n));
                        }
                        let name = match n.primary.first() {
                            Some((Value::Ident(s), _)) => s.clone(),
                            _ => return Err(bad_descriptor(n)),
                        };
                        let spec = parse_node_spec(n)?;
                        desc.nodes.insert(name, spec);
                    }
                }
                "branches" => {
                    for b in child.child_nodes() {
                        if b.tag != "branch" {
                            return Err(bad_descriptor(b));
                        }
                        let mut key = None;
                        let mut spec = BranchSpec::default();
                        for f in b.all_fields() {
                            match f.name.as_str() {
                                "key" => key = str_value(&f.value),
                                "require" => spec.require = ident_list(&f.value),
                                "forbid" => spec.forbid = ident_list(&f.value),
                                _other => return Err(bad_descriptor(b)),
                            }
                        }
                        match key {
                            Some(k) => {
                                desc.branches.insert(k, spec);
                            }
                            None => return Err(bad_descriptor(b)),
                        }
                    }
                }
                other => {
                    return Err(Diagnostic::new(
                        Stage::Bind,
                        "descriptor.unknown-section",
                        format!("unknown descriptor section `{}`", other),
                        child.tag_span,
                    ))
                }
            }
        }
        Ok(desc)
    }

    fn enum_cases(&self, name: &str) -> Option<&Vec<String>> {
        self.enums.get(name)
    }

    fn has_branches_for(&self, tag: &str) -> bool {
        let prefix = format!("{}.", tag);
        self.branches.contains_key(tag) || self.branches.keys().any(|k| k.starts_with(&prefix))
    }
}

fn bad_descriptor(n: &SyntaxNode) -> Diagnostic {
    Diagnostic::new(
        Stage::Bind,
        "descriptor.malformed",
        format!("malformed descriptor entry at `{}`", n.tag),
        n.tag_span,
    )
}

fn parse_node_spec(n: &SyntaxNode) -> Result<NodeSpec, Diagnostic> {
    let mut spec = NodeSpec::default();
    for f in n.all_fields() {
        match f.name.as_str() {
            "slots" => {
                if let Value::List(items) = &f.value {
                    for it in items {
                        if let Value::Ident(word) = it {
                            if let Some(k) = builtin_slot_kind(word) {
                                spec.slots.push(k);
                            } else {
                                spec.slots.push(SlotKind::Enum(word.clone()));
                            }
                        }
                    }
                }
            }
            "statements" => spec.statements = ident_list(&f.value),
            "inline" => spec.inline = ident_list(&f.value),
            _ => return Err(bad_descriptor(n)),
        }
    }
    for child in n.child_nodes() {
        match child.tag.as_str() {
            "fields" => {
                for f in child.all_fields() {
                    let word = match &f.value {
                        Value::Ident(s) => s.as_str(),
                        _ => return Err(bad_descriptor(child)),
                    };
                    let ft = builtin_field_type(word).unwrap_or(FieldType::Enum);
                    spec.fields.insert(f.name.clone(), ft);
                }
            }
            "containers" => {
                for f in child.all_fields() {
                    spec.containers.insert(f.name.clone(), ident_list(&f.value));
                }
            }
            _ => return Err(bad_descriptor(child)),
        }
    }
    Ok(spec)
}

fn ident_list(v: &Value) -> Vec<String> {
    match v {
        Value::List(items) => items
            .iter()
            .filter_map(|it| match it {
                Value::Ident(s) => Some(s.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Binder
// ---------------------------------------------------------------------------

/// Bind an Atom source against the descriptor. Collects all binder-level
/// diagnostics; returns them if any occurred.
pub fn bind(desc: &Descriptor, source: &str) -> Result<hir::Bundle, Vec<Diagnostic>> {
    let mut b = Binder::new(desc, source);
    match crate::atom_text::parse(source) {
        Ok(root) => b.bind_root(&root),
        Err(d) => b.diags.push(d),
    }
    if b.diags.is_empty() {
        Ok(hir::Bundle {
            requires: b.out_requires,
            modules: b.out_modules,
        })
    } else {
        Err(b.diags)
    }
}

/// A field set merged from head + body with duplicate detection.
struct FieldSet {
    by_name: BTreeMap<String, NamedValue>,
}

impl FieldSet {
    fn build(node: &SyntaxNode, b: &mut Binder) -> FieldSet {
        let mut by_name: BTreeMap<String, NamedValue> = BTreeMap::new();
        for f in node.all_fields() {
            if let Some(prev) = by_name.get(&f.name) {
                b.error(
                    "bind.duplicate-field",
                    format!(
                        "field `{}` declared more than once (previous at byte {})",
                        f.name, prev.name_span.start
                    ),
                    f.name_span,
                );
            } else {
                by_name.insert(f.name.clone(), f.clone());
            }
        }
        FieldSet { by_name }
    }

    fn get(&self, name: &str) -> Option<&NamedValue> {
        self.by_name.get(name)
    }

    fn names(&self) -> impl Iterator<Item = &String> {
        self.by_name.keys()
    }
}

/// ID tables of one function body, keyed by id text. Registration order is
/// the value order (`LocalId(i)` etc. index the hir vectors).
#[derive(Default)]
struct BodyTables {
    locals: BTreeMap<String, u32>,
    places: BTreeMap<String, u32>,
    exprs: BTreeMap<String, u32>,
    blocks: BTreeMap<String, u32>,
    loops: BTreeMap<String, u32>,
}

impl BodyTables {
    fn iter(&self) -> [(&'static str, &BTreeMap<String, u32>); 5] {
        [
            ("local", &self.locals),
            ("place", &self.places),
            ("expression", &self.exprs),
            ("block", &self.blocks),
            ("loop", &self.loops),
        ]
    }
}

/// ID tables of one module.
#[derive(Default)]
struct ModuleTables {
    types: BTreeMap<String, u32>,
    defs: BTreeMap<String, u32>,
    bodies: BTreeMap<String, u32>,
}

struct Binder<'d> {
    desc: &'d Descriptor,
    src: &'d str,
    diags: Vec<Diagnostic>,
    out_modules: Vec<hir::Module>,
    out_requires: Vec<String>,
}

impl<'d> Binder<'d> {
    fn new(desc: &'d Descriptor, src: &'d str) -> Self {
        Binder {
            desc,
            src,
            diags: Vec::new(),
            out_modules: Vec::new(),
            out_requires: Vec::new(),
        }
    }

    fn error(&mut self, code: &str, msg: impl Into<String>, span: Span) {
        self.diags
            .push(Diagnostic::new(Stage::Bind, code, msg, span));
    }

    fn text_of(&self, v: &Value) -> Option<String> {
        match v {
            Value::Ident(s) => Some(s.clone()),
            Value::Str(s) => Some(s.clone()),
            _ => None,
        }
    }

    // -- slot resolution ----------------------------------------------------

    /// Resolve a primary slot that may equivalently be written as a named
    /// field (`module m_add` vs `module(id: "m_add")`). Errors when both
    /// forms are present or neither is.
    fn slot_value(
        &mut self,
        node: &SyntaxNode,
        fields: &FieldSet,
        idx: usize,
        field_name: &str,
        what: &str,
    ) -> Option<(Value, Span)> {
        let in_primary = node.primary.get(idx).map(|(v, s)| (v.clone(), *s));
        let in_field = fields
            .get(field_name)
            .map(|f| (f.value.clone(), f.value_span));
        match (in_primary, in_field) {
            (Some(_), Some(f)) => {
                self.error(
                    "bind.duplicate-field",
                    format!(
                        "slot given both positionally and as field `{}` on {}",
                        field_name, what
                    ),
                    f.1,
                );
                None
            }
            (Some(p), None) => Some(p),
            (None, Some(f)) => Some(f),
            (None, None) => {
                self.error(
                    "bind.slot-arity",
                    format!(
                        "{} expects {} value (positional slot or field `{}`)",
                        what,
                        idx + 1,
                        field_name
                    ),
                    node.tag_span,
                );
                None
            }
        }
    }

    fn slot_text(
        &mut self,
        node: &SyntaxNode,
        fields: &FieldSet,
        idx: usize,
        field_name: &str,
        what: &str,
    ) -> Option<(String, Span)> {
        let (v, span) = self.slot_value(node, fields, idx, field_name, what)?;
        match self.text_of(&v) {
            Some(t) if !t.is_empty() => Some((t, span)),
            _ => {
                self.error(
                    "bind.slot-type",
                    format!("{} slot `{}` must be text", what, field_name),
                    span,
                );
                None
            }
        }
    }

    /// Registration-pass id peek: positional slot or explicit `id:` field,
    /// without diagnostics (pass B performs full slot validation).
    fn peek_id(node: &SyntaxNode) -> Option<String> {
        if let Some((v, _)) = node.primary.first() {
            if let Some(t) = Binder::static_text_of(v) {
                return Some(t);
            }
        }
        node.all_fields()
            .iter()
            .find(|f| f.name == "id")
            .and_then(|f| Binder::static_text_of(&f.value))
    }

    fn static_text_of(v: &Value) -> Option<String> {
        match v {
            Value::Ident(s) => Some(s.clone()),
            Value::Str(s) => Some(s.clone()),
            _ => None,
        }
    }

    fn check_enum_case(&mut self, enum_name: &str, word: &str, span: Span) -> bool {
        match self.desc.enum_cases(enum_name) {
            Some(cases) if cases.iter().any(|c| c == word) => true,
            Some(cases) => {
                self.error(
                    "bind.unknown-case",
                    format!(
                        "`{}` is not a case of enum {} (cases: {})",
                        word,
                        enum_name,
                        cases.join(", ")
                    ),
                    span,
                );
                false
            }
            None => {
                self.error(
                    "bind.unknown-enum",
                    format!("descriptor has no enum `{}`", enum_name),
                    span,
                );
                false
            }
        }
    }

    /// Compute the branch key for a node and enforce its require/forbid sets.
    fn branch_and_check(
        &mut self,
        tag: &str,
        primary: &[(Value, Span)],
        fields: &FieldSet,
        subject: &str,
        subject_span: Span,
    ) -> Option<String> {
        if !self.desc.has_branches_for(tag) {
            return None;
        }
        let disc1: Option<String> = if tag == "local" {
            fields.get("role").and_then(|f| self.text_of(&f.value))
        } else {
            primary
                .get(1)
                .and_then(|(v, _)| self.text_of(v))
                .or_else(|| fields.get("kind").and_then(|f| self.text_of(&f.value)))
        };
        let disc2: Option<String> = if tag == "expr" && disc1.as_deref() == Some("binary") {
            fields.get("op").and_then(|f| self.text_of(&f.value))
        } else {
            None
        };

        let mut candidates: Vec<String> = Vec::new();
        if let (Some(d1), Some(d2)) = (&disc1, &disc2) {
            candidates.push(format!("{}.{}:{}", tag, d1, d2));
        }
        if let Some(d1) = &disc1 {
            candidates.push(format!("{}.{}", tag, d1));
        }
        candidates.push(tag.to_string());

        for key in &candidates {
            if let Some(spec) = self.desc.branches.get(key).cloned() {
                for r in &spec.require {
                    if fields.get(r).is_none() {
                        self.error(
                            "bind.missing-field",
                            format!("{} requires field `{}`", subject, r),
                            subject_span,
                        );
                    }
                }
                for name in fields.names() {
                    if spec.forbid.contains(name) {
                        self.error(
                            "bind.branch-forbidden-field",
                            format!("field `{}` is not allowed on {} here", name, subject),
                            fields.get(name).unwrap().name_span,
                        );
                    }
                }
                return Some(key.clone());
            }
        }
        let shown = match (&disc1, &disc2) {
            (Some(d1), Some(d2)) => format!("{}.{}:{}", tag, d1, d2),
            (Some(d1), None) => format!("{}.{}", tag, d1),
            (None, _) => tag.to_string(),
        };
        let hint = if disc1.is_none() {
            " (discriminator field is missing)"
        } else {
            ""
        };
        self.error(
            "bind.unknown-case",
            format!("no accepted case `{}` in descriptor{}", shown, hint),
            primary.get(1).map(|(_, s)| *s).unwrap_or(subject_span),
        );
        None
    }

    fn check_fields_against_spec(&mut self, tag: &str, fields: &FieldSet) {
        let (spec_fields, slot_aliases) = match self.desc.nodes.get(tag) {
            Some(s) => {
                let mut aliases: Vec<&'static str> = Vec::new();
                for (i, k) in s.slots.iter().enumerate() {
                    match k {
                        SlotKind::Id => {
                            if i == 0 {
                                aliases.push("id");
                            }
                        }
                        SlotKind::TypeUse | SlotKind::TypeRef => aliases.push("type"),
                        SlotKind::Enum(_) => aliases.push("kind"),
                    }
                }
                (s.fields.clone(), aliases)
            }
            None => (BTreeMap::new(), Vec::new()),
        };
        for name in fields.names() {
            let is_slot_field = slot_aliases.contains(&name.as_str());
            if !spec_fields.contains_key(name) && !is_slot_field {
                self.error(
                    "bind.unknown-field",
                    format!("unknown field `{}` on {}", name, tag),
                    fields.get(name).unwrap().name_span,
                );
            }
        }
    }

    fn text_ref(&mut self, fields: &FieldSet, name: &str) -> Option<(String, Span)> {
        let f = fields.get(name)?;
        match self.text_of(&f.value) {
            Some(t) => Some((t, f.value_span)),
            None => {
                self.error(
                    "bind.type-shape",
                    format!("field `{}` must be text", name),
                    f.value_span,
                );
                None
            }
        }
    }

    /// Resolve a reference against its primary table; on miss, probe the
    /// sibling tables of the same scope to distinguish a category mismatch
    /// from a dangling reference.
    fn resolve_in(
        &mut self,
        text: &str,
        span: Span,
        category: &str,
        primary: &BTreeMap<String, u32>,
        siblings: &[(&str, &BTreeMap<String, u32>)],
    ) -> Option<u32> {
        if let Some(id) = primary.get(text) {
            return Some(*id);
        }
        for (label, table) in siblings {
            if table.contains_key(text) {
                self.error(
                    "bind.category-mismatch",
                    format!(
                        "`{}` is a {} id, not a {} (category mismatch)",
                        text, label, category
                    ),
                    span,
                );
                return None;
            }
        }
        self.error(
            "bind.dangling-ref",
            format!("dangling {} reference `{}`", category, text),
            span,
        );
        None
    }

    // -- root ---------------------------------------------------------------

    fn bind_root(&mut self, root: &SyntaxNode) {
        if root.tag != "hir" {
            self.error(
                "bind.unknown-root",
                format!("expected root node `hir`, found `{}`", root.tag),
                root.tag_span,
            );
            return;
        }
        let fields = FieldSet::build(root, self);
        self.branch_and_check("hir", &root.primary, &fields, "hir root", root.tag_span);
        self.check_fields_against_spec("hir", &fields);

        // Descriptor identity: only the published core-i32-draft triple.
        if let Some(f) = fields.get("schema") {
            match self.text_of(&f.value) {
                Some(s) if s == self.desc.schema => {}
                Some(s) => self.error(
                    "bind.schema-mismatch",
                    format!(
                        "schema `{}` does not match descriptor `{}`",
                        s, self.desc.schema
                    ),
                    f.value_span,
                ),
                None => self.error("bind.type-shape", "schema must be a string", f.value_span),
            }
        }
        if let Some(f) = fields.get("revision") {
            match int_value(&f.value) {
                Some(lit) => match lit.to_u32() {
                    Some(r) if matches!(lit.suffix.as_deref(), None | Some("u32")) => {
                        if r != self.desc.revision {
                            self.error(
                                "bind.revision-mismatch",
                                format!(
                                    "revision {} does not match descriptor revision {}",
                                    r, self.desc.revision
                                ),
                                f.value_span,
                            );
                        }
                    }
                    _ => self.error(
                        "bind.literal-suffix",
                        "revision must be a u32 literal",
                        f.value_span,
                    ),
                },
                None => self.error(
                    "bind.type-shape",
                    "revision must be an integer",
                    f.value_span,
                ),
            }
        }
        if let Some(f) = fields.get("profile") {
            match self.text_of(&f.value) {
                Some(s) if s == self.desc.profile => {}
                Some(s) => self.error(
                    "bind.profile-mismatch",
                    format!(
                        "profile `{}` is not accepted by this build (accepts only `{}`)",
                        s, self.desc.profile
                    ),
                    f.value_span,
                ),
                None => self.error("bind.type-shape", "profile must be a string", f.value_span),
            }
        }
        if let Some(f) = fields.get("requires") {
            match &f.value {
                Value::List(items) => {
                    for it in items {
                        match self.text_of(it) {
                            Some(s) => self.out_requires.push(s),
                            None => self.error(
                                "bind.type-shape",
                                "requires entries must be strings",
                                f.value_span,
                            ),
                        }
                    }
                }
                _ => self.error(
                    "bind.type-shape",
                    "requires must be a list of strings",
                    f.value_span,
                ),
            }
        }

        for child in root.child_nodes() {
            if child.tag == "module" {
                self.bind_module(child);
            } else {
                self.error(
                    "bind.unknown-node",
                    format!("`{}` is not allowed under hir", child.tag),
                    child.tag_span,
                );
            }
        }
    }

    // -- module -------------------------------------------------------------

    fn bind_module(&mut self, node: &SyntaxNode) {
        let spec = self.desc.nodes.get("module").cloned().unwrap_or_default();
        let fields = FieldSet::build(node, self);
        self.check_fields_against_spec("module", &fields);
        let Some((id_text, _)) = self.slot_text(node, &fields, 0, "id", "module") else {
            return;
        };
        let mut path = None;
        if let Some(f) = fields.get("path") {
            match self.text_of(&f.value) {
                Some(s) => path = Some(s),
                None => self.error("bind.type-shape", "path must be a string", f.value_span),
            }
        }

        // Containers: at most one of each; child tags validated.
        let mut containers: BTreeMap<String, Vec<&SyntaxNode>> = BTreeMap::new();
        let mut seen: BTreeMap<&str, ()> = BTreeMap::new();
        for child in node.child_nodes() {
            if let Some(allowed) = spec.containers.get(child.tag.as_str()) {
                if seen.insert(child.tag.as_str(), ()).is_some() {
                    self.error(
                        "bind.duplicate-container",
                        format!("duplicate `{}` container", child.tag),
                        child.tag_span,
                    );
                    continue;
                }
                for c in child.child_nodes() {
                    if allowed.contains(&c.tag) {
                        containers.entry(child.tag.clone()).or_default().push(c);
                    } else {
                        self.error(
                            "bind.unknown-node",
                            format!("`{}` is not allowed in {}", c.tag, child.tag),
                            c.tag_span,
                        );
                    }
                }
            } else {
                self.error(
                    "bind.unknown-node",
                    format!("`{}` is not allowed in module", child.tag),
                    child.tag_span,
                );
            }
        }

        let type_nodes: Vec<&SyntaxNode> = containers.get("types").cloned().unwrap_or_default();
        let def_nodes: Vec<&SyntaxNode> =
            containers.get("declarations").cloned().unwrap_or_default();
        let body_nodes: Vec<&SyntaxNode> = containers.get("bodies").cloned().unwrap_or_default();

        // Pass A: register table identities.
        let mut tables = ModuleTables::default();
        let mut types: Vec<hir::TypeDef> = Vec::new();
        let mut typed_nodes: Vec<(usize, &SyntaxNode)> = Vec::new();
        for t in &type_nodes {
            let Some(id) = Binder::peek_id(t) else {
                continue;
            };
            if tables.types.contains_key(&id) {
                self.error(
                    "bind.duplicate-id",
                    format!("duplicate type id `{}`", id),
                    t.tag_span,
                );
                continue;
            }
            tables.types.insert(id.clone(), types.len() as u32);
            types.push(hir::TypeDef {
                id_text: id,
                kind: hir::TypeKind::I32, // placeholder until pass B
                span: t.span,
            });
            typed_nodes.push((types.len() - 1, t));
        }

        let mut defs: Vec<hir::Def> = Vec::new();
        let mut def_entries: Vec<(usize, &SyntaxNode)> = Vec::new();
        for d in &def_nodes {
            let Some(id) = Binder::peek_id(d) else {
                continue;
            };
            if tables.defs.contains_key(&id) {
                self.error(
                    "bind.duplicate-id",
                    format!("duplicate declaration id `{}`", id),
                    d.tag_span,
                );
                continue;
            }
            tables.defs.insert(id.clone(), defs.len() as u32);
            defs.push(hir::Def {
                id_text: id,
                kind: hir::DefKind::Function {
                    name: String::new(),
                    ty: hir::TypeId(0),
                    body: hir::BodyId(0),
                }, // placeholder until pass B
                span: d.span,
            });
            def_entries.push((defs.len() - 1, d));
        }

        let mut body_entries: Vec<(usize, &SyntaxNode)> = Vec::new();
        for bnode in &body_nodes {
            let Some(id) = Binder::peek_id(bnode) else {
                continue;
            };
            if tables.bodies.contains_key(&id) {
                self.error(
                    "bind.duplicate-id",
                    format!("duplicate body id `{}`", id),
                    bnode.tag_span,
                );
                continue;
            }
            tables.bodies.insert(id, body_entries.len() as u32);
            body_entries.push((body_entries.len(), bnode));
        }

        // Pass B: resolve type kinds.
        for (ti, t) in &typed_nodes {
            let tfields = FieldSet::build(t, self);
            self.branch_and_check(
                "type",
                &t.primary,
                &tfields,
                &format!("type `{}`", types[*ti].id_text),
                t.tag_span,
            );
            self.check_fields_against_spec("type", &tfields);
            let Some((word, kind_span)) = self.slot_text(t, &tfields, 1, "kind", "type") else {
                continue;
            };
            let kind = match word.as_str() {
                "i32" => hir::TypeKind::I32,
                "bool" => hir::TypeKind::Bool,
                "function" => {
                    let mut params = Vec::new();
                    let mut ok = true;
                    if let Some(f) = tfields.get("params") {
                        match &f.value {
                            Value::List(items) => {
                                for it in items {
                                    match self.resolve_typeuse(it, f.value_span, &tables.types) {
                                        Some(u) => params.push(u),
                                        None => ok = false,
                                    }
                                }
                            }
                            _ => {
                                self.error(
                                    "bind.type-shape",
                                    "params must be a list",
                                    f.value_span,
                                );
                                ok = false;
                            }
                        }
                    }
                    let mut result = hir::TypeUse::Builtin(hir::BuiltinType::I32);
                    if let Some(f) = tfields.get("result") {
                        match self.resolve_typeuse(&f.value, f.value_span, &tables.types) {
                            Some(u) => result = u,
                            None => ok = false,
                        }
                    }
                    if !ok {
                        continue;
                    }
                    hir::TypeKind::Function(hir::FnSig { params, result })
                }
                _ => {
                    self.check_enum_case("TypeKind", &word, kind_span);
                    continue;
                }
            };
            types[*ti].kind = kind;
        }

        let mut bodies: Vec<Option<hir::Body>> = vec![None; body_entries.len()];
        for (bi, bnode) in &body_entries {
            let body = self.bind_body(bnode, &tables);
            bodies[*bi] = Some(body);
        }
        let bodies: Vec<hir::Body> = bodies
            .into_iter()
            .enumerate()
            .map(|(i, b)| {
                b.unwrap_or_else(|| hir::Body {
                    id_text: format!("<invalid body {}>", i),
                    owner: hir::DefId(0),
                    entry: hir::BlockId(0),
                    locals: vec![],
                    places: vec![],
                    exprs: vec![],
                    blocks: vec![],
                    loops: vec![],
                    span: Span { start: 0, end: 0 },
                })
            })
            .collect();

        for (di, d) in &def_entries {
            let dfields = FieldSet::build(d, self);
            self.branch_and_check(
                d.tag.as_str(),
                &d.primary,
                &dfields,
                &format!("declaration `{}`", defs[*di].id_text),
                d.tag_span,
            );
            self.check_fields_against_spec(d.tag.as_str(), &dfields);
            let Some((ty_text, ty_span)) = self.slot_text(d, &dfields, 1, "type", d.tag.as_str())
            else {
                continue;
            };
            let ty = match tables.types.get(&ty_text) {
                Some(tid) => hir::TypeId(*tid),
                None => {
                    self.error(
                        "bind.dangling-type-ref",
                        format!("dangling type reference `{}`", ty_text),
                        ty_span,
                    );
                    continue;
                }
            };
            let kind = match d.tag.as_str() {
                "function" => {
                    let name = dfields
                        .get("name")
                        .and_then(|f| self.text_of(&f.value))
                        .unwrap_or_default();
                    let Some((t, span)) = self.text_ref(&dfields, "body") else {
                        continue;
                    };
                    let body = match self.resolve_in(
                        &t,
                        span,
                        "BodyRef",
                        &tables.bodies,
                        &[("type", &tables.types), ("declaration", &tables.defs)],
                    ) {
                        Some(b) => hir::BodyId(b),
                        None => continue,
                    };
                    hir::DefKind::Function { name, ty, body }
                }
                "intrinsic" => {
                    let Some((symbol, _)) = self.text_ref(&dfields, "symbol") else {
                        continue;
                    };
                    let Some((word, esp)) = self.text_ref(&dfields, "effect") else {
                        continue;
                    };
                    if !self.check_enum_case("EffectKind", &word, esp) {
                        continue;
                    }
                    let effect = match word.as_str() {
                        "observable" => hir::Effect::Observable,
                        "pure" => hir::Effect::Pure,
                        _ => continue,
                    };
                    hir::DefKind::Intrinsic { symbol, effect, ty }
                }
                other => {
                    self.error(
                        "bind.unknown-node",
                        format!("`{}` is not a declaration", other),
                        d.tag_span,
                    );
                    continue;
                }
            };
            defs[*di].kind = kind;
        }

        self.out_modules.push(hir::Module {
            id_text,
            path,
            types,
            defs,
            bodies,
            span: node.span,
        });
    }

    // -- body ---------------------------------------------------------------

    fn bind_body(&mut self, node: &SyntaxNode, module: &ModuleTables) -> hir::Body {
        let fields = FieldSet::build(node, self);
        let id_text = self
            .slot_text(node, &fields, 0, "id", "body")
            .map(|(t, _)| t)
            .unwrap_or_default();
        self.branch_and_check(
            "body",
            &node.primary,
            &fields,
            &format!("body `{}`", id_text),
            node.tag_span,
        );
        self.check_fields_against_spec("body", &fields);
        let owner = match self.text_ref(&fields, "owner") {
            Some((t, span)) => match self.resolve_in(
                &t,
                span,
                "DefRef",
                &module.defs,
                &[("type", &module.types), ("body", &module.bodies)],
            ) {
                Some(d) => hir::DefId(d),
                None => hir::DefId(0),
            },
            None => hir::DefId(0),
        };
        let entry_field = self.text_ref(&fields, "entry");

        // Containers.
        let spec = self.desc.nodes.get("body").cloned().unwrap_or_default();
        let mut containers: BTreeMap<String, Vec<&SyntaxNode>> = BTreeMap::new();
        let mut seen: BTreeMap<&str, ()> = BTreeMap::new();
        for child in node.child_nodes() {
            if let Some(allowed) = spec.containers.get(child.tag.as_str()) {
                if seen.insert(child.tag.as_str(), ()).is_some() {
                    self.error(
                        "bind.duplicate-container",
                        format!("duplicate `{}` container", child.tag),
                        child.tag_span,
                    );
                    continue;
                }
                for c in child.child_nodes() {
                    if allowed.contains(&c.tag) {
                        containers.entry(child.tag.clone()).or_default().push(c);
                    } else {
                        self.error(
                            "bind.unknown-node",
                            format!("`{}` is not allowed in {}", c.tag, child.tag),
                            c.tag_span,
                        );
                    }
                }
            } else {
                self.error(
                    "bind.unknown-node",
                    format!("`{}` is not allowed in body", child.tag),
                    child.tag_span,
                );
            }
        }
        let local_nodes = containers.get("locals").cloned().unwrap_or_default();
        let place_nodes = containers.get("places").cloned().unwrap_or_default();
        let expr_nodes = containers.get("expressions").cloned().unwrap_or_default();
        let block_nodes = containers.get("blocks").cloned().unwrap_or_default();

        // Pass A: register identities.
        let mut tables = BodyTables::default();
        for l in &local_nodes {
            let Some(id) = Binder::peek_id(l) else {
                continue;
            };
            if tables.locals.contains_key(&id) {
                self.error(
                    "bind.duplicate-id",
                    format!("duplicate local id `{}`", id),
                    l.tag_span,
                );
                continue;
            }
            tables.locals.insert(id, tables.locals.len() as u32);
        }
        for p in &place_nodes {
            let Some(id) = Binder::peek_id(p) else {
                continue;
            };
            if tables.places.contains_key(&id) {
                self.error(
                    "bind.duplicate-id",
                    format!("duplicate place id `{}`", id),
                    p.tag_span,
                );
                continue;
            }
            tables.places.insert(id, tables.places.len() as u32);
        }
        for e in &expr_nodes {
            let Some(id) = Binder::peek_id(e) else {
                continue;
            };
            if tables.exprs.contains_key(&id) {
                self.error(
                    "bind.duplicate-id",
                    format!("duplicate expression id `{}`", id),
                    e.tag_span,
                );
                continue;
            }
            tables.exprs.insert(id, tables.exprs.len() as u32);
        }
        for bl in &block_nodes {
            let Some(id) = Binder::peek_id(bl) else {
                continue;
            };
            if tables.blocks.contains_key(&id) {
                self.error(
                    "bind.duplicate-id",
                    format!("duplicate block id `{}`", id),
                    bl.tag_span,
                );
                continue;
            }
            tables.blocks.insert(id, tables.blocks.len() as u32);
        }
        // Loop ids: scan all blocks' `loop` statements.
        for bl in &block_nodes {
            for stmt in bl.child_nodes() {
                if stmt.tag == "loop" {
                    let sfields = FieldSet::build(stmt, self);
                    if let Some(f) = sfields.get("id") {
                        if let Some(t) = self.text_of(&f.value) {
                            if tables.loops.contains_key(&t) {
                                self.error(
                                    "bind.duplicate-id",
                                    format!("duplicate loop id `{}`", t),
                                    f.value_span,
                                );
                                continue;
                            }
                            tables.loops.insert(t, tables.loops.len() as u32);
                        }
                    }
                }
            }
        }

        // Sibling views for category-mismatch probing.
        let expr_sibs: Vec<(&str, &BTreeMap<String, u32>)> = tables
            .iter()
            .into_iter()
            .filter(|(l, _)| *l != "expression")
            .collect();
        let block_sibs: Vec<(&str, &BTreeMap<String, u32>)> = tables
            .iter()
            .into_iter()
            .filter(|(l, _)| *l != "block")
            .collect();
        let local_sibs: Vec<(&str, &BTreeMap<String, u32>)> = tables
            .iter()
            .into_iter()
            .filter(|(l, _)| *l != "local")
            .collect();
        let place_sibs: Vec<(&str, &BTreeMap<String, u32>)> = tables
            .iter()
            .into_iter()
            .filter(|(l, _)| *l != "place")
            .collect();
        let loop_sibs: Vec<(&str, &BTreeMap<String, u32>)> = tables
            .iter()
            .into_iter()
            .filter(|(l, _)| *l != "loop")
            .collect();
        let module_sibs_for_def: Vec<(&str, &BTreeMap<String, u32>)> =
            vec![("type", &module.types), ("body", &module.bodies)];

        // Pass B: resolve contents.
        let mut locals = Vec::new();
        for l in &local_nodes {
            if let Some(local) = self.bind_local(l, &module.types) {
                locals.push(local);
            }
        }
        let mut places = Vec::new();
        for p in &place_nodes {
            if let Some(place) = self.bind_place(p, &module.types, &tables, &local_sibs) {
                places.push(place);
            }
        }
        let mut exprs = Vec::new();
        for e in &expr_nodes {
            if let Some(ex) = self.bind_expr(
                e,
                &module.types,
                &module.defs,
                &module_sibs_for_def,
                &tables,
                &expr_sibs,
            ) {
                exprs.push(ex);
            }
        }
        let mut blocks = Vec::new();
        for bl in &block_nodes {
            if let Some(b) = self.bind_block(bl, &tables, &block_sibs) {
                blocks.push(b);
            }
        }
        let entry = match entry_field {
            Some((t, span)) => {
                match self.resolve_in(&t, span, "BlockRef", &tables.blocks, &block_sibs) {
                    Some(b) => hir::BlockId(b),
                    None => hir::BlockId(0),
                }
            }
            None => hir::BlockId(0),
        };

        let loops: Vec<String> = {
            let mut v = vec![String::new(); tables.loops.len()];
            for (text, idx) in &tables.loops {
                v[*idx as usize] = text.clone();
            }
            v
        };
        let _ = (loop_sibs, place_sibs); // reserved for future ref categories

        hir::Body {
            id_text,
            owner,
            entry,
            locals,
            places,
            exprs,
            blocks,
            loops,
            span: node.span,
        }
    }

    fn bind_local(
        &mut self,
        node: &SyntaxNode,
        type_ids: &BTreeMap<String, u32>,
    ) -> Option<hir::LocalDef> {
        let fields = FieldSet::build(node, self);
        let (id_text, _) = self.slot_text(node, &fields, 0, "id", "local")?;
        self.branch_and_check(
            "local",
            &node.primary,
            &fields,
            &format!("local `{}`", id_text),
            node.tag_span,
        );
        self.check_fields_against_spec("local", &fields);
        let (ty_val, ty_span) = self.slot_value(node, &fields, 1, "type", "local")?;
        let ty = match self.resolve_typeuse(&ty_val, ty_span, type_ids) {
            Some(u) => u,
            None => return None,
        };
        let name = fields
            .get("name")
            .and_then(|f| self.text_of(&f.value))
            .unwrap_or_default();
        let Some((role_word, role_span)) = self.text_ref(&fields, "role") else {
            return None;
        };
        if !self.check_enum_case("LocalRole", &role_word, role_span) {
            return None;
        }
        let role = match role_word.as_str() {
            "param" => hir::LocalRole::Param,
            "binding" => hir::LocalRole::Binding,
            _ => return None,
        };
        let index = match fields.get("index") {
            Some(f) => match int_value(&f.value) {
                Some(lit) => match lit.to_u32() {
                    Some(v) if matches!(lit.suffix.as_deref(), None | Some("u32")) => Some(v),
                    _ => {
                        self.error(
                            "bind.literal-suffix",
                            "index must be a u32 literal",
                            f.value_span,
                        );
                        return None;
                    }
                },
                None => {
                    self.error("bind.type-shape", "index must be an integer", f.value_span);
                    return None;
                }
            },
            None => None,
        };
        let mutable_explicit = fields.get("mutable").is_some();
        let mutable = match fields.get("mutable") {
            Some(f) => match self.text_of(&f.value) {
                Some(w) if w == "true" => true,
                Some(w) if w == "false" => false,
                _ => {
                    self.error(
                        "bind.type-shape",
                        "mutable must be true or false",
                        f.value_span,
                    );
                    return None;
                }
            },
            None => false,
        };
        Some(hir::LocalDef {
            id_text,
            ty,
            name,
            role,
            index,
            mutable,
            mutable_explicit,
            span: node.span,
        })
    }

    fn bind_place(
        &mut self,
        node: &SyntaxNode,
        type_ids: &BTreeMap<String, u32>,
        body: &BodyTables,
        local_sibs: &[(&str, &BTreeMap<String, u32>)],
    ) -> Option<hir::Place> {
        let fields = FieldSet::build(node, self);
        let (id_text, _) = self.slot_text(node, &fields, 0, "id", "place")?;
        self.branch_and_check(
            "place",
            &node.primary,
            &fields,
            &format!("place `{}`", id_text),
            node.tag_span,
        );
        self.check_fields_against_spec("place", &fields);
        let (kind_word, kind_span) = self.slot_text(node, &fields, 1, "kind", "place")?;
        if !self.check_enum_case("PlaceKind", &kind_word, kind_span) {
            return None;
        }
        let kind = hir::PlaceKind::Local;
        let Some((lt, lspan)) = self.text_ref(&fields, "local") else {
            return None;
        };
        let local = match self.resolve_in(&lt, lspan, "LocalRef", &body.locals, local_sibs) {
            Some(l) => hir::LocalId(l),
            None => return None,
        };
        let ty = match fields.get("type") {
            Some(f) => match self.resolve_typeuse(&f.value, f.value_span, type_ids) {
                Some(u) => u,
                None => return None,
            },
            None => return None,
        };
        let Some((aword, aspan)) = self.text_ref(&fields, "access") else {
            return None;
        };
        if !self.check_enum_case("AccessKind", &aword, aspan) {
            return None;
        }
        let access = match aword.as_str() {
            "mutable" => hir::Access::Mutable,
            "readonly" => hir::Access::ReadOnly,
            _ => return None,
        };
        Some(hir::Place {
            id_text,
            kind,
            local,
            ty,
            access,
            span: node.span,
        })
    }

    fn bind_expr(
        &mut self,
        node: &SyntaxNode,
        type_ids: &BTreeMap<String, u32>,
        def_ids: &BTreeMap<String, u32>,
        def_sibs: &[(&str, &BTreeMap<String, u32>)],
        body: &BodyTables,
        expr_sibs: &[(&str, &BTreeMap<String, u32>)],
    ) -> Option<hir::Expr> {
        let fields = FieldSet::build(node, self);
        let (id_text, _) = self.slot_text(node, &fields, 0, "id", "expr")?;
        self.branch_and_check(
            "expr",
            &node.primary,
            &fields,
            &format!("expr `{}`", id_text),
            node.tag_span,
        );
        self.check_fields_against_spec("expr", &fields);
        let ty = match fields.get("type") {
            Some(f) => match self.resolve_typeuse(&f.value, f.value_span, type_ids) {
                Some(u) => u,
                None => return None,
            },
            None => return None,
        };
        let (kind_word, _kind_span) = self.slot_text(node, &fields, 1, "kind", "expr")?;
        let kind = match kind_word.as_str() {
            "read_local" => {
                let Some((t, span)) = self.text_ref(&fields, "local") else {
                    return None;
                };
                let local = match self.resolve_in(&t, span, "LocalRef", &body.locals, expr_sibs) {
                    Some(l) => hir::LocalId(l),
                    None => return None,
                };
                hir::ExprKind::ReadLocal { local }
            }
            "constant" => {
                let Some(f) = fields.get("value") else {
                    return None;
                };
                match int_value(&f.value) {
                    Some(lit) => match lit.to_i32() {
                        Some(v) if matches!(lit.suffix.as_deref(), None | Some("i32")) => {
                            hir::ExprKind::Constant { value: v }
                        }
                        _ => {
                            self.error(
                                "bind.literal-range",
                                format!(
                                    "value `{}` is not representable as i32",
                                    lit.span.slice(self.src)
                                ),
                                f.value_span,
                            );
                            return None;
                        }
                    },
                    None => {
                        self.error(
                            "bind.type-shape",
                            "constant value must be an integer",
                            f.value_span,
                        );
                        return None;
                    }
                }
            }
            "binary" => {
                let Some((op_word, ospan)) = self.text_ref(&fields, "op") else {
                    return None;
                };
                if !self.check_enum_case("BinOp", &op_word, ospan) {
                    return None;
                }
                let overflow = match fields.get("overflow") {
                    Some(f) => {
                        let Some(w) = self.text_of(&f.value) else {
                            return None;
                        };
                        if !self.check_enum_case("OverflowPolicy", &w, f.value_span) {
                            return None;
                        }
                        hir::OverflowPolicy::Trap
                    }
                    None => hir::OverflowPolicy::Trap, // branch already reported
                };
                let Some((lhs_t, lhs_span)) = self.text_ref(&fields, "lhs") else {
                    return None;
                };
                let lhs = match self.resolve_in(&lhs_t, lhs_span, "ExprRef", &body.exprs, expr_sibs)
                {
                    Some(e) => hir::ExprId(e),
                    None => return None,
                };
                let Some((rhs_t, rhs_span)) = self.text_ref(&fields, "rhs") else {
                    return None;
                };
                let rhs = match self.resolve_in(&rhs_t, rhs_span, "ExprRef", &body.exprs, expr_sibs)
                {
                    Some(e) => hir::ExprId(e),
                    None => return None,
                };
                let op = match op_word.as_str() {
                    "add_i32" => hir::BinOp::AddI32,
                    "mul_i32" => hir::BinOp::MulI32,
                    "lt_i32" => hir::BinOp::LtI32,
                    _ => return None,
                };
                hir::ExprKind::Binary {
                    op,
                    overflow,
                    lhs,
                    rhs,
                }
            }
            "call" => {
                let Some((ct, cspan)) = self.text_ref(&fields, "callee") else {
                    return None;
                };
                let callee = match self.resolve_in(&ct, cspan, "DefRef", def_ids, def_sibs) {
                    Some(d) => hir::DefId(d),
                    None => return None,
                };
                let mut eval_args = Vec::new();
                if let Some(f) = fields.get("eval_args") {
                    match &f.value {
                        Value::List(items) => {
                            for it in items {
                                let Some(t) = self.text_of(it) else {
                                    self.error(
                                        "bind.type-shape",
                                        "eval_args entries must be text",
                                        f.value_span,
                                    );
                                    return None;
                                };
                                match self.resolve_in(
                                    &t,
                                    f.value_span,
                                    "ExprRef",
                                    &body.exprs,
                                    expr_sibs,
                                ) {
                                    Some(e) => eval_args.push(hir::ExprId(e)),
                                    None => return None,
                                }
                            }
                        }
                        _ => {
                            self.error("bind.type-shape", "eval_args must be a list", f.value_span);
                            return None;
                        }
                    }
                }
                let mut bindings = Vec::new();
                if let Some(f) = fields.get("bindings") {
                    match &f.value {
                        Value::List(items) => {
                            for it in items {
                                match it {
                                    Value::Obj(obj) => {
                                        let mut param = None;
                                        let mut arg = None;
                                        for of in obj {
                                            match of.name.as_str() {
                                                "param" => {
                                                    param = int_value(&of.value).and_then(|l| {
                                                        l.to_u32().filter(|_| {
                                                            matches!(
                                                                l.suffix.as_deref(),
                                                                None | Some("u32")
                                                            )
                                                        })
                                                    })
                                                }
                                                "arg" => {
                                                    arg = int_value(&of.value).and_then(|l| {
                                                        l.to_u32().filter(|_| {
                                                            matches!(
                                                                l.suffix.as_deref(),
                                                                None | Some("u32")
                                                            )
                                                        })
                                                    })
                                                }
                                                other => {
                                                    self.error(
                                                        "bind.unknown-field",
                                                        format!(
                                                            "unknown field `{}` in binding",
                                                            other
                                                        ),
                                                        of.name_span,
                                                    );
                                                    return None;
                                                }
                                            }
                                        }
                                        match (param, arg) {
                                            (Some(param), Some(arg)) => {
                                                bindings.push(hir::CallBinding { param, arg })
                                            }
                                            _ => {
                                                self.error(
                                                    "bind.type-shape",
                                                    "binding requires param and arg u32 fields",
                                                    f.value_span,
                                                );
                                                return None;
                                            }
                                        }
                                    }
                                    _ => {
                                        self.error(
                                            "bind.type-shape",
                                            "bindings entries must be objects",
                                            f.value_span,
                                        );
                                        return None;
                                    }
                                }
                            }
                        }
                        _ => {
                            self.error("bind.type-shape", "bindings must be a list", f.value_span);
                            return None;
                        }
                    }
                }
                hir::ExprKind::Call {
                    callee,
                    eval_args,
                    bindings,
                }
            }
            _ => return None,
        };
        Some(hir::Expr {
            id_text,
            ty,
            kind,
            span: node.span,
        })
    }

    fn bind_block(
        &mut self,
        node: &SyntaxNode,
        body: &BodyTables,
        block_sibs: &[(&str, &BTreeMap<String, u32>)],
    ) -> Option<hir::Block> {
        let spec = self.desc.nodes.get("block").cloned().unwrap_or_default();
        let fields = FieldSet::build(node, self);
        let (id_text, _) = self.slot_text(node, &fields, 0, "id", "block")?;
        for name in fields.names() {
            if name == "id" {
                continue; // explicit-form slot alias
            }
            self.error(
                "bind.unknown-node",
                format!("blocks contain statements, not field `{}`", name),
                fields.get(name).unwrap().name_span,
            );
        }
        let mut stmts = Vec::new();
        for stmt in node.child_nodes() {
            if !spec.statements.contains(&stmt.tag) {
                self.error(
                    "bind.unknown-node",
                    format!("`{}` is not a statement", stmt.tag),
                    stmt.tag_span,
                );
                continue;
            }
            let sfields = FieldSet::build(stmt, self);
            self.branch_and_check(
                stmt.tag.as_str(),
                &stmt.primary,
                &sfields,
                &format!("statement in block `{}`", id_text),
                stmt.tag_span,
            );
            self.check_fields_against_spec(stmt.tag.as_str(), &sfields);
            let stmt_value = match stmt.tag.as_str() {
                "let" => {
                    let (t, span) = self.text_ref(&sfields, "local")?;
                    let local = hir::LocalId(self.resolve_in(
                        &t,
                        span,
                        "LocalRef",
                        &body.locals,
                        block_sibs,
                    )?);
                    let (t, span) = self.text_ref(&sfields, "value")?;
                    let value = hir::ExprId(self.resolve_in(
                        &t,
                        span,
                        "ExprRef",
                        &body.exprs,
                        block_sibs,
                    )?);
                    hir::Stmt::Let { local, value }
                }
                "return" => {
                    let (t, span) = self.text_ref(&sfields, "value")?;
                    let value = hir::ExprId(self.resolve_in(
                        &t,
                        span,
                        "ExprRef",
                        &body.exprs,
                        block_sibs,
                    )?);
                    hir::Stmt::Return { value }
                }
                "if" => {
                    let (t, span) = self.text_ref(&sfields, "cond")?;
                    let cond = hir::ExprId(self.resolve_in(
                        &t,
                        span,
                        "ExprRef",
                        &body.exprs,
                        block_sibs,
                    )?);
                    let (t, span) = self.text_ref(&sfields, "then")?;
                    let then = hir::BlockId(self.resolve_in(
                        &t,
                        span,
                        "BlockRef",
                        &body.blocks,
                        block_sibs,
                    )?);
                    let els = match sfields.get("else") {
                        Some(f) => match self.text_of(&f.value) {
                            Some(t) => match body.blocks.get(&t) {
                                Some(bid) => Some(hir::BlockId(*bid)),
                                None => {
                                    self.resolve_in(
                                        &t,
                                        f.value_span,
                                        "BlockRef",
                                        &body.blocks,
                                        block_sibs,
                                    );
                                    return None;
                                }
                            },
                            None => {
                                self.error("bind.type-shape", "else must be text", f.value_span);
                                return None;
                            }
                        },
                        None => None,
                    };
                    hir::Stmt::If { cond, then, els }
                }
                "loop" => {
                    let (t, span) = self.text_ref(&sfields, "id")?;
                    let id = hir::LoopId(self.resolve_in(
                        &t,
                        span,
                        "LoopRef",
                        &body.loops,
                        block_sibs,
                    )?);
                    let (t, span) = self.text_ref(&sfields, "body")?;
                    let body_id = hir::BlockId(self.resolve_in(
                        &t,
                        span,
                        "BlockRef",
                        &body.blocks,
                        block_sibs,
                    )?);
                    hir::Stmt::Loop { id, body: body_id }
                }
                "assign" => {
                    let (t, span) = self.text_ref(&sfields, "place")?;
                    let place = hir::PlaceId(self.resolve_in(
                        &t,
                        span,
                        "PlaceRef",
                        &body.places,
                        block_sibs,
                    )?);
                    let (t, span) = self.text_ref(&sfields, "value")?;
                    let value = hir::ExprId(self.resolve_in(
                        &t,
                        span,
                        "ExprRef",
                        &body.exprs,
                        block_sibs,
                    )?);
                    hir::Stmt::Assign { place, value }
                }
                "break" => {
                    let (t, span) = self.text_ref(&sfields, "target")?;
                    let target = hir::LoopId(self.resolve_in(
                        &t,
                        span,
                        "LoopRef",
                        &body.loops,
                        block_sibs,
                    )?);
                    hir::Stmt::Break { target }
                }
                "continue" => {
                    let (t, span) = self.text_ref(&sfields, "target")?;
                    let target = hir::LoopId(self.resolve_in(
                        &t,
                        span,
                        "LoopRef",
                        &body.loops,
                        block_sibs,
                    )?);
                    hir::Stmt::Continue { target }
                }
                _ => continue,
            };
            stmts.push(stmt_value);
        }
        Some(hir::Block {
            id_text,
            stmts,
            span: node.span,
        })
    }

    // -- type use -----------------------------------------------------------

    fn resolve_typeuse(
        &mut self,
        v: &Value,
        span: Span,
        type_ids: &BTreeMap<String, u32>,
    ) -> Option<hir::TypeUse> {
        match v {
            Value::Ident(word) if word == "i32" => {
                Some(hir::TypeUse::Builtin(hir::BuiltinType::I32))
            }
            Value::Ident(word) if word == "bool" => {
                Some(hir::TypeUse::Builtin(hir::BuiltinType::Bool))
            }
            Value::Ident(_) | Value::Str(_) => {
                let t = match self.text_of(v) {
                    Some(t) => t,
                    None => {
                        self.error("bind.type-shape", "type reference must be text", span);
                        return None;
                    }
                };
                match type_ids.get(&t) {
                    Some(tid) => Some(hir::TypeUse::Ref(hir::TypeId(*tid))),
                    None => {
                        self.error(
                            "bind.dangling-type-ref",
                            format!("dangling type reference `{}`", t),
                            span,
                        );
                        None
                    }
                }
            }
            _ => {
                self.error(
                    "bind.type-shape",
                    "type use must be a bareword builtin or a type reference",
                    span,
                );
                None
            }
        }
    }
}

fn str_value(v: &Value) -> Option<String> {
    match v {
        Value::Str(s) => Some(s.clone()),
        _ => None,
    }
}

fn int_value(v: &Value) -> Option<IntLit> {
    match v {
        Value::Int(lit) => Some(lit.clone()),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Canonical writer
// ---------------------------------------------------------------------------

/// Write the bundle back in fully-named canonical form. Re-reading the output
/// yields a semantically equal bundle (see text_binding tests).
pub fn write_canonical(bundle: &hir::Bundle, desc: &Descriptor) -> String {
    let mut w = Writer::new();
    w.line(&format!(
        "hir(schema: {}, revision: {}u32, profile: {}) {{",
        quote(&desc.schema),
        desc.revision,
        quote(&desc.profile)
    ));
    w.depth += 1;
    if !bundle.requires.is_empty() {
        let items: Vec<String> = bundle.requires.iter().map(|r| quote(r)).collect();
        w.line(&format!("requires: [{}]", items.join(", ")));
    }
    for m in &bundle.modules {
        w.open(&format!("module(id: {})", quote(&m.id_text)));
        if let Some(p) = &m.path {
            w.line(&format!("path: {}", quote(p)));
        }
        w.open("types");
        for t in &m.types {
            match &t.kind {
                hir::TypeKind::I32 => {
                    w.line(&format!("type(id: {}, kind: i32) {{}}", quote(&t.id_text)))
                }
                hir::TypeKind::Bool => {
                    w.line(&format!("type(id: {}, kind: bool) {{}}", quote(&t.id_text)))
                }
                hir::TypeKind::Function(sig) => {
                    let params: Vec<String> =
                        sig.params.iter().map(|p| type_use_text(p, m)).collect();
                    w.line(&format!(
                        "type(id: {}, kind: function) {{ params: [{}]; result: {} }}",
                        quote(&t.id_text),
                        params.join(", "),
                        type_use_text(&sig.result, m)
                    ));
                }
            }
        }
        w.close();
        w.open("declarations");
        for d in &m.defs {
            match &d.kind {
                hir::DefKind::Function { name, ty, body } => {
                    let body_text = m
                        .bodies
                        .get(body.0 as usize)
                        .map(|b| b.id_text.clone())
                        .unwrap_or_default();
                    w.line(&format!(
                        "function(id: {}, type: {}) {{ name: {}; body: {} }}",
                        quote(&d.id_text),
                        quote(&m.types[ty.0 as usize].id_text),
                        quote(name),
                        quote(&body_text)
                    ));
                }
                hir::DefKind::Intrinsic { symbol, effect, ty } => {
                    w.line(&format!(
                        "intrinsic(id: {}, type: {}) {{ symbol: {}; effect: {} }}",
                        quote(&d.id_text),
                        quote(&m.types[ty.0 as usize].id_text),
                        quote(symbol),
                        effect.as_str()
                    ));
                }
            }
        }
        w.close();
        w.open("bodies");
        for b in &m.bodies {
            w.open(&format!("body(id: {})", quote(&b.id_text)));
            w.line(&format!(
                "owner: {}; entry: {}",
                quote(&m.defs[b.owner.0 as usize].id_text),
                quote(&b.blocks[b.entry.0 as usize].id_text)
            ));
            w.open("locals");
            for l in &b.locals {
                let mut parts = vec![format!("name: {}", quote(&l.name))];
                parts.push(format!("role: {}", l.role.as_str()));
                if let Some(idx) = l.index {
                    parts.push(format!("index: {}u32", idx));
                }
                // `mutable:` is forbidden on param locals; emit only when true
                // or explicitly declared so canonical output re-binds cleanly.
                if l.mutable || l.mutable_explicit {
                    parts.push(format!("mutable: {}", l.mutable));
                }
                w.line(&format!(
                    "local(id: {}, type: {}) {{ {} }}",
                    quote(&l.id_text),
                    type_use_text(&l.ty, m),
                    parts.join("; ")
                ));
            }
            w.close();
            w.open("places");
            for p in &b.places {
                let access = match p.access {
                    hir::Access::Mutable => "mutable",
                    hir::Access::ReadOnly => "readonly",
                };
                w.line(&format!(
                    "place(id: {}, kind: local) {{ local: {}; type: {}; access: {} }}",
                    quote(&p.id_text),
                    quote(&b.locals[p.local.0 as usize].id_text),
                    type_use_text(&p.ty, m),
                    access
                ));
            }
            w.close();
            w.open("expressions");
            for e in &b.exprs {
                write_expr(&mut w, e, b, m);
            }
            w.close();
            w.open("blocks");
            for bl in &b.blocks {
                if bl.stmts.is_empty() {
                    w.line(&format!("block(id: {}) {{}}", quote(&bl.id_text)));
                    continue;
                }
                w.open(&format!("block(id: {})", quote(&bl.id_text)));
                for s in &bl.stmts {
                    let text = write_stmt(s, b, m);
                    w.line(&text);
                }
                w.close();
            }
            w.close();
            w.close(); // body
        }
        w.close(); // bodies
        w.close(); // module
    }
    w.depth -= 1;
    w.line("}");
    w.out
}

fn write_expr(w: &mut Writer, e: &hir::Expr, b: &hir::Body, m: &hir::Module) {
    let kind_word = match &e.kind {
        hir::ExprKind::ReadLocal { .. } => "read_local",
        hir::ExprKind::Constant { .. } => "constant",
        hir::ExprKind::Binary { .. } => "binary",
        hir::ExprKind::Call { .. } => "call",
    };
    let mut body_fields: Vec<String> = vec![format!("type: {}", type_use_text(&e.ty, m))];
    match &e.kind {
        hir::ExprKind::ReadLocal { local } => {
            body_fields.push(format!(
                "local: {}",
                quote(&b.locals[local.0 as usize].id_text)
            ));
        }
        hir::ExprKind::Constant { value } => {
            body_fields.push(format!("value: {}i32", value));
        }
        hir::ExprKind::Binary {
            op,
            overflow,
            lhs,
            rhs,
        } => {
            body_fields.push(format!("op: {}", op.as_str()));
            // overflow policy only exists for arithmetic ops; comparison ops
            // forbid it at bind time, so the canonical form must not emit it.
            if !matches!(op, hir::BinOp::LtI32) {
                body_fields.push(format!("overflow: {}", overflow_str(*overflow)));
            }
            body_fields.push(format!("lhs: {}", quote(&b.exprs[lhs.0 as usize].id_text)));
            body_fields.push(format!("rhs: {}", quote(&b.exprs[rhs.0 as usize].id_text)));
        }
        hir::ExprKind::Call {
            callee,
            eval_args,
            bindings,
        } => {
            body_fields.push(format!(
                "callee: {}",
                quote(&m.defs[callee.0 as usize].id_text)
            ));
            let args: Vec<String> = eval_args
                .iter()
                .map(|a| quote(&b.exprs[a.0 as usize].id_text))
                .collect();
            body_fields.push(format!("eval_args: [{}]", args.join(", ")));
            let bs: Vec<String> = bindings
                .iter()
                .map(|bd| format!("{{ param: {}u32, arg: {}u32 }}", bd.param, bd.arg))
                .collect();
            body_fields.push(format!("bindings: [{}]", bs.join(", ")));
        }
    }
    w.line(&format!(
        "expr(id: {}, kind: {}) {{ {} }}",
        quote(&e.id_text),
        kind_word,
        body_fields.join("; ")
    ));
}

fn write_stmt(s: &hir::Stmt, b: &hir::Body, _m: &hir::Module) -> String {
    match s {
        hir::Stmt::Let { local, value } => format!(
            "let(local: {}, value: {}) {{}}",
            quote(&b.locals[local.0 as usize].id_text),
            quote(&b.exprs[value.0 as usize].id_text)
        ),
        hir::Stmt::Return { value } => format!(
            "return(value: {}) {{}}",
            quote(&b.exprs[value.0 as usize].id_text)
        ),
        hir::Stmt::If { cond, then, els } => {
            let els_text = match els {
                Some(e) => format!(", else: {}", quote(&b.blocks[e.0 as usize].id_text)),
                None => String::new(),
            };
            format!(
                "if(cond: {}, then: {}{}) {{}}",
                quote(&b.exprs[cond.0 as usize].id_text),
                quote(&b.blocks[then.0 as usize].id_text),
                els_text
            )
        }
        hir::Stmt::Loop { id, body } => format!(
            "loop(id: {}, body: {}) {{}}",
            quote(&b.loops[id.0 as usize]),
            quote(&b.blocks[body.0 as usize].id_text)
        ),
        hir::Stmt::Assign { place, value } => format!(
            "assign(place: {}, value: {}) {{}}",
            quote(&b.places[place.0 as usize].id_text),
            quote(&b.exprs[value.0 as usize].id_text)
        ),
        hir::Stmt::Break { target } => {
            format!("break(target: {}) {{}}", quote(&b.loops[target.0 as usize]))
        }
        hir::Stmt::Continue { target } => format!(
            "continue(target: {}) {{}}",
            quote(&b.loops[target.0 as usize])
        ),
    }
}

fn overflow_str(o: hir::OverflowPolicy) -> &'static str {
    match o {
        hir::OverflowPolicy::Trap => "trap",
    }
}

fn type_use_text(u: &hir::TypeUse, m: &hir::Module) -> String {
    match u {
        hir::TypeUse::Builtin(b) => b.as_str().to_string(),
        hir::TypeUse::Ref(tid) => quote(&m.types[tid.0 as usize].id_text),
    }
}

fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

struct Writer {
    out: String,
    depth: usize,
}

impl Writer {
    fn new() -> Self {
        Writer {
            out: String::new(),
            depth: 0,
        }
    }

    fn line(&mut self, s: &str) {
        for _ in 0..self.depth {
            self.out.push_str("    ");
        }
        self.out.push_str(s);
        self.out.push('\n');
    }

    fn open(&mut self, s: &str) {
        self.line(&format!("{} {{", s));
        self.depth += 1;
    }

    fn close(&mut self) {
        self.depth -= 1;
        self.line("}");
    }
}
