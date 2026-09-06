//! Plan 560 T05: 静态 py-known 分析——s2s 糖规则与窥孔直达（T13）共用。
//!
//! "静态已知 py"（设计 §1）：use.py 导入项、py_*/桥调用的返回值流。
//! B 族改写（属性/索引/赋值）只对 py-known 接收者动手——Auto 结构体
//! 字段/数组索引语义零打扰；A 族方法调用走 obj_call 双通道（接收者
//! 任意）。分析为保守单遍（分支不敏感，闭包不内视）——错报"已知"
//! 的代价=改写走 py 桥（运行期响亮错），错报"未知"的代价=糖保留
//! （静态语义不变），两者都安全。

use crate::ast::{Code, Expr, Stmt, UseKind};
use std::collections::{HashMap, HashSet};

#[derive(Default, Debug)]
pub struct PyKnowledge {
    /// use.py 导入项名 → 模块路径（item 形态 `use.py torch: arange`）。
    pub py_items: HashMap<String, String>,
    /// use.py 裸模块名（`use.py torch` 无 items 形态）。
    pub py_modules: HashSet<String>,
    /// 每函数体分析得的 py-known 局部名（含形参直传不可知——保守不含）。
    pub fn_locals: HashMap<String, HashSet<String>>,
}

impl PyKnowledge {
    pub fn item_module(&self, item: &str) -> Option<&String> {
        self.py_items.get(item)
    }

    /// 名字是否静态已知 py（导入项/裸模块/函数内 py-known 局部）。
    pub fn is_known(&self, fn_name: &str, ident: &str) -> bool {
        self.py_items.contains_key(ident)
            || self.py_modules.contains(ident)
            || self
                .fn_locals
                .get(fn_name)
                .map(|s| s.contains(ident))
                .unwrap_or(false)
    }
}

/// 表达式在**当前改写态**下是否 py-producing（随改写 walker 内联使用：
/// 已改写为 py_*/桥调用的节点即时可判）。
pub fn expr_is_py(e: &Expr) -> bool {
    match e {
        Expr::Call(c) => {
            if let Expr::Ident(n) = c.name.as_ref() {
                let s = n.as_str();
                s.starts_with("py_")
                    || s.starts_with("obj_")
                    || s == "import_module"
                    // item 调用形态由 is_known 判（需 items 表，此处无表）
                    || false
            } else {
                matches!(c.name.as_ref(), Expr::Dot(recv, _) if expr_is_py(recv))
            }
        }
        Expr::Ident(_) | Expr::GenName(_) => false, // 由 is_known 判
        Expr::Dot(recv, _) | Expr::Index(recv, _) => expr_is_py(recv),
        Expr::Some(inner) | Expr::Ok(inner) => expr_is_py(inner),
        _ => false,
    }
}

pub fn analyze(code: &Code) -> PyKnowledge {
    let mut k = PyKnowledge::default();
    for stmt in &code.stmts {
        if let Stmt::Use(u) = stmt {
            if matches!(u.kind, UseKind::Py) {
                let path = if let Some(mp) = &u.module_path {
                    mp.to_string()
                } else {
                    u.paths
                        .iter()
                        .map(|p| p.as_str().to_string())
                        .collect::<Vec<_>>()
                        .join(".")
                };
                if u.items.is_empty() {
                    let last = path.rsplit('.').next().unwrap_or(&path).to_string();
                    k.py_modules.insert(last);
                } else {
                    for item in &u.items {
                        k.py_items.insert(item.as_str().to_string(), path.clone());
                    }
                }
            }
        }
    }
    for stmt in &code.stmts {
        if let Stmt::Fn(f) = stmt {
            let mut locals = HashSet::new();
            collect_fn(&f.body.stmts, &k, &mut locals);
            k.fn_locals.insert(f.name.as_str().to_string(), locals);
        }
    }
    k
}

fn collect_fn(stmts: &[Stmt], k: &PyKnowledge, locals: &mut HashSet<String>) {
    for stmt in stmts {
        match stmt {
            Stmt::Store(s) => {
                // py-producing 三形态：桥/组合子调用、导入项直传、
                // **导入项调用**（`var t = arange(6)`——T05 实证补面）。
                let is_item_call = matches!(&s.expr, Expr::Call(c)
                    if matches!(c.name.as_ref(), Expr::Ident(n)
                        if k.py_items.contains_key(n.as_str()) || k.py_modules.contains(n.as_str())));
                if expr_is_py(&s.expr)
                    || is_item_call
                    || matches!(&s.expr, Expr::Ident(n) if k.py_items.contains_key(n.as_str()) || k.py_modules.contains(n.as_str()))
                {
                    locals.insert(s.name.as_str().to_string());
                }
            }
            Stmt::If(i) => {
                for b in &i.branches {
                    collect_fn(&b.body.stmts, k, locals);
                }
                if let Some(e) = &i.else_ {
                    collect_fn(&e.stmts, k, locals);
                }
            }
            Stmt::For(f) => collect_fn(&f.body.stmts, k, locals),
            Stmt::Try(t) => {
                collect_fn(&t.body.stmts, k, locals);
                collect_fn(&t.catch_body.stmts, k, locals);
            }
            Stmt::Block(b) => collect_fn(&b.stmts, k, locals),
            // Plan 567 T12: with-as 块形态产物是 Stmt::Expr(Expr::Block)
            // ——绑定变量（__w/f）的 py-known 收录需下钻块表达式体。
            Stmt::Expr(e) => {
                if let Expr::Block(b) = e {
                    collect_fn(&b.stmts, k, locals);
                }
            }
            _ => {}
        }
    }
}
