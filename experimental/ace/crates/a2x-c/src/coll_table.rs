//! PLAN-926 CA-C: the seed-derived NOMINAL type table of `generated.c` —
//! record structs, variant tagged unions, and the typed list
//! instantiations with their per-op helpers.
//!
//! Everything here is generated from the decoded collections seed
//! ([`crate::seed_coll::SeedColl`]) so the PURE-RESTORE re-assembly
//! (no live view) produces byte-identical output. Semantics follow the
//! T-02 freeze (validated by the T-02 spike against the real MSVC
//! toolchain):
//! * every mutation prechecks bounds/budget BEFORE committing; a failed
//!   op frees its uncommitted owned input EXACTLY ONCE and leaves the
//!   old target intact (failure atomicity);
//! * `set` frees the OLD element at commit; `remove` hands the element
//!   out (ownership transfer); `clear`/`free` release elements in
//!   REVERSE index order, buffer last;
//! * all frees are IDEMPOTENT (a freed slot zeroes its owned bytes — a
//!   second free is a no-op);
//! * billing: 2 WU per slot (new base 4 + growth), Str payload bytes
//!   inside clone paths, ctor charges per the Rust parity table (i32 /
//!   bool element clones and NON-resource record clones charge ctor;
//!   resource records clone field-by-field with the payload's own
//!   charges; variant clones charge ctor);
//! * wrong-arm variant projection is the typed `VariantBranch` trap and
//!   leaves the root untouched (the caller's error frees still own it);
//!   a successful take moves the payload out and zeroes the shell;
//! * deep clone rolls back its partial output at the failing seam.

use crate::lower::sanitize;
use crate::lower_rt::c_record_name;
use crate::seed_coll::{c_variant_name, SeedColl, SeedCollTy};

/// The C value type of a seed type.
fn c_ty(ty: &SeedCollTy) -> String {
    ty.c_ty()
}

/// Whether a seed type owns resources (needs scheduled release).
fn owns(ty: &SeedCollTy) -> bool {
    match ty {
        SeedCollTy::I32 | SeedCollTy::Bool => false,
        SeedCollTy::StrOwned => true,
        SeedCollTy::Record(_) | SeedCollTy::Variant(_) => true,
    }
}

/// The deep free helper of a seed type (record/variant resolve through
/// the table).
fn deep_free_call(ty: &SeedCollTy) -> String {
    match ty {
        SeedCollTy::StrOwned => "a2x_str_free".into(),
        SeedCollTy::Record(n) => format!("a2x_coll_free_rec_{}", sanitize(n)),
        SeedCollTy::Variant(n) => format!("a2x_coll_free_var_{}", sanitize(n)),
        SeedCollTy::I32 | SeedCollTy::Bool => String::new(),
    }
}

/// The deep clone helper of a seed type.
fn deep_clone_call(ty: &SeedCollTy) -> String {
    match ty {
        SeedCollTy::StrOwned => "a2x_str_clone".into(),
        SeedCollTy::Record(n) => format!("a2x_coll_clone_rec_{}", sanitize(n)),
        SeedCollTy::Variant(n) => format!("a2x_coll_clone_var_{}", sanitize(n)),
        SeedCollTy::I32 | SeedCollTy::Bool => String::new(),
    }
}

/// Validate the seed's name-level references and uniqueness (fail loudly
/// on a dangling struct name — the 925 discipline).
fn validate(seed: &SeedColl) -> Result<(), String> {
    let mut rec_names = std::collections::BTreeSet::new();
    for r in &seed.records {
        if !rec_names.insert(sanitize(&r.name)) {
            return Err(format!("duplicate record name `{}` in the seed", r.name));
        }
    }
    let mut var_names = std::collections::BTreeSet::new();
    for v in &seed.variants {
        if !var_names.insert(sanitize(&v.name)) {
            return Err(format!("duplicate variant name `{}` in the seed", v.name));
        }
    }
    let known = |ty: &SeedCollTy| match ty {
        SeedCollTy::Record(n) => rec_names.contains(&sanitize(n)),
        SeedCollTy::Variant(n) => var_names.contains(&sanitize(n)),
        _ => true,
    };
    for r in &seed.records {
        for (_, ty) in &r.fields {
            if !known(ty) {
                return Err(format!(
                    "record `{}` field references unknown type `{:?}`",
                    r.name, ty
                ));
            }
        }
    }
    for v in &seed.variants {
        for (_, p) in &v.arms {
            if let Some(ty) = p {
                if !known(ty) {
                    return Err(format!(
                        "variant `{}` arm references unknown type `{:?}`",
                        v.name, ty
                    ));
                }
            }
        }
    }
    for e in &seed.lists {
        if !known(e) {
            return Err(format!("list instantiation of unknown type `{:?}`", e));
        }
    }
    let mut etags = std::collections::BTreeSet::new();
    for e in &seed.lists {
        if !etags.insert(e.elem_tag()) {
            return Err(format!("duplicate list element tag `{:?}`", e));
        }
    }
    Ok(())
}

/// Emit the whole type-table block of `generated.c` (struct definitions
/// first, then prototypes, then definitions — nested clone helpers call
/// each other).
pub fn coll_type_table_block(seed: &SeedColl) -> Result<String, String> {
    validate(seed)?;
    let order = topo_order(seed)?;
    let mut out = String::new();
    // 1) struct definitions in DEPENDENCY-FIRST order (a variant may
    //    reference a variant declared later in the source — N889 nested
    //    shapes; stable: seed order breaks ties, cycles are rejected)
    for key in &order {
        match key {
            TableKey::Record(r) => {
                out.push_str("typedef struct {\n");
                for (name, ty) in &r.fields {
                    out.push_str(&format!("    {} f_{};\n", c_ty(ty), sanitize(name)));
                }
                out.push_str(&format!("}} {};\n\n", c_record_name(&r.name)));
            }
            TableKey::Variant(v) => {
                out.push_str("typedef struct {\n    int32_t disc;\n    union {\n");
                for (arm, payload) in &v.arms {
                    if let Some(p) = payload {
                        out.push_str(&format!("        {} p_{};\n", c_ty(p), sanitize(arm)));
                    }
                }
                out.push_str(&format!("    }} as;\n}} {};\n\n", c_variant_name(&v.name)));
            }
        }
    }
    // lists come after every element struct (they only need the pointer)
    for e in &seed.lists {
        out.push_str(&format!(
            "typedef struct {{ {}* data; size_t len; size_t cap; }} a2x_list_{};\n\n",
            c_ty(e),
            e.elem_tag()
        ));
    }
    // 2) result wrappers + value/fwd constructors
    for r in &seed.records {
        let cr = c_record_name(&r.name);
        out.push_str(&format!(
            "typedef struct {{ {cr} value; a2x_trap_code trap; }} a2x_res_{cr};\n",
            cr = cr
        ));
    }
    for v in &seed.variants {
        let cv = c_variant_name(&v.name);
        out.push_str(&format!(
            "typedef struct {{ {cv} value; a2x_trap_code trap; }} a2x_res_{cv};\n",
            cv = cv
        ));
    }
    out.push('\n');
    for r in &seed.records {
        let cr = c_record_name(&r.name);
        out.push_str(&format!(
            "a2x_res_{cr} a2x_{cr}_value({cr} v);\na2x_res_{cr} a2x_fwd_{cr}(a2x_trap_code t);\nvoid a2x_coll_free_rec_{n}({cr}* r);\na2x_trap_code a2x_coll_clone_rec_{n}(const {cr}* src, {cr}* out);\n\n",
            cr = cr,
            n = sanitize(&r.name)
        ));
    }
    for v in &seed.variants {
        let cv = c_variant_name(&v.name);
        out.push_str(&format!(
            "a2x_res_{cv} a2x_{cv}_value({cv} v);\na2x_res_{cv} a2x_fwd_{cv}(a2x_trap_code t);\nvoid a2x_coll_free_var_{n}({cv}* v);\na2x_trap_code a2x_coll_clone_var_{n}(const {cv}* src, {cv}* out);\n",
            cv = cv,
            n = sanitize(&v.name)
        ));
        for (arm, payload) in &v.arms {
            if let Some(p) = payload {
                out.push_str(&format!(
                    "a2x_trap_code a2x_coll_take_{n}_{a}({cv}* v, {p}* out);\n",
                    n = sanitize(&v.name),
                    a = sanitize(arm),
                    cv = cv,
                    p = c_ty(p)
                ));
            }
        }
        out.push('\n');
    }
    for e in &seed.lists {
        let tag = e.elem_tag();
        let et = c_ty(e);
        out.push_str(&format!(
            "a2x_trap_code a2x_coll_list_{tag}_new(a2x_list_{tag}* out);\n\
             int32_t a2x_coll_list_{tag}_len(const a2x_list_{tag}* l);\n\
             a2x_trap_code a2x_coll_list_{tag}_push(a2x_list_{tag}* l, {et} v);\n\
             a2x_trap_code a2x_coll_list_{tag}_insert(a2x_list_{tag}* l, int32_t index, {et} v);\n\
             a2x_trap_code a2x_coll_list_{tag}_get(const a2x_list_{tag}* l, int32_t index, {et}* out);\n\
             a2x_trap_code a2x_coll_list_{tag}_view(const a2x_list_{tag}* l, int32_t index, const {et}** out);\n\
             a2x_trap_code a2x_coll_list_{tag}_set(a2x_list_{tag}* l, int32_t index, {et} v);\n\
             a2x_trap_code a2x_coll_list_{tag}_remove(a2x_list_{tag}* l, int32_t index, {et}* out);\n\
             void a2x_coll_list_{tag}_clear(a2x_list_{tag}* l);\n\
             void a2x_coll_list_{tag}_free(a2x_list_{tag}* l);\n\
             void a2x_coll_elem_{tag}_free({et}* v);\n\
             a2x_trap_code a2x_coll_elem_{tag}_clone(const {et}* src, {et}* out);\n\n",
            tag = tag,
            et = et
        ));
    }
    // 3) definitions
    for r in &seed.records {
        emit_record_helpers(&mut out, r);
    }
    for v in &seed.variants {
        emit_variant_helpers(&mut out, v);
    }
    for e in &seed.lists {
        emit_list_helpers(&mut out, e);
    }
    Ok(out)
}

/// The value/fwd/free/clone quartet of one record (Rust parity: a
/// NON-resource record's clone is the ctor charge + copy; a resource
/// record clones field by field with partial-output rollback and no
/// blanket ctor charge).
fn emit_record_helpers(out: &mut String, r: &crate::seed_coll::SeedCollRecord) {
    let cr = c_record_name(&r.name);
    let n = sanitize(&r.name);
    out.push_str(&format!(
        "a2x_res_{cr} a2x_{cr}_value({cr} v) {{\n\
         \x20   a2x_res_{cr} r;\n\
         \x20   r.value = v; r.trap = A2X_TRAP_NONE;\n\
         \x20   return r;\n\
         }}\n\n\
         a2x_res_{cr} a2x_fwd_{cr}(a2x_trap_code t) {{\n\
         \x20   a2x_res_{cr} r;\n\
         \x20   memset(&r.value, 0, sizeof r.value);\n\
         \x20   r.trap = t;\n\
         \x20   return r;\n\
         }}\n\n",
        cr = cr
    ));
    let resource = r.fields.iter().any(|(_, t)| owns(t));
    // free: explicit field-order release; a freed slot zeroes its owned
    // bytes so a second free is a no-op (idempotent by construction)
    out.push_str(&format!(
        "void a2x_coll_free_rec_{n}({cr}* r) {{\n",
        n = n,
        cr = cr
    ));
    if resource {
        for (fname, ty) in &r.fields {
            if owns(ty) {
                out.push_str(&format!(
                    "    {}(&r->f_{});\n",
                    deep_free_call(ty),
                    sanitize(fname)
                ));
            }
        }
        out.push_str("    /* idempotent: zero the owned bytes */\n    memset(r, 0, sizeof *r);\n");
    } else {
        out.push_str("    (void)r; /* /W4: a field-free record frees nothing */\n");
    }
    out.push_str("}\n\n");
    // clone
    out.push_str(&format!(
        "a2x_trap_code a2x_coll_clone_rec_{n}(const {cr}* src, {cr}* out) {{\n\
         \x20   {cr} acc;\n\
         \x20   memset(&acc, 0, sizeof acc);\n",
        n = n,
        cr = cr
    ));
    if resource {
        for (fname, ty) in &r.fields {
            let f = sanitize(fname);
            match ty {
                SeedCollTy::StrOwned => {
                    out.push_str(&format!(
                        "\x20   {{ a2x_trap_code t = a2x_str_clone(src->f_{f}.data, src->f_{f}.len, &acc.f_{f});\n\
                         \x20       if (t != A2X_TRAP_NONE) {{ a2x_coll_free_rec_{n}(&acc); return t; }} }}\n",
                        f = f,
                        n = n
                    ));
                }
                SeedCollTy::Record(_) | SeedCollTy::Variant(_) => {
                    out.push_str(&format!(
                        "\x20   {{ a2x_trap_code t = {c}(&src->f_{f}, &acc.f_{f});\n\
                         \x20       if (t != A2X_TRAP_NONE) {{ a2x_coll_free_rec_{n}(&acc); return t; }} }}\n",
                        c = deep_clone_call(ty),
                        f = f,
                        n = n
                    ));
                }
                SeedCollTy::I32 | SeedCollTy::Bool => {
                    out.push_str(&format!("    acc.f_{f} = src->f_{f};\n", f = f));
                }
            }
        }
        out.push_str("    *out = acc;\n    return A2X_TRAP_NONE;\n}\n\n");
    } else {
        out.push_str(
            "    { a2x_trap_code t = a2x_coll_charge_ctor();\n        if (t != A2X_TRAP_NONE) { return t; } }\n    *out = *src;\n    return A2X_TRAP_NONE;\n}\n\n",
        );
    }
}

/// The helpers of one variant (ctor is emitted inline by the lowering —
/// charge + disc/payload init; the table carries free/clone/take).
fn emit_variant_helpers(out: &mut String, v: &crate::seed_coll::SeedCollVariant) {
    let cv = c_variant_name(&v.name);
    let n = sanitize(&v.name);
    out.push_str(&format!(
        "a2x_res_{cv} a2x_{cv}_value({cv} v) {{\n\
         \x20   a2x_res_{cv} r;\n\
         \x20   r.value = v; r.trap = A2X_TRAP_NONE;\n\
         \x20   return r;\n\
         }}\n\n\
         a2x_res_{cv} a2x_fwd_{cv}(a2x_trap_code t) {{\n\
         \x20   a2x_res_{cv} r;\n\
         \x20   memset(&r.value, 0, sizeof r.value);\n\
         \x20   r.trap = t;\n\
         \x20   return r;\n\
         }}\n\n",
        cv = cv
    ));
    // free: release the ACTIVE arm only (disc guards every access);
    // zero the payload afterwards so a second free is a no-op
    out.push_str(&format!(
        "void a2x_coll_free_var_{n}({cv}* v) {{\n",
        n = n,
        cv = cv
    ));
    let mut any = false;
    for (k, (arm, payload)) in v.arms.iter().enumerate() {
        if let Some(p) = payload {
            if owns(p) {
                any = true;
                out.push_str(&format!(
                    "    if (v->disc == {k}) {{ {}(&v->as.p_{a}); }}\n",
                    deep_free_call(p),
                    k = k,
                    a = sanitize(arm)
                ));
            }
        }
    }
    if !any {
        out.push_str("    (void)v; /* /W4: no owned payloads */\n");
    }
    out.push_str("    memset(&v->as, 0, sizeof v->as); /* idempotent */\n}\n\n");
    // clone: blanket ctor charge (Rust parity), then the active arm's
    // own payload clone with partial rollback
    out.push_str(&format!(
        "a2x_trap_code a2x_coll_clone_var_{n}(const {cv}* src, {cv}* out) {{\n\
         \x20   {cv} acc;\n\
         \x20   a2x_trap_code t = a2x_coll_charge_ctor();\n\
         \x20   if (t != A2X_TRAP_NONE) {{ return t; }}\n\
         \x20   memset(&acc, 0, sizeof acc);\n\
         \x20   acc.disc = src->disc;\n",
        n = n,
        cv = cv
    ));
    for (k, (arm, payload)) in v.arms.iter().enumerate() {
        if let Some(p) = payload {
            let a = sanitize(arm);
            match p {
                SeedCollTy::StrOwned => {
                    out.push_str(&format!(
                        "\x20   if (src->disc == {k}) {{\n\
                         \x20       t = a2x_str_clone(src->as.p_{a}.data, src->as.p_{a}.len, &acc.as.p_{a});\n\
                         \x20       if (t != A2X_TRAP_NONE) {{ a2x_coll_free_var_{n}(&acc); return t; }}\n\
                         \x20   }}\n",
                        k = k,
                        a = a,
                        n = n
                    ));
                }
                SeedCollTy::Record(_) | SeedCollTy::Variant(_) => {
                    out.push_str(&format!(
                        "\x20   if (src->disc == {k}) {{\n\
                         \x20       t = {c}(&src->as.p_{a}, &acc.as.p_{a});\n\
                         \x20       if (t != A2X_TRAP_NONE) {{ a2x_coll_free_var_{n}(&acc); return t; }}\n\
                         \x20   }}\n",
                        k = k,
                        a = a,
                        n = n,
                        c = deep_clone_call(p)
                    ));
                }
                SeedCollTy::I32 | SeedCollTy::Bool => {
                    out.push_str(&format!(
                        "\x20   if (src->disc == {k}) {{ acc.as.p_{a} = src->as.p_{a}; }}\n",
                        k = k,
                        a = a
                    ));
                }
            }
        }
    }
    out.push_str("    *out = acc;\n    return A2X_TRAP_NONE;\n}\n\n");
    // take: wrong arm is the typed VariantBranch and leaves the root
    // untouched; a successful take moves the payload out and zeroes the
    // shell (no double-free path)
    for (k, (arm, payload)) in v.arms.iter().enumerate() {
        if let Some(p) = payload {
            out.push_str(&format!(
                "a2x_trap_code a2x_coll_take_{n}_{a}({cv}* v, {p}* out) {{\n\
                 \x20   if (v->disc != {k}) {{ return A2X_TRAP_VARIANT_BRANCH; }}\n\
                 \x20   *out = v->as.p_{a};\n\
                 \x20   memset(&v->as, 0, sizeof v->as);\n\
                 \x20   return A2X_TRAP_NONE;\n\
                 }}\n\n",
                n = n,
                a = sanitize(arm),
                cv = cv,
                p = c_ty(p),
                k = k
            ));
        }
    }
}

/// The typed helpers of one list instantiation (the T-02-spike-verified
/// op bodies, generalized through the element free/clone pair).
fn emit_list_helpers(out: &mut String, e: &SeedCollTy) {
    let tag = e.elem_tag();
    let et = c_ty(e);
    let esz = format!("sizeof({})", et);
    // element free/clone (Rust A2xCollElem parity)
    match e {
        SeedCollTy::I32 | SeedCollTy::Bool => {
            out.push_str(&format!(
                "void a2x_coll_elem_{tag}_free({et}* v) {{ (void)v; }}\n\
                 a2x_trap_code a2x_coll_elem_{tag}_clone(const {et}* src, {et}* out) {{\n\
                 \x20   a2x_trap_code t = a2x_coll_charge_ctor();\n\
                 \x20   if (t != A2X_TRAP_NONE) {{ return t; }}\n\
                 \x20   *out = *src;\n\
                 \x20   return A2X_TRAP_NONE;\n\
                 }}\n\n",
                tag = tag,
                et = et
            ));
        }
        SeedCollTy::StrOwned => {
            out.push_str(&format!(
                "void a2x_coll_elem_{tag}_free({et}* v) {{ a2x_str_free(v); }}\n\
                 a2x_trap_code a2x_coll_elem_{tag}_clone(const {et}* src, {et}* out) {{\n\
                 \x20   return a2x_str_clone(src->data, src->len, out);\n\
                 }}\n\n",
                tag = tag,
                et = et
            ));
        }
        SeedCollTy::Record(_) | SeedCollTy::Variant(_) => {
            out.push_str(&format!(
                "void a2x_coll_elem_{tag}_free({et}* v) {{ {f}(v); }}\n\
                 a2x_trap_code a2x_coll_elem_{tag}_clone(const {et}* src, {et}* out) {{\n\
                 \x20   return {c}(src, out);\n\
                 }}\n\n",
                tag = tag,
                et = et,
                f = deep_free_call(e),
                c = deep_clone_call(e)
            ));
        }
    }
    out.push_str(&format!(
        "static a2x_trap_code a2x_coll_list_{tag}_reserve(a2x_list_{tag}* l, size_t want) {{\n\
         \x20   size_t new_cap, bytes, old_bytes;\n\
         \x20   a2x_trap_code t;\n\
         \x20   if (l->len + want <= l->cap) {{ return A2X_TRAP_NONE; }}\n\
         \x20   new_cap = (l->cap == 0) ? (size_t)(4 > want ? 4 : want) : l->cap;\n\
         \x20   while (new_cap < l->len + want) {{\n\
         \x20       if (new_cap > 65536) {{ return A2X_TRAP_LIST_INDEX; }}\n\
         \x20       if (new_cap > (size_t)-1 / 2) {{ return A2X_TRAP_RESOURCE; }}\n\
         \x20       new_cap *= 2;\n\
         \x20   }}\n\
         \x20   if (new_cap > 65536) {{ return A2X_TRAP_LIST_INDEX; }}\n\
         \x20   t = a2x_coll_charge(2 * (long)(new_cap - l->cap));\n\
         \x20   if (t != A2X_TRAP_NONE) {{ return t; }}\n\
         \x20   old_bytes = l->cap * {esz};\n\
         \x20   bytes = new_cap * {esz};\n\
         \x20   t = a2x_coll_alloc_gate(bytes - old_bytes);\n\
         \x20   if (t != A2X_TRAP_NONE) {{ return t; }}\n\
         \x20   {{\n\
         \x20       {et}* p = ({et}*)realloc(l->data, bytes);\n\
         \x20       if (p == NULL) {{ a2x_coll_st_live_backing_bytes -= (long)(bytes - old_bytes); return A2X_TRAP_RESOURCE; }}\n\
         \x20       l->data = p; l->cap = new_cap;\n\
         \x20   }}\n\
         \x20   return A2X_TRAP_NONE;\n\
         }}\n\n",
        tag = tag,
        et = et,
        esz = esz
    ));
    out.push_str(&format!(
        "a2x_trap_code a2x_coll_list_{tag}_new(a2x_list_{tag}* out) {{\n\
         \x20   a2x_trap_code t;\n\
         \x20   size_t bytes = 4 * {esz};\n\
         \x20   memset(out, 0, sizeof *out);\n\
         \x20   t = a2x_coll_charge(2 * 4);\n\
         \x20   if (t != A2X_TRAP_NONE) {{ return t; }}\n\
         \x20   t = a2x_coll_alloc_gate(bytes);\n\
         \x20   if (t != A2X_TRAP_NONE) {{ return t; }}\n\
         \x20   out->data = ({et}*)malloc(bytes);\n\
         \x20   if (out->data == NULL) {{ a2x_coll_st_live_backing_bytes -= (long)bytes; return A2X_TRAP_RESOURCE; }}\n\
         \x20   out->len = 0; out->cap = 4;\n\
         \x20   return A2X_TRAP_NONE;\n\
         }}\n\n\
         int32_t a2x_coll_list_{tag}_len(const a2x_list_{tag}* l) {{ return (int32_t)l->len; }}\n\n",
        tag = tag,
        et = et,
        esz = esz
    ));
    out.push_str(&format!(
        "a2x_trap_code a2x_coll_list_{tag}_push(a2x_list_{tag}* l, {et} v) {{\n\
         \x20   a2x_trap_code t;\n\
         \x20   if (l->len + 1 > 65536) {{ a2x_coll_elem_{tag}_free(&v); return A2X_TRAP_LIST_INDEX; }}\n\
         \x20   t = a2x_coll_list_{tag}_reserve(l, 1);\n\
         \x20   if (t != A2X_TRAP_NONE) {{ a2x_coll_elem_{tag}_free(&v); return t; }}\n\
         \x20   t = a2x_coll_charge(2);\n\
         \x20   if (t != A2X_TRAP_NONE) {{ a2x_coll_elem_{tag}_free(&v); return t; }}\n\
         \x20   l->data[l->len] = v; l->len++;\n\
         \x20   a2x_coll_st_live_nodes++;\n\
         \x20   return A2X_TRAP_NONE;\n\
         }}\n\n",
        tag = tag,
        et = et
    ));
    out.push_str(&format!(
        "a2x_trap_code a2x_coll_list_{tag}_insert(a2x_list_{tag}* l, int32_t index, {et} v) {{\n\
         \x20   a2x_trap_code t;\n\
         \x20   size_t idx;\n\
         \x20   if (index < 0) {{ a2x_coll_elem_{tag}_free(&v); return A2X_TRAP_LIST_INDEX; }}\n\
         \x20   idx = (size_t)index;\n\
         \x20   if (idx > l->len) {{ a2x_coll_elem_{tag}_free(&v); return A2X_TRAP_LIST_INDEX; }}\n\
         \x20   t = a2x_coll_list_{tag}_reserve(l, 1);\n\
         \x20   if (t != A2X_TRAP_NONE) {{ a2x_coll_elem_{tag}_free(&v); return t; }}\n\
         \x20   t = a2x_coll_charge(2);\n\
         \x20   if (t != A2X_TRAP_NONE) {{ a2x_coll_elem_{tag}_free(&v); return t; }}\n\
         \x20   if (idx < l->len) {{ memmove(&l->data[idx + 1], &l->data[idx], (l->len - idx) * {esz}); }}\n\
         \x20   l->data[idx] = v; l->len++;\n\
         \x20   a2x_coll_st_live_nodes++;\n\
         \x20   return A2X_TRAP_NONE;\n\
         }}\n\n",
        tag = tag,
        et = et,
        esz = esz
    ));
    out.push_str(&format!(
        "a2x_trap_code a2x_coll_list_{tag}_get(const a2x_list_{tag}* l, int32_t index, {et}* out) {{\n\
         \x20   size_t idx;\n\
         \x20   if (index < 0) {{ return A2X_TRAP_LIST_INDEX; }}\n\
         \x20   idx = (size_t)index;\n\
         \x20   if (idx >= l->len) {{ return A2X_TRAP_LIST_INDEX; }}\n\
         \x20   *out = l->data[idx];\n\
         \x20   return A2X_TRAP_NONE;\n\
         }}\n\n\
         a2x_trap_code a2x_coll_list_{tag}_view(const a2x_list_{tag}* l, int32_t index, const {et}** out) {{\n\
         \x20   size_t idx;\n\
         \x20   if (index < 0) {{ return A2X_TRAP_LIST_INDEX; }}\n\
         \x20   idx = (size_t)index;\n\
         \x20   if (idx >= l->len) {{ return A2X_TRAP_LIST_INDEX; }}\n\
         \x20   *out = &l->data[idx];\n\
         \x20   return A2X_TRAP_NONE;\n\
         }}\n\n",
        tag = tag,
        et = et
    ));
    out.push_str(&format!(
        "a2x_trap_code a2x_coll_list_{tag}_set(a2x_list_{tag}* l, int32_t index, {et} v) {{\n\
         \x20   a2x_trap_code t;\n\
         \x20   size_t idx;\n\
         \x20   if (index < 0) {{ a2x_coll_elem_{tag}_free(&v); return A2X_TRAP_LIST_INDEX; }}\n\
         \x20   idx = (size_t)index;\n\
         \x20   if (idx >= l->len) {{ a2x_coll_elem_{tag}_free(&v); return A2X_TRAP_LIST_INDEX; }}\n\
         \x20   t = a2x_coll_charge(2);\n\
         \x20   if (t != A2X_TRAP_NONE) {{ a2x_coll_elem_{tag}_free(&v); return t; }}\n\
         \x20   a2x_coll_elem_{tag}_free(&l->data[idx]); /* old frees at commit */\n\
         \x20   l->data[idx] = v;\n\
         \x20   return A2X_TRAP_NONE;\n\
         }}\n\n",
        tag = tag,
        et = et
    ));
    out.push_str(&format!(
        "a2x_trap_code a2x_coll_list_{tag}_remove(a2x_list_{tag}* l, int32_t index, {et}* out) {{\n\
         \x20   size_t idx;\n\
         \x20   if (index < 0) {{ return A2X_TRAP_LIST_INDEX; }}\n\
         \x20   idx = (size_t)index;\n\
         \x20   if (idx >= l->len) {{ return A2X_TRAP_LIST_INDEX; }}\n\
         \x20   *out = l->data[idx];\n\
         \x20   if (idx + 1 < l->len) {{ memmove(&l->data[idx], &l->data[idx + 1], (l->len - idx - 1) * {esz}); }}\n\
         \x20   l->len--;\n\
         \x20   a2x_coll_st_live_nodes--;\n\
         \x20   return A2X_TRAP_NONE;\n\
         }}\n\n\
         void a2x_coll_list_{tag}_clear(a2x_list_{tag}* l) {{\n\
         \x20   while (l->len > 0) {{\n\
         \x20       l->len--;\n\
         \x20       a2x_coll_elem_{tag}_free(&l->data[l->len]);\n\
         \x20       a2x_coll_st_live_nodes--;\n\
         \x20   }}\n\
         }}\n\n\
         void a2x_coll_list_{tag}_free(a2x_list_{tag}* l) {{\n\
         \x20   while (l->len > 0) {{\n\
         \x20       l->len--;\n\
         \x20       a2x_coll_elem_{tag}_free(&l->data[l->len]);\n\
         \x20       a2x_coll_st_live_nodes--;\n\
         \x20   }}\n\
         \x20   if (l->data != NULL) {{\n\
         \x20       free(l->data);\n\
         \x20       a2x_coll_backing_release(l->cap * {esz});\n\
         \x20       l->data = NULL; l->cap = 0;\n\
         \x20   }}\n\
         }}\n\n",
        tag = tag,
        et = et,
        esz = esz
    ));
}

/// One nominal type-table member (topological sort key).
#[derive(Clone, Copy)]
enum TableKey<'a> {
    Record(&'a crate::seed_coll::SeedCollRecord),
    Variant(&'a crate::seed_coll::SeedCollVariant),
}

/// The DEPENDENCY-FIRST definition order of the nominal types (stable:
/// seed order among independent members; a cycle is rejected — the
/// domain is acyclic by construction).
fn topo_order(seed: &SeedColl) -> Result<Vec<TableKey<'_>>, String> {
    use std::collections::BTreeMap;
    let mut index: BTreeMap<String, usize> = BTreeMap::new();
    let mut nodes: Vec<TableKey<'_>> = Vec::new();
    for r in &seed.records {
        if index.insert(sanitize(&r.name), nodes.len()).is_some() {
            return Err(format!("duplicate record name `{}`", r.name));
        }
        nodes.push(TableKey::Record(r));
    }
    for v in &seed.variants {
        if index.insert(sanitize(&v.name), nodes.len()).is_some() {
            return Err(format!("duplicate variant name `{}`", v.name));
        }
        nodes.push(TableKey::Variant(v));
    }
    let dep_of = |ty: &SeedCollTy| -> Option<usize> {
        match ty {
            SeedCollTy::Record(n) | SeedCollTy::Variant(n) => index.get(&sanitize(n)).copied(),
            _ => None,
        }
    };
    let n = nodes.len();
    let mut deps: Vec<Vec<usize>> = Vec::with_capacity(n);
    for key in &nodes {
        let mut ds = Vec::new();
        match key {
            TableKey::Record(r) => {
                for (_, ty) in &r.fields {
                    if let Some(d) = dep_of(ty) {
                        ds.push(d);
                    }
                }
            }
            TableKey::Variant(v) => {
                for (_, p) in &v.arms {
                    if let Some(ty) = p {
                        if let Some(d) = dep_of(ty) {
                            ds.push(d);
                        }
                    }
                }
            }
        }
        deps.push(ds);
    }
    // Kahn with a sorted (seed-order) frontier — deterministic
    let mut indeg = vec![0usize; n];
    let mut rdeps: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (i, ds) in deps.iter().enumerate() {
        for &d in ds {
            if d == i {
                let name = match &nodes[i] {
                    TableKey::Record(r) => r.name.clone(),
                    TableKey::Variant(v) => v.name.clone(),
                };
                return Err(format!("type `{}` nests itself", name));
            }
            indeg[i] += 1;
            rdeps[d].push(i);
        }
    }
    let mut order = Vec::with_capacity(n);
    let mut ready: Vec<usize> = (0..n).filter(|&i| indeg[i] == 0).collect();
    ready.sort_unstable();
    while let Some(i) = ready.first().copied() {
        ready.remove(0);
        order.push(i);
        for &j in &rdeps[i] {
            indeg[j] -= 1;
            if indeg[j] == 0 {
                ready.push(j);
                ready.sort_unstable();
            }
        }
    }
    if order.len() != n {
        return Err("cyclic nominal type nesting in the seed".to_string());
    }
    Ok(order.into_iter().map(|i| nodes[i]).collect())
}
