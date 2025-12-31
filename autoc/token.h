/**
 * Token Types
 * Token kinds and token structure
 */

#ifndef TOKEN_H
#define TOKEN_H

#include "common.h"

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

#endif // TOKEN_H
