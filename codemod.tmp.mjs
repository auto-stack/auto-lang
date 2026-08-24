import { readFileSync, writeFileSync } from 'node:fs'
const f = 'crates/auto-lang/src/error_spans_tests.rs'
let s = readFileSync(f, 'utf8')

// Test 1: bool literal in event arg now PARSES (plan 015 P1#5) — flip to a
// success assertion on a still-unsupported token form instead. Use a token
// that stays unsupported: `)` is consumed... use a stray operator? Keep it
// simple: assert parse succeeds now.
const t1old = `    let err = parse_ui(code).expect_err("bool literal event arg must fail");

    // The root-cause error must mention the offending token...
    let msg = err.to_string();
    assert!(
        msg.contains("unsupported event argument `false`"),
        "root-cause message must name the offending token, got: {}",
        msg
    );

    // ...and its span must be the `false` token, not a closing brace.
    let false_off = code.find("(false)").unwrap() + 1;
    let label_off = first_label_offset(&err).expect("error must carry a label");
    assert_eq!(
        label_off, false_off,`
const t1new = `    // Plan 015 P1#5: bool literals are now VALID event args — this must
    // parse. The loud-error contract (gap 37a) still holds for genuinely
    // unsupported tokens; see the char-literal case below.
    parse_ui(code).expect("bool literal event arg must parse (plan 015 P1#5)");
}

#[test]
fn gap37a_unsupported_event_arg_error_points_at_offending_token() {
    let code = concat!(
        "widget App {\n",
        "  model { var open bool = false }\n",
        "  view {\n",
        "    col {\n",
        "      button { onclick.self: .\\"update:open\\"(if) }\n",
        "    }\n",
        "  }\n",
        "}\n"
    );
    let err = parse_ui(code).expect_err("kw token event arg must fail");

    // The root-cause error must mention the offending token...
    let msg = err.to_string();
    assert!(
        msg.contains("unsupported event argument"),
        "root-cause message must name the offending token, got: {}",
        msg
    );

    // ...and its span must be the offending token, not a closing brace.
    let false_off = code.find("(if)").unwrap() + 1;
    let label_off = first_label_offset(&err).expect("error must carry a label");
    assert_eq!(
        label_off, false_off,`
if (!s.includes(t1old)) { console.error('t1 anchor missing'); process.exit(1) }
s = s.replace(t1old, t1new)
writeFileSync(f, s)
console.log('gap37a test1 rewritten')
