/**
 * auto-lang C Compiler
 * A C implementation of the auto-lang compiler
 */

#ifndef AUTOC_H
#define AUTOC_H

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>

// ============================================================================
// Common Types
// ============================================================================

typedef struct {
    char* data;
    size_t len;
    size_t capacity;
} AutoStr;

typedef struct {
    size_t line;
    size_t at;
    size_t pos;
    size_t len;
} Pos;

// ============================================================================
// Token Types
// ============================================================================

typedef enum {
    // Literals
    TOKEN_INT,
    TOKEN_UINT,
    TOKEN_U8,
    TOKEN_I8,
    TOKEN_FLOAT,
    TOKEN_DOUBLE,
    TOKEN_STR,
    TOKEN_CSTR,
    TOKEN_CHAR,
    TOKEN_IDENT,

    // Operators
    TOKEN_LPAREN,
    TOKEN_RPAREN,
    TOKEN_LSQUARE,
    TOKEN_RSQUARE,
    TOKEN_LBRACE,
    TOKEN_RBRACE,
    TOKEN_COMMA,
    TOKEN_SEMI,
    TOKEN_NEWLINE,
    TOKEN_ADD,
    TOKEN_SUB,
    TOKEN_STAR,
    TOKEN_DIV,
    TOKEN_NOT,
    TOKEN_LT,
    TOKEN_GT,
    TOKEN_LE,
    TOKEN_GE,
    TOKEN_ASN,
    TOKEN_EQ,
    TOKEN_NEQ,
    TOKEN_ADDEQ,
    TOKEN_SUBEQ,
    TOKEN_MULEQ,
    TOKEN_DIVEQ,
    TOKEN_DOT,
    TOKEN_RANGE,
    TOKEN_RANGEEQ,
    TOKEN_COLON,
    TOKEN_VBAR,
    TOKEN_COMMENT_LINE,
    TOKEN_COMMENT_CONTENT,
    TOKEN_COMMENT_START,
    TOKEN_COMMENT_END,
    TOKEN_ARROW,
    TOKEN_DOUBLE_ARROW,
    TOKEN_QUESTION,
    TOKEN_AT,
    TOKEN_HASH,

    // Keywords
    TOKEN_TRUE,
    TOKEN_FALSE,
    TOKEN_NIL,
    TOKEN_NULL,
    TOKEN_IF,
    TOKEN_ELSE,
    TOKEN_FOR,
    TOKEN_WHEN,
    TOKEN_BREAK,
    TOKEN_IS,
    TOKEN_VAR,
    TOKEN_IN,
    TOKEN_FN,
    TOKEN_TYPE,
    TOKEN_UNION,
    TOKEN_TAG,
    TOKEN_LET,
    TOKEN_MUT,
    TOKEN_HAS,
    TOKEN_USE,
    TOKEN_AS,
    TOKEN_ENUM,
    TOKEN_ON,
    TOKEN_ALIAS,

    // Format String
    TOKEN_FSTR_START,
    TOKEN_FSTR_PART,
    TOKEN_FSTR_END,
    TOKEN_FSTR_NOTE,

    // AutoData
    TOKEN_GRID,

    // EOF
    TOKEN_EOF,
} TokenKind;

typedef struct {
    TokenKind kind;
    Pos pos;
    AutoStr text;
} Token;

// ============================================================================
// AST Types
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

typedef struct {
    TypeKind kind;
    AutoStr name;
    size_t str_len;  // For TYPE_STR
    struct Type* elem_type;  // For TYPE_ARRAY, TYPE_PTR
} Type;

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

typedef struct {
    Stmt** stmts;
    size_t count;
    size_t capacity;
} Code;

// ============================================================================
// Lexer
// ============================================================================

typedef struct {
    const char* input;
    size_t input_len;
    size_t pos;
    size_t line;
    size_t at;
    char fstr_note;
    Token* buffer;
    size_t buffer_count;
    size_t buffer_capacity;
    Token last;
} Lexer;

Lexer* lexer_new(const char* input);
void lexer_free(Lexer* lexer);
void lexer_set_fstr_note(Lexer* lexer, char note);
Token lexer_next(Lexer* lexer);

// ============================================================================
// Parser
// ============================================================================

typedef struct {
    Lexer* lexer;
    Token current;
    Token peek;
    int scope_depth;
} Parser;

Parser* parser_new(Lexer* lexer);
void parser_free(Parser* parser);
Code* parser_parse(Parser* parser);
Stmt* parser_parse_stmt(Parser* parser);
Expr* parser_parse_expr(Parser* parser);

// ============================================================================
// Value Runtime
// ============================================================================

typedef enum {
    VAL_NIL,
    VAL_VOID,
    VAL_BOOL,
    VAL_BYTE,
    VAL_INT,
    VAL_UINT,
    VAL_FLOAT,
    VAL_DOUBLE,
    VAL_CHAR,
    VAL_STR,
    VAL_ARRAY,
    VAL_OBJECT,
    VAL_RANGE,
    VAL_ERROR,
} ValueKind;

typedef struct Value Value;

typedef struct {
    Value** values;
    size_t count;
    size_t capacity;
} ValueArray;

typedef struct {
    AutoStr key;
    Value* value;
} KeyValue;

typedef struct {
    KeyValue* pairs;
    size_t count;
    size_t capacity;
} ValueObject;

struct Value {
    ValueKind kind;
    union {
        bool bool_val;
        uint8_t byte_val;
        int32_t int_val;
        uint32_t uint_val;
        double float_val;
        char char_val;
        AutoStr str_val;
        ValueArray array_val;
        ValueObject object_val;
        struct {
            int32_t start;
            int32_t end;
            bool eq;
        } range_val;
        AutoStr error_val;
    } u;
};

// ============================================================================
// Scope and Universe
// ============================================================================

typedef enum {
    SCOPE_GLOBAL,
    SCOPE_MOD,
    SCOPE_TYPE,
    SCOPE_FN,
    SCOPE_BLOCK,
} ScopeKind;

typedef struct {
    AutoStr path;
} Sid;

typedef struct {
    ScopeKind kind;
    Sid sid;
    Sid* parent;
    Sid** kids;
    size_t kid_count;
    size_t kid_capacity;
    // symbol tables
    Value** values;
    AutoStr* keys;
    size_t val_count;
    size_t val_capacity;
} Scope;

typedef struct {
    Scope** scopes;
    size_t scope_count;
    size_t scope_capacity;
    Scope* global;
    Scope* current;
    Sid* cur_spot;
} Universe;

Universe* universe_new(void);
void universe_free(Universe* universe);
Scope* universe_enter_scope(Universe* universe, ScopeKind kind);
void universe_exit_scope(Universe* universe);
Value* universe_get(Universe* universe, const char* name);
void universe_set(Universe* universe, const char* name, Value* value);
Value* universe_lookup(Universe* universe, const char* name);

// ============================================================================
// Evaluator
// ============================================================================

typedef enum {
    EVAL_MODE_SCRIPT,
    EVAL_MODE_CONFIG,
    EVAL_MODE_TEMPLATE,
} EvalMode;

typedef struct {
    Universe* universe;
    EvalMode mode;
    bool skip_check;
} Evaler;

Evaler* evaler_new(Universe* universe);
void evaler_free(Evaler* evaler);
Value* evaler_eval(Evaler* evaler, Code* code);
Value* evaler_eval_stmt(Evaler* evaler, Stmt* stmt);
Value* evaler_eval_expr(Evaler* evaler, Expr* expr);

// ============================================================================
// Value Operations
// ============================================================================

Value* value_nil(void);
Value* value_void(void);
Value* value_bool(bool b);
Value* value_int(int32_t i);
Value* value_uint(uint32_t u);
Value* value_float(double f);
Value* value_str(const char* s);
Value* value_byte(uint8_t b);
Value* value_char(char c);
Value* value_array(ValueArray arr);
Value* value_object(ValueObject obj);
Value* value_range(int32_t start, int32_t end, bool eq);
Value* value_error(const char* msg);
void value_free(Value* value);
Value* value_clone(Value* value);
const char* value_repr(Value* value);
bool value_is_true(Value* value);
bool value_is_nil(Value* value);
bool value_is_void(Value* value);
bool value_is_error(Value* value);

Value* value_add(Value* a, Value* b);
Value* value_sub(Value* a, Value* b);
Value* value_mul(Value* a, Value* b);
Value* value_div(Value* a, Value* b);
Value* value_neg(Value* a);
Value* value_not(Value* a);
Value* value_eq(Value* a, Value* b);
Value* value_neq(Value* a, Value* b);
Value* value_lt(Value* a, Value* b);
Value* value_gt(Value* a, Value* b);
Value* value_le(Value* a, Value* b);
Value* value_ge(Value* a, Value* b);

// ============================================================================
// AutoString Utilities
// ============================================================================

AutoStr astr_new(const char* s);
AutoStr astr_from_len(const char* s, size_t len);
void astr_free(AutoStr* s);
AutoStr astr_clone(AutoStr* s);
bool astr_eq(AutoStr* a, AutoStr* b);
AutoStr astr_append(AutoStr* a, const char* s);
AutoStr astr_append_char(AutoStr* a, char c);

// ============================================================================
// Main API
// ============================================================================

typedef enum {
    AUTOC_OK,
    AUTOC_ERROR_LEX,
    AUTOC_ERROR_PARSE,
    AUTOC_ERROR_EVAL,
} AutoResult;

typedef struct {
    AutoResult result;
    Value* value;
    char* error_msg;
} AutoRunResult;

AutoRunResult autoc_run(const char* code);
void autorun_free(AutoRunResult* result);

// ============================================================================
// Transpilation API
// ============================================================================

typedef struct {
    AutoResult result;
    char* header_code;
    char* source_code;
    char* error_msg;
} AutoTransResult;

AutoTransResult autoc_trans(const char* code, const char* name);
void autotrans_free(AutoTransResult* result);

// ============================================================================
// Build System
// ============================================================================

#endif // AUTOC_H
