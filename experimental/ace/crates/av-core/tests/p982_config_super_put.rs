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
        dispatch(SUPER, &[Value::I32(root), Value::I32(obj), text("heap")])?;
        // 缺键：具名错误。
        let err = dispatch(SUPER, &[Value::I32(root), Value::I32(obj), text("nope")])
            .expect_err("missing key must be named");
        let message = format!("{err:?}");
        assert!(message.contains("no field"), "{message}");
        // 非对象源：具名错误。
        let err = dispatch(SUPER, &[Value::I32(root), Value::I32(root), text("heap")])
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
        .find(|(k, _)| k == "heap")
        .map(|(_, v)| v.clone())
        .expect("the moved field is a root property");
    assert_eq!(
        heap,
        ace_av_core::config_document::OwnedValue::Text("big".into())
    );
}
