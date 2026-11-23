use super::{js_string, Cx};
use ace_a2x_core::{A2xReject, Ty};
use ace_core::{hir, source::json_ops::JSON_CAPABILITY};

impl Cx<'_, '_> {
    pub(super) fn widget_json_enabled(&self) -> bool {
        self.lang == crate::ScriptLang::JavaScript
            && self.view.is_age_package()
            && self
                .view
                .capabilities
                .iter()
                .any(|cap| cap == JSON_CAPABILITY)
    }

    pub(super) fn lower_widget_json_expr(
        &mut self,
        id: hir::ExprId,
        kind: &hir::ExprKind,
    ) -> Result<Option<String>, A2xReject> {
        if !self.widget_json_enabled() {
            return Ok(None);
        }
        let expr = match kind {
            hir::ExprKind::NilLit => "null".into(),
            hir::ExprKind::DynCast { operand, test_nil } => {
                let source = self.expr_ty(*operand)?;
                let target = self.expr_ty(id)?;
                let operand = self.lower_expr(*operand)?;
                if *test_nil {
                    format!("a2x_dyn_is_nil({operand})")
                } else if target == Ty::Dyn {
                    format!(
                        "a2x_dyn_pack({operand}, {})",
                        self.widget_json_type(source)?
                    )
                } else {
                    format!(
                        "a2x_dyn_take({operand}, {})",
                        self.widget_json_type(target)?
                    )
                }
            }
            hir::ExprKind::ObjectCtor {
                eval_args, keys, ..
            } => {
                let mut entries = Vec::with_capacity(keys.len());
                for (key, value) in keys.iter().zip(eval_args) {
                    let key = match key {
                        hir::ObjectKey::Text(text) => js_string(text),
                        hir::ObjectKey::Int(n) => n.to_string(),
                        hir::ObjectKey::Bool(b) => b.to_string(),
                    };
                    let value = self.lower_expr(*value)?;
                    let saved = self.fresh_temp();
                    self.emit(&format!("const {saved} = {value};"));
                    entries.push(format!("[{key}, {saved}]"));
                }
                format!("a2x_obj_new([{}])", entries.join(", "))
            }
            hir::ExprKind::ObjectGet { base, key } => {
                let base = self.lower_expr(*base)?;
                let key = self.lower_expr(*key)?;
                format!("a2x_obj_get({base}, {key})")
            }
            hir::ExprKind::ObjectSet { base, key, value } => {
                let base = self.lower_expr(*base)?;
                let key = self.lower_expr(*key)?;
                let value = self.lower_expr(*value)?;
                format!("a2x_obj_set({base}, {key}, {value})")
            }
            _ => return Ok(None),
        };
        Ok(Some(expr))
    }

    fn widget_json_type(&self, ty: Ty) -> Result<String, A2xReject> {
        match ty {
            Ty::List(id) => Ok(format!(
                "[\"list\", {}]",
                self.widget_json_type(self.view.list_elem(id)?)?
            )),
            Ty::Object(id) => Ok(format!(
                "[\"object\", {}]",
                self.widget_json_type(self.view.object_elem(id)?)?
            )),
            Ty::Bool
            | Ty::Char
            | Ty::I8
            | Ty::U8
            | Ty::I32
            | Ty::U32
            | Ty::I64
            | Ty::U64
            | Ty::F32
            | Ty::F64
            | Ty::Str
            | Ty::StrOwned
            | Ty::Nil
            | Ty::Dyn => Ok(js_string(ty.as_str())),
            _ => Err(A2xReject::unsupported_expr(format!(
                "widget JSON conversion for {ty:?}"
            ))),
        }
    }
}
