//! PLAN-982（CONFIG 对象字段访问，step 1）：`cfg_super_put` 的会话级合同。
//!
//! `heap: kernel.heap` 需要"把对象的字段搬进文档"，而配置车道**没有**从宿主
//! 返回 Str 的通路（`cfg_*` 全返回 i32 句柄/状态）。本原语因此只做**搬运**：
//! `super-put(dst_node, src_obj, key)` 把源对象的同名字段挂到目标节点上。
//!
//! 断言四件事：①搬运成功（值/句柄都能落地）；②缺键**具名**错误（不静默丢）；
//! ③源不是对象 ⇒ 具名错误；④根级的 seeded 覆盖规则仍然生效（注入值赢）。

use ace_av_core::config_document as cfg;
use ace_av_core::contract::{StrHandle, Value};

fn text(s: &str) -> Value {
    // 会话内的文本句柄：detached 视图即可（`text_arg` 走视图读取）。
    Value::Str(StrHandle::detached(Box::from(s.as_bytes())))
}

const ROOT_SYM: &str = "auto.config.document.root.v1";
const OBJ_SYM: &str = "auto.config.document.obj-new.v1";
const PUT_TEXT: &str = "auto.config.document.obj-put.str.v1";
const SUPER: &str = "auto.config.document.super-put.v1";
const GET_OBJ: &str = "auto.config.document.obj-get-obj.v1";
const FINISH: &str = "auto.config.document.finish.v1";

fn dispatch(sym: &str, args: &[Value]) -> Result<i32, ace_av_core::host::HostError> {
    let cancel = ace_av_core::av::CancelFlag::default();
    cfg::dispatch(sym, args, &cancel)
}

#[test]
fn p982_config_super_put_moves_an_object_field_into_the_document() {
    cfg::open_session(cfg::DocumentLimits::default()).expect("session");
    let result = (|| -> Result<(), ace_av_core::host::HostError> {
        let root = dispatch(ROOT_SYM, &[])?;
        let obj = dispatch(OBJ_SYM, &[])?;
        dispatch(PUT_TEXT, &[Value::I32(obj), text("heap"), text("big")])?;
        // 把 obj 的 `heap` 搬进 root 的同名属性。
        dispatch(
            SUPER,
            &[
                Value::I32(root),
                Value::I32(obj),
                text("heap"),
                text("heap_copy"),
            ],
        )?;
        // 链式读的中间步：对象字段 ⇒ 返回对象句柄；非对象字段 ⇒ 具名错误。
        let inner = dispatch(GET_OBJ, &[Value::I32(obj), text("heap")])
            .expect_err("`heap` is text, not an object");
        assert!(format!("{inner:?}").contains("not an object"), "{inner:?}");
        let nested = dispatch(OBJ_SYM, &[])?;
        dispatch(PUT_TEXT, &[Value::I32(obj), text("mem"), text("x")])?;
        let _ = nested;

        // 缺键：具名错误。
        let err = dispatch(
            SUPER,
            &[Value::I32(root), Value::I32(obj), text("nope"), text("x")],
        )
        .expect_err("missing key must be named");
        let message = format!("{err:?}");
        assert!(message.contains("no field"), "{message}");
        // 非对象源：具名错误。
        let err = dispatch(
            SUPER,
            &[Value::I32(root), Value::I32(root), text("heap"), text("x")],
        )
        .expect_err("non-object source must be named");
        assert!(format!("{err:?}").contains("object source"), "{err:?}");
        dispatch(FINISH, &[])?;
        Ok(())
    })();
    assert!(result.is_ok(), "{result:?}");
    let doc = cfg::close_session().expect("close");
    let root = doc.root;
    let heap = root
        .props
        .iter()
        .find(|(k, _)| k == "heap_copy")
        .map(|(_, v)| v.clone())
        .expect("the moved field is a root property");
    assert_eq!(
        heap,
        ace_av_core::config_document::OwnedValue::Text("big".into())
    );
}
/// 注入（seeded）与搬运的交互：**注入值赢**，搬运不得覆盖根级同名属性。
#[test]
fn p982_config_super_put_does_not_override_a_seeded_root_property() {
    cfg::open_session(cfg::DocumentLimits::default()).expect("session");
    // 先把根属性 `x` 播种为注入值（`seed_root_prop` 是宿主注入的入口）。
    cfg::seed_root_prop(
        "x",
        &ace_av_core::config_document::OwnedValue::Text("injected".into()),
    )
    .expect("seed");
    let result = (|| -> Result<(), ace_av_core::host::HostError> {
        let root = dispatch(ROOT_SYM, &[])?;
        let obj = dispatch(OBJ_SYM, &[])?;
        dispatch(PUT_TEXT, &[Value::I32(obj), text("x"), text("from-doc")])?;
        dispatch(
            SUPER,
            &[Value::I32(root), Value::I32(obj), text("x"), text("x")],
        )?;
        dispatch(FINISH, &[])?;
        Ok(())
    })();
    assert!(result.is_ok(), "{result:?}");
    let doc = cfg::close_session().expect("close");
    let x = doc
        .root
        .props
        .iter()
        .find(|(k, _)| k == "x")
        .map(|(_, v)| v.clone())
        .expect("x is present");
    assert_eq!(
        x,
        ace_av_core::config_document::OwnedValue::Text("injected".into()),
        "the seeded (injected) value must win over the document's carried field"
    );
}
