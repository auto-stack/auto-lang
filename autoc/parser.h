/**
 * Parser Interface
 * Pratt parser for auto-lang
 */

#ifndef PARSER_H
#define PARSER_H

#include "lexer.h"
#include "ast.h"

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

#endif // PARSER_H
