//! PLAN-1107 T-02 (A2PY admission face): the A2PY target compiles a `use.py`
//! program under the shared declaration contract — no PyO3, no provider
//! binary, no AV session — and sees the shared py-op facts through the same
//! `SemanticView` every other target consumes.

use ace_a2x_core::SemanticView;
use ace_a2x_python::py_contract::py_declaration_contract;
use ace_core::py_ops::PyOpKind;
use ace_foreign_contract::manual::SOURCE_CAP;
use ace_foreign_contract::resource::RESOURCE_CAP;

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

fn caps() -> Vec<String> {
    vec![SOURCE_CAP.to_string(), RESOURCE_CAP.to_string()]
}

#[test]
fn a2py_compiles_use_py_under_the_declaration_contract() {
    let contract = py_declaration_contract().expect("declaration contract");
    let compiled = ace_core::source::compile_aggregates_source_with_foreign(
        SECTION_5_1,
        "p1107-a2py.at",
        &caps(),
        &contract,
    )
    .unwrap_or_else(|d| panic!("A2PY-side compile must admit the shared source: {d:?}"));
    let plan = compiled
        .checked
        .py_ops()
        .expect("the py lane pins the shared op plan");
    assert_eq!(plan.of_kind(PyOpKind::Import).count(), 6);
    assert_eq!(plan.of_kind(PyOpKind::ToFloat).count(), 2);

    let view = SemanticView::from_checked(&compiled.checked).expect("shared semantic view");
    let view_plan = view.py_ops().expect("the view exposes the same plan");
    assert_eq!(view_plan.len(), plan.len(), "same plan, no second derivation");

    // the declarations the view carries are exactly the admitted py op set
    let mut providers: Vec<&str> = view
        .foreign_declarations()
        .map(|d| d.provider.as_str())
        .collect();
    providers.sort_unstable();
    providers.dedup();
    assert_eq!(providers, vec!["py"], "all foreign declarations are py ops");
    let mut names: Vec<&str> = view
        .foreign_declarations()
        .map(|d| d.name.as_str())
        .collect();
    names.sort_unstable();
    let mut frozen: Vec<&str> = ace_core::py_ops::PY_OP_NAMES.to_vec();
    frozen.sort_unstable();
    assert_eq!(names, frozen, "the shared view carries the frozen op set");
}

#[test]
fn a2py_contract_rejects_unrelated_sources_through_the_same_front_door() {
    // Control: the declaration contract does not turn arbitrary foreign names
    // into Python ops — a `use.py` binding to an unknown member still fails
    // in the shared front door, and a non-Python foreign call is not admitted
    // by this contract at all.
    let contract = py_declaration_contract().expect("declaration contract");
    let err = ace_core::source::compile_aggregates_source_with_foreign(
        "use.py math: sqrt\nfn main() int {\nvar a = nosuch(4)\nreturn 0\n}",
        "neg.at",
        &caps(),
        &contract,
    )
    .expect_err("an unknown binding must be rejected");
    let text = format!("{err:?}");
    assert!(
        text.contains("source.undefined") || text.contains("no function named"),
        "the shared resolver rejects the unknown name: {text}"
    );
}