/**
 * AST String Representation
 * AutoLang atom format representation for AST nodes
 */

#include "autoc.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// ============================================================================
// Helper Functions
// ============================================================================

// Buffer for building string representations
typedef struct {
    char* data;
    size_t len;
    size_t capacity;
} ReprBuffer;

static ReprBuffer repr_buf_new(size_t initial_capacity) {
    ReprBuffer buf;
    buf.data = (char*)malloc(initial_capacity);
    buf.len = 0;
    buf.capacity = initial_capacity;
    buf.data[0] = '\0';
    return buf;
}

static void repr_buf_free(ReprBuffer* buf) {
    free(buf->data);
    buf->data = NULL;
    buf->len = 0;
    buf->capacity = 0;
}

static void repr_buf_append(ReprBuffer* buf, const char* s) {
    size_t slen = strlen(s);
    while (buf->len + slen + 1 > buf->capacity) {
        buf->capacity *= 2;
        buf->data = (char*)realloc(buf->data, buf->capacity);
    }
    strcpy(buf->data + buf->len, s);
    buf->len += slen;
}

static void repr_buf_append_int(ReprBuffer* buf, int64_t val) {
    char temp[32];
    snprintf(temp, sizeof(temp), "%lld", val);
    repr_buf_append(buf, temp);
}

static void repr_buf_append_uint(ReprBuffer* buf, uint64_t val) {
    char temp[32];
    snprintf(temp, sizeof(temp), "%llu", val);
    repr_buf_append(buf, temp);
}

static void repr_buf_append_double(ReprBuffer* buf, double val) {
    char temp[64];
    snprintf(temp, sizeof(temp), "%g", val);
    repr_buf_append(buf, temp);
}

static void repr_buf_append_str(ReprBuffer* buf, const char* s) {
    repr_buf_append(buf, "\"");
    repr_buf_append(buf, s);
    repr_buf_append(buf, "\"");
}

// ============================================================================
// Type Representation
// ============================================================================

static const char* type_kind_name(TypeKind kind) {
    switch (kind) {
        case TYPE_BYTE: return "byte";
        case TYPE_INT: return "int";
        case TYPE_UINT: return "uint";
        case TYPE_FLOAT: return "float";
        case TYPE_DOUBLE: return "double";
        case TYPE_BOOL: return "bool";
        case TYPE_CHAR: return "char";
        case TYPE_STR: return "str";
        case TYPE_CSTR: return "cstr";
        case TYPE_ARRAY: return "array";
        case TYPE_PTR: return "ptr";
        case TYPE_VOID: return "void";
        case TYPE_UNKNOWN: return "unknown";
        case TYPE_USER: return "user";
        default: return ".";
    }
}

char* type_to_string(Type* type) {
    if (!type) return strdup("Type(null)");

    ReprBuffer buf = repr_buf_new(128);
    repr_buf_append(&buf, "Type(");

    // Add kind
    repr_buf_append(&buf, "kind: ");
    repr_buf_append(&buf, type_kind_name(type->kind));

    // Add name if user-defined type
    if (type->kind == TYPE_USER && type->name.data) {
        repr_buf_append(&buf, ", name: ");
        repr_buf_append(&buf, type->name.data);
    }

    // Add element type for array/ptr
    if ((type->kind == TYPE_ARRAY || type->kind == TYPE_PTR) && type->elem_type) {
        char* elem_str = type_to_string(type->elem_type);
        repr_buf_append(&buf, ", elem: ");
        repr_buf_append(&buf, elem_str);
        free(elem_str);
    }

    repr_buf_append(&buf, ")");
    return buf.data;
}

const char* type_repr(Type* type) {
    static char buffer[512];
    char* str = type_to_string(type);
    strncpy(buffer, str, sizeof(buffer) - 1);
    buffer[sizeof(buffer) - 1] = '\0';
    free(str);
    return buffer;
}

// ============================================================================
// Expression Representation
// ============================================================================

static const char* expr_kind_name(ExprKind kind) {
    switch (kind) {
        case EXPR_BYTE: return "expr.byte";
        case EXPR_INT: return "expr.int";
        case EXPR_UINT: return "expr.uint";
        case EXPR_I8: return "expr.i8";
        case EXPR_U8: return "expr.u8";
        case EXPR_I64: return "expr.i64";
        case EXPR_FLOAT: return "expr.float";
        case EXPR_DOUBLE: return "expr.double";
        case EXPR_BOOL: return "expr.bool";
        case EXPR_CHAR: return "expr.char";
        case EXPR_STR: return "expr.str";
        case EXPR_CSTR: return "expr.cstr";
        case EXPR_IDENT: return "expr.ident";
        case EXPR_REF: return "expr.ref";
        case EXPR_UNARY: return "expr.unary";
        case EXPR_BINA: return "expr.binary";
        case EXPR_RANGE: return "expr.range";
        case EXPR_ARRAY: return "expr.array";
        case EXPR_PAIR: return "expr.pair";
        case EXPR_BLOCK: return "expr.block";
        case EXPR_OBJECT: return "expr.object";
        case EXPR_CALL: return "expr.call";
        case EXPR_INDEX: return "expr.index";
        case EXPR_IF: return "expr.if";
        case EXPR_NIL: return "expr.nil";
        case EXPR_NULL: return "expr.null";
        default: return "expr.other";
    }
}

static const char* token_kind_name(TokenKind kind) {
    switch (kind) {
        case TOKEN_ADD: return "+";
        case TOKEN_SUB: return "-";
        case TOKEN_STAR: return "*";
        case TOKEN_DIV: return "/";
        case TOKEN_NOT: return "!";
        case TOKEN_EQ: return "==";
        case TOKEN_NEQ: return "!=";
        case TOKEN_LT: return "<";
        case TOKEN_GT: return ">";
        case TOKEN_LE: return "<=";
        case TOKEN_GE: return ">=";
        case TOKEN_ASN: return "=";
        case TOKEN_ADDEQ: return "+=";
        case TOKEN_SUBEQ: return "-=";
        case TOKEN_MULEQ: return "*=";
        case TOKEN_DIVEQ: return "/=";
        case TOKEN_RANGE: return "..";
        case TOKEN_RANGEEQ: return "..=";
        case TOKEN_DOT: return ".";
        default: return "?";
    }
}

char* expr_to_string(Expr* expr) {
    if (!expr) return strdup("Expr(null)");

    ReprBuffer buf = repr_buf_new(256);
    repr_buf_append(&buf, expr_kind_name(expr->kind));
    repr_buf_append(&buf, "(");

    bool has_brace_content = false;  // Track if expr already added ") { ... }"

    switch (expr->kind) {
        case EXPR_BYTE:
            repr_buf_append(&buf, "value: ");
            repr_buf_append_uint(&buf, expr->u.byte_val);
            break;

        case EXPR_INT:
            repr_buf_append(&buf, "value: ");
            repr_buf_append_int(&buf, expr->u.int_val);
            break;

        case EXPR_UINT:
        case EXPR_U8:
            repr_buf_append(&buf, "value: ");
            repr_buf_append_uint(&buf, expr->u.uint_val);
            break;

        case EXPR_I8:
            repr_buf_append(&buf, "value: ");
            repr_buf_append_int(&buf, expr->u.i8_val);
            break;

        case EXPR_I64:
            repr_buf_append(&buf, "value: ");
            repr_buf_append_int(&buf, expr->u.i64_val);
            break;

        case EXPR_FLOAT:
        case EXPR_DOUBLE:
            repr_buf_append(&buf, "value: ");
            repr_buf_append_double(&buf, expr->u.float_val);
            break;

        case EXPR_BOOL:
            repr_buf_append(&buf, "value: ");
            repr_buf_append(&buf, expr->u.bool_val ? "true" : "false");
            break;

        case EXPR_CHAR:
            repr_buf_append(&buf, "value: ");
            repr_buf_append(&buf, "'");
            char char_buf[2] = {expr->u.char_val, '\0'};
            repr_buf_append(&buf, char_buf);
            repr_buf_append(&buf, "'");
            break;

        case EXPR_STR:
        case EXPR_CSTR:
            repr_buf_append(&buf, "value: ");
            repr_buf_append_str(&buf, expr->u.str_val.data);
            break;

        case EXPR_IDENT:
            repr_buf_append(&buf, "name: ");
            repr_buf_append(&buf, expr->u.ident_val.data);
            break;

        case EXPR_UNARY:
            repr_buf_append(&buf, "op: ");
            repr_buf_append(&buf, token_kind_name((TokenKind)expr->u.unary.op));
            if (expr->u.unary.expr) {
                char* sub_str = expr_to_string(expr->u.unary.expr);
                repr_buf_append(&buf, ") { ");
                repr_buf_append(&buf, sub_str);
                repr_buf_append(&buf, " }");
                free(sub_str);
            }
            break;

        case EXPR_BINA: {
            repr_buf_append(&buf, "op: ");
            repr_buf_append(&buf, token_kind_name((TokenKind)expr->u.bina.op));
            if (expr->u.bina.left) {
                char* left_str = expr_to_string(expr->u.bina.left);
                if (expr->u.bina.right) {
                    char* right_str = expr_to_string(expr->u.bina.right);
                    repr_buf_append(&buf, ") { ");
                    repr_buf_append(&buf, left_str);
                    repr_buf_append(&buf, ", ");
                    repr_buf_append(&buf, right_str);
                    repr_buf_append(&buf, " }");
                    free(right_str);
                } else {
                    repr_buf_append(&buf, ") { ");
                    repr_buf_append(&buf, left_str);
                    repr_buf_append(&buf, " }");
                }
                free(left_str);
                has_brace_content = true;
            }
            break;
        }

        case EXPR_RANGE: {
            repr_buf_append(&buf, "eq: ");
            repr_buf_append(&buf, expr->u.range.eq ? "true" : "false");
            if (expr->u.range.start || expr->u.range.end) {
                repr_buf_append(&buf, ") { ");
                if (expr->u.range.start) {
                    char* start_str = expr_to_string(expr->u.range.start);
                    repr_buf_append(&buf, "start: ");
                    repr_buf_append(&buf, start_str);
                    free(start_str);
                }
                if (expr->u.range.end) {
                    if (expr->u.range.start) repr_buf_append(&buf, ", ");
                    char* end_str = expr_to_string(expr->u.range.end);
                    repr_buf_append(&buf, "end: ");
                    repr_buf_append(&buf, end_str);
                    free(end_str);
                }
                repr_buf_append(&buf, " }");
                has_brace_content = true;
            }
            break;
        }

        case EXPR_ARRAY: {
            repr_buf_append(&buf, "count: ");
            repr_buf_append_uint(&buf, expr->u.array.count);
            if (expr->u.array.count > 0) {
                repr_buf_append(&buf, ") { ");
                for (size_t i = 0; i < expr->u.array.count; i++) {
                    if (i > 0) repr_buf_append(&buf, ", ");
                    char* elem_str = expr_to_string(expr->u.array.elems[i]);
                    repr_buf_append(&buf, elem_str);
                    free(elem_str);
                }
                repr_buf_append(&buf, " }");
                has_brace_content = true;
            }
            break;
        }

        case EXPR_PAIR: {
            if (expr->u.pair.key.data || expr->u.pair.value) {
                repr_buf_append(&buf, ") { ");
                if (expr->u.pair.key.data) {
                    repr_buf_append(&buf, "key: ");
                    repr_buf_append_str(&buf, expr->u.pair.key.data);
                }
                if (expr->u.pair.value) {
                    if (expr->u.pair.key.data) repr_buf_append(&buf, ", ");
                    char* value_str = expr_to_string(expr->u.pair.value);
                    repr_buf_append(&buf, "value: ");
                    repr_buf_append(&buf, value_str);
                    free(value_str);
                }
                repr_buf_append(&buf, " }");
                has_brace_content = true;
            }
            break;
        }

        case EXPR_BLOCK: {
            repr_buf_append(&buf, "count: ");
            repr_buf_append_uint(&buf, expr->u.block.count);
            if (expr->u.block.count > 0) {
                repr_buf_append(&buf, ") { ");
                for (size_t i = 0; i < expr->u.block.count; i++) {
                    if (i > 0) repr_buf_append(&buf, ", ");
                    char* stmt_str = expr_to_string(expr->u.block.stmts[i]);
                    repr_buf_append(&buf, stmt_str);
                    free(stmt_str);
                }
                repr_buf_append(&buf, " }");
                has_brace_content = true;
            }
            break;
        }

        case EXPR_OBJECT: {
            repr_buf_append(&buf, "count: ");
            repr_buf_append_uint(&buf, expr->u.object.count);
            if (expr->u.object.count > 0) {
                repr_buf_append(&buf, ") { ");
                for (size_t i = 0; i < expr->u.object.count; i++) {
                    if (i > 0) repr_buf_append(&buf, ", ");
                    Pair* p = &expr->u.object.pairs[i];
                    if (p->key.data) {
                        repr_buf_append_str(&buf, p->key.data);
                    }
                    repr_buf_append(&buf, ": ");
                    if (p->value) {
                        char* value_str = expr_to_string(p->value);
                        repr_buf_append(&buf, value_str);
                        free(value_str);
                    }
                }
                repr_buf_append(&buf, " }");
                has_brace_content = true;
            }
            break;
        }

        case EXPR_CALL: {
            if (expr->u.call.callee) {
                char* callee_str = expr_to_string(expr->u.call.callee);
                repr_buf_append(&buf, "callee: ");
                repr_buf_append(&buf, callee_str);
                free(callee_str);
            }
            repr_buf_append(&buf, ", args: ");
            repr_buf_append_uint(&buf, expr->u.call.count);
            if (expr->u.call.count > 0) {
                repr_buf_append(&buf, ") { ");
                for (size_t i = 0; i < expr->u.call.count; i++) {
                    if (i > 0) repr_buf_append(&buf, ", ");
                    char* arg_str = expr_to_string(expr->u.call.args[i]);
                    repr_buf_append(&buf, arg_str);
                    free(arg_str);
                }
                repr_buf_append(&buf, " }");
                has_brace_content = true;
            }
            break;
        }

        case EXPR_INDEX: {
            bool has_prev = false;
            if (expr->u.index.array) {
                char* array_str = expr_to_string(expr->u.index.array);
                repr_buf_append(&buf, "array: ");
                repr_buf_append(&buf, array_str);
                free(array_str);
                has_prev = true;
            }
            if (expr->u.index.index) {
                if (has_prev) repr_buf_append(&buf, ", ");
                char* index_str = expr_to_string(expr->u.index.index);
                repr_buf_append(&buf, "index: ");
                repr_buf_append(&buf, index_str);
                free(index_str);
            }
            break;
        }

        case EXPR_IF: {
            bool has_prev = false;
            if (expr->u.if_expr.cond) {
                char* cond_str = expr_to_string(expr->u.if_expr.cond);
                repr_buf_append(&buf, "cond: ");
                repr_buf_append(&buf, cond_str);
                free(cond_str);
                has_prev = true;
            }
            if (expr->u.if_expr.then_body) {
                if (has_prev) repr_buf_append(&buf, ", ");
                char* then_str = expr_to_string(expr->u.if_expr.then_body);
                repr_buf_append(&buf, "then: ");
                repr_buf_append(&buf, then_str);
                free(then_str);
                has_prev = true;
            }
            if (expr->u.if_expr.else_body) {
                if (has_prev) repr_buf_append(&buf, ", ");
                char* else_str = expr_to_string(expr->u.if_expr.else_body);
                repr_buf_append(&buf, "else: ");
                repr_buf_append(&buf, else_str);
                free(else_str);
            }
            break;
        }

        case EXPR_NIL:
            repr_buf_append(&buf, "value: nil");
            break;

        case EXPR_NULL:
            repr_buf_append(&buf, "value: null");
            break;

        default:
            repr_buf_append(&buf, "...");
            break;
    }

    // Only add closing ")" if the expression didn't already add ") { ... }"
    if (!has_brace_content) {
        repr_buf_append(&buf, ")");
    }

    return buf.data;
}

const char* expr_repr(Expr* expr) {
    static char buffer[4096];
    char* str = expr_to_string(expr);
    strncpy(buffer, str, sizeof(buffer) - 1);
    buffer[sizeof(buffer) - 1] = '\0';
    free(str);
    return buffer;
}

// ============================================================================
// Statement Representation
// ============================================================================

static const char* stmt_kind_name(StmtKind kind) {
    switch (kind) {
        case STMT_EXPR: return "stmt.expr";
        case STMT_IF: return "stmt.if";
        case STMT_FOR: return "stmt.for";
        case STMT_STORE: return "stmt.store";
        case STMT_BLOCK: return "stmt.block";
        case STMT_FN: return "stmt.fn";
        case STMT_EMPTY_LINE: return "stmt.empty_line";
        case STMT_BREAK: return "stmt.break";
        default: return "stmt.";
    }
}

char* stmt_to_string(Stmt* stmt) {
    if (!stmt) return strdup("Stmt(null)");

    ReprBuffer buf = repr_buf_new(256);
    repr_buf_append(&buf, stmt_kind_name(stmt->kind));
    repr_buf_append(&buf, "(");

    bool has_brace_content = false;  // Track if stmt already added ") { ... }"

    switch (stmt->kind) {
        case STMT_EXPR:
            if (stmt->u.expr) {
                char* expr_str = expr_to_string(stmt->u.expr);
                repr_buf_append(&buf, ") { ");
                repr_buf_append(&buf, expr_str);
                repr_buf_append(&buf, " }");
                free(expr_str);
                has_brace_content = true;
            }
            break;

        case STMT_IF: {
            bool has_prev = false;
            if (stmt->u.if_stmt.cond) {
                char* cond_str = expr_to_string(stmt->u.if_stmt.cond);
                repr_buf_append(&buf, "cond: ");
                repr_buf_append(&buf, cond_str);
                free(cond_str);
                has_prev = true;
            }
            if (stmt->u.if_stmt.then_body) {
                if (has_prev) repr_buf_append(&buf, ", ");
                char* then_str = expr_to_string(stmt->u.if_stmt.then_body);
                repr_buf_append(&buf, "then: ");
                repr_buf_append(&buf, then_str);
                free(then_str);
                has_prev = true;
            }
            if (stmt->u.if_stmt.else_body) {
                if (has_prev) repr_buf_append(&buf, ", ");
                char* else_str = expr_to_string(stmt->u.if_stmt.else_body);
                repr_buf_append(&buf, "else: ");
                repr_buf_append(&buf, else_str);
                free(else_str);
            }
            break;
        }

        case STMT_FOR: {
            bool has_prev = false;
            if (stmt->u.for_stmt.var_name.data) {
                repr_buf_append(&buf, "var: ");
                repr_buf_append(&buf, stmt->u.for_stmt.var_name.data);
                has_prev = true;
            }
            if (stmt->u.for_stmt.iter) {
                if (has_prev) repr_buf_append(&buf, ", ");
                char* iter_str = expr_to_string(stmt->u.for_stmt.iter);
                repr_buf_append(&buf, "iter: ");
                repr_buf_append(&buf, iter_str);
                free(iter_str);
                has_prev = true;
            }
            if (stmt->u.for_stmt.body) {
                // No comma before body since it includes ") {"
                char* body_str = expr_to_string(stmt->u.for_stmt.body);
                repr_buf_append(&buf, ") { ");
                repr_buf_append(&buf, body_str);
                repr_buf_append(&buf, " }");
                free(body_str);
                has_brace_content = true;
            }
            break;
        }

        case STMT_STORE: {
            bool has_prev = false;
            if (stmt->u.store.name.data) {
                repr_buf_append(&buf, "name: ");
                repr_buf_append(&buf, stmt->u.store.name.data);
                has_prev = true;
            }
            if (stmt->u.store.ty) {
                if (has_prev) repr_buf_append(&buf, ", ");
                char* type_str = type_to_string(stmt->u.store.ty);
                repr_buf_append(&buf, "type: ");
                repr_buf_append(&buf, type_str);
                free(type_str);
                has_prev = true;
            }
            if (stmt->u.store.expr) {
                // No comma before expr since it includes ") {"
                char* expr_str = expr_to_string(stmt->u.store.expr);
                repr_buf_append(&buf, ") { ");
                repr_buf_append(&buf, expr_str);
                repr_buf_append(&buf, " }");
                free(expr_str);
                has_brace_content = true;
            } else {
                repr_buf_append(&buf, ")");
            }
            break;
        }

        case STMT_BLOCK: {
            repr_buf_append(&buf, "count: ");
            repr_buf_append_uint(&buf, stmt->u.block.count);
            if (stmt->u.block.count > 0) {
                repr_buf_append(&buf, ") { ");
                for (size_t i = 0; i < stmt->u.block.count; i++) {
                    if (i > 0) repr_buf_append(&buf, ", ");
                    char* stmt_str = stmt_to_string(stmt->u.block.stmts[i]);
                    repr_buf_append(&buf, stmt_str);
                    free(stmt_str);
                }
                repr_buf_append(&buf, " }");
                has_brace_content = true;
            }
            break;
        }

        case STMT_FN:
        case STMT_EMPTY_LINE:
        case STMT_BREAK:
            // No additional fields
            break;

        default:
            repr_buf_append(&buf, "...");
            break;
    }

    // Only add closing ")" if the statement didn't already add ") { ... }"
    if (!has_brace_content) {
        repr_buf_append(&buf, ")");
    }

    return buf.data;
}

const char* stmt_repr(Stmt* stmt) {
    static char buffer[4096];
    char* str = stmt_to_string(stmt);
    strncpy(buffer, str, sizeof(buffer) - 1);
    buffer[sizeof(buffer) - 1] = '\0';
    free(str);
    return buffer;
}

// ============================================================================
// Code Representation
// ============================================================================

char* code_to_string(Code* code) {
    if (!code) return strdup("Code(null)");

    ReprBuffer buf = repr_buf_new(256);
    repr_buf_append(&buf, "Code(count: ");
    repr_buf_append_uint(&buf, code->count);

    if (code->count > 0) {
        repr_buf_append(&buf, ") { ");
        for (size_t i = 0; i < code->count; i++) {
            if (i > 0) repr_buf_append(&buf, ", ");
            char* stmt_str = stmt_to_string(code->stmts[i]);
            repr_buf_append(&buf, stmt_str);
            free(stmt_str);
        }
        repr_buf_append(&buf, " }");
    } else {
        repr_buf_append(&buf, ")");
    }

    return buf.data;
}

const char* code_repr(Code* code) {
    static char buffer[8192];
    char* str = code_to_string(code);
    strncpy(buffer, str, sizeof(buffer) - 1);
    buffer[sizeof(buffer) - 1] = '\0';
    free(str);
    return buffer;
}
