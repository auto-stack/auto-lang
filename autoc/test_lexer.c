/**
 * Lexer Test Runner
 * Reads test cases from lexer_tests.md and runs them
 */

#include "autoc.h"
#include "test.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>

// Get token kind name
static const char* token_kind_name(TokenKind kind) {
    switch (kind) {
        case TOKEN_INT: return "int";
        case TOKEN_UINT: return "uint";
        case TOKEN_U8: return "u8";
        case TOKEN_I8: return "i8";
        case TOKEN_FLOAT: return "float";
        case TOKEN_DOUBLE: return "double";
        case TOKEN_STR: return "str";
        case TOKEN_CSTR: return "cstr";
        case TOKEN_CHAR: return "'";
        case TOKEN_IDENT: return "ident";

        case TOKEN_LPAREN: return "(";
        case TOKEN_RPAREN: return ")";
        case TOKEN_LSQUARE: return "[";
        case TOKEN_RSQUARE: return "]";
        case TOKEN_LBRACE: return "{";
        case TOKEN_RBRACE: return "}";
        case TOKEN_COMMA: return ",";
        case TOKEN_SEMI: return ";";
        case TOKEN_NEWLINE: return "nl";
        case TOKEN_ADD: return "+";
        case TOKEN_SUB: return "-";
        case TOKEN_STAR: return "*";
        case TOKEN_DIV: return "/";
        case TOKEN_NOT: return "!";
        case TOKEN_LT: return "<";
        case TOKEN_GT: return ">";
        case TOKEN_LE: return "<=";
        case TOKEN_GE: return ">=";
        case TOKEN_ASN: return "=";
        case TOKEN_EQ: return "==";
        case TOKEN_NEQ: return "!=";
        case TOKEN_ADDEQ: return "+=";
        case TOKEN_SUBEQ: return "-=";
        case TOKEN_MULEQ: return "*=";
        case TOKEN_DIVEQ: return "/=";
        case TOKEN_DOT: return ".";
        case TOKEN_RANGE: return "..";
        case TOKEN_RANGEEQ: return "...";
        case TOKEN_COLON: return ":";
        case TOKEN_VBAR: return "|";
        case TOKEN_COMMENT_LINE: return "//";
        case TOKEN_COMMENT_CONTENT: return "comment";
        case TOKEN_COMMENT_START: return "/*";
        case TOKEN_COMMENT_END: return "*/";
        case TOKEN_ARROW: return "->";
        case TOKEN_DOUBLE_ARROW: return "=>";
        case TOKEN_QUESTION: return "?";
        case TOKEN_AT: return "@";
        case TOKEN_HASH: return "#";

        case TOKEN_TRUE: return "true";
        case TOKEN_FALSE: return "false";
        case TOKEN_NIL: return "nil";
        case TOKEN_NULL: return "null";
        case TOKEN_IF: return "if";
        case TOKEN_ELSE: return "else";
        case TOKEN_FOR: return "for";
        case TOKEN_WHEN: return "when";
        case TOKEN_BREAK: return "break";
        case TOKEN_IS: return "is";
        case TOKEN_VAR: return "var";
        case TOKEN_IN: return "in";
        case TOKEN_FN: return "fn";
        case TOKEN_TYPE: return "type";
        case TOKEN_UNION: return "union";
        case TOKEN_TAG: return "tag";
        case TOKEN_LET: return "let";
        case TOKEN_MUT: return "mut";
        case TOKEN_HAS: return "has";
        case TOKEN_USE: return "use";
        case TOKEN_AS: return "as";
        case TOKEN_ENUM: return "enum";
        case TOKEN_ON: return "on";
        case TOKEN_ALIAS: return "alias";

        case TOKEN_FSTR_START: return "fstrs";
        case TOKEN_FSTR_PART: return "fstrp";
        case TOKEN_FSTR_END: return "fstre";
        case TOKEN_FSTR_NOTE: return "$";

        case TOKEN_GRID: return "grid";
        case TOKEN_EOF: return "EOF";

        default: return "unknown";
    }
}

// Convert token to string representation
static char* token_to_string(Token* token) {
    if (!token) return strdup("");

    const char* kind_name = token_kind_name(token->kind);
    size_t len = strlen(kind_name) + 3; // <, >, and null terminator
    char* result = (char*)malloc(len);

    if (token->text.data && strlen(token->text.data) > 0) {
        // Only include text if it's not already in the kind name
        if (token->kind == TOKEN_INT || token->kind == TOKEN_UINT ||
            token->kind == TOKEN_U8 || token->kind == TOKEN_I8 ||
            token->kind == TOKEN_FLOAT || token->kind == TOKEN_DOUBLE ||
            token->kind == TOKEN_STR || token->kind == TOKEN_CSTR ||
            token->kind == TOKEN_IDENT || token->kind == TOKEN_FSTR_PART ||
            token->kind == TOKEN_COMMENT_CONTENT) {
            size_t total_len = len + strlen(token->text.data) + 1; // +1 for :
            result = (char*)realloc(result, total_len);
            snprintf(result, total_len, "<%s:%s>", kind_name, token->text.data);
        } else {
            snprintf(result, len, "<%s>", kind_name);
        }
    } else {
        snprintf(result, len, "<%s>", kind_name);
    }

    return result;
}

// Tokenize code and get token representation
static char* get_tokens_repr(const char* code) {
    if (!code) return strdup("Error: NULL code");

    Lexer* lexer = lexer_new(code);
    if (!lexer) return strdup("Error: Failed to create lexer");

    // Build result string using a dynamic array of token strings
    size_t capacity = 256;
    size_t count = 0;
    char** tokens = (char**)malloc(capacity * sizeof(char*));

    while (true) {
        Token token = lexer_next(lexer);
        if (token.kind == TOKEN_EOF) {
            break;
        }

        // Ensure capacity
        if (count >= capacity) {
            capacity *= 2;
            tokens = (char**)realloc(tokens, capacity * sizeof(char*));
        }

        tokens[count++] = token_to_string(&token);
    }

    lexer_free(lexer);

    // Strip trailing newlines
    while (count > 0) {
        if (strcmp(tokens[count - 1], "<nl>") == 0) {
            free(tokens[count - 1]);
            count--;
        } else {
            break;
        }
    }

    // Calculate total length
    size_t total_len = 0;
    for (size_t i = 0; i < count; i++) {
        total_len += strlen(tokens[i]);
    }

    // Build final result
    char* result = (char*)malloc(total_len + 1);
    result[0] = '\0';
    for (size_t i = 0; i < count; i++) {
        strcat(result, tokens[i]);
        free(tokens[i]);
    }
    free(tokens);

    return result;
}

// Test runner function
static bool run_lexer_test(MarkdownTestCase* tc, TestStatistics* stats) {
    char* actual = get_tokens_repr(tc->input);

    stats->run++;
    if (!compare_exact(actual, tc->expected)) {
        fprintf(stderr, "  \033[31mFAILED\033[0m: %s\n", tc->name);
        fprintf(stderr, "    Expected: %s\n", tc->expected);
        fprintf(stderr, "    Actual:   %s\n", actual);
        stats->failed++;
        free(actual);
        return false;
    }
    stats->passed++;
    printf("  \033[32mPASSED\033[0m: %s\n", tc->name);
    free(actual);
    return true;
}

int main() {
    return run_markdown_test_suite("tests/lexer_tests.md", "Lexer Test Runner", run_lexer_test);
}
