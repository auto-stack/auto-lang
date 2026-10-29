use auto_ac_prototype::{source as ac, verify};
fn environment(source: &str) -> ac::CheckedEnv {
    let program = ac::parse_program_with(source, ac::SourceProfile::Collections).unwrap();
    let mut env = ac::resolve_env(&program, &[], ac::SourceProfile::Collections).unwrap();
    env.typecheck_all().unwrap();
    env
}
fn first(source: &str) -> ac::ast::FnItem {
    let mut program = ac::parse_program_with(source, ac::SourceProfile::Collections).unwrap();
    match program.items.remove(0) {
        ac::ast::Item::Fn(item) => item,
        _ => panic!("first function"),
    }
}
const OLD:&str="fn helper() int { var xs=List<int>.new()\n xs.push(42)\n return xs.get(0) }\nfn main() int { return helper() }";
const NEW:&str="fn helper() int { var xs=List<Str>.new()\n xs.push(\"中文\")\n let value ?int=Some(41)\n is value { Some(n) -> { return n+xs.get(0).len() } None -> { return 0 } } }\nfn main() int { return helper() }";
#[test]
fn collection_candidate_interned_body_types_commit_and_cancel_with_the_body() {
    // The same source is legal through the original whole frontend. A body
    // candidate must not lose new List/Option pool entries or rebuild main.
    ac::compile_collections_source(NEW, "p889.at", &[]).unwrap();
    let mut env = environment(OLD);
    let item = first(NEW);
    let before = ac::emit_function_body(&env, 0, OLD, "p889.at")
        .unwrap()
        .0
        .atom_text;
    let (candidate, work) = auto_ac_prototype::body_work::capture_body_work(|| {
        ac::prepare_candidate_document(&mut env, &item, 0, NEW, "p889.at").unwrap()
    });
    assert_eq!(work.len(), 1, "only helper may resolve/check/plan/bind");
    verify::verify_function(candidate.parts().bundle.clone(), 0).unwrap();
    assert_eq!(
        ac::emit_function_body(&env, 0, OLD, "p889.at")
            .unwrap()
            .0
            .atom_text,
        before
    );
    candidate.cancel();
    assert_eq!(env.generation(), 0);
    assert_eq!(
        ac::emit_function_body(&env, 0, OLD, "p889.at")
            .unwrap()
            .0
            .atom_text,
        before
    );
    let candidate = ac::prepare_candidate_document(&mut env, &item, 0, NEW, "p889.at").unwrap();
    let expected = candidate.parts().atom_text.clone();
    candidate.commit(&mut env).unwrap();
    assert_eq!(env.generation(), 1);
    let (parts, _) = ac::emit_function_body(&env, 0, NEW, "p889.at").unwrap();
    verify::verify_function(parts.bundle, 0).unwrap();
    assert_eq!(parts.atom_text, expected);
}

#[test]
fn collection_fragment_uses_only_granted_utf8_range_and_absolute_coordinates() {
    let prefix = "this prefix is deliberately not source @@@\n// 中文\n";
    let function =
        "fn helper() int { var xs=List<Str>.new()\n xs.push(\"中\")\n return xs.get(0).len()+41 }";
    let source = format!("{prefix}{function}\nfn untouched() int {{ return 0 }}");
    let start = prefix.len();
    let end = start + function.len();
    let (parsed, work) = ac::parser::capture_syntax_work(|| {
        ac::parse_function_fragment(&source, start..end, ac::SourceProfile::Collections).unwrap()
    });
    assert_eq!(
        parsed.span,
        auto_ac_prototype::atom_text::Span { start, end }
    );
    assert_eq!(work.tokenized_ranges, vec![parsed.span]);
    assert_eq!(
        work.parsed_functions.into_iter().collect::<Vec<_>>(),
        vec![("helper".into(), 1)]
    );
    assert_eq!(
        &source[parsed.name.span.start..parsed.name.span.end],
        "helper"
    );
    let bad = function.replace("+41", "+@");
    let source = format!("{prefix}{bad}");
    let error =
        ac::parse_function_fragment(&source, start..source.len(), ac::SourceProfile::Collections)
            .unwrap_err();
    assert!(error.span.start >= start && error.span.end <= source.len());
    assert!(source.is_char_boundary(error.span.start) && source.is_char_boundary(error.span.end));
    assert!(ac::parse_function_fragment(
        &source,
        start - 2..source.len() + 1,
        ac::SourceProfile::Collections
    )
    .is_err());
}

#[test]
fn collection_scoped_nominal_metadata_does_not_reuse_old_utf8_source_anchors() {
    let original =
        format!("{NEW}\ntype Item {{ name Str; code int }}\nenum Token {{ Data(Item)\nEnd }}");
    let mut env = environment(&original);
    let rewritten = original.replace("var xs", "// 中文🙂\n var xs");
    ac::compile_collections_source(&rewritten, "p889.at", &[]).unwrap();
    let main = ac::env_fn_body_identity(&env, 1).unwrap();
    let candidate =
        ac::prepare_candidate_document(&mut env, &first(&rewritten), 0, &rewritten, "p889.at")
            .unwrap();
    verify::verify_function(candidate.parts().bundle.clone(), 0).unwrap();
    for ty in &candidate.parts().bundle.modules[0].types {
        if matches!(
            ty.kind,
            auto_ac_prototype::hir::TypeKind::Record(_)
                | auto_ac_prototype::hir::TypeKind::Variant { .. }
        ) {
            assert_eq!(
                ty.span,
                auto_ac_prototype::atom_text::Span { start: 0, end: 0 }
            );
        }
    }
    candidate.commit(&mut env).unwrap();
    assert_eq!(ac::env_fn_body_identity(&env, 1), Some(main));
}

#[test]
fn collection_failed_foreign_stale_candidates_keep_current_body_types_and_identity() {
    let mut env = environment(OLD);
    let main = ac::env_fn_body_identity(&env, 1).unwrap();
    let before = ac::emit_function_body(&env, 0, OLD, "p889.at")
        .unwrap()
        .0
        .atom_text;
    let bad = NEW.replace("\"中文\"", "42");
    assert!(ac::prepare_candidate_document(&mut env, &first(&bad), 0, &bad, "p889.at").is_err());
    assert_eq!(
        ac::emit_function_body(&env, 0, OLD, "p889.at")
            .unwrap()
            .0
            .atom_text,
        before
    );
    let first = first(NEW);
    let candidate = ac::prepare_candidate_document(&mut env, &first, 0, NEW, "p889.at").unwrap();
    let stale = ac::prepare_candidate_document(&mut env, &first, 0, NEW, "p889.at").unwrap();
    let foreign = environment(OLD);
    assert!(candidate
        .validate(&foreign)
        .unwrap_err()
        .iter()
        .any(|d| d.code == "source.foreign"));
    let expected = candidate.parts().atom_text.clone();
    candidate.commit(&mut env).unwrap();
    assert!(stale
        .commit(&mut env)
        .unwrap_err()
        .iter()
        .any(|d| d.code == "source.stale"));
    assert_eq!(
        ac::emit_function_body(&env, 0, NEW, "p889.at")
            .unwrap()
            .0
            .atom_text,
        expected
    );
    assert_eq!(ac::env_fn_body_identity(&env, 1), Some(main));
    let closing = ac::prepare_candidate_document(&mut env, &first, 0, NEW, "p889.at").unwrap();
    env.close();
    assert!(closing
        .commit(&mut env)
        .unwrap_err()
        .iter()
        .any(|d| d.code == "source.closed"));
}
