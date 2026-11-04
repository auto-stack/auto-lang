//! 名字驱动的 kind 解析（auto-source 裁剪程序的 token/rule 名 → kind id）。
//!
//! 与旧 syntax-core `CoreKinds` 同一模式：kind 通过名字解析（非硬编码 id），
//! 语法身份变化（换名/换键）会在装载时给出受控 Infra 而不是错读。

use ace_syntax_runtime::program::{GrammarProgram, KIND_CALL, RULE_KIND_BASE, TOKEN_KIND_BASE};

pub struct SourceKinds {
    /// 保留 token（旧 RESERVED_KEYWORDS 镜像——grammar r-* 名）kind 集。
    pub reserved_toks: Vec<u32>,
    // items
    pub item: u32,
    pub fn_item: u32,
    pub type_item: Option<u32>,
    pub script_item: Option<u32>,
    // members
    pub member: u32,
    pub field_member: u32,
    pub static_method: u32,
    pub view_method: u32,
    // fn tail
    pub fn_tail: u32,
    pub params: u32,
    pub param: u32,
    pub ret_ty: u32,
    pub ann_ty: u32,
    // stmts
    pub block: u32,
    pub stmt: u32,
    pub let_stmt: u32,
    pub var_stmt: u32,
    pub assign_stmt: u32,
    pub path_assign_stmt: Option<u32>,
    pub path_seg: Option<u32>,
    pub return_stmt: u32,
    pub if_stmt: u32,
    pub else_block: u32,
    pub while_stmt: Option<u32>,
    pub for_cond_stmt: Option<u32>,
    pub loop_stmt: u32,
    pub expr_stmt: u32,
    // expr
    pub expr: u32,
    pub unary_expr: u32,
    pub postfix_expr: u32,
    pub postfix_step: Option<u32>,
    pub atom_expr: u32,
    pub leading_dot: Option<u32>,
    pub ctor_expr: Option<u32>,
    pub ctor_fields: Option<u32>,
    pub ctor_field: Option<u32>,
    /// PLAN-928：lambda 表达式（closures 面原子；其余 profile 无此规则）。
    pub lambda_expr: Option<u32>,
    /// PLAN-928：lambda 参数项（ident + 类型，无 mode 词）。
    pub lambda_param: Option<u32>,
    /// PLAN-928：闭包函数类型（closures 面 ret/ann 位）。
    pub fn_type: Option<u32>,
    /// PLAN-928：函数类型的参数项（词或嵌套 fn 型）。
    pub fn_type_param: Option<u32>,
    /// PLAN-928（T-03）：collections/closures 面的声明/语句/类型族。
    pub enum_item: Option<u32>,
    pub enum_arm: Option<u32>,
    pub is_stmt: Option<u32>,
    pub for_in_stmt: Option<u32>,
    pub match_arm: Option<u32>,
    pub generic_ty: Option<u32>,
    pub optional_ty: Option<u32>,
    pub generic_atom: Option<u32>,
    pub type_arg: Option<u32>,
    pub int_literal: u32,
    pub args: u32,
    pub arg: u32,
    pub arg_tail: u32,
    pub named_arg: u32,
    // tokens
    pub tok_ident: u32,
    pub tok_int_lit: u32,
    pub tok_str_lit: u32,
    pub tok_char_lit: u32,
    pub tok_affix_num: u32,
    pub tok_block_open: u32,
    pub tok_minus: u32,
    pub tok_bang: u32,
    pub tok_dot: u32,
    pub tok_comma: u32,
    pub tok_colon: u32,
    pub tok_lparen: u32,
    pub tok_rparen: u32,
    pub tok_lbrace: u32,
    pub tok_rbrace: u32,
    pub tok_assign: u32,
    pub tok_true: u32,
    pub tok_false: u32,
    pub tok_let: u32,
    pub tok_var: u32,
    pub tok_while: u32,
    pub tok_for: u32,
    pub tok_return: u32,
    pub tok_if: u32,
    pub tok_else: u32,
    pub tok_loop: u32,
    pub tok_break: u32,
    pub tok_continue: u32,
    pub tok_int_kw: u32,
    pub tok_i32: u32,
    pub tok_bool: u32,
    pub tok_type_kw: u32,
    pub tok_static: u32,
    pub tok_mut: u32,
    pub tok_enum_kw: u32,
    pub tok_is: u32,
    pub tok_in: u32,
    pub tok_question: u32,
    pub tok_arrow: u32,
    pub tok_fat_arrow: u32,
    pub tok_r_move: u32,
    pub tok_r_to: u32,
    // 算子 token（二元）
    pub op_plus: u32,
    pub op_minus: u32,
    pub op_star: u32,
    pub op_slash: u32,
    pub op_percent: u32,
    pub op_lt: u32,
    pub op_gt: u32,
    pub op_le: u32,
    pub op_ge: u32,
    pub op_eq: u32,
    pub op_ne: u32,
    pub op_andand: u32,
    pub op_oror: u32,
}

fn rule_kind(p: &GrammarProgram, name: &str) -> Option<u32> {
    p.rules
        .iter()
        .position(|r| r.name == name)
        .map(|i| RULE_KIND_BASE + i as u32)
}

fn rk_opt(p: &GrammarProgram, name: &str) -> Option<u32> {
    p.rules
        .iter()
        .position(|r| r.name == name)
        .map(|i| RULE_KIND_BASE + i as u32)
}
fn token_kind(p: &GrammarProgram, name: &str) -> Option<u32> {
    p.tokens
        .iter()
        .position(|t| t.name == name)
        .map(|i| TOKEN_KIND_BASE + i as u32)
}

/// 缺失的 required kind 名单（profile 裁剪后形状不符 = 受控装载失败）。
fn missing_names(p: &GrammarProgram) -> Vec<&'static str> {
    let mut want: Vec<&'static str> = Vec::new();
    for n in [
        "item",
        "fn-item",
        "block",
        "stmt",
        "let-stmt",
        "var-stmt",
        "assign-stmt",
        "return-stmt",
        "if-stmt",
        "else-block",
        "loop-stmt",
        "expr-stmt",
        "expr",
        "unary-expr",
        "postfix-expr",
        "atom-expr",
        "int-literal",
        "args",
        "arg",
        "arg-tail",
        "named-arg",
        "fn-tail",
        "params",
        "param",
        "ret-ty",
        "ann-ty",
        "member",
        "field-member",
    ] {
        if rule_kind(p, n).is_none() {
            want.push(n);
        }
    }
    for n in [
        "ident",
        "int-lit",
        "str-lit",
        "char-lit",
        "affix-num",
        "block-open",
        "minus",
        "bang",
        "dot",
        "comma",
        "colon",
        "lparen",
        "rparen",
        "lbrace",
        "rbrace",
        "assign",
        "true-kw",
        "false-kw",
        "let",
        "var",
        "return",
        "if",
        "else",
        "loop",
        "break",
        "continue",
        "int-kw",
        "i32",
        "bool",
        "plus",
        "star",
        "lt",
        "r-move",
        "r-to",
    ] {
        if token_kind(p, n).is_none() {
            want.push(n);
        }
    }
    want
}

impl SourceKinds {
    pub fn from_program(p: &GrammarProgram) -> Result<SourceKinds, Vec<&'static str>> {
        let missing = missing_names(p);
        if !missing.is_empty() {
            return Err(missing);
        }
        let rk = |n: &str| rule_kind(p, n).expect("checked");
        let tk = |n: &str| token_kind(p, n).expect("checked");
        let reserved_toks: Vec<u32> = p
            .tokens
            .iter()
            .enumerate()
            .filter(|(_, t)| t.name.starts_with("r-"))
            .map(|(i, _)| TOKEN_KIND_BASE + i as u32)
            .collect();
        Ok(SourceKinds {
            reserved_toks,
            item: rk("item"),
            fn_item: rk("fn-item"),
            type_item: rk_opt(p, "type-item"),
            script_item: rk_opt(p, "script-item"),
            member: rk("member"),
            field_member: rk("field-member"),
            static_method: rk("static-method"),
            view_method: rk("view-method"),
            fn_tail: rk("fn-tail"),
            params: rk("params"),
            param: rk("param"),
            ret_ty: rk("ret-ty"),
            ann_ty: rk("ann-ty"),
            block: rk("block"),
            stmt: rk("stmt"),
            let_stmt: rk("let-stmt"),
            var_stmt: rk("var-stmt"),
            assign_stmt: rk("assign-stmt"),
            path_assign_stmt: rk_opt(p, "path-assign-stmt"),
            path_seg: rk_opt(p, "path-seg"),
            return_stmt: rk("return-stmt"),
            if_stmt: rk("if-stmt"),
            else_block: rk("else-block"),
            while_stmt: rk_opt(p, "while-stmt"),
            for_cond_stmt: rk_opt(p, "for-cond-stmt"),
            loop_stmt: rk("loop-stmt"),
            expr_stmt: rk("expr-stmt"),
            expr: rk("expr"),
            unary_expr: rk("unary-expr"),
            postfix_expr: rk("postfix-expr"),
            postfix_step: rk_opt(p, "postfix-step"),
            atom_expr: rk("atom-expr"),
            leading_dot: rk_opt(p, "leading-dot"),
            ctor_expr: rk_opt(p, "ctor-expr"),
            ctor_fields: rk_opt(p, "ctor-fields"),
            ctor_field: rk_opt(p, "ctor-field"),
            lambda_expr: rk_opt(p, "lambda-expr"),
            lambda_param: rk_opt(p, "lambda-param"),
            fn_type: rk_opt(p, "fn-type"),
            fn_type_param: rk_opt(p, "fn-type-param"),
            enum_item: rk_opt(p, "enum-item"),
            enum_arm: rk_opt(p, "enum-arm"),
            is_stmt: rk_opt(p, "is-stmt"),
            for_in_stmt: rk_opt(p, "for-in-stmt"),
            match_arm: rk_opt(p, "match-arm"),
            generic_ty: rk_opt(p, "generic-ty"),
            optional_ty: rk_opt(p, "optional-ty"),
            generic_atom: rk_opt(p, "generic-atom"),
            type_arg: rk_opt(p, "type-arg"),
            int_literal: rk("int-literal"),
            args: rk("args"),
            arg: rk("arg"),
            arg_tail: rk("arg-tail"),
            named_arg: rk("named-arg"),
            tok_ident: tk("ident"),
            tok_int_lit: tk("int-lit"),
            tok_str_lit: tk("str-lit"),
            tok_char_lit: tk("char-lit"),
            tok_affix_num: tk("affix-num"),
            tok_block_open: tk("block-open"),
            tok_minus: tk("minus"),
            tok_bang: tk("bang"),
            tok_dot: tk("dot"),
            tok_comma: tk("comma"),
            tok_colon: tk("colon"),
            tok_lparen: tk("lparen"),
            tok_rparen: tk("rparen"),
            tok_lbrace: tk("lbrace"),
            tok_rbrace: tk("rbrace"),
            tok_assign: tk("assign"),
            tok_true: tk("true-kw"),
            tok_false: tk("false-kw"),
            tok_let: tk("let"),
            tok_var: tk("var"),
            tok_while: tk("while"),
            tok_for: tk("for"),
            tok_return: tk("return"),
            tok_if: tk("if"),
            tok_else: tk("else"),
            tok_loop: tk("loop"),
            tok_break: tk("break"),
            tok_continue: tk("continue"),
            tok_int_kw: tk("int-kw"),
            tok_i32: tk("i32"),
            tok_bool: tk("bool"),
            tok_type_kw: tk("type-kw"),
            tok_static: tk("static"),
            tok_mut: tk("mut"),
            tok_enum_kw: tk("enum"),
            tok_is: tk("is"),
            tok_in: tk("in"),
            tok_question: tk("question"),
            tok_arrow: tk("arrow"),
            tok_fat_arrow: tk("fat-arrow"),
            tok_r_move: tk("r-move"),
            tok_r_to: tk("r-to"),
            op_plus: tk("plus"),
            op_minus: tk("minus"),
            op_star: tk("star"),
            op_slash: tk("slash"),
            op_percent: tk("percent"),
            op_lt: tk("lt"),
            op_gt: tk("gt"),
            op_le: tk("le"),
            op_ge: tk("ge"),
            op_eq: tk("eq"),
            op_ne: tk("ne"),
            op_andand: tk("andand"),
            op_oror: tk("oror"),
        })
    }

    /// 合成调用节点的 kind（KIND_CALL）。
    pub fn call_kind(&self) -> u32 {
        KIND_CALL
    }
}
