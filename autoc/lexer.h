/**
 * Lexer Interface
 * Lexical analysis for auto-lang
 */

#ifndef LEXER_H
#define LEXER_H

#include "token.h"

typedef struct {
    const char* input;
    size_t input_len;
    size_t pos;
    size_t line;
    size_t at;
    char fstr_note;
    bool in_fstr_expr;  // Flag to prevent re-entering f-string mode
    Token* buffer;
    size_t buffer_count;
    size_t buffer_capacity;
    Token last;
} Lexer;

Lexer* lexer_new(const char* input);
void lexer_free(Lexer* lexer);
void lexer_set_fstr_note(Lexer* lexer, char note);
Token lexer_next(Lexer* lexer);

#endif // LEXER_H
