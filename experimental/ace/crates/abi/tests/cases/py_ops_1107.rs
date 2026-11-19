//! PLAN-1107 T-02: the shared, sourced Python-operation projection.
//!
//! The facts are produced by the ONE shared front door/resolver (not recovered
//! from the AV `p967.frame/1` wire text, function names or variable names) and
//! pinned onto the verified module. This test drives the real §5.1 program
//! through the py lane and asserts the recorded operation plan.
#![cfg(windows)]

use ace_abi::rust_provider::open_py_bridge_session;
use ace_core::py_ops::{PyOpKind, PyValClass};

const SECTION_5_1: &str = r#"use.py torch: manual_seed, arange, ones
use.py torch.nn: Linear, MSELoss
use.py torch.optim: SGD

fn main() int {
    manual_seed(0)
    var x = arange(8).reshape(4, 2) / 2
    var target = ones(4, 1)
    var model = Linear(2, 1, bias: false)
    var loss_fn = MSELoss()
    var optimizer = SGD(model.parameters(), lr: 0.1)
    var step = 0
    for step < 60 {
        var prediction = model(x)
        var loss = loss_fn(prediction, target)
        optimizer.zero_grad()
        loss.backward()
        optimizer.step()
        print(loss.to(float))
        step = step + 1
    }
    var prediction = model(x)
    print(prediction.shape)
    print(prediction[0].to(float))
    return 0
}"#;

fn compile(src: &str) -> ace_core::verify::CheckedModule {
    let session = open_py_bridge_session(1).expect("py bridge session");
    let caps = vec![
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ];
    ace_core::source::compile_aggregates_source_with_foreign(src, "p1107.at", &caps, &session.admitted)
        .unwrap_or_else(|d| panic!("compile: {d:?}"))
        .checked
}

#[test]
fn p1107_t02_imports_are_recorded_in_source_order() {
    let checked = compile(SECTION_5_1);
    let plan = checked.py_ops().expect("the py lane pins an op plan");
    let imports: Vec<(String, String)> = plan
        .of_kind(PyOpKind::Import)
        .map(|f| (f.module.clone(), f.member.clone()))
        .collect();
    assert_eq!(
        imports,
        vec![
            ("torch".to_string(), "manual_seed".to_string()),
            ("torch".to_string(), "arange".to_string()),
            ("torch".to_string(), "ones".to_string()),
            ("torch.nn".to_string(), "Linear".to_string()),
            ("torch.nn".to_string(), "MSELoss".to_string()),
            ("torch.optim".to_string(), "SGD".to_string()),
        ],
        "one import fact per `use.py` member, in source order"
    );
    for f in plan.of_kind(PyOpKind::Import) {
        assert!(f.operands.is_empty(), "imports carry no operand slots");
    }
}

#[test]
fn p1107_t02_operations_are_recorded_with_kinds_and_members() {
    let checked = compile(SECTION_5_1);
    let plan = checked.py_ops().expect("plan");
    let calls: Vec<&str> = plan
        .of_kind(PyOpKind::Call)
        .map(|f| f.member.as_str())
        .collect();
    // A call of an imported binding is a DIRECT call of the receiver object —
    // the frozen provider ABI's member label is "" for that form (`SGD(...)`,
    // `manual_seed(0)`, `model(x)`); a bound method carries its member name.
    for expected in [
        "", "reshape", "parameters", "zero_grad", "backward", "step",
    ] {
        assert!(
            calls.contains(&expected),
            "call member `{expected}` missing from {calls:?}"
        );
    }
    assert_eq!(
        calls.len(),
        14,
        "one call fact per source call operation, source order: {calls:?}"
    );
    assert_eq!(
        calls.iter().filter(|m| m.is_empty()).count(),
        9,
        "direct calls: manual_seed, arange, ones, Linear, MSELoss, SGD, model(x) x2, loss_fn"
    );
    // every call has a receiver operand and the receiver is a Python object
    for f in plan.of_kind(PyOpKind::Call) {
        let recv = f.receiver().unwrap_or_else(|| panic!("{f:?} lacks a receiver"));
        assert_eq!(recv.class, PyValClass::Object, "{f:?}");
    }
    let attrs: Vec<&str> = plan
        .of_kind(PyOpKind::GetAttr)
        .map(|f| f.member.as_str())
        .collect();
    assert_eq!(attrs, vec!["shape"], "attribute read recorded");
    let bins: Vec<&str> = plan
        .of_kind(PyOpKind::BinOp)
        .map(|f| f.member.as_str())
        .collect();
    assert_eq!(bins, vec!["truediv"], "the `/` on a Python object");
    assert_eq!(
        plan.of_kind(PyOpKind::GetItem).count(),
        1,
        "one integer index read"
    );
    let floats: Vec<&str> = plan
        .of_kind(PyOpKind::ToFloat)
        .map(|f| f.member.as_str())
        .collect();
    assert_eq!(
        floats,
        vec!["float", "float"],
        "both `.to(float)` extractions, no separate low-word lane"
    );
}

#[test]
fn p1107_t02_operand_slots_bind_nesting_and_value_classes() {
    let checked = compile(SECTION_5_1);
    let plan = checked.py_ops().expect("plan");
    // the `SGD(model.parameters(), lr: 0.1)` call: identified by its `lr` kwarg
    // (the direct-call form has no member label) — never by argument text.
    let sgd = plan
        .of_kind(PyOpKind::Call)
        .find(|f| f.kwargs().iter().any(|(n, _)| *n == "lr"))
        .expect("SGD call fact");
    let recv = sgd.receiver().expect("receiver slot");
    assert_eq!(recv.class, PyValClass::Object, "import binding is an object");
    let pos = sgd.positionals();
    assert_eq!(pos.len(), 1, "`model.parameters()` is the one positional");
    assert_eq!(pos[0].class, PyValClass::Object);
    let kw = sgd.kwargs();
    assert_eq!(kw.len(), 1);
    assert_eq!(kw[0].0, "lr");
    assert_eq!(
        kw[0].1.class,
        PyValClass::Float,
        "plain `0.1` crosses as the frozen float literal"
    );
    // the positional's span is EXACTLY the nested `model.parameters()` call
    // span — so a target maps the slot to the checked expression by span and
    // evaluates the receiver/args once, in source order.
    let nested = plan
        .of_kind(PyOpKind::Call)
        .find(|f| f.member == "parameters")
        .expect("nested call fact");
    assert_eq!(
        pos[0].span, nested.span,
        "the argument slot names the nested operation, not its frame text"
    );
    // and the nested operation's own receiver is the `model` object
    assert_eq!(
        nested.receiver().expect("nested receiver").class,
        PyValClass::Object
    );
}

#[test]
fn p1107_t02_provider_declarations_match_the_frozen_op_list() {
    // The target-neutral declaration list (`PY_OP_NAMES`) is the source other
    // targets recognise; it must stay exactly the admitted provider's set, so
    // a provider-side op addition cannot silently widen a consumer's contract.
    let session = open_py_bridge_session(1).expect("py bridge session");
    let mut names: Vec<&str> = session
        .admitted
        .source_declarations_ref()
        .iter()
        .filter(|d| d.provider == ace_core::py_ops::PY_PROVIDER)
        .map(|d| d.name.as_str())
        .collect();
    names.sort_unstable();
    let mut frozen: Vec<&str> = ace_core::py_ops::PY_OP_NAMES.to_vec();
    frozen.sort_unstable();
    assert_eq!(
        names, frozen,
        "the frozen py op declaration list must equal the admitted provider's"
    );
    // every py op rides the frame resource contract (one view-text frame)
    for decl in session
        .admitted
        .source_declarations_ref()
        .iter()
        .filter(|d| d.provider == ace_core::py_ops::PY_PROVIDER)
    {
        let res = decl
            .resource()
            .unwrap_or_else(|| panic!("py op `{}` lacks the frame resource sig", decl.name));
        assert_eq!(res.params.len(), 1, "{}: one frame param", decl.name);
    }
}

#[test]
fn p1107_t02_plain_program_carries_no_python_plan() {
    let checked = compile("fn main() int {\nvar a = 14\nreturn a\n}");
    assert!(
        checked.py_ops().is_none(),
        "a program without Python operations carries no plan (None, not an empty fake)"
    );
}