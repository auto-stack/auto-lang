/**
 * AST Types
 * Abstract Syntax Tree node definitions
 */

#ifndef AST_H
#define AST_H

#include "common.h"
#include "token.h"

// ============================================================================
// Type System
// ============================================================================

typedef enum {
    TYPE_BYTE,
    TYPE_INT,
    TYPE_UINT,
    TYPE_FLOAT,
    TYPE_DOUBLE,
    TYPE_BOOL,
    TYPE_CHAR,
    TYPE_STR,
    TYPE_CSTR,
    TYPE_ARRAY,
    TYPE_PTR,
    TYPE_VOID,
    TYPE_UNKNOWN,
    TYPE_USER,  // User-defined type
} TypeKind;

typedef struct Type Type;

struct Type {
    TypeKind kind;
    AutoStr name;
    size_t str_len;           // For TYPE_STR
    struct Type* elem_type;   // For TYPE_ARRAY, TYPE_PTR
};

// ============================================================================
// Expressions
// ============================================================================

typedef enum {
    EXPR_BYTE,
    EXPR_INT,
    EXPR_UINT,
    EXPR_I8,
    EXPR_U8,
    EXPR_I64,
    EXPR_FLOAT,
    EXPR_DOUBLE,
    EXPR_BOOL,
    EXPR_CHAR,
    EXPR_STR,
    EXPR_CSTR,
    EXPR_IDENT,
    EXPR_REF,
    EXPR_UNARY,
    EXPR_BINA,
    EXPR_RANGE,
    EXPR_ARRAY,
    EXPR_PAIR,
    EXPR_BLOCK,
    EXPR_OBJECT,
    EXPR_CALL,
    EXPR_INDEX,
    EXPR_IF,
    EXPR_NIL,
    EXPR_NULL,
} ExprKind;

typedef struct Expr Expr;

typedef struct {
    AutoStr key;
    Expr* value;
} Pair;

typedef struct {
    Expr* start;
    Expr* end;
    bool eq;
} Range;

struct Expr {
    ExprKind kind;
    Pos pos;
    union {
        uint8_t byte_val;
        int32_t int_val;
        uint32_t uint_val;
        int8_t i8_val;
        uint8_t u8_val;
        int64_t i64_val;
        double float_val;
        bool bool_val;
        char char_val;
        AutoStr str_val;
        AutoStr ident_val;
        struct {
            int op;
            Expr* expr;
        } unary;
        struct {
            Expr* left;
            int op;
            Expr* right;
        } bina;
        Range range;
        struct {
            Expr** elems;
            size_t count;
            size_t capacity;
        } array;
        Pair pair;
        struct {
            Expr** stmts;
            size_t count;
            size_t capacity;
        } block;
        struct {
            Pair* pairs;
            size_t count;
            size_t capacity;
        } object;
        struct {
            Expr* callee;
            Expr** args;
            size_t count;
            size_t capacity;
        } call;
        struct {
            Expr* array;
            Expr* index;
        } index;
        struct {
            Expr* cond;
            Expr* then_body;
            Expr* else_body;
        } if_expr;
    } u;
};

// ============================================================================
// Statements
// ============================================================================

typedef enum {
    STMT_EXPR,
    STMT_IF,
    STMT_FOR,
    STMT_STORE,
    STMT_BLOCK,
    STMT_FN,
    STMT_EMPTY_LINE,
    STMT_BREAK,
} StmtKind;

typedef struct {
    StmtKind kind;
    union {
        Expr* expr;
        struct {
            Expr* cond;
            Expr* then_body;
            Expr* else_body;
        } if_stmt;
        struct {
            AutoStr var_name;
            Expr* iter;
            Expr* body;
        } for_stmt;
        struct {
            AutoStr name;
            Type* ty;
            Expr* expr;
        } store;
        struct {
            Expr** stmts;
            size_t count;
            size_t capacity;
        } block;
    } u;
} Stmt;

// ============================================================================
// Code
// ============================================================================

typedef struct {
    Stmt** stmts;
    size_t count;
    size_t capacity;
} Code;

#endif // AST_H
