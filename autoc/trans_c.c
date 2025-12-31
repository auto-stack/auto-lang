/**
 * C Transpiler Implementation
 * Translates auto-lang AST to C code
 */

#include "trans_c.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Windows compatibility: open_memstream is not available on Windows
#ifdef _WIN32
#define USE_CUSTOM_MEMSTREAM 1
#else
#define USE_CUSTOM_MEMSTREAM 0
#endif

#if USE_CUSTOM_MEMSTREAM
// Simple memstream implementation for Windows
typedef struct {
    char** buf_ptr;
    size_t* size_ptr;
    size_t capacity;
    size_t pos;
} MemStream;

static int memstream_write(void* cookie, const char* data, int size) {
    MemStream* stream = (MemStream*)cookie;
    size_t new_size = stream->pos + size;

    if (new_size >= stream->capacity) {
        stream->capacity = new_size * 2 + 4096;
        *stream->buf_ptr = (char*)realloc(*stream->buf_ptr, stream->capacity);
    }

    memcpy(*stream->buf_ptr + stream->pos, data, size);
    stream->pos += size;
    *stream->size_ptr = stream->pos;
    return size;
}

static fpos_t memstream_seek(void* cookie, fpos_t offset, int whence) {
    MemStream* stream = (MemStream*)cookie;
    size_t new_pos;

    switch (whence) {
        case SEEK_SET:
            new_pos = offset;
            break;
        case SEEK_CUR:
            new_pos = stream->pos + offset;
            break;
        case SEEK_END:
            new_pos = *stream->size_ptr + offset;
            break;
        default:
            return -1;
    }

    if (new_pos > stream->capacity) {
        stream->capacity = new_pos * 2 + 4096;
        *stream->buf_ptr = (char*)realloc(*stream->buf_ptr, stream->capacity);
    }

    stream->pos = new_pos;
    if (stream->pos > *stream->size_ptr) {
        *stream->size_ptr = stream->pos;
    }
    return stream->pos;
}

static int memstream_close(void* cookie) {
    free(cookie);
    return 0;
}

static FILE* open_memstream_win(char** buf, size_t* size) {
    *buf = NULL;
    *size = 0;

    MemStream* cookie = (MemStream*)malloc(sizeof(MemStream));
    cookie->buf_ptr = buf;
    cookie->size_ptr = size;
    cookie->capacity = 0;
    cookie->pos = 0;

    // Create a custom file stream
    // Note: This is a simplified version - full implementation would use funopen on BSD
    // or _fdopen on Windows with custom I/O
    return NULL;  // Fallback to direct buffer writing
}
#endif

// ============================================================================
// C Transpiler Creation
// ============================================================================

CTrans* ctrans_new(const char* name, Universe* universe) {
    CTrans* trans = (CTrans*)malloc(sizeof(CTrans));
    trans->header = NULL;
    trans->source = NULL;
    trans->header_buf = NULL;
    trans->source_buf = NULL;
    trans->header_size = 0;
    trans->header_capacity = 0;
    trans->source_size = 0;
    trans->source_capacity = 0;
    trans->name = strdup(name);
    trans->indent = 0;
    trans->style = C_STYLE_MODERN;
    trans->libs = NULL;
    trans->lib_count = 0;
    trans->lib_capacity = 0;
    trans->universe = universe;
    return trans;
}

void ctrans_free(CTrans* trans) {
    if (!trans) return;

    if (trans->header_buf) free(trans->header_buf);
    if (trans->source_buf) free(trans->source_buf);
    if (trans->name) free(trans->name);
    if (trans->libs) {
        for (size_t i = 0; i < trans->lib_count; i++) {
            free(trans->libs[i]);
        }
        free(trans->libs);
    }
    free(trans);
}

void ctrans_set_style(CTrans* trans, CStyle style) {
    trans->style = style;
}

// ============================================================================
// Utility Functions
// ============================================================================

int ctrans_indent(CTrans* trans) {
    trans->indent++;
    return 0;
}

int ctrans_dedent(CTrans* trans) {
    if (trans->indent > 0) trans->indent--;
    return 0;
}

int ctrans_print_indent(CTrans* trans, FILE* out) {
    for (int i = 0; i < trans->indent; i++) {
        fprintf(out, "    ");
    }
    return 0;
}

int ctrans_write(CTrans* trans, FILE* out, const char* text) {
    if (out == trans->header) {
        // Write to header buffer
        size_t len = strlen(text);
        if (trans->header_size + len >= trans->header_capacity) {
            trans->header_capacity = trans->header_capacity == 0 ? 4096 : trans->header_capacity * 2;
            trans->header_buf = (char*)realloc(trans->header_buf, trans->header_capacity);
        }
        memcpy(trans->header_buf + trans->header_size, text, len + 1);
        trans->header_size += len;
    } else {
        // Write to source buffer
        size_t len = strlen(text);
        if (trans->source_size + len >= trans->source_capacity) {
            trans->source_capacity = trans->source_capacity == 0 ? 4096 : trans->source_capacity * 2;
            trans->source_buf = (char*)realloc(trans->source_buf, trans->source_capacity);
        }
        memcpy(trans->source_buf + trans->source_size, text, len + 1);
        trans->source_size += len;
    }
    fprintf(out, "%s", text);
    return 0;
}

int ctrans_write_header(CTrans* trans, const char* text) {
    return ctrans_write(trans, trans->header, text);
}

int ctrans_eos(CTrans* trans, FILE* out) {
    return ctrans_write(trans, out, ";\n");
}

void ctrans_add_lib(CTrans* trans, const char* lib) {
    for (size_t i = 0; i < trans->lib_count; i++) {
        if (strcmp(trans->libs[i], lib) == 0) return;
    }
    if (trans->lib_count >= trans->lib_capacity) {
        trans->lib_capacity = trans->lib_capacity == 0 ? 8 : trans->lib_capacity * 2;
        trans->libs = (char**)realloc(trans->libs, trans->lib_capacity * sizeof(char*));
    }
    trans->libs[trans->lib_count++] = strdup(lib);
}

// ============================================================================
// Type Name Conversion
// ============================================================================

const char* ctrans_type_name(CTrans* trans, Type* ty) {
    if (!ty) return "void";

    switch (ty->kind) {
        case TYPE_BYTE: return "uint8_t";
        case TYPE_INT: return "int";
        case TYPE_UINT: return "unsigned int";
        case TYPE_FLOAT: return "float";
        case TYPE_DOUBLE: return "double";
        case TYPE_BOOL: return "bool";
        case TYPE_CHAR: return "char";
        case TYPE_STR: return "char*";
        case TYPE_CSTR: return "char*";
        case TYPE_ARRAY: return "array";  // Simplified
        case TYPE_PTR: return "ptr";
        case TYPE_VOID: return "void";
        case TYPE_USER: return ty->name.data;
        default: return "void";
    }
}

// ============================================================================
// Expression Transpilation
// ============================================================================

int ctrans_expr(CTrans* trans, Expr* expr, FILE* out) {
    if (!expr) return 0;

    switch (expr->kind) {
        case EXPR_INT:
            return ctrans_expr_int(trans, expr->u.int_val, out);

        case EXPR_UINT:
            fprintf(out, "%uu", expr->u.uint_val);
            return 0;

        case EXPR_DOUBLE:
            return ctrans_expr_float(trans, expr->u.float_val, out);

        case EXPR_BOOL:
            fprintf(out, "%s", expr->u.bool_val ? "true" : "false");
            return 0;

        case EXPR_CHAR:
            fprintf(out, "'%c'", expr->u.char_val);
            return 0;

        case EXPR_STR:
            return ctrans_expr_str(trans, expr->u.str_val.data, out);

        case EXPR_CSTR:
            return ctrans_expr_str(trans, expr->u.str_val.data, out);

        case EXPR_NIL:
            fprintf(out, "NULL");
            return 0;

        case EXPR_NULL:
            fprintf(out, "NULL");
            return 0;

        case EXPR_IDENT:
            return ctrans_expr_ident(trans, expr->u.ident_val.data, out);

        case EXPR_UNARY:
            return ctrans_expr_unary(trans, expr->u.unary.op, expr->u.unary.expr, out);

        case EXPR_BINA:
            return ctrans_expr_binary(trans, expr->u.bina.left, expr->u.bina.op, expr->u.bina.right, out);

        case EXPR_CALL:
            return ctrans_expr_call(trans, expr->u.call.callee, expr->u.call.args, expr->u.call.count, out);

        case EXPR_ARRAY:
            return ctrans_expr_array(trans, expr->u.array.elems, expr->u.array.count, out);

        case EXPR_OBJECT:
            return ctrans_expr_object(trans, expr->u.object.pairs, expr->u.object.count, out);

        case EXPR_INDEX:
            return ctrans_expr_index(trans, expr->u.index.array, expr->u.index.index, out);

        case EXPR_IF:
            // If expression as ternary
            fprintf(out, "(");
            ctrans_expr(trans, expr->u.if_expr.cond, out);
            fprintf(out, ") ? (");
            ctrans_expr(trans, expr->u.if_expr.then_body, out);
            fprintf(out, ") : (");
            if (expr->u.if_expr.else_body) {
                ctrans_expr(trans, expr->u.if_expr.else_body, out);
            } else {
                fprintf(out, "NULL");
            }
            fprintf(out, ")");
            return 0;

        default:
            fprintf(stderr, "Unsupported expression kind: %d\n", expr->kind);
            return -1;
    }
}

int ctrans_expr_int(CTrans* trans, int32_t value, FILE* out) {
    fprintf(out, "%d", value);
    return 0;
}

int ctrans_expr_float(CTrans* trans, double value, FILE* out) {
    fprintf(out, "%g", value);
    return 0;
}

int ctrans_expr_str(CTrans* trans, const char* value, FILE* out) {
    fprintf(out, "\"%s\"", value);
    return 0;
}

int ctrans_expr_ident(CTrans* trans, const char* name, FILE* out) {
    fprintf(out, "%s", name);
    return 0;
}

int ctrans_expr_unary(CTrans* trans, int op, Expr* expr, FILE* out) {
    const char* op_str = "";
    switch (op) {
        case TOKEN_ADD: op_str = "+"; break;
        case TOKEN_SUB: op_str = "-"; break;
        case TOKEN_NOT: op_str = "!"; break;
        default: op_str = "?"; break;
    }
    fprintf(out, "%s", op_str);
    return ctrans_expr(trans, expr, out);
}

int ctrans_expr_binary(CTrans* trans, Expr* left, int op, Expr* right, FILE* out) {
    const char* op_str = "?";
    switch (op) {
        case TOKEN_ADD: op_str = " + "; break;
        case TOKEN_SUB: op_str = " - "; break;
        case TOKEN_STAR: op_str = " * "; break;
        case TOKEN_DIV: op_str = " / "; break;
        case TOKEN_EQ: op_str = " == "; break;
        case TOKEN_NEQ: op_str = " != "; break;
        case TOKEN_LT: op_str = " < "; break;
        case TOKEN_GT: op_str = " > "; break;
        case TOKEN_LE: op_str = " <= "; break;
        case TOKEN_GE: op_str = " >= "; break;
        case TOKEN_ASN: op_str = " = "; break;
        case TOKEN_DOT: op_str = "."; break;
        default: op_str = " ? "; break;
    }

    if (op == (int)TOKEN_DOT) {
        // Special handling for dot access
        ctrans_expr(trans, left, out);
        fprintf(out, "%s", op_str);
        ctrans_expr(trans, right, out);
    } else {
        ctrans_expr(trans, left, out);
        fprintf(out, "%s", op_str);
        ctrans_expr(trans, right, out);
    }
    return 0;
}

int ctrans_expr_call(CTrans* trans, Expr* callee, Expr** args, size_t arg_count, FILE* out) {
    // Check for special functions
    if (callee->kind == EXPR_IDENT) {
        const char* name = callee->u.ident_val.data;
        if (strcmp(name, "print") == 0) {
            // Convert print to printf
            ctrans_add_lib(trans, "<stdio.h>");
            fprintf(out, "printf(\"");
            for (size_t i = 0; i < arg_count; i++) {
                // Simple format detection
                if (args[i]->kind == EXPR_STR || args[i]->kind == EXPR_CSTR) {
                    fprintf(out, "%%s");
                } else if (args[i]->kind == EXPR_INT || args[i]->kind == EXPR_UINT) {
                    fprintf(out, "%%d");
                } else if (args[i]->kind == EXPR_DOUBLE) {
                    fprintf(out, "%%g");
                } else {
                    fprintf(out, "%%d");
                }
                if (i < arg_count - 1) fprintf(out, " ");
            }
            fprintf(out, "\\n\"");
            for (size_t i = 0; i < arg_count; i++) {
                fprintf(out, ", ");
                ctrans_expr(trans, args[i], out);
            }
            fprintf(out, ")");
            return 0;
        }
    }

    // Regular function call
    ctrans_expr(trans, callee, out);
    fprintf(out, "(");
    for (size_t i = 0; i < arg_count; i++) {
        ctrans_expr(trans, args[i], out);
        if (i < arg_count - 1) fprintf(out, ", ");
    }
    fprintf(out, ")");
    return 0;
}

int ctrans_expr_array(CTrans* trans, Expr** elems, size_t count, FILE* out) {
    fprintf(out, "{");
    for (size_t i = 0; i < count; i++) {
        ctrans_expr(trans, elems[i], out);
        if (i < count - 1) fprintf(out, ", ");
    }
    fprintf(out, "}");
    return 0;
}

int ctrans_expr_object(CTrans* trans, Pair* pairs, size_t count, FILE* out) {
    fprintf(out, "{");
    for (size_t i = 0; i < count; i++) {
        fprintf(out, ".%s = ", pairs[i].key.data);
        ctrans_expr(trans, pairs[i].value, out);
        if (i < count - 1) fprintf(out, ", ");
    }
    fprintf(out, "}");
    return 0;
}

int ctrans_expr_index(CTrans* trans, Expr* array, Expr* index, FILE* out) {
    ctrans_expr(trans, array, out);
    fprintf(out, "[");
    ctrans_expr(trans, index, out);
    fprintf(out, "]");
    return 0;
}

// ============================================================================
// Statement Transpilation
// ============================================================================

int ctrans_stmt(CTrans* trans, Stmt* stmt) {
    if (!stmt) return 0;

    switch (stmt->kind) {
        case STMT_EXPR:
            return ctrans_stmt_expr(trans, stmt->u.expr);

        case STMT_STORE: {
            const char* kind = "let"; // Default
            return ctrans_stmt_store(trans, kind, stmt->u.store.name.data,
                                    stmt->u.store.ty, stmt->u.store.expr);
        }

        case STMT_IF:
            return ctrans_stmt_if(trans, stmt->u.if_stmt.cond, stmt->u.if_stmt.then_body, stmt->u.if_stmt.else_body);

        case STMT_FOR:
            return ctrans_stmt_for(trans, stmt->u.for_stmt.var_name.data, stmt->u.for_stmt.iter, stmt->u.for_stmt.body);

        case STMT_BLOCK: {
            fprintf(trans->source, "{\n");
            ctrans_indent(trans);
            for (size_t i = 0; i < stmt->u.block.count; i++) {
                ctrans_print_indent(trans, trans->source);
                ctrans_stmt(trans, stmt->u.block.stmts[i]);
                fprintf(trans->source, "\n");
            }
            ctrans_dedent(trans);
            ctrans_print_indent(trans, trans->source);
            fprintf(trans->source, "}");
            return 0;
        }

        default:
            fprintf(stderr, "Unsupported statement kind: %d\n", stmt->kind);
            return -1;
    }
}

int ctrans_stmt_expr(CTrans* trans, Expr* expr) {
    ctrans_expr(trans, expr, trans->source);
    ctrans_eos(trans, trans->source);
    return 0;
}

int ctrans_stmt_store(CTrans* trans, const char* kind, const char* name, Type* ty, Expr* expr) {
    const char* type_name = ctrans_type_name(trans, ty);
    fprintf(trans->source, "%s %s = ", type_name, name);
    ctrans_expr(trans, expr, trans->source);
    fprintf(trans->source, ";\n");
    return 0;
}

int ctrans_stmt_if(CTrans* trans, Expr* cond, Stmt* then_branch, Stmt* else_branch) {
    fprintf(trans->source, "if (");
    ctrans_expr(trans, cond, trans->source);
    fprintf(trans->source, ") ");
    ctrans_stmt(trans, then_branch);
    if (else_branch) {
        fprintf(trans->source, " else ");
        ctrans_stmt(trans, else_branch);
    }
    return 0;
}

int ctrans_stmt_for(CTrans* trans, const char* var_name, Expr* iter, Stmt* body) {
    // Only handle range iteration for now
    if (iter->kind == EXPR_RANGE) {
        Range* r = &iter->u.range;
        fprintf(trans->source, "for (int %s = ", var_name);
        ctrans_expr(trans, r->start, trans->source);
        fprintf(trans->source, "; %s %s ", var_name, r->eq ? "<=" : "<");
        ctrans_expr(trans, r->end, trans->source);
        fprintf(trans->source, "; %s++) ", var_name);
        ctrans_stmt(trans, body);
    }
    return 0;
}

int ctrans_stmt_fn(CTrans* trans, const char* name, Type* ret, Expr** params, size_t param_count, Stmt** body_stmts, size_t body_count) {
    // Function declaration in header
    const char* ret_name = ctrans_type_name(trans, ret);
    fprintf(trans->header, "%s %s(", ret_name, name);
    for (size_t i = 0; i < param_count; i++) {
        // Assume params are identifiers with types
        if (params[i] && params[i]->kind == EXPR_IDENT) {
            fprintf(trans->header, "int %s", params[i]->u.ident_val.data);
        } else {
            fprintf(trans->header, "int param%zu", i);
        }
        if (i < param_count - 1) fprintf(trans->header, ", ");
    }
    fprintf(trans->header, ");\n");

    // Function definition in source
    fprintf(trans->source, "%s %s(", ret_name, name);
    for (size_t i = 0; i < param_count; i++) {
        if (params[i] && params[i]->kind == EXPR_IDENT) {
            fprintf(trans->source, "int %s", params[i]->u.ident_val.data);
        } else {
            fprintf(trans->source, "int param%zu", i);
        }
        if (i < param_count - 1) fprintf(trans->source, ", ");
    }
    fprintf(trans->source, ") {\n");
    ctrans_indent(trans);
    for (size_t i = 0; i < body_count; i++) {
        ctrans_print_indent(trans, trans->source);
        if (body_stmts[i]->kind == STMT_EXPR) {
            ctrans_stmt_expr(trans, body_stmts[i]->u.expr);
        } else {
            ctrans_stmt(trans, body_stmts[i]);
        }
        fprintf(trans->source, "\n");
    }
    ctrans_dedent(trans);
    fprintf(trans->source, "}\n");
    return 0;
}

// ============================================================================
// Main Transpilation
// ============================================================================

int ctrans_trans(CTrans* trans, Code* code) {
    // Separate declarations and main code
    int has_main = 0;
    int decl_count = 0;
    int main_count = 0;

    // Count statements and determine structure
    for (size_t i = 0; i < code->count; i++) {
        if (code->stmts[i]->kind == STMT_FN) {
            decl_count++;
        } else {
            main_count++;
        }
    }

    // Write header guard
    if (trans->style == C_STYLE_TRADITIONAL) {
        char guard_name[256];
        snprintf(guard_name, sizeof(guard_name), "%s_H", trans->name);
        for (size_t i = 0; i < strlen(guard_name); i++) {
            if (guard_name[i] >= 'a' && guard_name[i] <= 'z') {
                guard_name[i] = guard_name[i] - 'a' + 'A';
            }
        }
        fprintf(trans->header, "#ifndef %s\n#define %s\n\n", guard_name, guard_name);
    } else {
        fprintf(trans->header, "#pragma once\n\n");
    }

    // Write includes
    for (size_t i = 0; i < trans->lib_count; i++) {
        fprintf(trans->header, "#include %s\n", trans->libs[i]);
    }
    if (trans->lib_count > 0) {
        fprintf(trans->header, "\n");
    }

    // Process declarations first
    for (size_t i = 0; i < code->count; i++) {
        if (code->stmts[i]->kind == STMT_FN) {
            // Convert STMT_FN to function - for now simplified
            fprintf(stderr, "Function declaration not fully implemented\n");
        }
    }

    // Write main function
    if (main_count > 0) {
        fprintf(trans->header, "int main(void);\n");

        fprintf(trans->source, "int main(void) {\n");
        ctrans_indent(trans);
        for (size_t i = 0; i < code->count; i++) {
            if (code->stmts[i]->kind != STMT_FN) {
                ctrans_print_indent(trans, trans->source);
                ctrans_stmt(trans, code->stmts[i]);
            }
        }
        ctrans_print_indent(trans, trans->source);
        fprintf(trans->source, "return 0;\n");
        ctrans_dedent(trans);
        fprintf(trans->source, "}\n");
    }

    // Close header guard
    if (trans->style == C_STYLE_TRADITIONAL) {
        char guard_name[256];
        snprintf(guard_name, sizeof(guard_name), "%s_H", trans->name);
        fprintf(trans->header, "#endif // %s\n", guard_name);
    }

    return 0;
}

// ============================================================================
// Output Functions
// ============================================================================

int ctrans_flush_header(CTrans* trans, FILE* header_file) {
    if (trans->header_buf && trans->header_size > 0) {
        fwrite(trans->header_buf, 1, trans->header_size, header_file);
    }
    return 0;
}

int ctrans_flush_source(CTrans* trans, FILE* source_file) {
    if (trans->source_buf && trans->source_size > 0) {
        fwrite(trans->source_buf, 1, trans->source_size, source_file);
    }
    return 0;
}

char* ctrans_get_header(CTrans* trans, size_t* size) {
    if (size) *size = trans->header_size;
    return trans->header_buf;
}

char* ctrans_get_source(CTrans* trans, size_t* size) {
    if (size) *size = trans->source_size;
    return trans->source_buf;
}
