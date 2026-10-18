//! msg/model 声明文法（PLAN-806 自 auto-lang Parser 迁出的单一份实现）。
//!
//! 逐字迁出基线：master `5d8834bcc` `crates/auto-lang/src/parser.rs`
//! `parse_msg_decl_inner`（原 14917–15001）/ `parse_model_block_inner`
//! （原 15010–15062）/ `parse_model_field_annotations`（原 15066–15132）。
//! 端口映射（T-01 D2/D3/D6）：
//! - `self.expect_ident(w)` → [`expect_word`]（逐字复刻：纯文本比较 +
//!   "Expected '{w}', got '{cur}'" + 当前 token 位 span）；
//! - `self.next()` → `ctx.advance()`；`self.is_kind(k)` → `ctx.is_kind(k)`；
//!   `self.expect(k)` → `ctx.expect(k)`（桥面→`Parser::expect` 同实现）；
//! - `self.cur.text` → `ctx.cur_text()`（owned String，`.into()` 回 Name）；
//! - named-payload lookahead `lexer.next()`+`push_token` 单 token 回推 →
//!   原始单 token 探测 `probe_next_kind`（成功路径零副作用/幂等，失败
//!   路径遵循旧回推前瞻失败消费协议；F806-R2-01 引入、R3-01 定形：
//!   checkpoint+advance/restore 形态曾引入注释跳过/docs 收集/词法错误
//!   记录的副作用扩散与文法扩大，已废弃）；
//! - `parse_type`/`parse_expr` → 端口委托核心文法（不复制）；
//! - 未知注解 `SyntaxError::Generic{.., pos_to_span(self.cur.pos)}` →
//!   `ctx.syntax_error(msg)`（桥面实现逐字同构）。

use auto_parser_api::ast::{
    Decorator, Expr, ModelBlock, ModelField, MsgDecl, MsgVariant, Name, Type,
};
use auto_parser_api::error::AutoResult;
use auto_parser_api::token::TokenKind;
use auto_parser_api::ParseContext;

/// `Parser::expect_ident` 的端口等价复刻（T-01 D3）：纯文本比较（无
/// TokenKind 检查），错误文本与位置逐字同构。桥面 `expect_keyword`
/// （"expected …" + Ident kind 检查）与旧臂不等价，不得替代。
fn expect_word(ctx: &mut dyn ParseContext, word: &str) -> AutoResult<()> {
    if ctx.cur_text() == word {
        ctx.advance();
        Ok(())
    } else {
        let got = ctx.cur_text();
        Err(ctx.syntax_error(format!("Expected '{}', got '{}'", word, got)))
    }
}

/// 完整 msg 声明（游标在 `msg` 关键字上），返回 [`MsgDecl`]。
///
/// 语法（冻结）：`msg [Name] { Variant(payload), … }`——名字（旧形态）
/// 读弃；variant 名可带引号（合同性 emit 名，`quoted` 旗标）；payload
/// 空 = unit、多元素 = 元组形态；`Name(type…)` 中元素可命名
/// （`Change(value: str)`，名字入 `payload_names` 逐位平行，位置形态
/// 为 `None`）。
pub fn parse_msg_decl(ctx: &mut dyn ParseContext) -> AutoResult<MsgDecl> {
    expect_word(ctx, "msg")?;
    // Plan 448 A: the name token after `msg` was never semantic (event
    // lookup always takes the tail segment; every backend derives the
    // enum name from the widget name or hardcodes "Msg"). Canonical form
    // is `msg { ... }`; a legacy `msg Name { ... }` still parses with the
    // name read and discarded.
    if !ctx.is_kind(TokenKind::LBrace) {
        ctx.advance();
    }

    ctx.expect(TokenKind::LBrace)?;
    ctx.skip_empty_lines(); // Skip empty lines after opening brace

    let mut variants = Vec::new();

    while !ctx.is_kind(TokenKind::RBrace) {
        ctx.skip_empty_lines(); // Skip empty lines before each variant
        if ctx.is_kind(TokenKind::RBrace) {
            break;
        }

        // Quoted variant names ("update" / "link-click") are
        // contractual emit names — record the quote so the Vue codegen
        // always declares/emits them (see MsgVariant.quoted).
        let quoted = ctx.is_kind(TokenKind::Str);
        let variant_name: Name = ctx.cur_text().into();
        ctx.advance();

        // Check for payload type. Plan 043 M5 #1: support multi-param
        // payloads `Complete(str, int)` / `RunSmart(int, str, []str)`
        // (was single-type only). Empty vec = unit variant.
        // Plan 435 挂账②:support named params `Change(value: str)` —
        // gallery 官方组件在用;名字记入 payload_names(位置形态为 None)。
        // lookahead 用原始单 token 探测 probe_next_kind（F806-R2-01 引入/
        // R3-01 定形：成功路径零副作用/幂等，失败路径遵循旧回推前瞻的
        // 失败消费协议——与旧 lexer.next()+push_token 逐字同构；Colon
        // 命中后 advance 消费的正是被回推的同一 token）。
        let mut payload = vec![];
        let mut payload_names = vec![];
        if ctx.is_kind(TokenKind::LParen) {
            ctx.advance();
            loop {
                let name = if ctx.is_kind(TokenKind::Ident) || ctx.is_kind(TokenKind::Str) {
                    if ctx.probe_next_kind() == Some(TokenKind::Colon) {
                        let candidate = ctx.cur_text();
                        ctx.advance(); // cur = 回推的 Colon
                        ctx.expect(TokenKind::Colon)?;
                        Some(candidate)
                    } else {
                        None // 位置形态,token 原样在流中
                    }
                } else {
                    None
                };
                payload.push(ctx.parse_type()?);
                payload_names.push(name);
                if ctx.is_kind(TokenKind::Comma) {
                    ctx.advance();
                } else {
                    break;
                }
            }
            ctx.expect(TokenKind::RParen)?;
        }

        variants.push(MsgVariant {
            name: variant_name,
            quoted,
            payload,
            payload_names,
        });

        if ctx.is_kind(TokenKind::Comma) {
            ctx.advance();
        }
        ctx.skip_empty_lines(); // Skip empty lines after comma
    }
    ctx.expect(TokenKind::RBrace)?;

    Ok(MsgDecl { variants })
}

/// model 块文法策略（T-01 D4）：内建完整文法与 778 冻结外部子集共用
/// 一份块循环与字段算法，差异点按策略开关。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelGrammar {
    /// 内建完整文法（生产默认）：`#[…]` 注解（可多个）+ `var` + name +
    /// [type] [= expr]。**无裸字段**——`=` 缺省仍必 parse_type（`name = e`
    /// → ty=Unknown 推断；无 `=` 无类型 → 类型文法报错）。EOF 不在循环
    /// 守卫中（经由 parse_type 的错误出口，冻结行为）。
    Builtin,
    /// 778 冻结外部子集（PLAN-778/801 pilot 合同面，**仅显式启用**）：
    /// 无注解（字段首遇 `#` → [`ModelSubset::NotClaimed`，调用方整块
    /// 回滚后让行内建）；字段名必为 Ident（否则 Err "model field name
    /// expected identifier, got '…'"）；**裸字段允许**——类型与初值
    /// 同时省略，终止于行界/块界/EOF（→ ty=Unknown + init=Nil）；EOF
    /// 在循环守卫中（778 冻结形态）。
    ExternalSubset,
}

/// [`parse_model_block_subset`] 的产出：声明块，或"注解不在子集内"
/// （调用方 checkpoint 回滚后 `Ok(None)` 让行内建——778 冻结恢复策略）。
#[derive(Debug, Clone)]
pub enum ModelSubset {
    Block(ModelBlock),
    NotClaimed,
}

/// 完整 model 块（游标在 `model` 关键字上），内建文法（生产默认入口）。
pub fn parse_model_block(ctx: &mut dyn ParseContext) -> AutoResult<ModelBlock> {
    match parse_model_block_impl(ctx, ModelGrammar::Builtin)? {
        ModelSubset::Block(block) => Ok(block),
        // Builtin 策略下 `#` 是注解起始、由注解解析器处理，NotClaimed
        // 不可达。
        ModelSubset::NotClaimed => Err(ctx.syntax_error(
            "internal error: builtin model grammar cannot yield NotClaimed".to_string(),
        )),
    }
}

/// 外部子集 model 块（游标在 `model` 关键字上；显式 pilot 策略入口）。
pub fn parse_model_block_subset(ctx: &mut dyn ParseContext) -> AutoResult<ModelSubset> {
    parse_model_block_impl(ctx, ModelGrammar::ExternalSubset)
}

fn parse_model_block_impl(
    ctx: &mut dyn ParseContext,
    grammar: ModelGrammar,
) -> AutoResult<ModelSubset> {
    expect_word(ctx, "model")?;
    ctx.expect(TokenKind::LBrace)?;
    ctx.skip_empty_lines();

    let mut fields = Vec::new();

    // 循环守卫按策略冻结：Builtin 仅 RBrace（EOF 经 parse_type 报错）；
    // ExternalSubset 另含 EOF（778 冻结：块未闭合时经 expect(RBrace) 报错）。
    loop {
        if ctx.is_kind(TokenKind::RBrace) {
            break;
        }
        if grammar == ModelGrammar::ExternalSubset && ctx.is_kind(TokenKind::EOF) {
            break;
        }
        ctx.skip_empty_lines();
        if ctx.is_kind(TokenKind::RBrace) {
            break;
        }

        // Plan 119 & 05-Nav: Parse annotations for model fields
        //（子集策略：注解未迁 → 整块让行内建，调用方负责回滚到语句原位）
        let (is_primary, decorators) = match grammar {
            ModelGrammar::Builtin => parse_model_field_annotations(ctx)?,
            ModelGrammar::ExternalSubset => {
                if ctx.is_kind(TokenKind::Hash) {
                    return Ok(ModelSubset::NotClaimed);
                }
                (false, Vec::new())
            }
        };

        // Plan 130: Support `var` keyword for mutable model fields
        // Default is immutable (like `let`), use `var` to allow modification in `on` handlers
        //（var 是真实 TokenKind，按文本判定——两臂同构）
        let mutable = if ctx.cur_text() == "var" {
            ctx.advance();
            true
        } else {
            false
        };

        // 字段名：内建取任意 token 文本（冻结行为）；子集要求 Ident。
        let name: Name = match grammar {
            ModelGrammar::Builtin => {
                let n = ctx.cur_text();
                ctx.advance();
                n.into()
            }
            ModelGrammar::ExternalSubset => {
                if ctx.cur_kind() != TokenKind::Ident {
                    let got = ctx.cur_text();
                    return Err(ctx.syntax_error(format!(
                        "model field name expected identifier, got '{}'",
                        got
                    )));
                }
                let n = ctx.cur_text();
                ctx.advance();
                n.into()
            }
        };

        // Support optional type annotation: `name = expr` or `name type = expr`
        //（子集裸字段：类型/初值可同时省略，终止于行界/块界/EOF——778 R2 冻结；
        // 内建无裸字段：非 `=` 前缀必走核心类型文法）
        let ty = match grammar {
            ModelGrammar::Builtin => {
                if ctx.is_kind(TokenKind::Asn) {
                    Type::Unknown
                } else {
                    ctx.parse_type()?
                }
            }
            ModelGrammar::ExternalSubset => {
                if ctx.is_kind(TokenKind::Asn)
                    || ctx.is_kind(TokenKind::Newline)
                    || ctx.is_kind(TokenKind::RBrace)
                    || ctx.is_kind(TokenKind::EOF)
                {
                    Type::Unknown
                } else {
                    ctx.parse_type()?
                }
            }
        };
        let init = if ctx.is_kind(TokenKind::Asn) {
            ctx.advance();
            ctx.parse_expr()?
        } else {
            Expr::Nil
        };
        fields.push(ModelField {
            name,
            ty,
            init,
            mutable,
            is_primary,
            decorators,
        });
        ctx.skip_empty_lines();
    }
    ctx.expect(TokenKind::RBrace)?;

    Ok(ModelSubset::Block(ModelBlock { fields }))
}

/// Parse annotations for model fields (Plan 119: #[primary], Plan 05-Nav: decorators)
/// Returns (is_primary, decorators) tuple
///（内建专属；逐字迁出——多注解循环、`#` 后非 `[` 宽容返回、空注解跳过、
/// Consume/Provide/NavParam 可选单 Str 参数宽容、未知注解错误文本/span 冻结）
fn parse_model_field_annotations(ctx: &mut dyn ParseContext) -> AutoResult<(bool, Vec<Decorator>)> {
    let mut is_primary = false;
    let mut decorators = Vec::new();

    // Parse multiple annotations: #[primary] #[Consume("key")]
    while ctx.is_kind(TokenKind::Hash) {
        ctx.advance(); // skip #

        if !ctx.is_kind(TokenKind::LSquare) {
            // Not an annotation, unexpected # - just return what we have
            return Ok((is_primary, decorators));
        }

        ctx.advance(); // skip [

        if !ctx.is_kind(TokenKind::Ident) {
            // Empty annotation, skip
            if ctx.is_kind(TokenKind::RSquare) {
                ctx.advance();
            }
            continue;
        }

        let annot_name = ctx.cur_text();
        ctx.advance(); // skip annotation name

        match annot_name.as_str() {
            "primary" => {
                is_primary = true;
            }
            "Consume" | "Provide" | "NavParam" => {
                // Parse arguments: ("key") or ("routeName")
                let mut args = Vec::new();
                if ctx.is_kind(TokenKind::LParen) {
                    ctx.advance(); // skip (
                                   // Parse string argument
                    if ctx.is_kind(TokenKind::Str) {
                        args.push(ctx.cur_text());
                        ctx.advance();
                    }
                    if ctx.is_kind(TokenKind::RParen) {
                        ctx.advance(); // skip )
                    }
                }
                decorators.push(Decorator {
                    name: annot_name.into(),
                    args,
                });
            }
            _ => {
                return Err(ctx.syntax_error(format!(
                    "Unknown model field annotation '{}'. Valid: #[primary], #[Consume(\"key\")], #[Provide(\"key\")], #[NavParam(\"routeName\")]",
                    annot_name
                )));
            }
        }

        if ctx.is_kind(TokenKind::RSquare) {
            ctx.advance(); // skip ]
        }
    }

    Ok((is_primary, decorators))
}
