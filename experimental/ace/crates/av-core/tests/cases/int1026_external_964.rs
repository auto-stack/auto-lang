//! PLAN-964（T-08/INT-1026，IF-VAL）：通用外部值/可调用值的中立接口见证。
//!
//! 独立非 Python provider（本文件内的 counter provider）验证：
//! - 双别名共享修改（一处写、另一处读可见）——provider 细胞的 RefCell 共享；
//! - 函数参数返回/分支/循环传引用保身份（同 token，多次 retain/drop）；
//! - 最后 drop 恰一次（drop_hook 计数）；
//! - 错 kind 拒（`take_as` External{other} → DynTakeError 携标签）；
//! - 活外部值**不可** portable freeze（`av.dyn.unsupported`）；
//! - 原生近邻：相邻 pack/borrow 的普通 i32 不受影响。
//!
//! provider 不需要 Python/torch/967 runtime——本包以中立引用与 retain/drop
//! 协议消费 `ace-foreign-contract` 的 `ResKind::Handle{provider}` 事实形状。

use ace_av_core::contract::{ExternalCell, Value};
use ace_av_core::dynamic::{DynType, DynValue};
use std::cell::RefCell;
use std::rc::Rc;

/// 独立 provider：一个共享的 (kind, token) 计数器 + drop 钩子。
struct Provider {
    drop_count: Rc<RefCell<u32>>,
}

impl Provider {
    fn new() -> (Self, Rc<RefCell<u32>>) {
        let drop_count = Rc::new(RefCell::new(0u32));
        (
            Provider {
                drop_count: Rc::clone(&drop_count),
            },
            drop_count,
        )
    }

    fn mint(&self, kind: u32, token: u64) -> Value {
        let counter = Rc::clone(&self.drop_count);
        Value::External(Rc::new(RefCell::new(ExternalCell {
            provider: "p964.provider.v1".to_string(),
            kind,
            token,
            drop_hook: Some(Rc::new(move || {
                *counter.borrow_mut() += 1;
            })),
        })))
    }

    fn ty(&self, kind: u32) -> DynType {
        DynType::External {
            provider: "p964.provider.v1".to_string(),
            kind,
        }
    }
}

#[test]
fn int1026_aliases_share_the_cell_and_drop_exactly_once() {
    let (provider, drops) = Provider::new();
    let ty = provider.ty(1);
    // 双别名：retain = clone（Rc 增 owner），写经任一别名可见。
    let a = DynValue::pack(provider.mint(1, 0xABCD), ty.clone()).expect("pack external");
    let v1 = a.as_value().clone();
    let Value::External(cell) = &v1 else {
        panic!("expected external")
    };
    let Value::External(cell_a) = a.as_value() else {
        panic!("expected external")
    };
    // 一处写，另一处读——共享修改可见。
    cell.borrow_mut().token = 0x1234;
    assert_eq!(cell_a.borrow().token, 0x1234, "alias sees the write");

    // 别名经函数参数返回/分支/循环传引用保身份（同一 token 重复观察）。
    let mut cur = a.as_value().clone();
    for _ in 0..3 {
        // 分支：两个路径都保留同一引用。
        if cur.as_i32().is_none() {
            cur = cur.clone();
        } else {
            cur = cur.clone();
        }
    }
    let Value::External(c) = &cur else {
        panic!("expected external")
    };
    assert_eq!(
        c.borrow().token,
        0x1234,
        "identity preserved through aliases"
    );

    // 释放全部引用 → drop 钩子恰一次。
    drop(a);
    drop(v1);
    drop(cur);
    assert_eq!(*drops.borrow(), 1, "last-owner drop fires exactly once");
}

#[test]
fn int1026_wrong_kind_rejected_with_labels() {
    let (provider, _drops) = Provider::new();
    let v = DynValue::pack(provider.mint(1, 7), provider.ty(1)).expect("pack external");
    let wrong = DynType::External {
        provider: "p964.provider.v1".to_string(),
        kind: 2,
    };
    let err = v.take_as(&wrong).expect_err("different kind must fail");
    assert_eq!(err.code(), "av.dyn.take-type");
    assert!(err.expected.contains("external"), "label: {}", err.expected);
    assert!(err.actual.contains("external"), "label: {}", err.actual);
}

#[test]
fn int1026_live_external_is_not_portable_freezeable() {
    let (provider, _drops) = Provider::new();
    let v = DynValue::pack(provider.mint(1, 9), provider.ty(1)).expect("pack external");
    let opts = ace_av_core::dynamic::TraversalOptions::default();
    let err = ace_av_core::portable::freeze(&v, &opts).expect_err("live external never freezes");
    assert!(
        matches!(
            err,
            ace_av_core::portable::PortableError::Unsupported {
                category: "external"
            }
        ),
        "got {err:?}"
    );
}

#[test]
fn int1026_native_neighbor_untouched() {
    // 原生近邻：普通 i32 与外部值相邻 pack/borrow，互不影响。
    let i32_ty = DynType::I32;
    let n = DynValue::pack(Value::I32(5), i32_ty.clone()).expect("pack i32");
    assert_eq!(n.borrow_as(&DynType::I32), Some(&Value::I32(5)));
    assert_eq!(n.borrow_as(&DynType::I64), None, "no cross-kind coercion");

    let (provider, _drops) = Provider::new();
    let e = DynValue::pack(provider.mint(1, 11), provider.ty(1)).expect("pack external");
    // i32 不受外部值影响，反之亦然。
    assert_eq!(e.borrow_as(&provider.ty(1)).is_some(), true);
    assert_eq!(e.borrow_as(&DynType::I32), None);
    assert_eq!(n.borrow_as(&DynType::I32), Some(&Value::I32(5)));
}
