/**
 * C Transpiler Implementation
 * Translates auto-lang AST to C code
 */

#ifndef TRANS_C_H
#define TRANS_C_H

#include "autoc.h"
#include <stdio.h>

// ============================================================================
// C Transpiler
// ============================================================================

typedef enum {
    C_STYLE_MODERN,    // #pragma once
    C_STYLE_TRADITIONAL, // #ifndef guards
} CStyle;

typedef struct {
    FILE* header;       // Header file output
    FILE* source;       // Source file output
    char* header_buf;   // Header buffer
    char* source_buf;   // Source buffer
    size_t header_size;
    size_t header_capacity;
    size_t source_size;
    size_t source_capacity;
    char* name;
    int indent;
    CStyle style;
    char** libs;
    size_t lib_count;
    size_t lib_capacity;
    Universe* universe;
} CTrans;

// Create C transpiler
CTrans* ctrans_new(const char* name, Universe* universe);
void ctrans_free(CTrans* trans);

// Set C style
void ctrans_set_style(CTrans* trans, CStyle style);

// Main transpilation function
int ctrans_trans(CTrans* trans, Code* code);

// Statement transpilation
int ctrans_stmt(CTrans* trans, Stmt* stmt);
int ctrans_stmt_expr(CTrans* trans, Expr* expr);
int ctrans_stmt_store(CTrans* trans, const char* kind, const char* name, Type* ty, Expr* expr);
int ctrans_stmt_if(CTrans* trans, Expr* cond, Stmt* then_branch, Stmt* else_branch);
int ctrans_stmt_for(CTrans* trans, const char* var_name, Expr* iter, Stmt* body);
int ctrans_stmt_fn(CTrans* trans, const char* name, Type* ret, Expr** params, size_t param_count, Stmt** body_stmts, size_t body_count);

// Expression transpilation
int ctrans_expr(CTrans* trans, Expr* expr, FILE* out);
int ctrans_expr_int(CTrans* trans, int32_t value, FILE* out);
int ctrans_expr_float(CTrans* trans, double value, FILE* out);
int ctrans_expr_str(CTrans* trans, const char* value, FILE* out);
int ctrans_expr_ident(CTrans* trans, const char* name, FILE* out);
int ctrans_expr_call(CTrans* trans, Expr* callee, Expr** args, size_t arg_count, FILE* out);
int ctrans_expr_binary(CTrans* trans, Expr* left, int op, Expr* right, FILE* out);
int ctrans_expr_unary(CTrans* trans, int op, Expr* expr, FILE* out);
int ctrans_expr_array(CTrans* trans, Expr** elems, size_t count, FILE* out);
int ctrans_expr_object(CTrans* trans, Pair* pairs, size_t count, FILE* out);
int ctrans_expr_index(CTrans* trans, Expr* array, Expr* index, FILE* out);

// Type transpilation
const char* ctrans_type_name(CTrans* trans, Type* ty);

// Utility functions
int ctrans_indent(CTrans* trans);
int ctrans_dedent(CTrans* trans);
int ctrans_print_indent(CTrans* trans, FILE* out);
int ctrans_write(CTrans* trans, FILE* out, const char* text);
int ctrans_write_header(CTrans* trans, const char* text);
int ctrans_eos(CTrans* trans, FILE* out);  // End of statement

// Output functions
int ctrans_flush_header(CTrans* trans, FILE* header_file);
int ctrans_flush_source(CTrans* trans, FILE* source_file);
char* ctrans_get_header(CTrans* trans, size_t* size);
char* ctrans_get_source(CTrans* trans, size_t* size);

// Add library include
void ctrans_add_lib(CTrans* trans, const char* lib);

#endif // TRANS_C_H
