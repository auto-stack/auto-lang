use auto_lang::ast::{Expr, Name, Stmt, Store, StoreKind, Type};
use auto_lang::infer::{check_stmt, InferenceContext};

#[test]
fn test_infer_stmt_integration() {
    let mut ctx = InferenceContext::new();

    // Create a Let statement: let x = 42
    let store = Store {
        kind: StoreKind::Let,
        name: Name::from("integrated_x"),
        ty: Type::Int,
        expr: Expr::Int(42),
    };

    let stmt = Stmt::Store(store);

    // Check it
    let result = check_stmt(&mut ctx, &stmt);
    assert!(result.is_ok());

    // Verify variable is bound in context
    let bound_ty = ctx.lookup_type(&Name::from("integrated_x"));
    assert!(matches!(bound_ty, Some(Type::Int)));
}

#[test]
fn test_infer_block_integration() {
    let mut ctx = InferenceContext::new();
    let mut stmts = Vec::new();

    // let y = true
    stmts.push(Stmt::Store(Store {
        kind: StoreKind::Let,
        name: Name::from("y"),
        ty: Type::Bool,
        expr: Expr::Bool(true),
    }));

    // y (expression statement)
    stmts.push(Stmt::Expr(Expr::Ident(Name::from("y"))));

    let body = auto_lang::ast::Body {
        stmts,
        has_new_line: false,
    };
    let stmt = Stmt::Block(body);

    let result = check_stmt(&mut ctx, &stmt);
    assert!(result.is_ok());

    // Block should return type of last expression (Bool)
    assert!(matches!(result.unwrap(), Type::Bool));
}
