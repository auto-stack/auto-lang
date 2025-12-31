/**
 * Parser Implementation
 * Implements a Pratt parser for auto-lang
 */

#include "autoc.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// ============================================================================
// Operator Precedence
// ============================================================================

typedef enum {
    PREC_NONE = 0,
    PREC_ASN = 1,       // = += -= etc
    PREC_OR = 2,
    PREC_AND = 3,
    PREC_CMP = 4,       // == != < > <= >=
    PREC_ADD = 10,      // + -
    PREC_MUL = 11,      // * /
    PREC_UNARY = 12,
    PREC_CALL = 15,
    PREC_DOT = 17,
} Precedence;

// ============================================================================
// Parser Creation
// ============================================================================

Parser* parser_new(Lexer* lexer) {
    Parser* parser = (Parser*)malloc(sizeof(Parser));
    parser->lexer = lexer;
    parser->scope_depth = 0;

    // Initialize current and peek tokens
    parser->current = lexer_next(lexer);
    parser->peek = lexer_next(lexer);

    return parser;
}

void parser_free(Parser* parser) {
    if (!parser) return;
    astr_free(&parser->current.text);
    astr_free(&parser->peek.text);
    free(parser);
}

// ============================================================================
// Parser Utilities
// ============================================================================

static void parser_advance(Parser* parser) {
    astr_free(&parser->current.text);
    parser->current = parser->peek;
    parser->peek = lexer_next(parser->lexer);
}

static bool parser_check(Parser* parser, TokenKind kind) {
    return parser->current.kind == kind;
}

static bool parser_match(Parser* parser, TokenKind kind) {
    if (parser_check(parser, kind)) {
        parser_advance(parser);
        return true;
    }
    return false;
}

static bool parser_expect(Parser* parser, TokenKind kind, const char* msg) {
    if (parser_check(parser, kind)) {
        parser_advance(parser);
        return true;
    }
    // TODO: Add proper error reporting
    fprintf(stderr, "Error: %s, got %d\n", msg, parser->current.kind);
    return false;
}

// ============================================================================
// Expression Parsing (Pratt Parser)
// ============================================================================

static Expr* parse_expr_primary(Parser* parser);
static Expr* parse_expr_unary(Parser* parser);
static Expr* parse_expr_binary(Parser* parser, Expr* left, Precedence prec);
static Expr* parse_expr_call(Parser* parser, Expr* callee);
static Expr* parse_expr_grouping(Parser* parser);

static Precedence get_infix_prec(TokenKind kind) {
    switch (kind) {
        case TOKEN_ASN:
        case TOKEN_ADDEQ:
        case TOKEN_SUBEQ:
        case TOKEN_MULEQ:
        case TOKEN_DIVEQ:
            return PREC_ASN;
        case TOKEN_EQ:
        case TOKEN_NEQ:
        case TOKEN_LT:
        case TOKEN_GT:
        case TOKEN_LE:
        case TOKEN_GE:
            return PREC_CMP;
        case TOKEN_ADD:
        case TOKEN_SUB:
            return PREC_ADD;
        case TOKEN_STAR:
        case TOKEN_DIV:
            return PREC_MUL;
        case TOKEN_RANGE:
        case TOKEN_RANGEEQ:
            return PREC_ADD;  // Range has same precedence as addition
        case TOKEN_DOT:
            return PREC_DOT;
        default:
            return PREC_NONE;
    }
}

static Precedence get_postfix_prec(TokenKind kind) {
    switch (kind) {
        case TOKEN_LPAREN:
        case TOKEN_LSQUARE:
            return PREC_CALL;
        default:
            return PREC_NONE;
    }
}

Expr* parser_parse_expr(Parser* parser) {
    return parse_expr_binary(parser, parse_expr_unary(parser), PREC_NONE);
}

static Expr* parse_expr_primary(Parser* parser) {
    Expr* expr = (Expr*)malloc(sizeof(Expr));
    expr->pos = parser->current.pos;

    if (parser_check(parser, TOKEN_INT)) {
        int32_t int_val = atoi(parser->current.text.data);
        parser_match(parser, TOKEN_INT);
        expr->kind = EXPR_INT;
        expr->u.int_val = int_val;
        return expr;
    }

    if (parser_check(parser, TOKEN_UINT)) {
        uint32_t uint_val = (uint32_t)atoi(parser->current.text.data);
        parser_match(parser, TOKEN_UINT);
        expr->kind = EXPR_UINT;
        expr->u.uint_val = uint_val;
        return expr;
    }

    if (parser_check(parser, TOKEN_FLOAT)) {
        double float_val = atof(parser->current.text.data);
        parser_match(parser, TOKEN_FLOAT);
        expr->kind = EXPR_DOUBLE;
        expr->u.float_val = float_val;
        return expr;
    }

    if (parser_check(parser, TOKEN_DOUBLE)) {
        double float_val = atof(parser->current.text.data);
        parser_match(parser, TOKEN_DOUBLE);
        expr->kind = EXPR_DOUBLE;
        expr->u.float_val = float_val;
        return expr;
    }

    if (parser_match(parser, TOKEN_TRUE)) {
        expr->kind = EXPR_BOOL;
        expr->u.bool_val = true;
        return expr;
    }

    if (parser_match(parser, TOKEN_FALSE)) {
        expr->kind = EXPR_BOOL;
        expr->u.bool_val = false;
        return expr;
    }

    if (parser_match(parser, TOKEN_NIL)) {
        expr->kind = EXPR_NIL;
        return expr;
    }

    if (parser_match(parser, TOKEN_NULL)) {
        expr->kind = EXPR_NULL;
        return expr;
    }

    if (parser_check(parser, TOKEN_STR)) {
        AutoStr str_val = parser->current.text;
        parser_match(parser, TOKEN_STR);
        expr->kind = EXPR_STR;
        expr->u.str_val = str_val;
        return expr;
    }

    if (parser_check(parser, TOKEN_CSTR)) {
        AutoStr str_val = parser->current.text;
        parser_match(parser, TOKEN_CSTR);
        expr->kind = EXPR_CSTR;
        expr->u.str_val = str_val;
        return expr;
    }

    if (parser_check(parser, TOKEN_CHAR)) {
        char char_val = parser->current.text.len > 0 ? parser->current.text.data[0] : '\0';
        parser_match(parser, TOKEN_CHAR);
        expr->kind = EXPR_CHAR;
        expr->u.char_val = char_val;
        return expr;
    }

    if (parser_check(parser, TOKEN_IDENT)) {
        AutoStr ident_val = astr_clone(&parser->current.text);
        parser_match(parser, TOKEN_IDENT);
        expr->kind = EXPR_IDENT;
        expr->u.ident_val = ident_val;
        return expr;
    }

    if (parser_match(parser, TOKEN_LPAREN)) {
        Expr* group = parse_expr_grouping(parser);
        return group;
    }

    if (parser_match(parser, TOKEN_LSQUARE)) {
        // Array literal
        expr->kind = EXPR_ARRAY;
        expr->u.array.elems = NULL;
        expr->u.array.count = 0;
        expr->u.array.capacity = 0;

        while (!parser_check(parser, TOKEN_RSQUARE) && !parser_check(parser, TOKEN_EOF)) {
            Expr* elem = parser_parse_expr(parser);
            if (elem) {
                if (expr->u.array.count >= expr->u.array.capacity) {
                    expr->u.array.capacity = expr->u.array.capacity == 0 ? 8 : expr->u.array.capacity * 2;
                    expr->u.array.elems = (Expr**)realloc(expr->u.array.elems, expr->u.array.capacity * sizeof(Expr*));
                }
                expr->u.array.elems[expr->u.array.count++] = elem;
            }
            if (!parser_match(parser, TOKEN_COMMA)) break;
        }
        parser_expect(parser, TOKEN_RSQUARE, "Expected ']' to close array");
        return expr;
    }

    if (parser_match(parser, TOKEN_LBRACE)) {
        // Object literal
        expr->kind = EXPR_OBJECT;
        expr->u.object.pairs = NULL;
        expr->u.object.count = 0;
        expr->u.object.capacity = 0;

        while (!parser_check(parser, TOKEN_RBRACE) && !parser_check(parser, TOKEN_EOF)) {
            // Parse key (identifier or string)
            AutoStr key = astr_new("");
            if (parser_check(parser, TOKEN_IDENT)) {
                key = astr_clone(&parser->current.text);
                parser_advance(parser);
            } else if (parser_check(parser, TOKEN_STR)) {
                key = astr_clone(&parser->current.text);
                parser_advance(parser);
            }

            parser_expect(parser, TOKEN_COLON, "Expected ':' after object key");

            Expr* value = parser_parse_expr(parser);
            if (value && key.len > 0) {
                if (expr->u.object.count >= expr->u.object.capacity) {
                    expr->u.object.capacity = expr->u.object.capacity == 0 ? 8 : expr->u.object.capacity * 2;
                    expr->u.object.pairs = (Pair*)realloc(expr->u.object.pairs, expr->u.object.capacity * sizeof(Pair));
                }
                Pair p;
                p.key = key;
                p.value = value;
                expr->u.object.pairs[expr->u.object.count++] = p;
            }

            if (!parser_match(parser, TOKEN_COMMA)) break;
        }
        parser_expect(parser, TOKEN_RBRACE, "Expected '}' to close object");
        return expr;
    }

    // Unknown expression
    expr->kind = EXPR_NIL;
    return expr;
}

static Expr* parse_expr_unary(Parser* parser) {
    if (parser_match(parser, TOKEN_ADD) ||
        parser_match(parser, TOKEN_SUB) ||
        parser_match(parser, TOKEN_NOT)) {

        Token op = parser->current; // Already advanced, need to track
        int op_code = 0;
        // Simple hack - in real impl, track the operator properly
        Expr* expr = (Expr*)malloc(sizeof(Expr));
        expr->kind = EXPR_UNARY;
        expr->u.unary.expr = parse_expr_unary(parser);
        return expr;
    }

    return parse_expr_primary(parser);
}

static Expr* parse_expr_binary(Parser* parser, Expr* left, Precedence prec) {
    while (1) {
        TokenKind op = parser->current.kind;
        Precedence next_prec = get_infix_prec(op);
        if (next_prec <= prec) break;

        parser_advance(parser); // Consume operator

        Expr* right = parse_expr_unary(parser);

        // Handle right-associative operators
        Precedence next_next = get_infix_prec(parser->current.kind);
        if (next_prec < next_next) {
            right = parse_expr_binary(parser, right, next_prec);
        }

        Expr* expr = (Expr*)malloc(sizeof(Expr));
        expr->kind = EXPR_BINA;
        expr->pos = parser->current.pos;
        expr->u.bina.left = left;
        expr->u.bina.op = (int)op;
        expr->u.bina.right = right;
        left = expr;
    }

    // Handle postfix operators
    while (1) {
        Precedence post_prec = get_postfix_prec(parser->current.kind);
        if (post_prec <= prec) break;

        if (parser_match(parser, TOKEN_LPAREN)) {
            left = parse_expr_call(parser, left);
        } else if (parser_match(parser, TOKEN_LSQUARE)) {
            // Check if this is an array literal or index operation
            // Look ahead to see if there's a comma (array literal) or just an expression (index)
            bool is_array_literal = false;
            if (parser_check(parser, TOKEN_RSQUARE)) {
                // Empty array [] is an array literal
                is_array_literal = true;
            } else {
                // Parse first element
                Expr* first_elem = parser_parse_expr(parser);
                if (parser_check(parser, TOKEN_COMMA) || parser_check(parser, TOKEN_RSQUARE)) {
                    // Has comma or closing bracket after first expr -> array literal
                    is_array_literal = true;
                    // We've already consumed the first element, need to handle this
                    // For now, create an array with this single element
                    Expr* array_expr = (Expr*)malloc(sizeof(Expr));
                    array_expr->kind = EXPR_ARRAY;
                    array_expr->pos = parser->current.pos;
                    array_expr->u.array.elems = (Expr**)malloc(sizeof(Expr*));
                    array_expr->u.array.elems[0] = first_elem;
                    array_expr->u.array.count = 1;
                    array_expr->u.array.capacity = 1;

                    // If there's a comma, there are more elements
                    if (parser_match(parser, TOKEN_COMMA)) {
                        while (!parser_check(parser, TOKEN_RSQUARE) && !parser_check(parser, TOKEN_EOF)) {
                            Expr* elem = parser_parse_expr(parser);
                            if (elem) {
                                array_expr->u.array.count++;
                                array_expr->u.array.elems = (Expr**)realloc(array_expr->u.array.elems,
                                    array_expr->u.array.count * sizeof(Expr*));
                                array_expr->u.array.elems[array_expr->u.array.count - 1] = elem;
                            }
                            if (!parser_match(parser, TOKEN_COMMA)) break;
                        }
                    }

                    parser_expect(parser, TOKEN_RSQUARE, "Expected ']' to close array");
                    left = array_expr;
                } else {
                    // It's an index operation
                    parser_expect(parser, TOKEN_RSQUARE, "Expected ']' after index");

                    Expr* expr = (Expr*)malloc(sizeof(Expr));
                    expr->kind = EXPR_INDEX;
                    expr->pos = parser->current.pos;
                    expr->u.index.array = left;
                    expr->u.index.index = first_elem;
                    left = expr;
                }
            }

            if (!is_array_literal) {
                // Empty array case - handle as array literal
                Expr* array_expr = (Expr*)malloc(sizeof(Expr));
                array_expr->kind = EXPR_ARRAY;
                array_expr->pos = parser->current.pos;
                array_expr->u.array.elems = NULL;
                array_expr->u.array.count = 0;
                array_expr->u.array.capacity = 0;
                parser_expect(parser, TOKEN_RSQUARE, "Expected ']' to close array");
                left = array_expr;
            }
        } else if (parser_match(parser, TOKEN_DOT)) {
            if (parser_match(parser, TOKEN_IDENT)) {
                // Create a pair expression for dot access
                // In a full implementation, this would be a separate expression type
                // For now, treat as a special BINA
                Expr* expr = (Expr*)malloc(sizeof(Expr));
                expr->kind = EXPR_BINA;
                expr->pos = parser->current.pos;
                expr->u.bina.left = left;
                expr->u.bina.op = (int)TOKEN_DOT;
                expr->u.bina.right = parse_expr_primary(parser);
                left = expr;
            }
        } else {
            break;
        }
    }

    return left;
}

static Expr* parse_expr_call(Parser* parser, Expr* callee) {
    Expr* expr = (Expr*)malloc(sizeof(Expr));
    expr->kind = EXPR_CALL;
    expr->pos = parser->current.pos;
    expr->u.call.callee = callee;
    expr->u.call.args = NULL;
    expr->u.call.count = 0;
    expr->u.call.capacity = 0;

    while (!parser_check(parser, TOKEN_RPAREN) && !parser_check(parser, TOKEN_EOF)) {
        Expr* arg = parser_parse_expr(parser);
        if (arg) {
            if (expr->u.call.count >= expr->u.call.capacity) {
                expr->u.call.capacity = expr->u.call.capacity == 0 ? 8 : expr->u.call.capacity * 2;
                expr->u.call.args = (Expr**)realloc(expr->u.call.args, expr->u.call.capacity * sizeof(Expr*));
            }
            expr->u.call.args[expr->u.call.count++] = arg;
        }
        if (!parser_match(parser, TOKEN_COMMA)) break;
    }

    parser_expect(parser, TOKEN_RPAREN, "Expected ')' to close function call");
    return expr;
}

static Expr* parse_expr_grouping(Parser* parser) {
    Expr* expr = parser_parse_expr(parser);
    parser_expect(parser, TOKEN_RPAREN, "Expected ')' after grouping");
    return expr;
}

// ============================================================================
// Statement Parsing
// ============================================================================

Stmt* parser_parse_stmt(Parser* parser) {
    // Skip empty lines
    while (parser_match(parser, TOKEN_NEWLINE)) {
        // Skip
    }

    if (parser_check(parser, TOKEN_EOF)) {
        return NULL;
    }

    // Check for variable declarations
    if (parser_match(parser, TOKEN_VAR) || parser_match(parser, TOKEN_LET) || parser_match(parser, TOKEN_MUT)) {
        // TODO: Track which keyword was used
        if (parser_check(parser, TOKEN_IDENT)) {
            AutoStr name = astr_clone(&parser->current.text);
            parser_advance(parser);

            Type* ty = NULL; // TODO: Parse type annotation

            parser_expect(parser, TOKEN_ASN, "Expected '=' after variable name");

            Expr* init = parser_parse_expr(parser);

            Stmt* stmt = (Stmt*)malloc(sizeof(Stmt));
            stmt->kind = STMT_STORE;
            stmt->u.store.name = name;
            stmt->u.store.ty = ty;
            stmt->u.store.expr = init;

            // Skip trailing newline/semicolon
            parser_match(parser, TOKEN_NEWLINE);
            parser_match(parser, TOKEN_SEMI);

            return stmt;
        }
    }

    // Check for if statement
    if (parser_match(parser, TOKEN_IF)) {
        Expr* cond = parser_parse_expr(parser);

        Stmt* then_block = parser_parse_stmt(parser);
        Stmt* else_block = NULL;

        if (parser_match(parser, TOKEN_ELSE)) {
            else_block = parser_parse_stmt(parser);
        }

        Stmt* stmt = (Stmt*)malloc(sizeof(Stmt));
        stmt->kind = STMT_IF;
        stmt->u.if_stmt.cond = cond;
        stmt->u.if_stmt.then_body = then_block;
        stmt->u.if_stmt.else_body = else_block;

        return stmt;
    }

    // Check for for loop
    if (parser_match(parser, TOKEN_FOR)) {
        AutoStr var_name = astr_new("");
        if (parser_check(parser, TOKEN_IDENT)) {
            var_name = astr_clone(&parser->current.text);
            parser_advance(parser);
        }

        parser_expect(parser, TOKEN_IN, "Expected 'in' in for loop");

        Expr* iter = parser_parse_expr(parser);

        Stmt* body = parser_parse_stmt(parser);

        Stmt* stmt = (Stmt*)malloc(sizeof(Stmt));
        stmt->kind = STMT_FOR;
        stmt->u.for_stmt.var_name = var_name;
        stmt->u.for_stmt.iter = iter;
        stmt->u.for_stmt.body = body;

        return stmt;
    }

    // Block statement
    if (parser_match(parser, TOKEN_LBRACE)) {
        Stmt** stmts = NULL;
        size_t count = 0;
        size_t capacity = 0;

        while (!parser_check(parser, TOKEN_RBRACE) && !parser_check(parser, TOKEN_EOF)) {
            Stmt* s = parser_parse_stmt(parser);
            if (s) {
                if (count >= capacity) {
                    capacity = capacity == 0 ? 16 : capacity * 2;
                    stmts = (Stmt**)realloc(stmts, capacity * sizeof(Stmt*));
                }
                stmts[count++] = s;
            }
        }

        parser_expect(parser, TOKEN_RBRACE, "Expected '}' to close block");

        // Convert block to STMT_BLOCK or STMT_EXPR
        Stmt* stmt = (Stmt*)malloc(sizeof(Stmt));
        stmt->kind = STMT_BLOCK;
        stmt->u.block.stmts = stmts;
        stmt->u.block.count = count;

        return stmt;
    }

    // Expression statement
    Expr* expr = parser_parse_expr(parser);

    // Skip trailing newline/semicolon
    parser_match(parser, TOKEN_NEWLINE);
    parser_match(parser, TOKEN_SEMI);

    if (expr) {
        Stmt* stmt = (Stmt*)malloc(sizeof(Stmt));
        stmt->kind = STMT_EXPR;
        stmt->u.expr = expr;
        return stmt;
    }

    return NULL;
}

// ============================================================================
// Code Parsing
// ============================================================================

Code* parser_parse(Parser* parser) {
    Code* code = (Code*)malloc(sizeof(Code));
    code->stmts = NULL;
    code->count = 0;
    code->capacity = 0;

    while (!parser_check(parser, TOKEN_EOF)) {
        Stmt* stmt = parser_parse_stmt(parser);
        if (stmt) {
            if (code->count >= code->capacity) {
                code->capacity = code->capacity == 0 ? 64 : code->capacity * 2;
                code->stmts = (Stmt**)realloc(code->stmts, code->capacity * sizeof(Stmt*));
            }
            code->stmts[code->count++] = stmt;
        }
    }

    return code;
}
