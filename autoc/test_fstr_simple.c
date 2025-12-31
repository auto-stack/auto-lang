#include "autoc.h"
#include <stdio.h>

static const char* token_kind_name(TokenKind kind) {
    switch (kind) {
        case TOKEN_INT: return "int";
        case TOKEN_UINT: return "uint";
        case TOKEN_STR: return "str";
        case TOKEN_IDENT: return "ident";
        case TOKEN_LPAREN: return "(";
        case TOKEN_RPAREN: return ")";
        case TOKEN_LBRACE: return "{";
        case TOKEN_RBRACE: return "}";
        case TOKEN_FSTR_START: return "fstrs";
        case TOKEN_FSTR_PART: return "fstrp";
        case TOKEN_FSTR_END: return "fstre";
        case TOKEN_FSTR_NOTE: return "$";
        case TOKEN_EOF: return "EOF";
        default: return "?";
    }
}

int main() {
    const char* input = "f\"hello ${2}\"";
    printf("Testing: %s\n", input);

    Lexer* lexer = lexer_new(input);
    if (!lexer) {
        printf("Failed to create lexer\n");
        return 1;
    }

    printf("Tokens:\n");
    int count = 0;
    while (count < 100) {  // Limit to 100 tokens to avoid infinite loop
        Token token = lexer_next(lexer);
        printf("[%d] %s", count, token_kind_name(token.kind));
        if (token.text.data && strlen(token.text.data) > 0) {
            printf(":%s", token.text.data);
        }
        printf("\n");

        if (token.kind == TOKEN_EOF) {
            break;
        }
        count++;
    }

    if (count >= 100) {
        printf("ERROR: Too many tokens (%d) - possible infinite loop\n", count);
    }

    lexer_free(lexer);
    return 0;
}
